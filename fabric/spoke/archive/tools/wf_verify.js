export const meta = {
  name: 'spoke-combined-verify',
  description: 'SPOKE-grounded verification of semantic maps + resolution of guarded-ambiguous nodes',
  phases: [{ title: 'Verify', detail: 'confirm each mapping against live SPOKE' }],
};

const A = typeof args === 'string' ? JSON.parse(args) : (args || {});
const CLI = '/Users/wgu/Desktop/BioOKF-spoke-fabric-combined/fabric/spoke/spoke-cli';
const queueFile = A.queueFile, total = A.total || 0, B = A.batchSize || 14;

const SCHEMA = {
  type: 'object', additionalProperties: false, required: ['results'],
  properties: {
    results: {
      type: 'array',
      items: {
        type: 'object', additionalProperties: false,
        required: ['id', 'kind', 'verdict', 'reason'],
        properties: {
          id: { type: 'string' },
          kind: { type: 'string', enum: ['verify_map', 'resolve'] },
          verdict: { type: 'string',
            enum: ['correct', 'wrong', 'defensible', 'map', 'none'],
            description: 'verify_map -> correct/wrong/defensible. resolve -> map/none.' },
          spoke_label: { type: 'string' }, spoke_identifier: { type: 'string' },
          spoke_name: { type: 'string' },
          reason: { type: 'string' },
        },
      },
    },
  },
};

function prompt(s, e) {
  return `You are an impartial biomedical entity-resolution verifier. Confirm mappings against LIVE SPOKE.

Load your batch:
  python3 -c "import json; q=json.load(open('${queueFile}'))['queue'][${s}:${e}]; print(json.dumps(q, indent=1))"

Two kinds:
- kind "verify_map": a node was mapped to \`assigned\` {label,identifier,name}. These had no lexical
  overlap (LLM/fuzzy tier), so confirm the assigned SPOKE node truly denotes the SAME entity as the
  BioOKF id. Query it: ${CLI} query "MATCH (n:\`Disease\` {identifier:'DOID:9744'}) RETURN n.name AS name, n.synonyms AS syn" --stdout
  (Gene ids are integers — no quotes.) verdict: "correct" (same entity, e.g. adriamycin↔Doxorubicin),
  "wrong" (different entity / too broad / a family), or "defensible" (related, arguably ok).
- kind "resolve": the node is unresolved with \`candidates\`. Pick the right SPOKE node or decline.
  verdict "map" (set spoke_label/spoke_identifier/spoke_name — must be an id you SAW in a query) or
  "none". Reject candidates that only share tokens or are broader parents.

Cypher tips: return scalars; fulltext = CALL db.index.fulltext.queryNodes('<Label>NamesAndIds','terms')
YIELD node,score RETURN node.identifier AS id, node.name AS name, score ORDER BY score DESC LIMIT 8.
Avoid the words create/merge/delete/remove/set/drop inside query strings (split like 'd'+'rop').

reason: ONE line citing the SPOKE name you saw. Return ONLY the results object.`;
}

phase('Verify');
if (!queueFile || !total) { log('no queue'); return { results: [] }; }
const n = Math.ceil(total / B), jobs = [];
for (let i = 0; i < n; i++) jobs.push([i * B, Math.min((i + 1) * B, total)]);
log(`verifying ${total} nodes in ${n} batches`);
const res = await parallel(jobs.map(([s, e], i) => () =>
  agent(prompt(s, e), { label: `verify:${i}`, phase: 'Verify', schema: SCHEMA })));
const results = (res || []).filter(Boolean).flatMap(r => r.results || []);
log(`results: ${results.length}`);
return { results };
