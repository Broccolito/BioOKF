/* Deterministic BioOKF topology metrics and dependency-free SVG plots. */
(function(){
  'use strict';
  const $=id=>document.getElementById(id),modal=$('network-metrics-modal');
  if(!modal) return;
  let report=null,reportBase=null,busy=false;
  const fmt=(value,digits=3)=>value==null||!Number.isFinite(Number(value))?'—':Number(value).toLocaleString(undefined,{maximumFractionDigits:digits});
  const pct=value=>value==null?'—':fmt(Number(value)*100,1)+'%';
  const xml=value=>String(value==null?'':value).replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));

  function activeBase(){return (window.BASES||BASES||[]).find(base=>base.id===(window.activeBaseId||activeBaseId));}
  function open(){const base=activeBase();if(!base){showToast('Select a knowledge base first.','err');return;}$('metrics-base').textContent=`${base.name||base.id} · ${base.node_count||0} nodes · ${base.edge_count||0} displayed graph edges`;modal.hidden=false;if(reportBase!==base.id){report=null;$('metrics-export').disabled=true;}reportBase=base.id;if(!report)calculate();}
  function close(){if(!busy)modal.hidden=true;}

  function linePlot(points,xKey,yKey,{logX=false,logY=false,xLabel='',yLabel=''}={}){
    const clean=points.map(point=>({x:Number(point[xKey]),y:Number(point[yKey])})).filter(point=>Number.isFinite(point.x)&&Number.isFinite(point.y)&&(!logX||point.x>0)&&(!logY||point.y>0));
    if(!clean.length)return '<div class="metrics-empty">Insufficient non-zero values for this plot.</div>';
    const tx=value=>logX?Math.log10(value):value,ty=value=>logY?Math.log10(value):value;
    const xs=clean.map(point=>tx(point.x)),ys=clean.map(point=>ty(point.y));let xmin=Math.min(...xs),xmax=Math.max(...xs),ymin=Math.min(...ys),ymax=Math.max(...ys);if(xmin===xmax){xmin-=.5;xmax+=.5}if(ymin===ymax){ymin-=.5;ymax+=.5}
    const W=520,H=230,L=52,R=14,T=12,B=38,sx=value=>L+(tx(value)-xmin)/(xmax-xmin)*(W-L-R),sy=value=>T+(1-(ty(value)-ymin)/(ymax-ymin))*(H-T-B);
    const path=clean.sort((a,b)=>a.x-b.x).map((point,index)=>(index?'L':'M')+sx(point.x).toFixed(1)+' '+sy(point.y).toFixed(1)).join(' ');
    const dots=clean.map(point=>`<circle class="metric-point" cx="${sx(point.x).toFixed(1)}" cy="${sy(point.y).toFixed(1)}" r="2.4"/>`).join('');
    let grid='';for(let i=0;i<=4;i++){const x=L+i*(W-L-R)/4,y=T+i*(H-T-B)/4;grid+=`<line class="metric-grid" x1="${x}" y1="${T}" x2="${x}" y2="${H-B}"/><line class="metric-grid" x1="${L}" y1="${y}" x2="${W-R}" y2="${y}"/>`;}
    return `<svg viewBox="0 0 ${W} ${H}" role="img"><g>${grid}</g><line class="metric-axis" x1="${L}" y1="${H-B}" x2="${W-R}" y2="${H-B}"/><line class="metric-axis" x1="${L}" y1="${T}" x2="${L}" y2="${H-B}"/><path class="metric-line" d="${path}"/>${dots}<text class="metric-label" x="${L}" y="${H-20}">${xml(logX?'log '+xLabel:xLabel)} · ${fmt(clean[0].x)}</text><text class="metric-label" text-anchor="end" x="${W-R}" y="${H-20}">${fmt(clean[clean.length-1].x)}</text><text class="metric-label" x="6" y="${T+7}">${xml(logY?'log '+yLabel:yLabel)} · ${fmt(Math.max(...clean.map(p=>p.y)))}</text></svg>`;
  }
  function bars(values){
    if(!values.length)return '<div class="metrics-empty">No Leiden communities.</div>';const W=520,H=230,L=42,R=12,T=12,B=36,max=Math.max(...values,1),bar=(W-L-R)/values.length;
    const shapes=values.map((value,index)=>{const height=value/max*(H-T-B);return `<rect class="metric-bar" x="${L+index*bar+2}" y="${H-B-height}" width="${Math.max(2,bar-4)}" height="${height}"/><text class="metric-label" text-anchor="middle" x="${L+(index+.5)*bar}" y="${H-20}">${index+1}</text>`;}).join('');return `<svg viewBox="0 0 ${W} ${H}"><line class="metric-axis" x1="${L}" y1="${H-B}" x2="${W-R}" y2="${H-B}"/>${shapes}<text class="metric-label" x="6" y="${T+7}">nodes · ${max}</text></svg>`;
  }
  function sourceYearPlot(points,unknown){
    if(!points.length)return `<div class="metrics-empty">No explicit source years found. ${unknown||0} source${unknown===1?' is':'s are'} undated.</div>`;
    const W=520,H=230,L=45,R=38,T=16,B=42,maxCount=Math.max(...points.map(point=>point.sources),1),maxCumulative=Math.max(...points.map(point=>point.cumulative_sources),1),slot=(W-L-R)/points.length;
    const x=index=>L+(index+.5)*slot,yCount=value=>T+(1-value/maxCount)*(H-T-B),yCumulative=value=>T+(1-value/maxCumulative)*(H-T-B);
    const bars=points.map((point,index)=>{const y=yCount(point.sources);return `<rect class="metric-bar" x="${x(index)-Math.min(16,slot*.35)}" y="${y}" width="${Math.min(32,slot*.7)}" height="${H-B-y}"/><text class="metric-label" text-anchor="middle" x="${x(index)}" y="${H-23}">${point.year}</text>`;}).join('');
    const cumulative=points.map((point,index)=>(index?'L':'M')+x(index).toFixed(1)+' '+yCumulative(point.cumulative_sources).toFixed(1)).join(' ');
    return `<svg viewBox="0 0 ${W} ${H}" role="img"><line class="metric-axis" x1="${L}" y1="${H-B}" x2="${W-R}" y2="${H-B}"/>${bars}<path class="metric-line metric-line--cumulative" d="${cumulative}"/>${points.map((point,index)=>`<circle class="metric-point" cx="${x(index)}" cy="${yCumulative(point.cumulative_sources)}" r="2.7"/>`).join('')}<text class="metric-label" x="5" y="${T+6}">per year · ${maxCount}</text><text class="metric-label" text-anchor="end" x="${W-4}" y="${T+6}">cumulative · ${maxCumulative}</text><text class="metric-label" x="${L}" y="${H-7}">Undated: ${unknown||0}</text></svg>`;
  }
  function betweennessCcdf(nodes){
    const values=nodes.map(node=>Number(node.betweenness)).filter(value=>value>0).sort((a,b)=>a-b);const unique=[...new Set(values)];return unique.map(value=>({value,probability:values.filter(item=>item>=value).length/values.length}));
  }
  function card(label,value,note){return `<div class="metric-card"><em>${xml(label)}</em><strong>${xml(value)}</strong><small>${xml(note)}</small></div>`;}
  function render(value){
    const g=value.global,nodes=value.nodes||[],between=betweennessCcdf(nodes);
    const cards=[card('Density',fmt(g.density,4),`mean degree ${fmt(g.average_degree)}`),card('Transitivity',fmt(g.transitivity), 'global triangle closure'),card('Distance scale',fmt(g.average_shortest_path_giant),`giant ASP · efficiency ${fmt(g.global_efficiency)}`),card('Leiden modularity Q',fmt(g.modularity_leiden),`${g.communities} communities · γ=1`),card('Degree assortativity r',fmt(g.degree_assortativity),'hub mixing pattern'),card('Giant component',pct(g.giant_component_fraction),`${g.components} connected components`),card('Algebraic connectivity',fmt(g.algebraic_connectivity,5),'λ₂ of the combinatorial Laplacian'),card('Analyzed projection',`${g.nodes} / ${g.edges}`, 'nodes / unique edges'),card('Dated sources',`${value.sources_with_year||0} / ${value.source_count||0}`,`${value.sources_without_year||0} unknown year`)].join('');
    const rows=nodes.slice(0,100).map(node=>`<tr><td>${xml(node.id)}</td><td>${xml(node.node_type)}</td><td>${node.degree}</td><td>${fmt(node.betweenness,5)}</td><td>${fmt(node.clustering)}</td><td>${node.coreness}</td><td>${node.community==null?'—':node.community+1}</td><td>${fmt(node.participation)}</td></tr>`).join('');
    $('metrics-body').innerHTML=`<div class="metrics-summary">${cards}</div><section class="metrics-section"><div class="metrics-section-head"><h3>Evidence timeline</h3><span>Explicit source metadata only; unknown years remain visible</span></div><div class="metrics-plots metrics-plots--timeline"><article class="metric-plot"><h4>Sources by year</h4><p>Bars show annual sources; the line shows cumulative evidence growth.</p>${sourceYearPlot(value.source_years||[],value.sources_without_year||0)}</article></div></section><section class="metrics-section"><div class="metrics-section-head"><h3>Node-level distributions</h3><span>Descriptive topology; no unsupported power-law inference</span></div><div class="metrics-plots"><article class="metric-plot"><h4>Degree CCDF P(K ≥ k)</h4><p>Log–log representation; preferable to a degree histogram.</p>${linePlot(value.degree_ccdf||[],'degree','probability',{logX:true,logY:true,xLabel:'degree k',yLabel:'CCDF'})}</article><article class="metric-plot"><h4>Betweenness CCDF</h4><p>Highlights low-degree bridge nodes that degree alone misses.</p>${linePlot(between,'value','probability',{logX:true,logY:true,xLabel:'betweenness',yLabel:'CCDF'})}</article><article class="metric-plot"><h4>Local clustering C(k)</h4><p>Mean local clustering conditional on degree; inspect for hierarchical decay.</p>${linePlot(value.clustering_by_degree||[],'degree','mean_clustering',{logX:true,logY:false,xLabel:'degree k',yLabel:'C(k)'})}</article><article class="metric-plot"><h4>Leiden community sizes</h4><p>Node counts in deterministic Leiden communities, descending.</p>${bars(value.community_sizes||[])}</article></div></section><section class="metrics-section"><div class="metrics-section-head"><h3>Structurally important nodes</h3><span>Top 100 ordered by betweenness, then degree</span></div><div class="metrics-table-wrap"><table class="metrics-table"><thead><tr><th>Node</th><th>Type</th><th>Degree</th><th>Betweenness</th><th>Clustering</th><th>k-core</th><th>Community</th><th>Participation</th></tr></thead><tbody>${rows}</tbody></table></div></section><div class="metrics-notes"><strong>Method</strong><ul>${(value.notes||[]).map(note=>`<li>${xml(note)}</li>`).join('')}</ul><div>${xml(value.projection)}</div></div>`;
  }
  async function calculate(){
    const base=activeBase();if(!base)return;busy=true;$('metrics-calculate').disabled=true;$('metrics-calculate').textContent='Calculating…';$('metrics-body').innerHTML='<div class="metrics-empty">Computing shortest paths, betweenness and Leiden communities…</div>';
    try{report=await tauriInvoke('network_metrics',{id:base.id,excludeProvenance:$('metrics-exclude-provenance').checked});reportBase=base.id;render(report);$('metrics-export').disabled=false;$('metrics-base').textContent=`${base.name||base.id} · ${report.global.nodes} analyzed nodes · ${report.global.edges} unique edges`;}
    catch(error){$('metrics-body').innerHTML=`<div class="metrics-empty">Metrics failed: ${xml((error&&error.message)||error)}</div>`;showToast('Network metrics failed','err');}
    finally{busy=false;$('metrics-calculate').disabled=false;$('metrics-calculate').textContent='Calculate';}
  }
  async function exportJson(){
    if(!report)return;const dialog=window.__TAURI__&&window.__TAURI__.dialog;if(!dialog||!dialog.save)return;const base=activeBase(),path=await dialog.save({title:'Export network metrics',defaultPath:(base&&base.id||'biookf')+'-network-metrics.json',filters:[{name:'JSON',extensions:['json']}]});if(!path)return;try{const saved=await tauriInvoke('write_network_metrics_json',{path,content:JSON.stringify(report,null,2)});showToast('Metrics exported to '+saved);}catch(error){showToast('Export failed: '+String(error),'err');}
  }
  $('networkMetricsBtn').onclick=open;$('metrics-close').onclick=close;$('metrics-calculate').onclick=calculate;$('metrics-export').onclick=exportJson;
  $('metrics-exclude-provenance').onchange=()=>{report=null;$('metrics-export').disabled=true;};
  window.addEventListener('keydown',event=>{if(event.key==='Escape'&&!modal.hidden)close();});
  setInterval(()=>{const base=activeBase();if(!modal.hidden&&base&&base.id!==reportBase){report=null;reportBase=base.id;$('metrics-base').textContent=base.name||base.id;$('metrics-export').disabled=true;$('metrics-body').innerHTML='<div class="metrics-empty">The selected knowledge base changed. Calculate its metrics.</div>'; }},500);
})();
