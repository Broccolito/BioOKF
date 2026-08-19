/* Native Paperclip → BioOKF generation window.
   The Rust command launches the machine-local subscription-backed harness with
   argv (never a shell), streams progress events, registers the completed base,
   and returns its manifest. */
(function(){
  'use strict';
  const $=id=>document.getElementById(id);
  const modal=$('paperclip-modal'), form=$('paperclip-form'), openBtn=$('paperclipGenerateBtn');
  if(!modal||!form||!openBtn) return;
  const submit=$('paperclip-submit'), cancel=$('paperclip-cancel'), close=$('paperclip-close');
  const provider=$('pc-provider'), model=$('pc-model'), customWrap=$('pc-custom-model-wrap');
  const run=$('pc-run'), events=$('pc-events'), progress=$('pc-progress'), result=$('pc-result');
  let catalog={codex:[],claude:[]}, running=false, progressCount=0, unlisten=null;
  const phaseWidth={search:18,snapshot:34,extract:62,build:82,studio:94,done:100};

  function esc(value){return String(value==null?'':value).replace(/[&<>'"]/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;',"'":'&#39;','"':'&quot;'}[c]));}
  function val(id){const el=$(id);return el&&String(el.value||'').trim()||null;}
  function number(id){const raw=val(id);return raw==null?null:Number(raw);}
  function setBusy(value){running=value;submit.disabled=value;cancel.disabled=value;close.disabled=value;submit.textContent=value?'Generating…':'Generate BioOKF KB';}
  function setHealth(message,kind){const el=$('pc-health');el.textContent=message;el.className='pc-health'+(kind?' '+kind:'');}

  function refreshModels(){
    const prior=model.value;
    const options=['<option value="">Subscription default (recommended)</option>'];
    (catalog[provider.value]||[]).forEach(item=>options.push(`<option value="${esc(item.id)}">${esc(item.label||item.id)}</option>`));
    options.push('<option value="__custom__">Custom model ID…</option>');
    model.innerHTML=options.join('');
    if([...model.options].some(option=>option.value===prior)) model.value=prior;
    toggleCustom();
  }
  function toggleCustom(){customWrap.hidden=model.value!=='__custom__';}
  function selectedModel(){return model.value==='__custom__'?val('pc-custom-model'):(model.value||null);}

  function updateLimitExplanation(){
    const selected=document.querySelectorAll('input[name="pc-source"]:checked').length;
    const perDatabase=Math.max(1,Number($('pc-limit').value)||1),maximum=selected*perDatabase;
    const resultWord=perDatabase===1?'result':'results',databaseWord=selected===1?'database':'databases',documentWord=maximum===1?'document':'documents';
    $('pc-limit-help').textContent=`${perDatabase} ${resultWord} × ${selected} selected ${databaseWord} = up to ${maximum} ${documentWord} overall before cross-database deduplication.`;
  }

  async function loadStatus(){
    setHealth('Checking Paperclip and subscription agents…','');
    try{
      const status=await tauriInvoke('paperclip_generator_status');
      catalog=status.models||catalog;refreshModels();
      if(!status.installed){setHealth('paperclip2bioOKF harness not found on this machine.','err');submit.disabled=true;return;}
      const doctor=status.doctor||{},agents=doctor.agents||{};
      const pc=doctor.paperclip&&doctor.paperclip.ok;
      const cx=agents.codex&&agents.codex.authenticated&&agents.codex.auth_method==='ChatGPT subscription';
      const cl=agents.claude&&agents.claude.authenticated&&agents.claude.auth_method==='claude.ai'&&!!agents.claude.subscription_type;
      setHealth(`Paperclip ${pc?'ready':'check required'} · Codex ${cx?'connected':'not connected'} · Claude ${cl?'connected':'not connected'} · output ${status.workspace}`,pc&&(cx||cl)?'ok':'err');
      submit.disabled=!!status.running;
    }catch(error){setHealth('Generator status failed: '+String(error),'err');submit.disabled=true;}
  }

  function resetRun(){run.hidden=true;result.hidden=true;result.innerHTML='';events.innerHTML='';progress.style.width='7%';progressCount=0;$('pc-run-title').textContent='Preparing generation';$('pc-run-phase').textContent='queued';}
  function addProgress(message){
    run.hidden=false;progressCount++;
    const match=String(message).match(/^\[([^\]]+)\]\s*(.*)$/),phase=match?match[1]:'working',text=match?match[2]:message;
    $('pc-run-title').textContent=text||'Working';$('pc-run-phase').textContent=phase;
    progress.style.width=(phaseWidth[phase]||Math.min(78,10+progressCount*5))+'%';
    const row=document.createElement('div');row.textContent='• '+message;events.appendChild(row);events.scrollTop=events.scrollHeight;
  }

  async function show(){
    if(!window.__TAURI__&&!window.__TAURI_INTERNALS__){showToast('Paperclip generation is available in the desktop app.','err');return;}
    modal.hidden=false;resetRun();await loadStatus();
  }
  function hide(){if(!running)modal.hidden=true;}

  async function generate(event){
    event.preventDefault();
    const sources=[...document.querySelectorAll('input[name="pc-source"]:checked')].map(input=>input.value);
    const request={query:val('pc-query')||'',sources,limit:number('pc-limit')||1,kbName:val('pc-kb-name')||'',provider:provider.value,model:selectedModel(),yearMin:number('pc-year-min'),yearMax:number('pc-year-max'),since:val('pc-since')};
    resetRun();run.hidden=false;setBusy(true);addProgress('[queued] Starting Paperclip generation');
    try{
      const output=await tauriInvoke('paperclip_generate_base',{request});
      progress.style.width='100%';$('pc-run-title').textContent='Knowledge base ready';$('pc-run-phase').textContent='done';
      const manifest=output.manifest||{},studio=output.studio||{};
      result.hidden=false;result.innerHTML=`<strong>${esc(request.kbName)}</strong><br>${manifest.source_count||0} sources · ${manifest.node_count||0} nodes · ${manifest.edge_count||0} edges · verified<br><code>${esc(output.bundle||'')}</code>`;
      bundleCache.clear();layoutCache.clear();lintCache.clear();pageLoadCache.clear();
      BASES=await loadBases();renderSidebar();lastBasesSig=basesSig(BASES);
      const generated=BASES.find(base=>base.id===studio.kb_id)||BASES.find(base=>base.path===output.bundle);
      if(generated) await selectBase(generated);
      showToast('Paperclip knowledge base generated and opened.');
      setTimeout(()=>{if(!running)modal.hidden=true;},900);
    }catch(error){
      const message=typeof error==='string'?error:(error&&error.message)||'Unknown generation error';
      $('pc-run-title').textContent='Generation stopped';$('pc-run-phase').textContent='error';result.hidden=false;result.innerHTML='<span style="color:var(--danger)">'+esc(message)+'</span>';setHealth(message,'err');
    }finally{setBusy(false);}
  }

  openBtn.onclick=show;close.onclick=hide;cancel.onclick=hide;modal.addEventListener('click',event=>{if(event.target===modal)hide();});
  form.addEventListener('submit',generate);provider.addEventListener('change',refreshModels);model.addEventListener('change',toggleCustom);
  $('pc-limit').addEventListener('input',updateLimitExplanation);document.querySelectorAll('input[name="pc-source"]').forEach(input=>input.addEventListener('change',updateLimitExplanation));updateLimitExplanation();
  window.addEventListener('keydown',event=>{if(event.key==='Escape'&&!modal.hidden)hide();});
  if(window.__TAURI__&&window.__TAURI__.event){window.__TAURI__.event.listen('paperclip2biookf-progress',event=>addProgress(event.payload&&event.payload.message||'Working')).then(fn=>{unlisten=fn;}).catch(()=>{});}
})();
