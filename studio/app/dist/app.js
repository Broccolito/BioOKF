/* BioOKF Studio frontend — data-driven from bokf-core.
   In the Tauri app it calls window.__TAURI__ invoke(); in a browser (and tests)
   it fetches the JSON the `bokf export` CLI emits. Visualization is identical. */

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
// External is NOT a node type — it's a referenced entity with no concept document yet
// (an entity to curate). Light, outline-less; grouped under provenance/context in the legend.
const EXTERNAL_COL='#D7DBE1';

const cv=document.getElementById('graph'), ctx=cv.getContext('2d');
let DPR=Math.max(1,window.devicePixelRatio||1), W=0,H=0;
let view={x:0,y:0,k:1};
let nodes=[], edges=[], byId={}, pages={};
let hover=null, hoverEdge=null, selected=null, selectedEdge=null;
let drag=null, panning=null, moved=false, alpha=1, searchTerm='';
let focusNeighbors=new Set();
let BASES=[], activeBaseId=null, currentLog='', currentUpdated=null, currentLint=null;
let currentDetailPath=null;           // path of the open node doc, for resolving body links
let currentNoteCtx=null;              // notes context (node/edge) for the open panel

/* ---------- data loading ---------- */
/* Graph data always loads from the exported static JSON, which carries the
   curated base names; the live `list_bases`/`base_info` backend returns
   name = dir-id, so we deliberately do NOT use it for display. (withGlobalTauri
   is on so the bridge + command invokes work — see tauriInvoke below.) */
const inTauri = false;
async function invoke(cmd, args){ return window.__TAURI__.core.invoke(cmd, args); }
/* Desktop webview detection — gates editor/notes/terminal (backend-only paths). */
const isDesktop = !!(window.__TAURI__ || window.__TAURI_INTERNALS__);
async function tauriInvoke(cmd, args){
  const core = window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.invoke;
  const fn = core || (window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.invoke);
  if(!fn) throw new Error('available in the desktop app only');
  return fn(cmd, args);
}
/* Debug MCP bridge: tauri-plugin-mcp's guest bindings (mcp_guest.js, injected
   debug-gated from main.rs) answer execute-js eval requests. A previous hand-rolled
   listener lived here and duplicated that handling, causing N-fold eval of every
   execute_js — removed. */
const cb = () => '?_=' + Date.now(); // dev cache-bust for the static JSON
async function loadBases(){
  // Desktop reads the LIVE registry (list_bases): bundles tracked anywhere on disk,
  // with broken/missing paths already filtered out by the backend. A KB whose folder
  // was deleted simply won't be returned, so it disappears from the sidebar. The
  // static data/bases.json snapshot is only the browser fallback.
  if(isDesktop){ try { return await tauriInvoke('list_bases'); } catch(e){ console.error('list_bases failed; using snapshot', e); } }
  return await (await fetch('data/bases.json'+cb())).json();
}
async function loadBundle(base){
  // On desktop read LIVE from the .md files so frontmatter/notes edits show up;
  // the curated display name still comes from bases.json (see selectBase).
  if(isDesktop){ try{ return await tauriInvoke('get_bundle', { id: base.id }); }catch(e){ console.error('get_bundle failed; using snapshot', e); } }
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
  const neg=isNegPred(e.predicate);  // negative (`not_<X>`) — render dashed + reddish
  let col="rgba(28,33,40,0.18)";
  if(emph===1)col="rgba(28,33,40,0.32)"; if(emph===2)col="rgba(28,33,40,0.46)"; if(dim)col="rgba(28,33,40,0.07)";
  if(neg){ // reddish so a refuted relation reads as a negation on the canvas
    col = dim?"rgba(193,75,75,0.10)":(emph===2?"rgba(193,75,75,0.62)":emph===1?"rgba(193,75,75,0.46)":"rgba(193,75,75,0.34)");
  }
  if(e.symmetric){
    ctx.save();if(neg)ctx.setLineDash([4,3]);
    ctx.strokeStyle=col;ctx.lineWidth=neg?1.1:0.9;ctx.beginPath();ctx.moveTo(sx,sy);ctx.lineTo(ex,ey);ctx.stroke();ctx.restore();
  }else if(neg){ // dashed stroked line (no solid taper) signals the negation
    ctx.save();ctx.setLineDash([4,3]);
    ctx.strokeStyle=col;ctx.lineWidth=1.1;ctx.beginPath();ctx.moveTo(sx,sy);ctx.lineTo(ex,ey);ctx.stroke();ctx.restore();
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
    const neg=isNegPred(e.predicate), lbl=predLabel(e.predicate);
    ctx.save();ctx.font="500 11px ui-monospace,Menlo,monospace";ctx.textAlign="center";ctx.textBaseline="middle";
    ctx.shadowColor="rgba(250,250,252,0.95)";ctx.shadowBlur=4;ctx.fillStyle=neg?"#c4564b":"#41474f";
    ctx.fillText(lbl,mx,my);ctx.fillText(lbl,mx,my);
    if(neg){ // strike-through reinforces the negation on the label
      const tw=ctx.measureText(lbl).width;ctx.strokeStyle="#c4564b";ctx.lineWidth=1;ctx.beginPath();ctx.moveTo(mx-tw/2,my);ctx.lineTo(mx+tw/2,my);ctx.stroke();
    }
    ctx.restore();ctx.textAlign="start";
  }
}
function drawNodeCircle(n,a,isFocus){
  const [x,y]=toScreen(n.x,n.y),r=nodeR(n)*view.k,col=n.external?EXTERNAL_COL:(n.color||TYPE_COLOR[n.type]||"#9aa1ab");
  ctx.globalAlpha=a;
  if(isFocus){
    const g=ctx.createRadialGradient(x,y,r,x,y,r+13);
    g.addColorStop(0,hexA(col,0.34));g.addColorStop(0.55,hexA(col,0.14));g.addColorStop(1,hexA(col,0));
    ctx.beginPath();ctx.arc(x,y,r+13,0,7);ctx.fillStyle=g;ctx.fill();
  }
  ctx.beginPath();ctx.arc(x,y,r,0,7);ctx.fillStyle=col;ctx.fill();
  ctx.lineWidth=1.1;ctx.strokeStyle="rgba(18,21,26,0.92)";ctx.stroke();
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
/* Predicate polarity — derived from the DATA (canonical `not_<X>` prefix), not a
   hard-coded list, so all 35 predicates (incl. the 11 negatives) are covered. */
function isNegPred(p){return typeof p==='string' && p.startsWith('not_');}
function negBase(p){return isNegPred(p)?p.slice(4):p;}
/* Display label: negatives spell out the word "not" on the underlying predicate
   (e.g. not_prevents -> "not prevents"), so the negation reads as plain language
   everywhere predLabel is used (canvas tooltip, edge groups, headline, incoming rows). */
function predLabel(p){return isNegPred(p)?('not '+negBase(p)):p;}
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
  else if(hoverEdge)showTip(sx,sy,predLabel(hoverEdge.predicate),hoverEdge.source+(hoverEdge.symmetric?' ⇄ ':' → ')+hoverEdge.target);
  else hideTip();
});
function showTip(sx,sy,a,b){tip.style.display='block';tip.style.left=(sx+14)+'px';tip.style.top=(sy+14)+'px';tip.innerHTML=`${esc(a)}<br><span class="tp">${esc(b)}</span>`;}
function hideTip(){tip.style.display='none';}
cv.addEventListener('mousedown',ev=>{const rect=cv.getBoundingClientRect(),sx=ev.clientX-rect.left,sy=ev.clientY-rect.top;moved=false;const n=pickNode(sx,sy);if(n)drag=n;else panning={x:sx,y:sy};cv.classList.add('grabbing');});
window.addEventListener('mouseup',ev=>{
  // Only handle releases that BEGAN on the canvas. A canvas mousedown always sets
  // drag (a node) or panning (empty space); a click on the detail panel, preview
  // popup, or terminal floating above the canvas sets neither. Without this guard,
  // releasing such a click ran pickNode→closeDetail and swallowed the click — so
  // Edit/Save/Notes/citation buttons appeared to "do nothing".
  if(!drag && !panning){return;}
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
    currentDetailPath=null;detail.classList.add('open');wireDetail();return;
  }
  const out=outEdges(n.id);
  const groups={};out.forEach(e=>{(groups[e.predicate]=groups[e.predicate]||[]).push(e);});
  let eh='';
  Object.keys(groups).forEach(p=>{
    const neg=isNegPred(p), sym=groups[p][0]&&groups[p][0].symmetric;
    eh+=`<div class="edge-group"><div class="eg-pred${neg?' neg':''}">${esc(predLabel(p))}</div>`;
    groups[p].forEach(e=>{const tc=nodeColor(e.target),st=fmtStat(e),isExt=byId[e.target]&&byId[e.target].external;
      eh+=`<div class="erow" data-edge="${eid(e)}"><span class="arrow">${sym?'⇄':'→'}</span><span class="tgt"><i style="background:${tc}"></i><span>${esc(e.target)}</span>${isExt?'<span class="ext">ext</span>':''}</span>${st?`<span class="stat">${esc(st)}</span>`:''}</div>`;});
    eh+=`</div>`;
  });
  detail.innerHTML=`<div class="d-head">${headBtns(pg.path)}
    <span class="d-badge" style="background:${col}">${esc(typeStr(pg.node_type))}</span>
    <div class="d-id">${esc(n.id)}</div></div>
    <div class="d-body">
      <div class="d-section">${nodeFrontmatterHtml(pg,n)}</div>
      <div class="d-section" id="sourceSection"></div>
      ${eh?`<div class="d-section"><h5>Edges · this node → object (${out.length})</h5>${eh}</div>`:''}
      ${incomingSection(n.id)}
      <div class="d-section" id="docSection">${docSectionHtml(pg)}</div>
      ${notesSectionHtml(noteCtxForNode(n,pg))}
    </div>`;
  currentDetailPath=pg.path||null;
  detail.classList.add('open');wireDetail();
  hydrateSourceProvenance(pg);
}
/* Shared header buttons (close, reveal-in-Finder, Edit). The Edit button enters
   full-file edit mode for the given file path; desktop-only and only when a path
   exists (external/source-less entities have no file to edit). */
