# Archive — the Python lineage of the SPOKE fabric

Provenance for `fabric/spoke`. The BioOKF → SPOKE mapping fabric was first built three times in
Python, then ported to the native Rust `spoke-fabric` that now lives one directory up. The Python
source is gone; **this directory holds the parts of it that the Rust port does not carry.**

Nothing here is built, run, or imported. It is documentation and reusable agent workflows.

## Why this exists

The Rust port supersedes the Python **code** completely — it reproduces every resolution tier and
adds three the Python never had (a token-specificity disambiguation guard, Tier-6 protein
recommended-name recovery, and viral taxonomic `name_aliases`). `data/type_crosswalk.yaml` and
`data/predicate_crosswalk.yaml` are supersets of the Python crosswalks, and
`data/curation_overrides.json` (1,005 decisions) strictly contains the Python lineage's 982.

What the port did **not** carry is what you find here: the measured precision, the curation process
that produced those 1,005 decisions, the SPOKE research corpus the crosswalks were authored from,
and the rationale for the design.

## The three lineages

| lineage | what it was | fate |
|---|---|---|
| **spoke-biookf-mapper** ("Claude") | first linker; LLM adjudication + canonical-name resolution | folded into *combined* |
| **spoke-fabric-linker** ("Codex") | independent single-file linker; synonym-as-identifier, audit tooling, richer edge evidence | folded into *combined*; `w1-*.md` docs below |
| **spoke-fabric-combined** | the merge of both, and the direct source of the Rust port | superseded by `../src/*.rs` |

`docs/04-comparison-claude-vs-codex.md` is the head-to-head evaluation that decided the merge.

## Contents

- **`docs/round-metrics.md`** — the only record of *measured precision* in this project. Per-round
  node coverage and precision (converged at 1,464 mapped, 99.9% true precision over a 206-node
  SPOKE-grounded audit), edge-classification accuracy (107/109 = 98%, ~99% after the RESEMBLES fix),
  the 128-organism test (118/128 mapped, 100% verified precision), the fulltext-starvation bug
  postmortem, and an honest list of six standing limitations. `../docs/BENCHMARK.md` records
  coverage and speed only — **precision has never been re-measured for the Rust engine.**
- **`tools/wf_*.js`** — the LLM curation *process*. `../data/curation_overrides.json` is this
  process's frozen output; these are the scripts that produced it. They are Claude Code Workflow
  scripts (they call `agent()` / `parallel()` / `phase()` / `log()`) and still run today. Without
  them, mapping a new knowledge base means rewriting the adjudication loop from scratch: the Rust
  emits a review queue and will apply and verify a decisions file, but nothing in the repo turns the
  queue *into* that file.
- **`schema/research/corpus.json`** — 42 SPOKE node labels with their identifier schemes (e.g. SPOKE
  `Gene.identifier` is a bare Entrez integer, not a CURIE), 129 relationship types with meanings,
  properties, sources and examples, and 63 BioOKF types/predicates with definitions. Authored by
  `tools/wf_research.js` against live SPOKE. Not regenerable by a single query — the crosswalks were
  written from it.
- **`docs/01-lookup-mechanics.md`** — empirically verified SPOKE index inventory (RANGE on `name` for
  only 21 labels; per-label `<Label>NamesAndIds` fulltext). The Rust encodes this as
  `FALLBACK_NAME_INDEXED` in `../src/client.rs`; here is the evidence for it.
- **`docs/02-annotation-contract.md`** — the KG-agnostic contract. The annotation field vocabulary is
  deliberately not SPOKE-specific: a BioOKF→PrimeKG fabric would emit the same field names under a
  `node.primekg` key. Read this before adding a second target graph.
- **`docs/05-guardrails.md`**, **`docs/03-iteration-log.md`**, **`docs/28-type-stress-test.md`**,
  **`docs/non-disease-types.md`**, **`docs/00-mapper-overview.md`** — largely superseded by
  `../docs/BENCHMARK.md` and the Rust source, retained for the reasoning behind them.
- **`docs/w1-*.md`** — the Codex lineage's design and SPOKE schema research.
- **`docs/comparison-report.html`**, **`docs/mapping-story.html`** — narrative reports.

## Findings that lived only in the discarded Python source

**The disambiguation guard's justification** (`linker/resolver.py:298-302`), quoted verbatim because
the file no longer exists:

> Only auto-accept a synonym match when it is unambiguous and specific. Ambiguous / short-acronym /
> broad-parent matches are routed to the LLM/review tier, which disambiguates them far more reliably
> (**measured 96% precision, 0 false positives**) than a deterministic alias match (Codex's alias
> errors: `cyclooxygenase` → COX8A, `PI3K`).

The Rust implements this guard in `../src/resolver.rs` and adds a fourth condition, but records
neither the measurement nor the two failure cases that motivated it.

**Per-tier confidence constants**, deliberately dropped from the Rust: the Python carried
`exact_identifier 1.0 · exact_name 0.98 · exact_name_ci 0.95 · exact_synonym 0.90 ·
fuzzy_accept 0.72 · llm_accept 0.80`, used only to drive the fuzzy gate and never emitted. The Rust
drops self-reported confidence entirely — acceptance is structural (the reviewer names a target; the
harness re-verifies the id exists). See `../src/annotate.rs`.

## Recovering the discarded Python

The full Python trees, their run artifacts and the legacy `bin/spoke-cli` binary were removed with
the `codex/*` branches. Until git garbage-collects them they remain reachable by SHA:

```
codex/spoke-fabric-combined   2858c75    modular Python package + tools + docs
codex/spoke-biookf-mapper     a889b02    earlier Claude-lineage Python
codex/spoke-fabric-linker     aac0bfd    single-file spoke_fabric.py, run artifacts, bin/spoke-cli
```

`git show 2858c75:fabric/spoke/linker/resolver.py`, or `git worktree add /tmp/x 2858c75` for the
full tree. These SHAs are unreferenced; do not rely on them surviving a `git gc --prune`.
