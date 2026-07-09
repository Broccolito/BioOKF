export const meta = {
  name: 'spoke-biookf-adjudicate',
  description: 'LLM adjudication of the BioOKF->SPOKE review queue (ambiguous / not_found nodes)',
  phases: [{ title: 'Adjudicate', detail: 'one agent per batch of hard nodes' }],
};

// args = { queueFile: '<abs path to {queue:[...]}>', total: <int>, batchSize?: <int> }
// (args can arrive as a JSON string; parse defensively.)
const A = typeof args === 'string' ? JSON.parse(args) : (args || {});
const WT = '/Users/wgu/Desktop/BioOKF-spoke-mapper';
const CLI = WT + '/fabric/spoke/spoke-cli';
const queueFile = A.queueFile;
const total = A.total || 0;
const B = A.batchSize || 18;

const DECISION_SCHEMA = {
  type: 'object', additionalProperties: false, required: ['decisions'],
  properties: {
    decisions: {
      type: 'array',
      items: {
        type: 'object', additionalProperties: false,
        required: ['id', 'decision', 'confidence', 'reason'],
        properties: {
          id: { type: 'string' },
          decision: { type: 'string', enum: ['map', 'none'] },
          spoke_label: { type: 'string' },
          spoke_identifier: { type: 'string' },
          spoke_name: { type: 'string' },
          confidence: { type: 'number' },
          reason: { type: 'string' },
        },
      },
    },
  },
};

function prompt(start, end) {
  return `You are a biomedical entity-resolution adjudicator. Resolve BioOKF nodes to SPOKE nodes.
These failed the deterministic waterfall, so they need judgment.

STEP 1 — load your batch of nodes (a slice of the review queue):
  python3 -c "import json; q=json.load(open('${queueFile}'))['queue'][${start}:${end}]; print(json.dumps(q, indent=1))"
Each node has: id (the BioOKF name), type, subtype, synonyms, candidate_labels (allowed SPOKE
labels), candidates (SPOKE nodes the fulltext tier surfaced — often WRONG token matches).

STEP 2 — for EACH node, decide. You MAY run read-only SPOKE queries to confirm or find a match:
  ${CLI} query "<CYPHER>" --stdout
  - Scalars only: RETURN n.identifier AS id, n.name AS name, n.synonyms AS syn
  - Exact (indexed labels Gene/Disease/Compound/Anatomy/CellType/Symptom/SideEffect/Pathway/
    BiologicalProcess/MolecularFunction/Organism/PharmacologicClass/Protein/Food/Reaction):
      MATCH (n:\`Disease\` {name:'...'}) RETURN n.identifier AS id, n.name AS name
  - Fulltext (any biomedical label; sanitize special chars +-():><*? to spaces first):
      CALL db.index.fulltext.queryNodes('DiseaseNamesAndIds','<terms>') YIELD node,score
      RETURN node.identifier AS id, node.name AS name, node.synonyms AS syn, score ORDER BY score DESC LIMIT 10
  - AVOID the word-boundary tokens create/merge/delete/remove/set/drop inside a query string
    (the CLI blocks them) — split them like 'd'+'rop' if a name contains one.

DECISION RULES:
- "map" ONLY when confident: the SPOKE node is the SAME entity (name/synonym/semantic equivalence)
  AND label-appropriate. Set spoke_label + spoke_identifier + spoke_name.
  Examples of GOOD maps: "type 1 diabetes"->DOID:9744 "type 1 diabetes mellitus";
    "adriamycin"->the Doxorubicin Compound. Pick the CLOSEST specific match, not a broader parent.
- REJECT bad fulltext candidates that merely share tokens (e.g. "intervertebral disc" is NOT
  "intercalated disc"; "Atorvastatin lactone" is NOT "atorvastatin"; a specific subtype should not
  map to a broad parent unless no specific node exists).
- "none" if the entity is genuinely absent from SPOKE, is provenance/context, or you cannot
  confidently disambiguate. DO NOT guess. For non-rsID variants (e.g. "C282Y","+2740 A>G") SPOKE is
  keyed by dbSNP rsID, so "none" unless you can confirm a specific Variant node.
- spoke_identifier MUST be an exact SPOKE identifier you have SEEN in a query result or the
  candidates list — never invented.

Return ONLY the decisions object (one entry per node in your batch).`;
}

phase('Adjudicate');
log('args seen: ' + JSON.stringify(args));
if (!queueFile || !total) {
  log('no queueFile/total provided; nothing to adjudicate');
  return { decisions: [] };
}
const nBatches = Math.ceil(total / B);
log(`adjudicating ${total} nodes in ${nBatches} batches of <=${B}`);
const jobs = [];
for (let i = 0; i < nBatches; i++) jobs.push([i * B, Math.min((i + 1) * B, total)]);
const results = await parallel(jobs.map(([s, e], i) => () =>
  agent(prompt(s, e), { label: `adj:${i}`, phase: 'Adjudicate', schema: DECISION_SCHEMA })));
const decisions = (results || []).filter(Boolean).flatMap(r => r.decisions || []);
const mapped = decisions.filter(d => d.decision === 'map').length;
log(`adjudication done: ${mapped}/${decisions.length} resolved to a SPOKE node`);
return { decisions };
