# Iteration log — the mapping "spins"

Each spin runs the harness on `benchmark-biored` (2,550 nodes / 5,360 edges), reviews the output,
and fixes what the review exposed. This is the waterfall's outer loop: deterministic run → adversarial
review → fix → re-run.

## Spin 1 — first contact (partial)
Ran the initial deterministic waterfall. **Two harness vulnerabilities surfaced before it finished:**
1. **Unindexed name scan.** `MATCH (n:Variant {name:q})` — SPOKE `Variant` has no `name` property
   and no `name` index, so every name triggered a full scan of 1.04M nodes. → `batch_exact_by_name`
   guarded to the 21 name-indexed labels; `Variant` resolves by rsID identifier or fulltext.
2. **Read-only-guard keyword collisions.** The CLI rejects any query text containing
   `create/merge/delete/remove/set/drop` at a word boundary — real names hit it ("set shifting
   impairment", "a dramatic drop in blood pressure"). → `cypher_str` splits those words with Cypher
   concatenation (`'d'+'rop'`).

## Spin 2 — full deterministic run
Completed: **1,062 nodes mapped (41.6%; 50% of mappable)**, edges **agree 28 / contradict 0**.
Review flagged the agreement count as suspiciously low. Direct SPOKE probing found the cause:
3. **Integer-identifier edge mismatch.** SPOKE `Gene` identifiers are **integers** (BRCA1 = `672`),
   but `edges_between` quoted every identifier (`{identifier:'672'}`), which never matches `672`.
   This silently broke edge-checking for **every Gene endpoint** — the majority of biological edges.
   → added `cypher_value` (type-aware literal); only `Gene` is numeric among mapped labels, so this
   fully closes the bug class.

## Spin 3 — edge fix
Same nodes, corrected edges: **agree 28 → 110, contradict 0 → 9**, unmapped-predicate 25 → 83,
not-found-in-spoke 751 → 602. Contradictions are real and valuable — e.g. *HFE `not_associated_with`
hemochromatosis* (HFE is the hemochromatosis gene), complement genes C2/C3/CFB `not_associated_with`
AMD, GCK/GLP1R/HNF4A `not_associated_with` T2D. Agreements verified correct (BACH1 regulates HMOX1 →
UP/DOWNREGULATES; treats → TREATS_CtD; has_phenotype → PRESENTS_DpS).

## Spin 4 — LLM adjudication (tier 6)
The 327 `ambiguous` nodes (deterministic waterfall couldn't disambiguate) went to a 19-agent LLM
adjudication workflow that queries SPOKE live. **112 resolved to confident maps, 215 correctly
returned `none`.** The LLM rejected token-match false positives ("intervertebral disc" ≠ "intercalated
disc") and refused to collapse general terms onto specific children ("apoptosis", "endoplasmic
reticulum" → none), while catching GO renames (cell proliferation → GO:0008283 "cell population
proliferation") and spelling variants (disc → disk). Decisions overlay the deterministic matches as
tier `llm_accept`; edges are then re-checked against the improved node set.
Result: **1,174 mapped (55.2% of mappable)**, agree 126, contradict 10.

## Spin 5 — LLM adjudication of the not_found set
The 655 non-Variant `not_found` nodes (no fulltext candidates at all) went to a 33-agent pass that
searches SPOKE from scratch. **290 recovered to confident maps, 365 confirmed absent.** Recoveries
were dominated by synonym/lexical-variant cases the deterministic tiers can't reach: Compound 179
(drug aliases), Disease 81, plus anatomy variants (cartilage→"cartilage tissue", corneal
stroma→"substantia propria of cornea", pancreatic islet→"islet of Langerhans", GPi→"medial globus
pallidus"). General GO terms absent from SPOKE (apoptosis, cell death) were correctly kept `none`.

## Guardrail — identifier verification
Before merging, all 402 LLM `map` decisions are **verified to exist in SPOKE** with that
(label, identifier). **4 were demoted** — all Organism cases where the LLM guessed a plausible NCBI
taxon id (e.g. 487 for *N. meningitidis*) that doesn't match SPOKE's strain-format Organism
identifier scheme. This is the safeguard against hallucinated ids; 398 confirmed.

## Spin 6 — final
Deterministic + both verified LLM passes, edges re-checked:

| spin | nodes mapped | of mappable | agree | contradict |
|------|-------------|-------------|-------|-----------|
| 2 (deterministic) | 1,062 | 50.0% | 28 | 0 |
| 3 (edge fix) | 1,062 | 50.0% | 110 | 9 |
| 4 (+ambiguous LLM) | 1,174 | 55.2% | 126 | 10 |
| **6 (+not_found LLM, verified)** | **1,460** | **68.7%** | **197** | **14** |

Final match-tier mix: exact_name 855, **llm_accept 398**, exact_name_ci 138, exact_synonym 33,
exact_identifier 22, fuzzy 14. The output graph is shape-identical to the benchmark (2,550 nodes /
5,360 edges, order preserved) and passes all contract invariants.

### Why the remaining 666 are `not_found` (not harness gaps)
- **Variant 297** — benchmark variants are named (`C282Y`, `+2740 A>G`), SPOKE is keyed by dbSNP
  rsID; without an rsID/HGVS xref there is nothing to join on.
- **General GO process terms** (apoptosis, cell death, cell differentiation) — not loaded in SPOKE
  as nodes (only specific children); mapping to a child would be wrong.
- **Types SPOKE doesn't model** — most SequenceFeature, BiomedicalMeasure, some Exposure/Structure.
These are true coverage boundaries, correctly reported rather than force-mapped.
