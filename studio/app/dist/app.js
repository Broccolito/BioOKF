/* BioOKF Studio frontend — data-driven from okf-core.
   In the Tauri app it calls window.__TAURI__ invoke(); in a browser (and tests)
   it fetches the JSON the `okf export` CLI emits. Visualization is identical. */

const TYPE_COLOR = {
  Gene:"#6366A8", Variant:"#8A86C4", SequenceFeature:"#AAA6DA", Structure:"#7C8FC9",
  Molecule:"#2E8C84", MolecularClass:"#5FB0A8", BiologicalPathway:"#4FA38C", BiologicalFunction:"#7CC3B0",
  Anatomy:"#3F9E6E", CellType:"#62B889", Organism:"#8FCBA6",
  Disease:"#C45B6B", Phenotype:"#D98AA0", BiomedicalMeasure:"#D98C5A", MethodOrProcedure:"#C99750",
  Exposure:"#B79A52", SocialFactor:"#C2A86A", Food:"#CBB87E",
  Device:"#6E87A3", MaterialSample:"#92A6BC",
  Publication:"#6B7280", Study:"#7A828E", Dataset:"#88909C", Agent:"#5E6672",
  Population:"#7E8896", GeographicLocation:"#94A0A0", Concept:"#9AA0A8", Other:"#AEB2B8"
};
const FAMILIES = [
  ["Genomic", ["Gene","Variant","SequenceFeature","Structure"]],
  ["Molecular & process", ["Molecule","MolecularClass","BiologicalPathway","BiologicalFunction"]],
  ["Anatomy & organism", ["Anatomy","CellType","Organism"]],
  ["Clinical", ["Disease","Phenotype","BiomedicalMeasure","MethodOrProcedure"]],
  ["Exposome", ["Exposure","SocialFactor","Food"]],
  ["Physical", ["Device","MaterialSample"]],
  ["Provenance & context", ["Publication","Study","Dataset","Agent","Population","GeographicLocation","Concept","Other"]]
];

const cv=document.getElementById('graph'), ctx=cv.getContext('2d');
let DPR=Math.max(1,window.devicePixelRatio||1), W=0,H=0;
let view={x:0,y:0,k:1};
let nodes=[], edges=[], byId={}, pages={};
let hover=null, hoverEdge=null, selected=null, selectedEdge=null;
let drag=null, panning=null, moved=false, alpha=1, searchTerm='';
let focusNeighbors=new Set();
let BASES=[], activeBaseId=null, currentLog='', currentUpdated=null;

/* ---------- data loading ---------- */
const inTauri = !!(window.__TAURI__ && window.__TAURI__.core);
async function invoke(cmd, args){ return window.__TAURI__.core.invoke(cmd, args); }
const cb = () => '?_=' + Date.now(); // dev cache-bust for the static JSON
async function loadBases(){
  if(inTauri){ try { return await invoke('list_bases'); } catch(e){ console.error(e); return []; } }
  return await (await fetch('data/bases.json'+cb())).json();
}
async function loadBundle(base){
  if(inTauri){ return await invoke('get_bundle', { id: base.id }); }
  return await (await fetch(base.file+cb())).json();
}

function loadGraph(g){
  byId={};
  nodes = g.nodes.map((n,i)=>{
    const a=(i/g.nodes.length)*Math.PI*2, r=170+(i%6)*18;
    const node=Object.assign({}, n, {x:Math.cos(a)*r, y:Math.sin(a)*r, vx:0, vy:0});
    byId[n.id]=node; return node;
  });
  edges = g.edges.map(e=>Object.assign({}, e));
  // hub = top-6 by degree (real nodes)
  const ranked=[...nodes].filter(n=>!n.external).sort((a,b)=>b.degree-a.degree);
  const hubSet=new Set(ranked.slice(0,6).map(n=>n.id));
  nodes.forEach(n=>n.hub=hubSet.has(n.id));
  selected=null;selectedEdge=null;hover=null;hoverEdge=null;focusNeighbors=new Set();closeDetail();
  alpha=1;
  for(let i=0;i<300;i++) tick(0.9*Math.pow(0.985,i)+0.02);
  fitView();
}

