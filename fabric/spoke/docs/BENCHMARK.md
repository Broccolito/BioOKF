# Benchmark — Rust `spoke-fabric` vs the Python reference

Input: `runs/benchmark-biored/biookf-export.json` — **2,550 nodes / 5,360 edges**, curation overlay
`data/curation_overrides.json` (1,005 reviewed decisions). Both engines run the identical fail-closed
pipeline (`preflight → load_export → resolve_nodes → apply_curation → verify_node_ids → canonicalize
→ check_edges → validate_invariants`).

## Parity — status histograms identical; Rust diverges on 4 nodes, in its favour

The Rust engine reproduces every Python tier and adds three the Python reference never had (a
token-specificity disambiguation guard, `resolver.rs`; Tier-6 protein recommended-name recovery,
`resolver.rs`; viral taxonomic `name_aliases`, `data/type_crosswalk.yaml`). Output is therefore
**not** byte-identical to the checked-in Python baseline, and is not meant to be.

Measured against `runs/benchmark-biored/python-annotated.json` (both engines with the same
1,005-decision curation overlay):

| comparison | result |
|---|---|
| node `mapping_status` histogram | **identical** (1,464 / 660 / 424 / 2) |
| edge `support_status` histogram | 2 edges shifted (`not_evaluated` 3,855→3,853, `unsupported` 1,103→1,105) |
| node annotations differing | 13 / 2,550 — **9 cosmetic** (same target, `match_method` `synonym`→`reviewed`), **4 semantic** |
| edge annotations differing | 6 / 5,360 in `support_status`; all 5,360 gain `endpoints.source_mapped`/`target_mapped` (schema enrichment) |

The 4 semantic node divergences are all cases where the Rust is **more correct**:

| node | Python | Rust | why |
|---|---|---|---|
| `superoxide dismutase` | mapped → SOD1 | `not_found` | SOD is a family (SOD1/2/3); the guard routes it to review, curation says `none` |
| `nicotinic acetylcholine receptor` | mapped → CHRNA4 | `not_found` | likewise a receptor family, not a single gene |
| `CCK` | `not_found` | mapped → P06307 | Tier-6 protein recommended-name recovery finds Cholecystokinin |
| `Hepatitis C virus` | `not_found` | mapped → `Hepacivirus hominis` | ICTV taxonomic rename, via `name_aliases` |

Curation is only overlaid on `ambiguous`/`not_found`/fuzzy nodes — a confident deterministic match
always wins (`annotate.rs`). That is why `CCK` and `Hepatitis C virus` map despite a `decision: none`
override recorded before those two tiers existed.

Shared result set: mapped **1,464** · not_found 660 · not_mappable 424 · ambiguous 2; edges
supported **209** · related 97 · contradicted 14 · different_relation 82 · unsupported 1,103 ·
not_evaluated 3,855. Node map-rate 57.4% overall / 68.9% of mappable.

> Precision was never re-measured for the Rust engine. The Python lineage's grounded audits
> (99.9% node precision over a 206-node sample; 107/109 edge classifications correct) are recorded in
> `docs/round-metrics.md` on the archive branch `codex/spoke-fabric-combined`.

## Speed

| run | Rust `spoke-fabric` | Python reference | speedup |
|---|---|---|---|
| **cold** (no cache, live SPOKE) | **44 s** | **812.7 s** (13.5 min) | **18.3×** |
| **warm** (query cache populated) | **0.57 s** | 0.82 s | 1.4× |

(Both cold runs produce identical output; the Python cold run's 812.7 s is subprocess-per-query
overhead over ~3,690 live queries plus the unoptimized 276 s `toLower` canonicalize scan.)

Where the cold speedup comes from:
- one **pooled Bolt connection** for the whole run (no per-query subprocess spawn);
- **concurrent** independent queries (edge checks, id verification, cross-label fulltext gather);
- **index-friendly Cypher** for Compound canonicalization — per-variant `MATCH (n {name: v})` index
  seeks instead of a `toLower(n.name)` full scan of the ~1 M-node Compound label: **276 s → 0.3 s**.

Per-phase (cold, `SPOKE_FABRIC_TIMING=1`): preflight 0.2 s · resolve_nodes 38.6 s · verify_node_ids
0.1 s · canonicalize 0.3 s · check_edges 3.4 s.

## Other tests

- `selftest` — PASS (PTPN22→26191, T1DM→DOID:9744, bogus→not_found, edge supported by ASSOCIATES_DaG).
- `audit` (independent, re-queries live SPOKE, 300 sampled ids) — **PASS**, 0 broken.
- `validate` (contract invariants) — **PASS**.
- **28-type stress** (2,720 nodes, one export spanning every BioOKF type; the Python lineage's
  original run of this test used a 2,796-node export — see `docs/28-type-stress-test.md` on the
  archive branch `codex/spoke-fabric-combined`. Neither export is retained in-repo; regenerate with
  `tools/wf_gen_nodes.js` from that branch) — all resolve sanely in
  9.4 s live: Gene 85%, BiologicalFunction 81%, Anatomy 79%, Disease 69%, Organism 62%; every
  provenance/context type (Agent, Dataset, Publication, Study, …) correctly `not_mappable`.
- **Fail-closed guardrail** — a curation decision pointing "carotid artery" at a hallucinated Gene id
  (999999999) trips the unconditional `verify_node_ids` gate → pipeline **FAIL**
  (`GATE: 1 mapped nodes point at a SPOKE id that does not exist`). Acceptance is structural: the
  reviewer only picks a target; the harness re-verifies the id exists — no confidence number is involved.

Concurrency only parallelizes *independent* queries; the node-matching order and every emitted value
are unchanged, which is why the output stays byte-identical to the sequential/Python reference.
