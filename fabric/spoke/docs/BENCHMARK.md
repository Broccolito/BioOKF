# Benchmark — Rust `spoke-fabric` vs the Python reference

Input: `runs/benchmark-biored/biookf-export.json` — **2,550 nodes / 5,360 edges**, curation overlay
`data/curation_overrides.json` (1,005 reviewed decisions). Both engines run the identical fail-closed
pipeline (`preflight → load_export → resolve_nodes → apply_curation → verify_node_ids → canonicalize
→ check_edges → validate_invariants`).

## Parity — byte-for-byte identical output

| comparison | result |
|---|---|
| full node annotation (all 9 fields × 2,550 nodes) | **0 mismatches** |
| full edge annotation (all 8 fields × 5,360 edges, incl. `evidence`) | **0 mismatches** |
| `spoke_report` (totals, per-type, per-predicate, contradictions) | **identical** |

Shared result set (both engines): mapped **1,464** · not_found 660 · not_mappable 424 · ambiguous 2;
edges supported **209** · related 97 · contradicted 14 · different_relation 82 · unsupported 1,103 ·
not_evaluated 3,855. Node map-rate 57.4% overall / 68.9% of mappable.

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
- **28-type stress** (2,720 nodes, one export spanning every BioOKF type) — all resolve sanely in
  9.4 s live: Gene 85%, BiologicalFunction 81%, Anatomy 79%, Disease 69%, Organism 62%; every
  provenance/context type (Agent, Dataset, Publication, Study, …) correctly `not_mappable`.
- **Fail-closed guardrail** — a curation decision pointing "carotid artery" at a hallucinated Gene id
  (999999999) trips the unconditional `verify_node_ids` gate → pipeline **FAIL**
  (`GATE: 1 mapped nodes point at a SPOKE id that does not exist`). Acceptance is structural: the
  reviewer only picks a target; the harness re-verifies the id exists — no confidence number is involved.

Concurrency only parallelizes *independent* queries; the node-matching order and every emitted value
are unchanged, which is why the output stays byte-identical to the sequential/Python reference.