/* ---------- force sim ---------- */
function tick(a){
  const M=nodes.length;
  for(let i=0;i<M;i++){const ni=nodes[i];
    for(let j=i+1;j<M;j++){const nj=nodes[j];
      let dx=ni.x-nj.x,dy=ni.y-nj.y,d2=dx*dx+dy*dy;
      if(d2<0.01){d2=0.01;dx=Math.random()-0.5;dy=Math.random()-0.5;}
      const d=Math.sqrt(d2),rep=4200/d2,fx=(dx/d)*rep,fy=(dy/d)*rep;
      ni.vx+=fx*a;ni.vy+=fy*a;nj.vx-=fx*a;nj.vy-=fy*a;}}
  const L=92;
  edges.forEach(e=>{const s=byId[e.source],t=byId[e.target]; if(!s||!t)return;
    let dx=t.x-s.x,dy=t.y-s.y,d=Math.sqrt(dx*dx+dy*dy)||0.01;
    const f=(d-L)*0.045*a,fx=(dx/d)*f,fy=(dy/d)*f;
    s.vx+=fx;s.vy+=fy;t.vx-=fx;t.vy-=fy;});
  nodes.forEach(n=>{n.vx+=-n.x*0.012*a;n.vy+=-n.y*0.012*a;});
  nodes.forEach(n=>{if(n===drag){n.vx=0;n.vy=0;return;}n.x+=n.vx;n.y+=n.vy;n.vx*=0.82;n.vy*=0.82;});
}
function toScreen(x,y){return [(x*view.k)+view.x+W/2,(y*view.k)+view.y+H/2];}
function toWorld(sx,sy){return [(sx-W/2-view.x)/view.k,(sy-H/2-view.y)/view.k];}
function nodeR(n){return n.external?5:(n.hub?10:6);}
function fitView(){
  if(!nodes.length)return;
  let mnx=1e9,mny=1e9,mxx=-1e9,mxy=-1e9;
  nodes.forEach(n=>{mnx=Math.min(mnx,n.x);mny=Math.min(mny,n.y);mxx=Math.max(mxx,n.x);mxy=Math.max(mxy,n.y);});
  const pad=72,gw=mxx-mnx||1,gh=mxy-mny||1,k=Math.min((W-pad*2)/gw,(H-pad*2)/gh,1.6);
  view.k=k;view.x=-((mnx+mxx)/2)*k;view.y=-((mny+mxy)/2)*k;
}
function neighborsOf(id){const s=new Set();edges.forEach(e=>{if(e.source===id)s.add(e.target);if(e.target===id)s.add(e.source);});return s;}
function recomputeFocus(){const id=(selected&&selected.id)||(hover&&hover.id)||null;focusNeighbors=id?neighborsOf(id):new Set();}
function matches(n){const q=searchTerm;if(!q)return true;return (n.id||'').toLowerCase().includes(q)||(n.type||'').toLowerCase().includes(q)||(n.subtype||'').toLowerCase().includes(q);}

