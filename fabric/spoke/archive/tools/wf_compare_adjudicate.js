export const meta = {
  name: 'spoke-compare-adjudicate',
  description: 'SPOKE-grounded adjudication of Claude-vs-Codex mapping disagreements + unique maps',
  phases: [{ title: 'Adjudicate', detail: 'verify which mapping is correct, grounded in SPOKE' }],
};

const A = typeof args === 'string' ? JSON.parse(args) : (args || {});
const CLI = '/Users/wgu/Desktop/BioOKF-spoke-mapper/fabric/spoke/spoke-cli';
const queueFile = A.queueFile;
const total = A.total || 0;
const B = A.batchSize || 12;

const SCHEMA = {
  type: 'object', additionalProperties: false, required: ['verdicts'],
  properties: {
    verdicts: {
      type: 'array',
      items: {
        type: 'object', additionalProperties: false,
        required: ['id', 'kind', 'verdict', 'reason'],
        properties: {
          id: { type: 'string' },
          kind: { type: 'string', enum: ['disagreement', 'codex_only', 'claude_only'] },
          verdict: {
            type: 'string',
            enum: ['claude_correct', 'codex_correct', 'both_valid', 'both_wrong', 'unclear'],
            description: 'disagreement: who picked the right SPOKE node. codex_only/claude_only: ' +
                         'claude_correct=the sole map is right, codex_correct=the sole map is right, ' +
                         'both_wrong=the sole map is a false positive, both_valid=defensible.',
          },
          correct_identifier: { type: 'string', description: 'the SPOKE identifier you judge correct (or empty if none)' },
          reason: { type: 'string', description: 'one line, cite the SPOKE name/desc you saw' },
        },
      },
    },
  },
};

function prompt(start, end) {
  return `You are an impartial biomedical entity-resolution judge. Two mapping systems (Claude, Codex)
linked BioOKF nodes to SPOKE nodes. Decide, grounded in live SPOKE, which mapping is correct. Be
strictly neutral — do not favor either system.

STEP 1 — load your batch:
  python3 -c "import json; q=json.load(open('${queueFile}'))['queue'][${start}:${end}]; print(json.dumps(q, indent=1))"
Each item: id (BioOKF name), type, subtype, synonyms, and up to two proposed mappings:
  claude={label,identifier} and/or codex={label,identifier}. kind tells you the case:
  - "disagreement": both mapped to DIFFERENT SPOKE ids — decide which id is the correct entity.
  - "codex_only": only Codex mapped — is that mapping correct or a false positive?
  - "claude_only": only Claude mapped — is that mapping correct or a false positive?

STEP 2 — verify each proposed SPOKE node exists and inspect what it actually is:
  ${CLI} query "MATCH (n:\`Gene\`) WHERE n.identifier IN [367,231] RETURN n.identifier AS id, n.name AS name, n.description AS desc, n.synonyms AS syn" --stdout
  (Gene identifiers are integers — no quotes. Disease/Compound/etc. are strings — quote them.)
  Check: does the SPOKE node's name/description/synonyms actually denote the SAME entity as the
  BioOKF node? For genes, the CANONICAL gene is the one whose n.name equals the official symbol
  (e.g. "AR"=androgen receptor id 367), NOT a different gene that merely lists the symbol as an alias
  (AKR1B1 id 231 lists "AR" as a synonym — that is a WRONG match). For a generic name ("cyclooxygenase",
  "PI3K", "superoxide dismutase") that denotes a FAMILY, mapping to one specific gene is a false
  positive unless the BioOKF context clearly means that gene. For diseases/compounds, prefer the node
  that is the same specific entity (watch parent-vs-child and salt/lactone/stereoisomer differences).

STEP 3 — verdict per item:
  - disagreement -> claude_correct | codex_correct | both_valid (truly equivalent nodes, e.g. duplicate
    Compound records for the same molecule) | both_wrong | unclear. Put the right id in correct_identifier.
  - codex_only / claude_only -> claude_correct/codex_correct (the map is right), both_wrong (false
    positive), both_valid (defensible but loose), or unclear.
  reason: ONE line citing the SPOKE name/desc you saw (e.g. "id 231 is AKR1B1, 'AR' only an alias").

Return ONLY the verdicts object, one entry per item in your batch.`;
}

phase('Adjudicate');
if (!queueFile || !total) { log('no queue'); return { verdicts: [] }; }
const n = Math.ceil(total / B);
log(`adjudicating ${total} comparison cases in ${n} batches`);
const jobs = [];
for (let i = 0; i < n; i++) jobs.push([i * B, Math.min((i + 1) * B, total)]);
const res = await parallel(jobs.map(([s, e], i) => () =>
  agent(prompt(s, e), { label: `judge:${i}`, phase: 'Adjudicate', schema: SCHEMA })));
const verdicts = (res || []).filter(Boolean).flatMap(r => r.verdicts || []);
log(`verdicts: ${verdicts.length}`);
return { verdicts };
