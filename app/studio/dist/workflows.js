/* BioOKF Studio machine-local subscription workflows. */
(function(){
  'use strict';
  const $=id=>document.getElementById(id), desktop=!!(window.__TAURI__||window.__TAURI_INTERNALS__);
  if(!desktop) return;
  let status=null, catalog={codex:[],claude:[]}, workflowBusy=false, chatBaseId=null,doctorBaseId=null,doctorProgressEl=null;
  const chats=new Map(),doctorChats=new Map();

  function open(id){const el=$(id);if(el)el.hidden=false;if(id==='kb-chat-modal'||id==='doctor-modal'){const other=id==='kb-chat-modal'?'doctor-modal':'kb-chat-modal';if($(other))$(other).hidden=true;document.querySelector('.window').classList.add('chat-open');requestAnimationFrame(()=>{if(typeof resize==='function')resize();});}}
  function close(id){if(workflowBusy&&(id==='local-kb-modal'||id==='merge-kb-modal'||id==='doctor-modal'))return;const el=$(id);if(el)el.hidden=true;if(id==='kb-chat-modal'||id==='doctor-modal'){document.querySelector('.window').classList.remove('chat-open');requestAnimationFrame(()=>{if(typeof resize==='function')resize();});}}
  function errorText(error){return typeof error==='string'?error:(error&&error.message)||String(error||'Unknown error');}
  function selectedModel(prefix){const select=$(prefix+'-model');return select.value==='__custom__'?($(prefix+'-custom').value.trim()||null):(select.value||null);}
  function modelOptions(prefix){
    const provider=$(prefix+'-provider'),select=$(prefix+'-model'),wrap=$(prefix+'-custom-wrap');if(!provider||!select)return;
    const prior=select.value,items=catalog[provider.value]||[];
    select.innerHTML='<option value="">Subscription default (recommended)</option>'+items.map(item=>`<option value="${esc(item.id)}">${esc(item.label||item.id)}</option>`).join('')+'<option value="__custom__">Custom model ID…</option>';
    if([...select.options].some(option=>option.value===prior))select.value=prior;
    const toggle=()=>{if(wrap)wrap.hidden=select.value!=='__custom__';};select.onchange=toggle;toggle();
  }
  function bindProvider(prefix){const provider=$(prefix+'-provider');if(provider)provider.onchange=()=>modelOptions(prefix);modelOptions(prefix);}
  function setBusy(button,value,label){if(button){button.disabled=value;button.textContent=value?'Working…':label;}}
  function resetCaches(){bundleCache.clear();layoutCache.clear();lintCache.clear();pageLoadCache.clear();}
  async function openGenerated(output){
    resetCaches();BASES=await loadBases();renderSidebar();lastBasesSig=basesSig(BASES);
    const studio=output.studio||{},generated=BASES.find(base=>base.id===studio.kb_id)||BASES.find(base=>base.path===output.bundle);
    if(generated)await selectBase(generated);
  }

  async function loadConnections(){
    status=await tauriInvoke('local_connections_status');catalog=status.models||catalog;
    ['chat','doctor','local','merge'].forEach(modelOptions);
    const configured=status.configured||{},detected=status.detected||{},agents=detected.agents||{};
    $('conn-codex').checked=configured.codex!==false;$('conn-claude').checked=configured.claude!==false;$('conn-paperclip').checked=configured.paperclip!==false;
    const cx=agents.codex||{},cl=agents.claude||{},pc=detected.paperclip||{};
    const cxSubscription=cx.authenticated&&cx.auth_method==='ChatGPT subscription';
    const clSubscription=cl.authenticated&&cl.auth_method==='claude.ai'&&!!cl.subscription_type;
    $('conn-codex-status').textContent=cxSubscription?`Subscription connected · ${cx.auth_method} · ${cx.binary||''}`:'Requires `codex login` with a ChatGPT subscription';
    $('conn-claude-status').textContent=clSubscription?`Subscription connected · ${cl.subscription_type} · ${cl.binary||''}`:'Requires `claude auth login` with a claude.ai subscription';
    $('conn-paperclip-status').textContent=pc.ok?`Connected · ${pc.binary||''}`:'Not authenticated or CLI not found';
    document.querySelectorAll('select[id$="-provider"]').forEach(select=>{
      [...select.options].forEach(option=>{const subscriptionOk=option.value==='codex'?cxSubscription:option.value==='claude'?clSubscription:true;option.disabled=configured[option.value]===false||!subscriptionOk;});
      if(select.selectedOptions[0]&&select.selectedOptions[0].disabled){const first=[...select.options].find(option=>!option.disabled);if(first)select.value=first.value;}
    });
    ['chat','doctor','local','merge'].forEach(modelOptions);
    return status;
  }

  document.querySelectorAll('[data-wf-close]').forEach(button=>button.onclick=()=>close(button.dataset.wfClose));
  document.querySelectorAll('.wf-modal').forEach(modal=>modal.addEventListener('click',event=>{if(event.target===modal)close(modal.id);}));
  window.addEventListener('keydown',event=>{if(event.key==='Escape')document.querySelectorAll('.wf-modal:not([hidden])').forEach(modal=>close(modal.id));});

  $('connectionsBtn').onclick=async()=>{open('connections-modal');try{await loadConnections();}catch(error){showToast('Connection check failed: '+errorText(error),'err');}};
  $('connections-refresh').onclick=async()=>{try{await loadConnections();showToast('Connection status refreshed.');}catch(error){showToast(errorText(error),'err');}};
  $('connections-form').onsubmit=async event=>{
    event.preventDefault();
    try{
      status=await tauriInvoke('save_local_connections',{connections:{codex:$('conn-codex').checked,claude:$('conn-claude').checked,paperclip:$('conn-paperclip').checked}});
      catalog=status.models||catalog;showToast('Local connections saved.');close('connections-modal');
    }catch(error){showToast('Could not save connections: '+errorText(error),'err');}
  };

  function renderChat(baseId){
    const host=$('chat-messages'),messages=chats.get(baseId)||[];host.innerHTML='';
    if(!messages.length){host.innerHTML='<div class="wf-empty">Ask about mechanisms, evidence, contradictions, provenance, populations, or quantitative findings in the selected KB.</div>';return;}
    messages.forEach(message=>{const el=document.createElement('div');el.className='wf-message '+message.role;el.textContent=message.content;if(message.meta){const small=document.createElement('small');small.textContent=message.meta;el.appendChild(small);}host.appendChild(el);});
    host.scrollTop=host.scrollHeight;
  }
  function selectChatBase(baseId){
    chatBaseId=baseId;const base=BASES.find(item=>item.id===baseId);$('kb-chat-base').textContent=base?`${base.name||base.id} · ${base.node_count||0} nodes · ${base.edge_count||0} edges`:baseId;renderChat(baseId);
  }
  $('kbChatBtn').onclick=async()=>{
    if(!activeBaseId){showToast('Select a knowledge base first.','err');return;}
    selectChatBase(activeBaseId);open('kb-chat-modal');try{await loadConnections();}catch(error){showToast(errorText(error),'err');}
    $('chat-question').focus();
  };
  $('chat-form').onsubmit=async event=>{
    event.preventDefault();if(!chatBaseId)return;
    const question=$('chat-question').value.trim();if(!question)return;
    const baseId=chatBaseId,messages=chats.get(baseId)||[];messages.push({role:'user',content:question});chats.set(baseId,messages);$('chat-question').value='';renderChat(baseId);
    const button=$('chat-send');setBusy(button,true,'Send');
    const pending=document.createElement('div');pending.className='wf-message assistant';pending.textContent='Retrieving relevant nodes and asking '+$('chat-provider').value+'…';$('chat-messages').appendChild(pending);
    try{
      const output=await tauriInvoke('chat_with_knowledge_base',{request:{baseId,provider:$('chat-provider').value,model:selectedModel('chat'),question,history:messages.slice(-8)}});
      messages.push({role:'assistant',content:output.answer,meta:`${output.provider} · ${output.model} · ${output.contextNodes} retrieved nodes`});renderChat(baseId);
    }catch(error){messages.push({role:'assistant',content:'Chat failed: '+errorText(error),meta:'error'});renderChat(baseId);}
    finally{setBusy(button,false,'Send');}
  };

  function renderDoctor(baseId){
    const host=$('doctor-messages'),messages=doctorChats.get(baseId)||[];host.innerHTML='';
    if(!messages.length){host.innerHTML='<div class="wf-empty">Ask Doctor to recheck an edge against its papers, correct provenance, split a conflated node, or merge genuine duplicates.</div>';return;}
    messages.forEach(message=>{const el=document.createElement('div');el.className='wf-message '+message.role+(message.audit?' audit':'');el.textContent=message.content;if(message.meta){const small=document.createElement('small');small.textContent=message.meta;el.appendChild(small);}host.appendChild(el);});host.scrollTop=host.scrollHeight;
  }
  function selectDoctorBase(baseId){
    doctorBaseId=baseId;const base=BASES.find(item=>item.id===baseId);$('doctor-base').textContent=base?`${base.name||base.id} · ${base.node_count||0} nodes · ${base.edge_count||0} edges`:baseId;renderDoctor(baseId);
  }
  $('kbDoctorBtn').onclick=async()=>{
    if(!activeBaseId){showToast('Select a knowledge base first.','err');return;}
    selectDoctorBase(activeBaseId);open('doctor-modal');try{await loadConnections();}catch(error){showToast(errorText(error),'err');}$('doctor-instruction').focus();
  };
  $('doctor-form').onsubmit=async event=>{
    event.preventDefault();if(!doctorBaseId||workflowBusy)return;
    const instruction=$('doctor-instruction').value.trim();if(!instruction)return;
    const baseId=doctorBaseId,messages=doctorChats.get(baseId)||[];messages.push({role:'user',content:instruction});doctorChats.set(baseId,messages);$('doctor-instruction').value='';renderDoctor(baseId);
    const button=$('doctor-send');workflowBusy=true;setBusy(button,true,'Review & apply');doctorProgressEl=document.createElement('div');doctorProgressEl.className='wf-message assistant';doctorProgressEl.textContent='Creating an isolated revision candidate…';$('doctor-messages').appendChild(doctorProgressEl);$('doctor-messages').scrollTop=$('doctor-messages').scrollHeight;
    try{
      const output=await tauriInvoke('doctor_knowledge_base',{request:{baseId,provider:$('doctor-provider').value,model:selectedModel('doctor'),instruction,history:messages.slice(0,-1).slice(-6)}});
      const evidence=(output.evidence_checked||[]).join(', ')||'No evidence identifiers reported';const changed=(output.changed_files||[]).join(', ');const unresolved=(output.unresolved||[]).join('; ');
      let content=output.summary||'Verified revision applied.';if(output.rationale)content+='\n\nRationale: '+output.rationale;content+='\n\nEvidence checked: '+evidence;content+='\nChanged files: '+changed;if(unresolved)content+='\nUnresolved: '+unresolved;
      messages.push({role:'assistant',audit:true,content,meta:`Verified · committed · ${output.agent&&output.agent.provider||$('doctor-provider').value} · reversible in Log`});renderDoctor(baseId);
      resetCaches();BASES=await loadBases();renderSidebar();lastBasesSig=basesSig(BASES);const refreshed=BASES.find(base=>base.id===baseId);if(refreshed)await selectBase(refreshed);selectDoctorBase(baseId);showToast('Doctor revision verified, committed, and applied.');
    }catch(error){messages.push({role:'assistant',content:'Doctor stopped without applying changes: '+errorText(error),meta:'KB left unchanged'});renderDoctor(baseId);showToast(errorText(error),'err');}
    finally{doctorProgressEl=null;workflowBusy=false;setBusy(button,false,'Review & apply');}
  };

  $('localKbBtn').onclick=async()=>{open('local-kb-modal');$('local-progress').hidden=true;try{await loadConnections();}catch(error){showToast(errorText(error),'err');}};
  $('local-browse').onclick=async()=>{
    const dialog=window.__TAURI__&&window.__TAURI__.dialog;if(!dialog||!dialog.open){showToast('Folder picker unavailable.','err');return;}
    const path=await dialog.open({directory:true,multiple:false,title:'Select a folder containing local papers'});if(path)$('local-source').value=Array.isArray(path)?path[0]:path;
  };
  $('local-kb-form').onsubmit=async event=>{
    event.preventDefault();const button=$('local-submit'),panel=$('local-progress'),log=$('local-progress-log');panel.hidden=false;log.textContent='';workflowBusy=true;setBusy(button,true,'Create knowledge base');
    try{
      const output=await tauriInvoke('create_local_knowledge_base',{request:{sourcePath:$('local-source').value,kbName:$('local-name').value.trim(),provider:$('local-provider').value,model:selectedModel('local'),maxFiles:Number($('local-max').value)||25}});
      await openGenerated(output);showToast('Local knowledge base generated, verified, and opened.');workflowBusy=false;close('local-kb-modal');
    }catch(error){$('local-progress-title').textContent='Creation stopped';log.textContent+='\n'+errorText(error);showToast(errorText(error),'err');}
    finally{workflowBusy=false;setBusy(button,false,'Create knowledge base');}
  };

  function renderMergeBases(){
    const host=$('merge-base-picker');host.innerHTML='';
    BASES.forEach((base,index)=>{const label=document.createElement('label');label.className='wf-base-option';label.innerHTML=`<input type="checkbox" value="${esc(base.id)}"><span><strong>${esc(base.name||base.id)}</strong><em>${base.node_count||0} nodes · ${base.edge_count||0} edges${index===0?' · canonical':''}</em></span>`;host.appendChild(label);});
  }
  $('mergeKbBtn').onclick=async()=>{if(BASES.length<2){showToast('Load at least two knowledge bases before merging.','err');return;}renderMergeBases();$('merge-progress').hidden=true;open('merge-kb-modal');try{await loadConnections();}catch(error){showToast(errorText(error),'err');}};
  $('merge-kb-form').onsubmit=async event=>{
    event.preventDefault();const ids=[...document.querySelectorAll('#merge-base-picker input:checked')].map(input=>input.value);if(ids.length<2){showToast('Select at least two knowledge bases.','err');return;}
    const button=$('merge-submit'),panel=$('merge-progress'),log=$('merge-progress-log');panel.hidden=false;log.textContent='';workflowBusy=true;setBusy(button,true,'Merge selected bases');
    try{
      const output=await tauriInvoke('merge_knowledge_bases',{request:{baseIds:ids,kbName:$('merge-name').value.trim(),provider:$('merge-provider').value,model:selectedModel('merge')}});
      await openGenerated(output);showToast('Merged knowledge base verified and opened.');workflowBusy=false;close('merge-kb-modal');
    }catch(error){$('merge-progress-title').textContent='Merge stopped';log.textContent+='\n'+errorText(error);showToast(errorText(error),'err');}
    finally{workflowBusy=false;setBusy(button,false,'Merge selected bases');}
  };

  ['chat','doctor','local','merge'].forEach(bindProvider);
  if(window.__TAURI__&&window.__TAURI__.event){window.__TAURI__.event.listen('biookf-agent-progress',event=>{
    const message=event.payload&&event.payload.message||'Working';
    if(!$('doctor-modal').hidden&&doctorProgressEl){doctorProgressEl.textContent=message.replace(/^\[[^\]]+\]\s*/, '');return;}
    const target=!$('local-kb-modal').hidden?'local':!$('merge-kb-modal').hidden?'merge':null;if(!target)return;
    $(target+'-progress').hidden=false;$(target+'-progress-title').textContent=message.replace(/^\[[^\]]+\]\s*/, '');const log=$(target+'-progress-log');log.textContent+=(log.textContent?'\n':'')+'• '+message;log.scrollTop=log.scrollHeight;
  }).catch(()=>{});}
  setInterval(()=>{if(!$('kb-chat-modal').hidden&&activeBaseId&&activeBaseId!==chatBaseId)selectChatBase(activeBaseId);},500);
  setInterval(()=>{if(!$('doctor-modal').hidden&&activeBaseId&&activeBaseId!==doctorBaseId)selectDoctorBase(activeBaseId);},500);
  loadConnections().catch(()=>{});
})();