/* ---------- render ---------- */
function draw(){
  ctx.setTransform(DPR,0,0,DPR,0,0);ctx.clearRect(0,0,W,H);drawGrid();
  const focusId=(selected&&selected.id)||(hover&&hover.id)||null;
  const focusEdge=selectedEdge||hoverEdge;
  edges.forEach(e=>{
    const s=byId[e.source],t=byId[e.target]; if(!s||!t)return;
    let emph=0,dim=false;
    if(focusEdge===e)emph=2; else if(focusId&&(e.source===focusId||e.target===focusId))emph=1;
    if(focusId&&!(e.source===focusId||e.target===focusId))dim=true;
    if(focusEdge&&focusEdge!==e)dim=true;
    if(searchTerm){dim=!(matches(s)&&matches(t));emph=0;}
    drawEdge(s,t,e,emph,dim);
  });
  nodes.forEach(n=>{
    const isFocus=focusId===n.id, isNb=focusNeighbors.has(n.id);
    let a=1; if(searchTerm)a=matches(n)?1:0.12; else if(focusId&&!isFocus&&!isNb)a=0.26;
    drawNodeCircle(n,a,isFocus);
  });
  drawLabels(focusId);
}
function drawGrid(){
  const step=34*view.k; if(step<11)return;
  const ox=((view.x+W/2)%step+step)%step, oy=((view.y+H/2)%step+step)%step;
  ctx.fillStyle="rgba(20,24,31,0.045)";
  for(let x=ox;x<W;x+=step)for(let y=oy;y<H;y+=step){ctx.beginPath();ctx.arc(x,y,0.8,0,7);ctx.fill();}
}
function drawEdge(s,t,e,emph,dim){
  const [x1,y1]=toScreen(s.x,s.y),[x2,y2]=toScreen(t.x,t.y);
  let dx=x2-x1,dy=y2-y1,len=Math.hypot(dx,dy)||1;
  const ux=dx/len,uy=dy/len,px=-uy,py=ux;
  const rs=nodeR(s)*view.k+1.5,rt=nodeR(t)*view.k+1.5;
  const sx=x1+ux*rs,sy=y1+uy*rs,ex=x2-ux*rt,ey=y2-uy*rt;
  if(e.synthesized){ // provenance edge — faint dashed, no taper
    ctx.save();ctx.setLineDash([2,3]);
    ctx.strokeStyle=dim?"rgba(28,33,40,0.05)":"rgba(28,33,40,0.13)";ctx.lineWidth=0.8;
    ctx.beginPath();ctx.moveTo(sx,sy);ctx.lineTo(ex,ey);ctx.stroke();ctx.restore();
    maybeLabel(e,emph,sx,sy,ex,ey,len);return;
  }
  let col="rgba(28,33,40,0.18)";
  if(emph===1)col="rgba(28,33,40,0.32)"; if(emph===2)col="rgba(28,33,40,0.46)"; if(dim)col="rgba(28,33,40,0.07)";
  if(e.symmetric){
    ctx.strokeStyle=col;ctx.lineWidth=0.9;ctx.beginPath();ctx.moveTo(sx,sy);ctx.lineTo(ex,ey);ctx.stroke();
  }else{
    const w0=0.85,w1=0.42; // subtle taper: source slightly thicker than object end
    ctx.fillStyle=col;ctx.beginPath();
    ctx.moveTo(sx+px*w0,sy+py*w0);ctx.lineTo(ex+px*w1,ey+py*w1);ctx.lineTo(ex-px*w1,ey-py*w1);ctx.lineTo(sx-px*w0,sy-py*w0);
    ctx.closePath();ctx.fill();
  }
  maybeLabel(e,emph,sx,sy,ex,ey,len);
}
function maybeLabel(e,emph,sx,sy,ex,ey,len){
  if(emph===2&&len>26){
    const mx=(sx+ex)/2,my=(sy+ey)/2;
    ctx.save();ctx.font="500 11px ui-monospace,Menlo,monospace";ctx.textAlign="center";ctx.textBaseline="middle";
    ctx.shadowColor="rgba(250,250,252,0.95)";ctx.shadowBlur=4;ctx.fillStyle="#41474f";
    ctx.fillText(e.predicate,mx,my);ctx.fillText(e.predicate,mx,my);ctx.restore();ctx.textAlign="start";
  }
}
function drawNodeCircle(n,a,isFocus){
  const [x,y]=toScreen(n.x,n.y),r=nodeR(n)*view.k,col=n.color||TYPE_COLOR[n.type]||"#9aa1ab";
  ctx.globalAlpha=a;
  if(isFocus){
    const g=ctx.createRadialGradient(x,y,r,x,y,r+13);
    g.addColorStop(0,hexA(col,0.34));g.addColorStop(0.55,hexA(col,0.14));g.addColorStop(1,hexA(col,0));
    ctx.beginPath();ctx.arc(x,y,r+13,0,7);ctx.fillStyle=g;ctx.fill();
  }
  ctx.beginPath();ctx.arc(x,y,r,0,7);ctx.fillStyle=col;ctx.fill();
  if(n.external){ctx.setLineDash([2,2]);ctx.strokeStyle="rgba(18,21,26,0.5)";ctx.lineWidth=1;ctx.stroke();ctx.setLineDash([]);}
  else{ctx.lineWidth=1.1;ctx.strokeStyle="rgba(18,21,26,0.92)";ctx.stroke();}
  ctx.globalAlpha=1;
}
function drawLabels(focusId){
  const cands=[];
  nodes.forEach(n=>{
    const isFocus=focusId===n.id, isNb=focusNeighbors.has(n.id), isHover=hover===n;
    let pr=-1,a=1;
    if(searchTerm){ if(matches(n))pr=2; else return; }
    else{ if(isFocus)pr=5;else if(isHover)pr=4;else if(n.hub)pr=3;else if(isNb)pr=2;else if(view.k>=1.55)pr=1; if(focusId&&!isFocus&&!isNb)a=0.3; }
    if(pr<0)return; cands.push({n,pr,a});
  });
  cands.sort((p,q)=>q.pr-p.pr);
  const placed=[];ctx.textBaseline="middle";
  cands.forEach(({n,pr,a})=>{
    const [x,y]=toScreen(n.x,n.y),r=nodeR(n)*view.k,fs=n.hub?12:11.5;
    ctx.font=`${pr>=4||n.hub?600:450} ${fs}px -apple-system,system-ui,sans-serif`;
    const label=(n.label||n.id).length>32?(n.label||n.id).slice(0,31)+"…":(n.label||n.id);
    const tw=ctx.measureText(label).width,lx=x+r+6,ly=y;
    const rect={x:lx-1,y:ly-fs/2-1,w:tw+2,h:fs+2};
    if(placed.some(p=>!(rect.x>p.x+p.w||rect.x+rect.w<p.x||rect.y>p.y+p.h||rect.y+rect.h<p.y)))return;
    placed.push(rect);
    ctx.save();ctx.globalAlpha=a;ctx.shadowColor="rgba(250,250,252,0.95)";ctx.shadowBlur=4;ctx.fillStyle="#1c2128";
    ctx.fillText(label,lx,ly);ctx.fillText(label,lx,ly);ctx.restore();
  });
}
function hexA(hex,a){const h=(hex||'#999').replace('#','');return `rgba(${parseInt(h.slice(0,2),16)},${parseInt(h.slice(2,4),16)},${parseInt(h.slice(4,6),16)},${a})`;}
function loop(){
  const needW=Math.round(cv.clientWidth*DPR), needH=Math.round(cv.clientHeight*DPR);
  if(needW>0&&needH>0&&(cv.width!==needW||cv.height!==needH)){W=cv.clientWidth;H=cv.clientHeight;cv.width=needW;cv.height=needH;}
  if(alpha>0.005){tick(alpha);alpha*=0.94;} draw(); requestAnimationFrame(loop);
}