function headBtns(path){
  const canEdit=isDesktop && !!path;
  const hasReveal=isDesktop&&path;
  const reveal=hasReveal?'<button class="d-reveal" id="dReveal" title="Reveal markdown file in Finder"><svg width="13" height="13" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M9.5 2.5h4v4"/><path d="M13.5 2.5l-6 6"/><path d="M11.5 9v2.5a1 1 0 0 1-1 1h-6a1 1 0 0 1-1-1v-6a1 1 0 0 1 1-1H7"/></svg></button>':'';
  // Edit sits just left of the icon buttons (right of reveal when reveal is shown).
  const edit=canEdit?`<button class="d-edit${hasReveal?' has-reveal':''}" id="dEdit" data-edit-path="${esc(path)}" title="Edit the markdown file">✎ Edit</button>`:'';
  return `<button class="d-close" id="dClose">×</button>${reveal}${edit}`;
}
/* Plain key/value row (SIMPLE tier) — value-agnostic, assumes no vocabulary. */
function kvRow(k,v){ return `<div class="kv-row"><div class="kv-k">${esc(k)}</div><div class="kv-v">${v}</div></div>`; }
function kvPlain(k,v){ return (v===undefined||v===null||v==='')?'':kvRow(k,esc(String(v))); }
/* Every list of short tokens (xref, synonyms, tags, …) renders identically as chips
   so similar data looks the same across the panel. */
function chips(arr){ return (arr||[]).map(x=>`<span class="chip">${esc(x)}</span>`).join(''); }
/* Node frontmatter — ONE ordered block matching the source .md (SPEC §4 template
   order), de-duplicated. Controlled `type` stays rich (header badge); every other
   field renders as a plain `label: value` row, value-agnostic. */
function nodeFrontmatterHtml(pg,n){
  if(!pg) return '';
  const rows=[];
  rows.push(kvPlain('subtype', pg.subtype||n.subtype));
  if(pg.xref&&pg.xref.length) rows.push(kvRow('xref', chips(pg.xref)));
  if(pg.synonyms&&pg.synonyms.length) rows.push(kvRow('synonyms', chips(pg.synonyms)));
  rows.push(kvPlain('in_taxon', pg.in_taxon));
  rows.push(kvPlain('description', pg.description));
  rows.push(kvPlain('note', pg.note));
  if(pg.tags&&pg.tags.length) rows.push(kvRow('tags', chips(pg.tags)));
  if(pg.raw_source&&pg.raw_source.length) rows.push(kvRow('raw_source', pg.raw_source.map(p=>`<code>${esc(p)}</code>`).join('<br>')));
  rows.push(kvPlain('timestamp', pg.timestamp));
  if(pg.extra && typeof pg.extra==='object'){
    Object.keys(pg.extra).forEach(k=>{ const v=pg.extra[k]; rows.push(kvPlain(k, typeof v==='object'?JSON.stringify(v):v)); });
  }
  const body=rows.filter(Boolean).join('');
  return body?`<div class="kv">${body}</div>`:'';
}
/* Source / Provenance — for a source node (one with raw_source[]) on desktop,
   pull the ingested source's raw/<id>/meta.yaml via the source_info connector and
   render origin + credibility + ids + figures. Mirrors loadRawSource()'s
   desktop-only behaviour; a missing meta.yaml just omits the section. */
async function hydrateSourceProvenance(pg){
  const sec=document.getElementById('sourceSection'); if(!sec) return;
  if(!pg || !pg.raw_source || !pg.raw_source.length) return;
  if(!isDesktop) return;   // desktop-only (the connector reads files from disk)
  // raw/<id>/source.md -> <id>
  const parts=String(pg.raw_source[0]).split('/');
  const ri=parts.indexOf('raw');
  const source_id = ri>=0 ? parts[ri+1] : parts[parts.length-2];
  if(!source_id) return;
  let info;
  try{ info=await tauriInvoke('source_info', { base: activeBaseId, sourceId: source_id }); }
  catch(e){ return; }   // no meta.yaml / connector unavailable — omit the section
  if(!info || typeof info!=='object') return;
  const cred=info.credibility||{}, ids=info.ids||{};
  const TIER_RANK={peer_reviewed:6,preprint:5,archive:4,gray_lit:3,web:2,unknown:1};
  const tier=cred.tier||'unknown';
  const tierClass = TIER_RANK[tier]>=5?'tier-hi':TIER_RANK[tier]>=3?'tier-mid':'tier-lo';
  const idLinks=[];
  const idLink=(k,v,url)=>{ if(v) idLinks.push(url?`<a class="cite" href="${esc(url)}" target="_blank" rel="noopener">${esc(k)}:${esc(v)}</a>`:`<code>${esc(k)}:${esc(v)}</code>`); };
  idLink('doi', ids.doi, ids.doi?('https://doi.org/'+ids.doi):null);
  idLink('pmid', ids.pmid, ids.pmid?('https://pubmed.ncbi.nlm.nih.gov/'+ids.pmid):null);
  idLink('pmcid', ids.pmcid, ids.pmcid?('https://www.ncbi.nlm.nih.gov/pmc/articles/'+ids.pmcid):null);
  idLink('arxiv', ids.arxiv, ids.arxiv?('https://arxiv.org/abs/'+ids.arxiv):null);
  idLink('isbn', ids.isbn, null);
  const urlLinks=[];
  if(info.url) urlLinks.push(`<a class="cite" href="${esc(info.url)}" target="_blank" rel="noopener">url</a>`);
  if(info.final_url && info.final_url!==info.url) urlLinks.push(`<a class="cite" href="${esc(info.final_url)}" target="_blank" rel="noopener">final_url</a>`);
  const figs=Array.isArray(info.figures)?info.figures:[];
  let figHtml='';
  if(figs.length){
    figHtml=`<div class="src-figs">${figs.map((f,i)=>{
      const flags=[]; if(f.provisional)flags.push('<span class="fig-flag prov">provisional</span>'); if(f.described===false||f.described==='false')flags.push('<span class="fig-flag undesc">undescribed</span>'); if(f.origin)flags.push(`<span class="fig-flag">${esc(f.origin)}</span>`);
      // FigureMeta.file is already relative to raw/<id>/ (e.g. "figures/foo.png") — do not re-prefix.
      return `<figure class="src-fig"><img class="md-img" data-md-raw="raw/${esc(source_id)}/${esc(f.file||'')}" alt="${esc(f.file||('figure '+i))}"><figcaption>${esc(f.file||'')} ${flags.join(' ')}</figcaption></figure>`;
    }).join('')}</div>`;
  }
  const credChips=`${info.source_type?`<span class="src-origin">${esc(info.source_type)}</span>`:''}<span class="src-tier ${tierClass}">${esc(tier)}</span>${cred.confidence!=null?`<span class="src-conf">conf ${esc(Number(cred.confidence).toFixed(2))}</span>`:''}${cred.retracted?'<span class="src-retracted">⚠ RETRACTED</span>':''}`;
  // Same .kv rows as the rest of the panel (the credibility badges ride inline in one row),
  // so the Source / Provenance block is stylistically consistent with the fields above/below it.
  sec.innerHTML=`<h5>Source / Provenance</h5>
    <div class="kv">
      ${kvRow('credibility', credChips)}
      ${info.title?kvRow('title', esc(info.title)):''}
      ${info.format?kvRow('format', esc(info.format)):''}
      ${cred.venue?kvRow('venue', esc(cred.venue)):''}
      ${cred.publisher?kvRow('publisher', esc(cred.publisher)):''}
      ${cred.reasoning?kvRow('reasoning', esc(cred.reasoning)):''}
      ${cred.classifier_version?kvRow('classifier', esc(String(cred.classifier_version))):''}
      ${idLinks.length?kvRow('ids', idLinks.join(' ')):''}
      ${urlLinks.length?kvRow('links', urlLinks.join(' ')):''}
    </div>${figHtml}`;
  hydrateMdImages(sec);
}
/* Document section — read-only rendered body (editing now happens on the
   frontmatter above, per the requested workflow). */
