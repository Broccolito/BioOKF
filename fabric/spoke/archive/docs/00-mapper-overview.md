# BioOKF → SPOKE Mapper

Maps an LLM-curated **BioOKF** knowledge graph onto **SPOKE** (UCSF Baranzini Lab), annotating every
node with its SPOKE identity and every edge with whether SPOKE agrees, contradicts, or is silent —
**without dropping anything**. The output graph is shape-identical to the input; downstream users
filter on the annotations.

> Built in an isolated worktree (`codex/spoke-biookf-mapper`), independent of the
> `codex/spoke-fabric-linker` effort — designed from a fresh 25-agent research pass, reusing none of
> its files.

## Result on `benchmark-biored` (2,550 nodes / 5,360 edges)

| | value |
|---|---|
| nodes mapped | **1,460 / 2,550** (57.3% overall, **68.7% of mappable**) |
| edge agreements | **197** |
| edge contradictions | **14** (e.g. HFE `not_associated_with` hemochromatosis vs SPOKE ASSOCIATES_DaG) |
| output shape | identical to benchmark (2,550 / 5,360, order preserved) — all invariants pass |
| match tiers | exact_name 855 · llm_accept 398 · exact_name_ci 138 · exact_synonym 33 · exact_id 22 · fuzzy 14 |

Per-type: Anatomy 94%, Gene 92%, Molecule 90%, BiologicalPathway 90%, CellType 81%, Disease 72%,
Phenotype 63%. (Variant 7% and BiomedicalMeasure 0% are true coverage boundaries — see the log.)

## How it works — a confidence waterfall (deterministic first, LLM for the residue)

1. **Research** (`schema/research/corpus.json`) characterized all 42 SPOKE labels, 129 reltypes, and
   63 BioOKF types/predicates from real nodes/edges → the crosswalks.
2. **Deterministic tiers** (`linker/resolver.py`): exact identifier → exact name → case-insensitive
   name → synonym → gated fuzzy. Index-backed, high precision.
3. **LLM adjudication** (`tools/wf_adjudicate.js`, tier `llm_accept`): the `ambiguous`/`not_found`
   residue is handed to subagents that query SPOKE live and decide — rejecting token-match false
   positives and refusing to collapse general terms onto specific children.
4. **Guardrail** (`tools/verify_adjudications.py`): every LLM identifier is verified to exist in
   SPOKE before it is accepted (4 hallucinated ids were caught and demoted).
5. **Edge agreement** (`linker/resolver.py::EdgeChecker`): for edges with both endpoints mapped,
   look for a SPOKE relationship in the predicate's family → agrees / contradicts (for `not_*`) /
   found-but-different-predicate / not-found.

## Layout

```
linker/     the harness: crosswalk YAMLs + engine (see linker/README.md)
mcp/        MCP server (spoke_lookup_node, spoke_check_edge, spoke_link_graph, spoke_query)
bin/        spoke-cli (SPOKE query CLI) + bokf-spoke-link (mapper CLI)
docs/       01 lookup mechanics · 02 annotation contract · 03 iteration log
schema/     SPOKE schema skeleton + research corpus
tools/      recon, research/adjudication workflows, QA reviewer, verifier, summarizer
runs/benchmark-biored/
            biookf-export.json (input)  ·  spin6-final-annotated.json (DELIVERABLE)
            spin6-final-report.json  ·  adjudication-all-verified.json
```

## Reproduce

```bash
# deterministic + verified LLM decisions -> final annotated graph
bin/bokf-spoke-link link runs/benchmark-biored/biookf-export.json \
  --out out.json --report report.json \
  --adjudications runs/benchmark-biored/adjudication-all-verified.json --validate

python3 tools/summarize.py runs/benchmark-biored/spin6-final-annotated.json   # cross-spin summary
python3 tools/review_mappings.py runs/benchmark-biored/spin6-final-annotated.json   # QA triage
```

The two LLM passes are regenerated with `tools/wf_research.js` / `tools/wf_adjudicate.js` (multi-agent
workflows). See `docs/03-iteration-log.md` for the full spin-by-spin history and the three harness
bugs found and fixed along the way.