/* ---------- interaction ---------- */
function pickNode(sx,sy){for(let i=nodes.length-1;i>=0;i--){const n=nodes[i],[x,y]=toScreen(n.x,n.y),r=nodeR(n)*view.k+4;if((sx-x)**2+(sy-y)**2<=r*r)return n;}return null;}
function pickEdge(sx,sy){let best=null,bd=6;edges.forEach(e=>{const s=byId[e.source],t=byId[e.target];if(!s||!t)return;const [x1,y1]=toScreen(s.x,s.y),[x2,y2]=toScreen(t.x,t.y),d=distSeg(sx,sy,x1,y1,x2,y2);if(d<bd){bd=d;best=e;}});return best;}
function distSeg(px,py,x1,y1,x2,y2){const dx=x2-x1,dy=y2-y1,l2=dx*dx+dy*dy;if(l2===0)return Math.hypot(px-x1,py-y1);let t=((px-x1)*dx+(py-y1)*dy)/l2;t=Math.max(0,Math.min(1,t));return Math.hypot(px-(x1+t*dx),py-(y1+t*dy));}
const tip=document.getElementById('tip');
window.addEventListener('mousemove',ev=>{
  const rect=cv.getBoundingClientRect(),sx=ev.clientX-rect.left,sy=ev.clientY-rect.top;
  if(drag){const [wx,wy]=toWorld(sx,sy);drag.x=wx;drag.y=wy;moved=true;alpha=Math.max(alpha,0.25);hideTip();return;}
  if(panning){view.x+=sx-panning.x;view.y+=sy-panning.y;panning={x:sx,y:sy};moved=true;hideTip();return;}
  if(sx<0||sy<0||sx>rect.width||sy>rect.height){if(hover||hoverEdge){hover=null;hoverEdge=null;recomputeFocus();}hideTip();return;}
  const n=pickNode(sx,sy);hover=n;hoverEdge=n?null:pickEdge(sx,sy);recomputeFocus();
  cv.style.cursor=(n||hoverEdge)?'pointer':'grab';
  if(n)showTip(sx,sy,n.label||n.id,(n.type||'')+(n.subtype?' · '+n.subtype:''));
  else if(hoverEdge)showTip(sx,sy,hoverEdge.predicate,hoverEdge.source+' → '+hoverEdge.target);
  else hideTip();
});
function showTip(sx,sy,a,b){tip.style.display='block';tip.style.left=(sx+14)+'px';tip.style.top=(sy+14)+'px';tip.innerHTML=`${esc(a)}<br><span class="tp">${esc(b)}</span>`;}
function hideTip(){tip.style.display='none';}
cv.addEventListener('mousedown',ev=>{const rect=cv.getBoundingClientRect(),sx=ev.clientX-rect.left,sy=ev.clientY-rect.top;moved=false;const n=pickNode(sx,sy);if(n)drag=n;else panning={x:sx,y:sy};cv.classList.add('grabbing');});
window.addEventListener('mouseup',ev=>{
  const rect=cv.getBoundingClientRect(),sx=ev.clientX-rect.left,sy=ev.clientY-rect.top;cv.classList.remove('grabbing');
  if(!moved){
    const n=pickNode(sx,sy);
    if(n){selected=n;selectedEdge=null;recomputeFocus();showNodeDetail(n);}
    else{const e=pickEdge(sx,sy);if(e){selectedEdge=e;selected=null;recomputeFocus();showEdgeDetail(e);}else{selected=null;selectedEdge=null;recomputeFocus();closeDetail();}}
  }
  drag=null;panning=null;
});
cv.addEventListener('wheel',ev=>{ev.preventDefault();const rect=cv.getBoundingClientRect(),sx=ev.clientX-rect.left,sy=ev.clientY-rect.top,[wx,wy]=toWorld(sx,sy);const nk=Math.max(0.25,Math.min(5,view.k*Math.exp(-ev.deltaY*0.0014)));view.k=nk;view.x=sx-W/2-wx*view.k;view.y=sy-H/2-wy*view.k;},{passive:false});