function docSectionHtml(pg){
  return `<h5>Document</h5><div class="md" id="docMd">${renderMd(stripNotesSection(pg.body||''))}</div>`;
}
function incomingSection(id){
  const inc=inEdges(id); if(!inc.length)return '';
  let h=`<div class="d-section"><h5>Referenced by (${inc.length})</h5>`;
  inc.slice(0,10).forEach(e=>{const sc=nodeColor(e.source),neg=isNegPred(e.predicate);
    h+=`<div class="erow" data-edge="${eid(e)}"><span class="tgt"><i style="background:${sc}"></i><span>${esc(e.source)}</span></span><span class="stat${neg?' neg':''}">${esc(predLabel(e.predicate))} ${e.symmetric?'⇄':'→'}</span></div>`;});
  return h+`</div>`;
}
function showEdgeDetail(e){
  const sc=nodeColor(e.source),tc=nodeColor(e.target),sym=e.symmetric,neg=isNegPred(e.predicate);
  // Prefer the full page-edge stats (complete bundle); fall back to the graph edge.
  const full=edgeFull(e), st=(full&&full.stats)||e.stats||{};
  // stats — every key rendered uniformly as `key: value`; only the ci_lower–ci_upper
  // merge is privileged (a display convenience), nothing else.
  const statRows=[];
  if(st.ci_lower!=null && st.ci_upper!=null) statRows.push(kvRow('95% CI', esc(st.ci_lower+'–'+st.ci_upper)));
  Object.keys(st).forEach(k=>{ if(k==='ci_lower'||k==='ci_upper') return; const v=st[k]; if(v!=null&&v!==''&&typeof v!=='object') statRows.push(kvRow(k, esc(String(v)))); });
  const statHtml=statRows.length?`<div class="d-section"><h5>Stats</h5><div class="kv">${statRows.join('')}</div></div>`:'';
  // qualifiers{} — every key rendered uniformly, no hard-coded ordering.
  const qobj=(full&&full.qualifiers)||{};
  const qRows=Object.keys(qobj).filter(k=>qobj[k]!=null&&qobj[k]!==''&&typeof qobj[k]!=='object').map(k=>kvRow(k, esc(String(qobj[k]))));
  const qualHtml=qRows.length?`<div class="d-section"><h5>Qualifiers</h5><div class="kv">${qRows.join('')}</div></div>`:'';
  // publications[] (out-links) from the full page edge — BEFORE stats (file order).
  const pubs=(full&&full.publications)||[];
  const pubHtml=pubs.length?`<div class="d-section"><h5>Publications (${pubs.length})</h5><div class="pub-list">${pubs.map(p=>{const ext=/^https?:\/\//i.test(p);return ext?`<a class="cite" href="${esc(p)}" target="_blank" rel="noopener">${esc(p)}</a>`:`<span class="cite" data-cite="${esc(p)}">${esc(p)}</span>`;}).join('')}</div></div>`:'';
  // direction / aspect — plain rows, no privileged labels.
  const dir=(full&&full.direction)||st.direction, asp=(full&&full.aspect)||st.aspect;
  const dirRows=[kvPlain('direction',dir),kvPlain('aspect',asp)].filter(Boolean).join('');
  const dirHtml=dirRows?`<div class="d-section"><div class="kv">${dirRows}</div></div>`:'';
  const isExtT=byId[e.target]&&byId[e.target].external;
  // Edges live in their SOURCE node's file — Edit edits that file in context.
  const srcPath=pages[e.source]&&pages[e.source].path;
  detail.innerHTML=`<div class="d-head">${headBtns(srcPath)}
    <span class="d-badge" style="background:${neg?'#c4564b':'#7a828e'}">${neg?'NEGATIVE EDGE':'EDGE'}</span><span class="d-sub">${e.synthesized?'provenance (from primary_source)':(neg?'refuted relation · ':'')+(sym?'symmetric':'directed')}</span>
    <div class="edge-headline"><span class="n" data-node="${esc(e.source)}"><i style="background:${sc}"></i>${esc(e.source)}</span>
    <span class="p${neg?' neg':''}">${esc(predLabel(e.predicate))}${sym?' ⇄':' →'}</span>
    <span class="n" data-node="${esc(e.target)}"><i style="background:${tc}"></i>${esc(e.target)}${isExtT?' <span class="ext">ext</span>':''}</span></div></div>
    <div class="d-body">
    ${e.synthesized?'<div class="d-desc" style="margin-bottom:10px">Implicit link synthesized from the cited <code>primary_source</code> so provenance is visible. Author an explicit <code>reported_in</code> edge to make it first-class.</div>':`
    <div class="d-section"><h5>Provenance triplet</h5><div class="prov">
      <div class="cell"><div class="k">knowledge_level</div><div class="v">${esc(e.knowledge_level||'—')}</div></div>
      <div class="cell"><div class="k">agent_type</div><div class="v">${esc(e.agent_type||'—')}</div></div>
      <div class="cell" style="grid-column:1/3"><div class="k">primary_source</div><div class="v${pages[e.primary_source]?' cite':''}"${pages[e.primary_source]?` data-cite="${esc(e.primary_source)}"`:''}>${esc(e.primary_source||'—')}</div></div>
    </div></div>`}
    ${dirHtml}
    ${pubHtml}
    ${statHtml}
    ${qualHtml}
    ${notesSectionHtml(noteCtxForEdge(e))}
    </div>`;
  // currentDetailPath = the file Edit/cites resolve against (the source node's file).
  currentDetailPath=srcPath||null;detail.classList.add('open');wireDetail();
}
function wireDetail(){
  const c=document.getElementById('dClose');if(c)c.onclick=()=>{selected=null;selectedEdge=null;recomputeFocus();closeDetail();};
  detail.querySelectorAll('[data-node]').forEach(el=>{el.onclick=()=>{const n=byId[el.getAttribute('data-node')];if(n){selected=n;selectedEdge=null;recomputeFocus();focusOn(n);showNodeDetail(n);}};});
  detail.querySelectorAll('[data-edge]').forEach(el=>{el.onclick=()=>{const e=edges[+el.getAttribute('data-edge')];if(e){selectedEdge=e;selected=null;recomputeFocus();showEdgeDetail(e);}};});
  wireCites(detail, currentDetailPath);
  const rv=document.getElementById('dReveal');
  if(rv) rv.onclick=()=>{ if(isDesktop && currentDetailPath) tauriInvoke('reveal_in_finder', { base: activeBaseId, path: currentDetailPath }).catch(()=>{}); };
  const ed=document.getElementById('dEdit');
  if(ed && isDesktop) ed.onclick=()=>openFileEditor(ed.getAttribute('data-edit-path'));
  wireNotes();
  hydrateMdImages(detail);
}

