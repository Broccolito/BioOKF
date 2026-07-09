export const meta = {
  name: 'spoke-combined-verify-edges',
  description: 'SPOKE-grounded verification of edge classifications (agrees / contradicts / related)',
  phases: [{ title: 'VerifyEdges', detail: 'confirm each edge verdict against live SPOKE' }],
};

const A = typeof args === 'string' ? JSON.parse(args) : (args || {});
const CLI = '/Users/wgu/Desktop/BioOKF-spoke-fabric-combined/fabric/spoke/spoke-cli';
const queueFile = A.queueFile, total = A.total || 0, B = A.batchSize || 12;

const SCHEMA = {
  type: 'object', additionalProperties: false, required: ['verdicts'],
  properties: {
    verdicts: {
      type: 'array',
      items: {
        type: 'object', additionalProperties: false,
        required: ['index', 'verdict', 'reason'],
        properties: {
          index: { type: 'integer', description: 'the item index within your batch slice (0-based)' },
          verdict: { type: 'string', enum: ['correct', 'wrong', 'borderline'] },
          reason: { type: 'string' },
        },
      },
    },
  },
};

function prompt(s, e) {
  return `You are an impartial biomedical knowledge-graph auditor. A combined mapper classified each
BioOKF edge against SPOKE. Confirm the classification is right, grounded in LIVE SPOKE.

Load your batch (note the 0-based index of each within THIS slice):
  python3 -c "import json; q=json.load(open('${queueFile}'))['queue'][${s}:${e}]; [print(i, json.dumps(x)) for i,x in enumerate(q)]"

Each item: status (the mapper's verdict), source/predicate/target (BioOKF), spoke_reltypes (what SPOKE
has between the two mapped nodes), src_spoke/tgt_spoke (the SPOKE identifiers).

Confirm the SPOKE relationship exists and supports the status:
  ${CLI} query "MATCH (a)-[r]-(b) WHERE a.identifier=<src> AND b.identifier=<tgt> RETURN type(r) AS rt LIMIT 20" --stdout
  (Gene ids are integers, no quotes; others are strings, quote them.)

status meanings to check:
- "agrees_with_spoke": a SPOKE relationship of the SAME family as the BioOKF predicate connects the
  two nodes (e.g. treats -> TREATS_CtD; part_of -> MAPS_VmG/PARTICIPATES_GpCC; has_phenotype ->
  PRESENTS_DpS). verdict "correct" if the reltype genuinely means that predicate between these entities.
- "contradicts_spoke": the BioOKF claim is a not_* negative, yet SPOKE positively asserts the base
  relation. "correct" if SPOKE truly asserts the positive (e.g. HFE not_associated_with hemochromatosis
  but SPOKE ASSOCIATES_DaG).
- "related_in_spoke": SPOKE has a RELATED but not identical relationship (causes vs ASSOCIATES; binds
  vs UP/DOWNREGULATES; regulates vs PARTICIPATES). "correct" if it's genuinely related-not-identical,
  "wrong" if it's actually the same family (should be agrees) or unrelated.

verdict per item: "correct", "wrong" (misclassified), or "borderline". index = its position in your
slice. reason = ONE line. Return ONLY the verdicts object.`;
}

phase('VerifyEdges');
if (!queueFile || !total) { log('no queue'); return { verdicts: [] }; }
const n = Math.ceil(total / B), jobs = [];
for (let i = 0; i < n; i++) jobs.push([i * B, Math.min((i + 1) * B, total), i]);
log(`verifying ${total} edge classifications in ${n} batches`);
const res = await parallel(jobs.map(([s, e, bi]) => () =>
  agent(prompt(s, e), { label: `edge:${bi}`, phase: 'VerifyEdges', schema: SCHEMA })
    .then(r => ({ base: s, r }))));
// remap batch-local index -> global index
const verdicts = [];
for (const item of (res || []).filter(Boolean)) {
  for (const v of (item.r.verdicts || [])) verdicts.push({ global: item.base + v.index, ...v });
}
log(`edge verdicts: ${verdicts.length}`);
return { verdicts };