/* ---------- detail panels ---------- */
const detail=document.getElementById('detail');
function closeDetail(){detail.classList.remove('open');detail.innerHTML='';}
function nodeColor(id){const n=byId[id];return (n&&n.color)|| (pages[id]&&TYPE_COLOR[typeStr(pages[id].node_type)]) ||"#b6bbc4";}
function typeStr(t){ if(typeof t==='string')return t; if(t&&typeof t==='object')return Object.keys(t)[0]; return 'Other'; }
function outEdges(id){return edges.filter(e=>e.source===id&&!e.synthesized);}
function inEdges(id){return edges.filter(e=>e.target===id&&!e.synthesized);}

function showNodeDetail(n){
  const pg=pages[n.id];
  const col=n.color||TYPE_COLOR[n.type]||"#9aa1ab";
  if(!pg && n.external){
    const inc=inEdges(n.id), out=outEdges(n.id);
    detail.innerHTML=`<div class="d-head"><button class="d-close" id="dClose">×</button>
      <span class="d-badge" style="background:#8a929c">EXTERNAL</span><span class="d-sub">referenced, not yet a concept doc</span>
      <div class="d-id">${esc(n.id)}</div>
      <div class="d-desc">This identifier is referenced by ${inc.length+out.length} edge(s) but has no concept document in this bundle. Create one to enrich the graph.</div></div>
      <div class="d-body">${incomingSection(n.id)}</div>`;
    detail.classList.add('open');wireDetail();return;
  }
  const out=outEdges(n.id);
  const groups={};out.forEach(e=>{(groups[e.predicate]=groups[e.predicate]||[]).push(e);});
  let eh='';
  Object.keys(groups).forEach(p=>{
    eh+=`<div class="edge-group"><div class="eg-pred">${esc(p)}</div>`;
    groups[p].forEach(e=>{const tc=nodeColor(e.target),st=fmtStat(e),isExt=byId[e.target]&&byId[e.target].external;
      eh+=`<div class="erow" data-edge="${eid(e)}"><span class="arrow">→</span><span class="tgt"><i style="background:${tc}"></i><span>${esc(e.target)}</span>${isExt?'<span class="ext">ext</span>':''}</span>${st?`<span class="stat">${esc(st)}</span>`:''}</div>`;});
    eh+=`</div>`;
  });
  const synonyms=(pg.synonyms||[]).map(s=>`<span class="chip">${esc(s)}</span>`).join('');
  const xr=(pg.xref||[]).map(x=>`<span class="chip xref">${esc(x)}</span>`).join('');
  detail.innerHTML=`<div class="d-head"><button class="d-close" id="dClose">×</button>
    <span class="d-badge" style="background:${col}">${esc(typeStr(pg.node_type))}</span><span class="d-sub">${esc(pg.subtype||n.subtype||'')}</span>
    <div class="d-id">${esc(n.id)}</div>${pg.description?`<div class="d-desc">${esc(pg.description)}</div>`:''}
    ${synonyms?`<div class="chips">${synonyms}</div>`:''}${xr?`<div class="chips">${xr}</div>`:''}</div>
    <div class="d-body">
      <div class="d-section"><h5>Frontmatter</h5><div class="fm">${esc(buildFm(pg,out))}</div></div>
      ${eh?`<div class="d-section"><h5>Edges · this node → object (${out.length})</h5>${eh}</div>`:''}
      ${incomingSection(n.id)}
      <div class="d-section"><h5>Document</h5><div class="md">${renderMd(pg.body||'')}</div></div>
    </div>`;
  detail.classList.add('open');wireDetail();
}
function incomingSection(id){
  const inc=inEdges(id); if(!inc.length)return '';
  let h=`<div class="d-section"><h5>Referenced by (${inc.length})</h5>`;
  inc.slice(0,10).forEach(e=>{const sc=nodeColor(e.source);
    h+=`<div class="erow" data-edge="${eid(e)}"><span class="tgt"><i style="background:${sc}"></i><span>${esc(e.source)}</span></span><span class="stat">${esc(e.predicate)} →</span></div>`;});
  return h+`</div>`;
}
function showEdgeDetail(e){
  const sc=nodeColor(e.source),tc=nodeColor(e.target),st=e.stats||{},sym=e.symmetric;
  const cells=[];
  const add=(v,k)=>{if(v!==undefined&&v!==null)cells.push(`<div class="cell"><div class="v">${esc(String(v))}</div><div class="k">${esc(k)}</div></div>`);};
  add(st.effect_size, st.effect_metric||'effect'); add(st.p_value,'p-value');
  if(st.ci_lower!=null) add(st.ci_lower+'–'+st.ci_upper,'95% CI');
  add(st.sample_size!=null?('n='+st.sample_size):null,'sample'); add(st.sensitivity,'sensitivity'); add(st.specificity,'specificity');
  add(st.direction,'direction'); add(st.frequency,'frequency'); add(st.unit,'unit'); add(st.clinical_phase,'phase'); add(st.response_direction,'response');
  const isExtT=byId[e.target]&&byId[e.target].external;
  detail.innerHTML=`<div class="d-head"><button class="d-close" id="dClose">×</button>
    <span class="d-badge" style="background:#7a828e">EDGE</span><span class="d-sub">${e.synthesized?'provenance (from primary_source)':(sym?'symmetric':'directed')}</span>
    <div class="edge-headline"><span class="n" data-node="${esc(e.source)}"><i style="background:${sc}"></i>${esc(e.source)}</span>
    <span class="p">${esc(e.predicate)}${sym?' ⇄':' →'}</span>
    <span class="n" data-node="${esc(e.target)}"><i style="background:${tc}"></i>${esc(e.target)}${isExtT?' <span class="ext">ext</span>':''}</span></div></div>
    <div class="d-body">
    ${e.synthesized?'<div class="d-desc" style="margin-bottom:10px">Implicit link synthesized from the cited <code>primary_source</code> so provenance is visible. Author an explicit <code>reported_in</code> edge to make it first-class.</div>':`
    <div class="d-section"><h5>Provenance triplet</h5><div class="prov">
      <div class="cell"><div class="k">knowledge_level</div><div class="v">${esc(e.knowledge_level||'—')}</div></div>
      <div class="cell"><div class="k">agent_type</div><div class="v">${esc(e.agent_type||'—')}</div></div>
      <div class="cell" style="grid-column:1/3"><div class="k">primary_source</div><div class="v" data-node="${esc(e.primary_source||'')}" style="${byId[e.primary_source]?'cursor:pointer;color:#4f5a8a':''}">${esc(e.primary_source||'—')}</div></div>
    </div></div>`}
    ${cells.length?`<div class="d-section"><h5>Quantitative attributes</h5><div class="statgrid">${cells.join('')}</div></div>`:''}
    </div>`;
  detail.classList.add('open');wireDetail();
}
function wireDetail(){
  const c=document.getElementById('dClose');if(c)c.onclick=()=>{selected=null;selectedEdge=null;recomputeFocus();closeDetail();};
  detail.querySelectorAll('[data-node]').forEach(el=>{el.onclick=()=>{const n=byId[el.getAttribute('data-node')];if(n){selected=n;selectedEdge=null;recomputeFocus();focusOn(n);showNodeDetail(n);}};});
  detail.querySelectorAll('[data-edge]').forEach(el=>{el.onclick=()=>{const e=edges[+el.getAttribute('data-edge')];if(e){selectedEdge=e;selected=null;recomputeFocus();showEdgeDetail(e);}};});
}
function eid(e){return edges.indexOf(e);}
function focusOn(n){view.x=W*0.4-W/2-n.x*view.k;view.y=H*0.5-H/2-n.y*view.k;}
function buildFm(pg,out){
  let s=`type: ${typeStr(pg.node_type)}\nidentifier: ${pg.identifier}\n`;
  if(pg.subtype)s+=`subtype: ${pg.subtype}\n`;
  if(pg.synonyms&&pg.synonyms.length)s+=`synonyms: [${pg.synonyms.join(', ')}]\n`;
  if(pg.xref&&pg.xref.length)s+=`xref: [${pg.xref.join(', ')}]\n`;
  if(pg.raw_source&&pg.raw_source.length)s+=`raw_source: [${pg.raw_source.join(', ')}]\n`;
  if(pg.in_taxon)s+=`in_taxon: ${pg.in_taxon}\n`;
  if(pg.note)s+=`note: ${pg.note}\n`;
  if(pg.description)s+=`description: ${pg.description}\n`;
  if(out.length){s+=`edges:\n`;out.slice(0,3).forEach(e=>{s+=`  - predicate: ${e.predicate}\n    object: ${e.target}\n    knowledge_level: ${e.knowledge_level||'—'}\n    agent_type: ${e.agent_type||'—'}\n    primary_source: ${e.primary_source||'—'}\n`;});if(out.length>3)s+=`  # … ${out.length-3} more\n`;}
  return s;
}
function fmtStat(e){const st=e.stats||{};if(st.effect_size!=null)return (st.effect_metric?st.effect_metric.replace(/_/g,' ')+' ':'')+st.effect_size;if(st.sensitivity!=null)return 'sens '+st.sensitivity;if(st.frequency)return st.frequency;if(st.direction)return st.direction;if(st.unit)return st.unit;return '';}
function renderMd(md){const lines=(md||'').split('\n');let h='',inL=false;for(let line of lines){if(/^#\s+/.test(line)){if(inL){h+='</ul>';inL=false;}h+='<h1>'+inl(line.replace(/^#\s+/,''))+'</h1>';continue;}if(/^##\s+/.test(line)){if(inL){h+='</ul>';inL=false;}h+='<h2>'+inl(line.replace(/^##\s+/,''))+'</h2>';continue;}if(/^\s*[-*]\s+/.test(line)){if(!inL){h+='<ul>';inL=true;}h+='<li>'+inl(line.replace(/^\s*[-*]\s+/,''))+'</li>';continue;}if(line.trim()===''){if(inL){h+='</ul>';inL=false;}continue;}if(inL){h+='</ul>';inL=false;}h+='<p>'+inl(line)+'</p>';}if(inL)h+='</ul>';return h;}
function inl(s){s=esc(s);s=s.replace(/\*\*([^*]+)\*\*/g,'<b>$1</b>');s=s.replace(/`([^`]+)`/g,'<code>$1</code>');s=s.replace(/\[([^\]]+)\]\(([^)]*)\)/g,'<a>$1</a>');return s;}
function esc(s){return String(s==null?'':s).replace(/[&<>"]/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;'}[c]));}

/* ---------- chrome ---------- */
function monogram(name){return (name||'').split(/\s+/).slice(0,2).map(w=>w[0]||'').join('').toUpperCase();}
function renderSidebar(){
  const list=document.getElementById('kbList');list.innerHTML='';
  BASES.forEach(b=>{const el=document.createElement('div');el.className='kb'+(b.id===activeBaseId?' active':'');el.title=b.name;
    const when = b.updated ? `<span class="kb-when">updated ${esc(b.updated)}</span>` : '';
    el.innerHTML=`<span class="kb-mono">${esc(monogram(b.name))}</span><span class="kb-text"><span class="kb-name">${esc(b.name)}</span><span class="kb-meta">${b.node_count!=null?b.node_count+' nodes':''}${b.edge_count!=null?' · '+b.edge_count+' edges':''}</span>${when}</span>`;
    el.onclick=()=>selectBase(b);list.appendChild(el);});
}
function renderLegend(){
  let h='';FAMILIES.forEach(([fam,types])=>{h+=`<div class="legend-fam"><div class="fam-name">${fam}</div><div class="swatches">`;types.forEach(t=>{h+=`<span class="sw"><i style="background:${TYPE_COLOR[t]}"></i><span>${t}</span></span>`;});h+=`</div></div>`;});
  document.getElementById('legendBody').innerHTML=h;
}
function updateChrome(b){
  document.getElementById('tbTitle').textContent=b.name;
  document.getElementById('tbSub').textContent=`${b.node_count!=null?b.node_count:nodes.filter(n=>!n.external).length} nodes · ${b.edge_count!=null?b.edge_count:edges.filter(e=>!e.synthesized).length} edges`;
  const pill=document.getElementById('lintPill');
  if(b.lint){pill.style.display='inline-flex';pill.innerHTML=`<span class="e">${b.lint.errors}</span> err · <span class="w">${b.lint.warnings}</span> warn`;}
  else pill.style.display='none';
}
async function selectBase(b){
  activeBaseId=b.id;renderSidebar();closeLog();
  const bundle=await loadBundle(b);
  pages=bundle.pages||{};
  currentLog=bundle.log||''; currentUpdated=bundle.updated||null;
  loadGraph(bundle.graph);
  // merge counts/lint from bundle if base index lacked them
  const merged=Object.assign({}, b, {node_count:bundle.node_count, edge_count:bundle.edge_count, lint:bundle.lint, name:bundle.name||b.name, updated:bundle.updated});
  updateChrome(merged);
  window.__OKF_READY=true;
}

/* ---------- change-log drawer (BioRouter-style history sheet) ---------- */
function openLog(){
  const d=document.getElementById('logDrawer'), s=document.getElementById('logScrim');
  document.getElementById('logBody').innerHTML = (currentLog && currentLog.trim())
    ? renderMd(currentLog)
    : '<div class="empty">No change-log entries yet for this knowledge base.</div>';
  document.getElementById('logMeta').textContent = currentUpdated ? ('updated '+currentUpdated) : '';
  d.classList.add('open'); s.classList.add('open');
}
function closeLog(){
  const d=document.getElementById('logDrawer'), s=document.getElementById('logScrim');
  if(d) d.classList.remove('open'); if(s) s.classList.remove('open');
}

document.getElementById('collapseBtn').onclick=()=>{const wb=document.getElementById('wbody');wb.classList.toggle('collapsed');document.getElementById('collapseBtn').textContent=wb.classList.contains('collapsed')?'›':'‹';setTimeout(resize,280);};
document.getElementById('legendToggle').onclick=()=>{const lg=document.getElementById('legend');lg.classList.toggle('min');document.getElementById('legendToggle').textContent=lg.classList.contains('min')?'show':'hide';};
const searchInput=document.getElementById('searchInput');searchInput.addEventListener('input',e=>{searchTerm=e.target.value.trim().toLowerCase();});
function zoomBy(f){const cx=W/2,cy=H/2,[wx,wy]=toWorld(cx,cy);view.k=Math.max(0.25,Math.min(5,view.k*f));view.x=cx-W/2-wx*view.k;view.y=cy-H/2-wy*view.k;}
document.getElementById('zoomIn').onclick=()=>zoomBy(1.25);document.getElementById('zoomOut').onclick=()=>zoomBy(0.8);document.getElementById('zoomFit').onclick=()=>fitView();
document.getElementById('logBtn').onclick=openLog;
document.getElementById('logClose').onclick=closeLog;
document.getElementById('logScrim').onclick=closeLog;
window.addEventListener('keydown',e=>{if(e.key==='Escape')closeLog();});
function resize(){const rect=cv.getBoundingClientRect();W=rect.width;H=rect.height;DPR=Math.max(1,window.devicePixelRatio||1);cv.width=W*DPR;cv.height=H*DPR;}
window.addEventListener('resize',resize);

// expose for tests / Tauri automation
window.__okf = { selectBase:(id)=>{const b=BASES.find(x=>x.id===id);if(b)return selectBase(b);}, getState:()=>({nodes:nodes.length,edges:edges.length,base:activeBaseId}), selectNode:(id)=>{const n=byId[id];if(n){selected=n;selectedEdge=null;recomputeFocus();focusOn(n);showNodeDetail(n);}}, search:(q)=>{searchTerm=(q||'').toLowerCase();searchInput.value=q||'';} };

async function boot(){
  renderLegend();resize();
  BASES=await loadBases();renderSidebar();
  if(BASES.length)await selectBase(BASES[0]);
  requestAnimationFrame(loop);
}
boot();