/* ---------- citation previews ---------- */
/* Resolve a citation reference (a node id, or a relative `.md` link from a node
   body) to a page id we can preview. */
function resolveCite(ref, fromPath){
  if(!ref) return null;
  if(pages[ref]) return ref;                       // direct node id (e.g. primary_source)
  let href=ref.split('#')[0].split('?')[0];
  const seg=(fromPath||'').split('/').slice(0,-1); // start in the current doc's folder
  href.split('/').forEach(p=>{ if(p===''||p==='.')return; if(p==='..'){seg.pop();return;} seg.push(p); });
  const norm=seg.join('/');
  for(const id in pages){ if(pages[id].path===norm) return id; }
  const fn=norm.split('/').pop();                  // fallback: match by filename
  for(const id in pages){ const p=pages[id].path||''; if(p.split('/').pop()===fn) return id; }
  return null;
}
function wireCites(root, fromPath){
  if(!root) return;
  root.querySelectorAll('[data-cite]').forEach(el=>{
    el.onclick=(ev)=>{ ev.stopPropagation(); openPreview(el.getAttribute('data-cite'), fromPath); };
  });
}
const previewEl=document.getElementById('preview'), previewScrim=document.getElementById('previewScrim');
function openPreview(ref, fromPath){
  const id=resolveCite(ref, fromPath);
  if(!id || !pages[id]){
    previewEl.innerHTML=`<div class="pv-head"><div><div class="pv-eyebrow">Citation</div><div class="pv-title">Source not in this base</div></div><button class="pv-close" id="pvClose">×</button></div>
      <div class="pv-body"><div class="pv-missing">“${esc(ref||'')}” isn’t a document in this knowledge base.</div></div>`;
  }else{
    const pg=pages[id], col=nodeColor(id);
    const hasRaw = pg.raw_source && pg.raw_source.length;
    const rawBlock = hasRaw
      ? `<div class="pv-raw"><div class="pv-rawhead">Original paper · <code>${esc(pg.raw_source[0])}</code></div><div class="md" id="pvRawBody"><span class="pv-missing">Loading source…</span></div></div>`
      : '';
    previewEl.innerHTML=`<div class="pv-head"><span class="pv-badge" style="background:${col}">${esc(typeStr(pg.node_type))}</span><div><div class="pv-eyebrow">Cited source</div><div class="pv-title">${esc(id)}</div></div><button class="pv-close" id="pvClose">×</button></div>
      <div class="pv-body">${pg.description?`<div class="d-desc" style="margin-bottom:10px">${esc(pg.description)}</div>`:''}<div class="md">${renderMd(pg.body||'')}</div>${rawBlock}</div>`;
    wireCites(previewEl, pg.path||null);
    hydrateMdImages(previewEl);
    if(hasRaw) loadRawSource(activeBaseId, pg.raw_source[0]);
  }
  previewEl.classList.add('open'); previewScrim.classList.add('open');
  const c=document.getElementById('pvClose'); if(c) c.onclick=closePreview;
}
/* Load the original ingested paper (raw/source.md) into an open preview. */
async function loadRawSource(base, path){
  const el=document.getElementById('pvRawBody'); if(!el) return;
  if(!isDesktop){ el.innerHTML='<span class="pv-missing">Open the desktop app to read the original source paper.</span>'; return; }
  try{
    const text=await tauriInvoke('read_bundle_file', { base, path });
    el.innerHTML=renderMd(text||'');
    hydrateMdImages(el);
  }catch(e){ el.innerHTML='<span class="pv-missing">Could not load source: '+esc(String((e&&e.message)||e))+'</span>'; }
}
function closePreview(){ previewEl.classList.remove('open'); previewScrim.classList.remove('open'); }

/* ---------- full-file editor (header Edit -> replaces the whole panel) ----------
   Loads the ENTIRE .md (frontmatter AND body) into one textarea. Save writes the
   whole file via save_node_file and regenerates the panel from fresh disk data;
   Cancel re-renders the current node/edge detail, discarding edits. For an edge
   panel the edited file is the SOURCE node's file (where the edge is defined). */
let editorReturn=null;   // { kind:'node', node } | { kind:'edge', edge } — what to re-render on Save/Cancel
async function openFileEditor(path){
  if(!isDesktop || !path) return;
  // capture what to re-render afterwards (current selection)
  editorReturn = selectedEdge ? {kind:'edge', edge:selectedEdge} : (selected ? {kind:'node', node:selected} : null);
  const label = (selected && selected.id) || (selectedEdge && (selectedEdge.source+' '+selectedEdge.predicate+' '+selectedEdge.target)) || path;
  const keepH = detail.offsetHeight;   // pin the panel to its current (formatted) size so the editor doesn't shrink
  detail.innerHTML=`<div class="d-head"><button class="d-close" id="dClose">×</button>
    <div class="d-id">${esc((selected&&selected.id)||(selectedEdge&&selectedEdge.source)||'Edit')}</div>
    <div class="d-desc">Editing <code>${esc(path)}</code></div></div>
    <div class="file-editor">
      <textarea class="md-edit file-edit" id="fileEditArea" spellcheck="false" disabled>Loading file…</textarea>
      <div class="edit-bar">
        <button class="btn primary" id="fileSave" disabled>Save</button>
        <button class="btn" id="fileCancel">Cancel</button>
        <span class="edit-status" id="fileMsg"></span>
      </div>
    </div>`;
  detail.classList.add('open');
  if(keepH) detail.style.height=keepH+'px';
  const ta=document.getElementById('fileEditArea'), save=document.getElementById('fileSave'), cancel=document.getElementById('fileCancel');
  document.getElementById('dClose').onclick=()=>cancelFileEditor();
  cancel.onclick=()=>cancelFileEditor();
  save.onclick=()=>saveFileEditor(path, label);
  try{
    const raw=await tauriInvoke('read_bundle_file', { base: activeBaseId, path });
    ta.value=raw||''; ta.disabled=false; save.disabled=false; ta.focus();
  }catch(err){ ta.value='# could not read file: '+String((err&&err.message)||err); ta.disabled=false; }
}
function cancelFileEditor(){
  detail.style.height='';
  const r=editorReturn; editorReturn=null;
  if(r&&r.kind==='node') showNodeDetail(r.node);
  else if(r&&r.kind==='edge') showEdgeDetail(r.edge);
  else closeDetail();
}
async function saveFileEditor(path, label){
  const ta=document.getElementById('fileEditArea'), save=document.getElementById('fileSave'), msg=document.getElementById('fileMsg');
  if(!ta) return;
  save.disabled=true; msg.className='edit-status'; msg.textContent='Saving…';
  try{
    await tauriInvoke('save_node_file', { base: activeBaseId, path, content: ta.value, label, date: today() });
    detail.style.height='';
    // The desktop app reads live from disk — refresh the bundle so pages/graph
    // reflect the edit, then re-open the same node/edge detail.
    const r=editorReturn; editorReturn=null;
    const b=BASES.find(x=>x.id===activeBaseId);
    if(b) await selectBase(b);   // reloads pages/graph; clears selection + detail
    if(r&&r.kind==='node'){ const n=byId[r.node.id]; if(n){ selected=n;selectedEdge=null;recomputeFocus();showNodeDetail(n); } }
    else if(r&&r.kind==='edge'){ const e=edges.find(x=>x.source===r.edge.source&&x.predicate===r.edge.predicate&&x.target===r.edge.target); if(e){ selectedEdge=e;selected=null;recomputeFocus();showEdgeDetail(e); } else closeDetail(); }
  }catch(e){ save.disabled=false; msg.className='edit-status err'; msg.textContent='Save failed: '+String((e&&e.message)||e); }
}

/* ---------- notes — stored in markdown (node: `# Notes` body section; edge: the
   edge's frontmatter `note:` field). notes.json is gone. ---------- */
function today(){ const d=new Date(); return d.getFullYear()+'-'+String(d.getMonth()+1).padStart(2,'0')+'-'+String(d.getDate()).padStart(2,'0'); }
function extractNotesSection(body){
  const lines=(body||'').split('\n');
  const i=lines.findIndex(l=>l.trim()==='# Notes');
  if(i<0) return '';
  const out=[];
  for(let j=i+1;j<lines.length;j++){ if(/^#\s+/.test(lines[j])) break; out.push(lines[j]); }
  return out.join('\n').trim();
}
function stripNotesSection(body){
  const lines=(body||'').split('\n');
  const i=lines.findIndex(l=>l.trim()==='# Notes');
  if(i<0) return body||'';
  let end=lines.length;
  for(let j=i+1;j<lines.length;j++){ if(/^#\s+/.test(lines[j])){ end=j; break; } }
  return lines.slice(0,i).concat(lines.slice(end)).join('\n').replace(/\n{3,}/g,'\n\n').trim();
}
function upsertNotesInBody(body, notes){
  const stripped=stripNotesSection(body);
  if(!notes.trim()) return stripped;
  return (stripped?stripped+'\n\n':'')+'# Notes\n\n'+notes.trim()+'\n';
}
/* Resolve a graph edge back to its full page Edge record, which carries the
   richer bundle (publications[], qualifiers{}, the complete stats map) that the
   flattened graph edge omits. The graph predicate is the canonical lowercase
   form, matched against the page edge's raw_predicate. */
function edgeFull(e){
  const pg=pages[e.source]; if(!pg||!pg.edges) return null;
  return pg.edges.find(x=>(x.raw_predicate===e.predicate||x.predicate===e.predicate)&&x.object===e.target)||null;
}
function edgeNote(e){
  const m=edgeFull(e);
  return (m&&m.note)||'';
}
function noteCtxForNode(n,pg){ return { kind:'node', id:n.id, path:pg&&pg.path, note:extractNotesSection(pg&&pg.body) }; }
function noteCtxForEdge(e){ const pg=pages[e.source];
  return { kind:'edge', source:e.source, srcPath:pg&&pg.path, predicate:e.predicate, object:e.target, label:e.source+' '+e.predicate+' '+e.target, note:edgeNote(e) }; }
function notesSectionHtml(ctx){
  currentNoteCtx=ctx;
  const txt=ctx.note||'';
  const canEdit=isDesktop && (ctx.kind==='node'? !!ctx.path : !!ctx.srcPath);
  return `<div class="d-section notes-section" id="notesSection">
    <h5>Notes<span class="sec-actions">
      <button class="btn-mini" id="noteEdit"${canEdit?'':' disabled title="Notes are saved in the desktop app"'}>✎ Add note</button>
      <button class="btn-mini primary" id="noteSave" disabled>Save</button>
      <span class="edit-status" id="noteMsg"></span></span></h5>
    <div class="notes-view" id="notesView">${txt?renderMd(txt):'<span class="notes-empty">No notes yet — click Add note to attach one.</span>'}</div>
  </div>`;
}
function wireNotes(){
  const e=document.getElementById('noteEdit'), s=document.getElementById('noteSave');
  if(e && isDesktop && !e.disabled) e.onclick=()=>toggleNoteEdit();
  if(s) s.onclick=()=>saveNote();
}
function toggleNoteEdit(){
  if(document.getElementById('noteEditArea')){ renderNotesView(); return; }
  const view=document.getElementById('notesView'); if(!view) return;
  view.outerHTML='<textarea class="md-edit notes-edit" id="noteEditArea" placeholder="Type your notes (markdown supported)…"></textarea>';
  const ta=document.getElementById('noteEditArea'); ta.value=(currentNoteCtx&&currentNoteCtx.note)||''; ta.focus();
  document.getElementById('noteEdit').textContent='Cancel';
  document.getElementById('noteSave').disabled=false;
}
function renderNotesView(){
  const sec=document.getElementById('notesSection'); if(!sec) return;
  const txt=(currentNoteCtx&&currentNoteCtx.note)||'';
  const e=sec.querySelector('#noteEdit'); if(e) e.textContent='✎ Add note';
  const s=sec.querySelector('#noteSave'); if(s) s.disabled=true;
  const area=document.getElementById('noteEditArea');
  if(area) area.outerHTML=`<div class="notes-view" id="notesView">${txt?renderMd(txt):'<span class="notes-empty">No notes yet — click Add note to attach one.</span>'}</div>`;
}
async function saveNote(){
  const ta=document.getElementById('noteEditArea'); if(!ta||!currentNoteCtx) return;
  const ctx=currentNoteCtx, text=ta.value, s=document.getElementById('noteSave'), msg=document.getElementById('noteMsg');
  s.disabled=true; msg.className='edit-status'; msg.textContent='Saving…';
  try{
    if(ctx.kind==='node'){
      await tauriInvoke('save_node_notes', { base: activeBaseId, path: ctx.path, notes: text, label: ctx.id, date: today() });
      if(pages[ctx.id]) pages[ctx.id].body=upsertNotesInBody(pages[ctx.id].body||'', text);
    }else{
      await tauriInvoke('save_edge_note', { base: activeBaseId, path: ctx.srcPath, predicate: ctx.predicate, object: ctx.object, note: text, label: ctx.label, date: today() });
      const pg=pages[ctx.source]; if(pg&&pg.edges){ const m=pg.edges.find(x=>(x.raw_predicate===ctx.predicate||x.predicate===ctx.predicate)&&x.object===ctx.object); if(m) m.note=text; }
    }
    ctx.note=text;
    msg.className='edit-status ok'; msg.textContent='Saved ✓';
    renderNotesView();
  }catch(e){ s.disabled=false; msg.className='edit-status err'; msg.textContent='Save failed: '+String((e&&e.message)||e); }
}

function eid(e){return edges.indexOf(e);}
function focusOn(n){view.x=W*0.4-W/2-n.x*view.k;view.y=H*0.5-H/2-n.y*view.k;}
function fmtStat(e){const st=e.stats||{};if(st.effect_size!=null)return (st.effect_metric?st.effect_metric.replace(/_/g,' ')+' ':'')+st.effect_size;if(st.sensitivity!=null)return 'sens '+st.sensitivity;if(st.frequency)return st.frequency;if(st.direction)return st.direction;if(st.unit)return st.unit;return '';}
function renderMd(md){const lines=(md||'').split('\n');let h='',inL=false;for(let line of lines){if(/^#\s+/.test(line)){if(inL){h+='</ul>';inL=false;}h+='<h1>'+inl(line.replace(/^#\s+/,''))+'</h1>';continue;}if(/^##\s+/.test(line)){if(inL){h+='</ul>';inL=false;}h+='<h2>'+inl(line.replace(/^##\s+/,''))+'</h2>';continue;}if(/^\s*[-*]\s+/.test(line)){if(!inL){h+='<ul>';inL=true;}h+='<li>'+inl(line.replace(/^\s*[-*]\s+/,''))+'</li>';continue;}if(line.trim()===''){if(inL){h+='</ul>';inL=false;}continue;}if(inL){h+='</ul>';inL=false;}h+='<p>'+inl(line)+'</p>';}if(inL)h+='</ul>';return h;}
function inl(s){
  s=esc(s);
  s=s.replace(/\*\*([^*]+)\*\*/g,'<b>$1</b>');
  s=s.replace(/`([^`]+)`/g,'<code>$1</code>');
  // IMAGE rule — must run BEFORE the citation-link rule so `![alt](url)` isn't
  // turned into a broken cite link. http(s) URLs render inline as-is; relative
  // `raw/` paths are hydrated to a data URI on desktop (see hydrateMdImages).
  s=s.replace(/!\[([^\]]*)\]\(([^)]*)\)/g,(m,alt,url)=>{
    const u=(url||'').trim();
    if(/^https?:\/\//i.test(u)) return `<img class="md-img" src="${u}" alt="${alt}">`;
    return `<img class="md-img" data-md-raw="${u}" alt="${alt}">`;
  });
  s=s.replace(/\[([^\]]+)\]\(([^)]*)\)/g,(m,t,u)=>`<a class="cite" data-cite="${u}">${t}</a>`);
  return s;
}
/* Hydrate relative `raw/` markdown images to data URIs (desktop only): figures in
   ingested source papers live as binary files inside the bundle, so they can't be
   loaded by relative URL in the webview. We read their raw bytes via the
   read_bundle_bytes connector (base64) and inline them. http(s) images are left untouched. */
async function hydrateMdImages(root){
  if(!root || !isDesktop) return;
  const imgs=root.querySelectorAll && root.querySelectorAll('img.md-img[data-md-raw]');
  if(!imgs || !imgs.length) return;
  for(const img of imgs){
    const path=img.getAttribute('data-md-raw'); img.removeAttribute('data-md-raw');
    if(!path || /^https?:\/\//i.test(path)){ if(path) img.src=path; continue; }
    try{
      const b64=await tauriInvoke('read_bundle_bytes', { base: activeBaseId, path });
      if(typeof b64==='string' && b64){
        img.src = b64.startsWith('data:') ? b64 : ('data:'+mimeForPath(path)+';base64,'+b64);
      }
    }catch(e){ /* missing figure — leave it unresolved */ }
  }
}
function mimeForPath(p){
  const ext=((p||'').split('.').pop()||'').toLowerCase();
  return ({png:'image/png',jpg:'image/jpeg',jpeg:'image/jpeg',gif:'image/gif',webp:'image/webp',svg:'image/svg+xml'}[ext])||'image/png';
}
function esc(s){return String(s==null?'':s).replace(/[&<>"]/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;'}[c]));}

/* ---------- chrome ---------- */
function monogram(name){return (name||'').split(/\s+/).slice(0,2).map(w=>w[0]||'').join('').toUpperCase();}
function renderSidebar(){
  const list=document.getElementById('kbList');list.innerHTML='';
  BASES.forEach(b=>{const el=document.createElement('div');el.className='kb'+(b.id===activeBaseId?' active':'');el.title=b.path?b.name+'\n'+b.path:b.name;
    // Path lives in the hover tooltip (el.title) only — permanent gray text is just counts + updated.
    const when = b.updated ? `<span class="kb-when">updated ${esc(b.updated)}</span>` : '';
    const focus = b.id===aiFocusKb ? `<span class="kb-focus" title="AI agent is focused on this knowledge base"></span>` : '';
    el.innerHTML=`<span class="kb-mono">${esc(monogram(b.name))}</span><span class="kb-text"><span class="kb-name">${esc(b.name)}</span><span class="kb-meta">${b.node_count!=null?b.node_count+' nodes':''}${b.edge_count!=null?' · '+b.edge_count+' edges':''}</span>${when}</span>${focus}`;
    el.onclick=()=>selectBase(b);list.appendChild(el);});
}
function renderLegend(){
  let h='';FAMILIES.forEach(([fam,types])=>{h+=`<div class="legend-fam"><div class="fam-name">${fam}</div><div class="swatches">`;types.forEach(t=>{h+=`<span class="sw"><i style="background:${TYPE_COLOR[t]}"></i><span>${t}</span></span>`;});
    // External isn't a node type — a referenced entity with no concept doc yet (curate it).
    // Grouped under provenance/context; solid light swatch like the rest (no special outline).
    if(fam==='Provenance & context'){h+=`<span class="sw" title="Referenced by an edge but has no concept document yet — an entity to curate."><i style="background:${EXTERNAL_COL}"></i><span>External</span></span>`;}
    h+=`</div></div>`;});
  document.getElementById('legendBody').innerHTML=h;
}
function updateChrome(b){
  document.getElementById('tbTitle').textContent=b.name;
  document.getElementById('tbSub').textContent=`${b.node_count!=null?b.node_count:nodes.filter(n=>!n.external).length} nodes · ${b.edge_count!=null?b.edge_count:edges.filter(e=>!e.synthesized).length} edges`;
  const pill=document.getElementById('lintPill');
  if(b.lint){pill.style.display='inline-flex';pill.innerHTML=`<span class="e">${b.lint.errors}</span> err · <span class="w">${b.lint.warnings}</span> warn`;currentLintFindings=(b.lint&&b.lint.findings)||[];}
  else {pill.style.display='none';currentLintFindings=[];}
  closeLintPop();
}
async function selectBase(b){
  window.__bokfLoading=true;            // agent-visible: a bundle load is in flight
  activeBaseId=b.id;renderSidebar();closeLog();
  // Sync the shared .active-kb pointer so a CLI/agent sees the GUI's selection
  // (fire-and-forget; the poll below mirrors changes the other way).
  if(isDesktop) tauriInvoke('set_active_kb',{id:b.id}).catch(()=>{});
  const bundle=await loadBundle(b);
  pages=bundle.pages||{};
  currentLog=bundle.log||''; currentUpdated=bundle.updated||null; currentLint=bundle.lint||null;
  loadGraph(bundle.graph);
  // merge counts/lint from bundle if base index lacked them
  const merged=Object.assign({}, b, {node_count:bundle.node_count, edge_count:bundle.edge_count, lint:bundle.lint, name:b.name||bundle.name, updated:bundle.updated});
  updateChrome(merged);
  window.__BOKF_READY=true;
  window.__bokfLoading=false;
}

/* ---------- change-log drawer (BioRouter-style history sheet) ---------- */
function openLog(){
  const d=document.getElementById('logDrawer'), s=document.getElementById('logScrim');
  document.getElementById('logBody').innerHTML = (currentLog && currentLog.trim())
    ? renderMd(currentLog)
    : '<div class="empty">No change-log entries yet for this knowledge base.</div>';
  document.getElementById('logMeta').textContent = currentUpdated ? ('updated '+currentUpdated) : '';
  hydrateMdImages(document.getElementById('logBody'));
  d.classList.add('open'); s.classList.add('open');
}
function closeLog(){
  const d=document.getElementById('logDrawer'), s=document.getElementById('logScrim');
  if(d) d.classList.remove('open'); if(s) s.classList.remove('open');
}

/* ---------- lint findings popup (anchored under the lint pill) ---------- */
let currentLintFindings=[];
const SEV_ORDER=[['error','Errors'],['warn','Warnings'],['info','Infos']];
function esc(s){return String(s==null?'':s).replace(/[&<>"]/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;'}[c]));}
function renderLintPop(){
  const pop=document.getElementById('lintPop');
  if(!currentLintFindings.length){ pop.innerHTML='<div class="lp-empty">No issues for this knowledge base.</div>'; return; }
  let h='';
  SEV_ORDER.forEach(([sev,label])=>{
    const rows=currentLintFindings.filter(f=>f.severity===sev);
    if(!rows.length) return;
    h+=`<div class="lp-group">${label}</div>`;
    rows.forEach(f=>{
      h+=`<div class="lp-row"><span class="lp-dot ${sev}"></span><span class="lp-text">`+
         `<span class="lp-rule">${esc(f.rule)}</span><span class="lp-subj">${esc(f.subject)}</span> ${esc(f.message)}`+
         (f.path?`<span class="lp-path">${esc(f.path)}</span>`:'')+`</span></div>`;
    });
  });
  pop.innerHTML=h;
}
function openLintPop(){ renderLintPop(); const pop=document.getElementById('lintPop'), pill=document.getElementById('lintPill'); pop.classList.add('open'); const r=pill.getBoundingClientRect(), pw=pop.offsetWidth||380; let left=r.left+r.width/2-pw/2; left=Math.max(8, Math.min(left, window.innerWidth-pw-8)); pop.style.left=left+'px'; pop.style.top=(r.bottom+6)+'px'; }
function closeLintPop(){ const p=document.getElementById('lintPop'); if(p) p.classList.remove('open'); }
function toggleLintPop(){ document.getElementById('lintPop').classList.contains('open') ? closeLintPop() : openLintPop(); }

document.getElementById('collapseBtn').onclick=()=>{const wb=document.getElementById('wbody'),btn=document.getElementById('collapseBtn');wb.classList.toggle('collapsed');const c=wb.classList.contains('collapsed');btn.textContent=c?'›':'‹';btn.title=c?'Expand':'Collapse';setTimeout(resize,280);};
document.getElementById('legendToggle').onclick=()=>{const lg=document.getElementById('legend');lg.classList.toggle('min');document.getElementById('legendToggle').textContent=lg.classList.contains('min')?'show':'hide';};
const searchInput=document.getElementById('searchInput');searchInput.addEventListener('input',e=>{searchTerm=e.target.value.trim().toLowerCase();});
function zoomBy(f){const cx=W/2,cy=H/2,[wx,wy]=toWorld(cx,cy);view.k=Math.max(0.25,Math.min(5,view.k*f));view.x=cx-W/2-wx*view.k;view.y=cy-H/2-wy*view.k;}
document.getElementById('zoomIn').onclick=()=>zoomBy(1.25);document.getElementById('zoomOut').onclick=()=>zoomBy(0.8);document.getElementById('zoomFit').onclick=()=>fitView();
document.getElementById('logBtn').onclick=()=>{ document.getElementById('logDrawer').classList.contains('open') ? closeLog() : openLog(); };
document.getElementById('logClose').onclick=closeLog;
document.getElementById('logScrim').onclick=closeLog;
previewScrim.onclick=closePreview;
(function(){ const pill=document.getElementById('lintPill');
  pill.addEventListener('click',e=>{ e.stopPropagation(); toggleLintPop(); });
  pill.addEventListener('keydown',e=>{ if(e.key==='Enter'||e.key===' '){ e.preventDefault(); toggleLintPop(); } });
  document.addEventListener('click',e=>{ const pop=document.getElementById('lintPop');
    if(pop.classList.contains('open') && !pop.contains(e.target) && e.target!==pill && !pill.contains(e.target)) closeLintPop(); });
})();
window.addEventListener('keydown',e=>{if(e.key==='Escape'){ if(document.getElementById('lintPop').classList.contains('open'))closeLintPop(); else if(previewEl.classList.contains('open'))closePreview(); else closeLog(); }});
// Cmd/Ctrl+K focuses the search box. On macOS WKWebView swallows Cmd-key combos, so
// the keydown often never fires — a native "Go ▸ Search" menu accelerator emits
// 'menu-search' as the reliable path; this keydown handler covers the other platforms.
window.addEventListener('keydown',e=>{ if((e.metaKey||e.ctrlKey)&&(e.key==='k'||e.key==='K')){ e.preventDefault(); searchInput.focus(); searchInput.select(); } });
if(window.__TAURI__&&window.__TAURI__.event){ window.__TAURI__.event.listen('menu-search',()=>{ searchInput.focus(); searchInput.select(); }); }

/* ---------- integrated terminal (multi-tab xterm.js front-ends + N PTYs) ---------- */
let terms=[], activeTid=null, termSeq=0, termFallbackShown=false;
const termPanel=document.getElementById('termPanel'), termTabsEl=document.getElementById('termTabs'),
      termHostsEl=document.getElementById('termHosts');
const TERM_THEME={ background:'#ffffff', foreground:'#1f242c', cursor:'#0ea5e9', cursorAccent:'#ffffff', selectionBackground:'rgba(14,165,233,0.20)',
  black:'#1f242c', red:'#c4564b', green:'#0c7a5e', yellow:'#9a6b1a', blue:'#3a6ea5', magenta:'#8a4f9e', cyan:'#1f7a8c', white:'#5b5d66',
  brightBlack:'#8e8ea0', brightRed:'#d4685c', brightGreen:'#10a37f', brightYellow:'#b07d3a', brightBlue:'#4f7fb5', brightMagenta:'#9a5fae', brightCyan:'#2e8c84', brightWhite:'#2b3038' };
function termById(id){ return terms.find(t=>t.id===id); }
function showFallback(){
  if(termFallbackShown) return; termFallbackShown=true;
  termHostsEl.innerHTML='<div style="padding:16px;color:var(--muted);font-size:12.5px">The integrated terminal runs in the desktop app.</div>';
}
// togglePanel(): the toolbar Terminal button only collapses/expands; never kills sessions.
function togglePanel(){
  if(termPanel.classList.contains('open')){ termPanel.classList.remove('open'); return; }
  termPanel.classList.add('open');
  if(!isDesktop || typeof Terminal==='undefined'){ showFallback(); return; }
  if(!terms.length) addTab(); else activateTab(activeTid);
}
async function addTab(){
  if(!isDesktop || typeof Terminal==='undefined'){ termPanel.classList.add('open'); showFallback(); return; }
  const n=++termSeq, label='Terminal '+n;
  const host=document.createElement('div'); host.className='term-host'; termHostsEl.appendChild(host);
  const term=new Terminal({
    fontFamily:'ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, monospace',
    fontSize:12.5, lineHeight:1.2, cursorBlink:true, scrollback:5000, theme:TERM_THEME
  });
  const fit=new FitAddon.FitAddon(); term.loadAddon(fit); term.open(host); fit.fit();
  const sess={ id:null, n, label, term, fit, host, unlisten:[] };
  let id;
  try{ id=await tauriInvoke('term_open', { rows: term.rows, cols: term.cols }); }
  catch(e){ term.write('\r\n[could not start shell: '+String((e&&e.message)||e)+']\r\n'); }
  sess.id=id;
  if(id){
    const un1=await window.__TAURI__.event.listen('term-output', ev=>{ if(ev.payload && ev.payload.id===id) term.write(ev.payload.data); });
    const un2=await window.__TAURI__.event.listen('term-exit', ev=>{ if(ev.payload===id) term.write('\r\n[process exited — close this tab and open a new one]\r\n'); });
    sess.unlisten=[un1,un2];
    term.onData(d=>{ if(id) tauriInvoke('term_write', { id, data: d }); });
  }
  terms.push(sess); activeTid=id; renderTabs(); activateTab(id);
}
function activateTab(id){
  const sess=termById(id); if(!sess) return;
  activeTid=id;
  terms.forEach(t=>{ t.host.style.display = (t.id===id)?'block':'none'; });
  try{ sess.fit.fit(); if(sess.id) tauriInvoke('term_resize', { id: sess.id, rows: sess.term.rows, cols: sess.term.cols }); }catch(e){}
  sess.term.focus(); renderTabs();
}
async function closeTab(id){
  const i=terms.findIndex(t=>t.id===id); if(i<0) return;
  const sess=terms[i];
  sess.unlisten.forEach(u=>{ try{ u(); }catch(e){} });
  if(sess.id){ try{ await tauriInvoke('term_close', { id: sess.id }); }catch(e){} }
  try{ sess.term.dispose(); }catch(e){}
  try{ sess.host.remove(); }catch(e){}
  terms.splice(i,1);
  if(!terms.length){ activeTid=null; termPanel.classList.remove('open'); renderTabs(); return; }
  if(activeTid===id){ const next=terms[Math.min(i, terms.length-1)]; activateTab(next.id); }
  else renderTabs();
}
async function closeAll(){
  for(const sess of terms.slice()){
    sess.unlisten.forEach(u=>{ try{ u(); }catch(e){} });
    if(sess.id){ try{ await tauriInvoke('term_close', { id: sess.id }); }catch(e){} }
    try{ sess.term.dispose(); }catch(e){}
    try{ sess.host.remove(); }catch(e){}
  }
  terms=[]; activeTid=null; termPanel.classList.remove('open'); renderTabs();
}
function renderTabs(){
  termTabsEl.innerHTML='';
  terms.forEach(sess=>{
    const tab=document.createElement('div'); tab.className='term-tab'+(sess.id===activeTid?' active':'');
    const lbl=document.createElement('span'); lbl.textContent=sess.label; tab.appendChild(lbl);
    const x=document.createElement('button'); x.className='term-tab-x'; x.title='Close '+sess.label; x.textContent='×';
    x.onclick=ev=>{ ev.stopPropagation(); closeTab(sess.id); };
    tab.appendChild(x);
    tab.onclick=()=>{ if(sess.id!==activeTid) activateTab(sess.id); };
    termTabsEl.appendChild(tab);
  });
}
function fitActive(){ const sess=termById(activeTid); if(sess){ try{ sess.fit.fit(); if(sess.id) tauriInvoke('term_resize', { id: sess.id, rows: sess.term.rows, cols: sess.term.cols }); }catch(e){} } }
document.getElementById('termBtn').onclick=togglePanel;
document.getElementById('termAddTab').onclick=addTab;
document.getElementById('termClose').onclick=closeAll;
window.addEventListener('resize', ()=>{ if(termPanel.classList.contains('open')) fitActive(); });
(function(){ const h=document.getElementById('termResize'); let dragging=false;
  h.addEventListener('mousedown', e=>{ dragging=true; e.preventDefault(); document.body.style.cursor='ns-resize'; });
  window.addEventListener('mousemove', e=>{ if(!dragging) return;
    const main=document.querySelector('.main').getBoundingClientRect();
    let hgt=Math.max(120, Math.min(main.height-70, main.bottom - e.clientY));
    termPanel.style.height=hgt+'px'; fitActive();
  });
  window.addEventListener('mouseup', ()=>{ if(dragging){ dragging=false; document.body.style.cursor=''; fitActive(); } });
})();

function resize(){const rect=cv.getBoundingClientRect();W=rect.width;H=rect.height;DPR=Math.max(1,window.devicePixelRatio||1);cv.width=W*DPR;cv.height=H*DPR;}
window.addEventListener('resize',resize);

/* ---------- live "AI agent" activity banner ----------------------------------
   Every __bokf.* call below is an action driven by an AI agent over the MCP
   control channel. We narrate each one in a flat banner so a human watching the
   GUI sees, in real time, exactly what the agent is doing as it reads the graph.
   (User-driven clicks call the internal selectBase/selectNode directly and are
   NOT narrated — only agent-driven __bokf.* calls are.) */
let aiBannerTimer=null;
function aiNarrate(action){
  window.__bokfLastAction={action, at:Date.now()};
  let el=document.getElementById('aiBanner');
  if(!el){ el=document.createElement('div'); el.id='aiBanner'; el.className='ai-banner'; (document.querySelector('.main')||document.body).appendChild(el); }
  el.innerHTML=`<span class="ai-tag">AI agent</span><span class="ai-act">${esc(action)}</span><span class="ai-dots"><i></i><i></i><i></i></span>`;
  el.classList.add('open');
  clearTimeout(aiBannerTimer); aiBannerTimer=setTimeout(()=>{ el.classList.remove('open'); }, 5000);
}

// expose for the MCP control channel / tests — the agent's window into the GUI.
window.__bokf = {
  // --- generic narration: the MCP server pushes whatever the agent is doing
  //     (linting, merging, building, querying, parsing…) so it shows live here ---
  narrate:(msg)=>{ if(msg!=null && String(msg).trim()) aiNarrate(String(msg)); return true; },
  // --- actions: drive the GUI (visible to the watching user) + narrate ---
  selectBase:(id)=>{const b=BASES.find(x=>x.id===id);if(b){aiNarrate('opening · '+b.name);return selectBase(b);}},
  selectNode:(id)=>{const n=byId[id];if(n){aiNarrate('inspecting node · '+(n.label||n.id));selected=n;selectedEdge=null;recomputeFocus();focusOn(n);showNodeDetail(n);return true;}return false;},
  search:(q)=>{aiNarrate(q?('searching · "'+q+'"'):'clearing search');searchTerm=(q||'').toLowerCase();if(searchInput)searchInput.value=q||'';return true;},
  reload:()=>{ const b=BASES.find(x=>x.id===activeBaseId); if(b){aiNarrate('reloading · '+b.name);return selectBase(b);} return null; },
  // --- observation: the complete app status, so the agent never needs a screenshot ---
  getState:()=>{
    const ab=BASES.find(x=>x.id===activeBaseId)||{};
    const sb=document.getElementById('wbody'), tp=document.getElementById('termPanel');
    return {
      base: activeBaseId,
      baseName: ab.name||null,
      basePath: ab.path||null,
      loading: !!window.__bokfLoading,
      counts:{nodes:nodes.length, edges:edges.length},
      query: searchInput?searchInput.value:'',
      selectedNode: selected?{id:(selected.identifier||selected.id), type:(selected.node_type||selected.type), label:selected.label||null}:null,
      selectedEdge: selectedEdge?{predicate:selectedEdge.predicate, source:selectedEdge.source, target:selectedEdge.target}:null,
      panelOpen: !!(typeof detail!=='undefined' && detail && detail.classList.contains('open')),
      sidebarCollapsed: !!(sb && sb.classList.contains('collapsed')),
      terminalOpen: !!(tp && tp.classList.contains('open')),
      lint: currentLint?{errors:currentLint.errors, warnings:currentLint.warnings, infos:currentLint.infos}:null,
      lastAgentAction: window.__bokfLastAction||null,
      bases: BASES.map(b=>({id:b.id,name:b.name,path:b.path,node_count:b.node_count,edge_count:b.edge_count}))
    };
  },
  getGraph:()=>({nodes: nodes.map(n=>({id:n.id, type:n.type, label:n.label, external:!!n.external, degree:n.degree})), edges: edges.map(e=>({source:e.source, target:e.target, predicate:e.predicate, symmetric:!!e.symmetric, synthesized:!!e.synthesized}))})
};

/* ---------- on-brand toast (flat banner, top-centre of the stage) ---------- */
let toastTimer=null;
function showToast(msg, kind){
  let t=document.getElementById('bokfToast');
  if(!t){ t=document.createElement('div'); t.id='bokfToast'; t.className='toast'; document.querySelector('.main').appendChild(t); }
  t.textContent=msg; t.className='toast'+(kind?' '+kind:'')+' open';
  clearTimeout(toastTimer); toastTimer=setTimeout(()=>{ t.classList.remove('open'); }, 4200);
}

/* ---------- "+ New base": native folder picker -> add_base -> refresh ---------- */
async function addNewBase(){
  if(!isDesktop) return;
  const dlg=window.__TAURI__&&window.__TAURI__.dialog;
  if(!dlg||!dlg.open){ showToast('Folder picker unavailable in this build.','err'); return; }
  let path;
  try{ path=await dlg.open({ directory:true, multiple:false, title:'Select a BioOKF knowledge base folder' }); }
  catch(e){ console.error(e); return; }
  if(path==null) return;                         // user cancelled
  if(Array.isArray(path)) path=path[0];
  try{
    const added=await tauriInvoke('add_base',{path});
    BASES=await loadBases();renderSidebar();
    const b=(added&&added.id&&BASES.find(x=>x.id===added.id)) || BASES.find(x=>x.path===path);
    if(b) await selectBase(b);
  }catch(e){
    showToast('Not a valid BioOKF knowledge base: '+(typeof e==='string'?e:(e&&e.message)||'unknown error'),'err');
  }
}

/* ---------- .active-kb poll: follow a CLI/agent changing the shared pointer ---------- */
let activeKbSyncing=false, aiFocusKb=null;
/* The AI agent setting the active KB (.active-kb, e.g. via bokf_set_active) does
   NOT force the user's view to switch — instead we mark that KB in the sidebar as
   the agent's current focus. When the agent explicitly drives the GUI (selectBase)
   the active KB equals the displayed base, so no separate marker is shown. */
async function pollActiveKb(){
  if(!isDesktop || !BASES.length) return;
  let id; try{ id=await tauriInvoke('get_active_kb'); }catch(e){ return; }
  const focus=(id && id!==activeBaseId && BASES.some(b=>b.id===id)) ? id : null;
  if(focus!==aiFocusKb){ aiFocusKb=focus; renderSidebar(); }
}

/* Re-discover the registry so the sidebar stays true to disk: a KB whose folder
   was deleted/unregistered drops out; one registered elsewhere (CLI `bokf
   register`, an agent, or "+ New base") appears — all without a restart. */
let lastBasesSig='';
function basesSig(arr){ return (arr||[]).map(b=>b.id+':'+(b.node_count||0)+'/'+(b.edge_count||0)+'@'+(b.updated||'')).join('|'); }
async function syncBases(){
  if(!isDesktop || activeKbSyncing || window.__bokfLoading) return;
  let list; try{ list=await tauriInvoke('list_bases'); }catch(e){ return; }
  if(!Array.isArray(list)) return;
  const sig=basesSig(list);
  if(sig===lastBasesSig) return;        // registry unchanged on disk
  lastBasesSig=sig;
  BASES=list; renderSidebar();
  if(activeBaseId && !BASES.find(b=>b.id===activeBaseId)){
    // the active KB's folder was deleted/unregistered — move off it
    if(BASES.length){ activeKbSyncing=true; try{ await selectBase(BASES[0]); } finally{ activeKbSyncing=false; } }
    else { activeBaseId=null; }
  }
}

async function boot(){
  renderLegend();resize();
  const nb=document.querySelector('.new-kb');
  if(nb){ if(isDesktop) nb.onclick=addNewBase; else nb.style.display='none'; }
  BASES=await loadBases();renderSidebar();
  lastBasesSig=basesSig(BASES);
  if(BASES.length)await selectBase(BASES[0]);
  if(isDesktop){ setInterval(pollActiveKb, 1500); setInterval(syncBases, 3000); }
  requestAnimationFrame(loop);
}
boot();
