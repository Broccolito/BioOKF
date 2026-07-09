# spoke-fabric — native Rust BioOKF → SPOKE mapping fabric

A single Rust binary that grounds a BioOKF knowledge-graph export into the
[SPOKE](https://spoke.ucsf.edu) biomedical knowledge graph: it resolves every BioOKF **node** to a
SPOKE identifier (or marks it not-found / not-mappable) and verifies every BioOKF **edge** against
the relationships SPOKE actually holds between the two resolved endpoints.

This is a faithful, **output-identical** port of the Python reference harness, rewritten in Rust with
a **native Neo4j Bolt connection** (the SPOKE query layer is a built-in subcommand, not a
subprocess). On the `benchmark-biored` graph (2,550 nodes / 5,360 edges) it produces byte-for-byte
the same annotations as the Python version while running **~10× faster end-to-end** (≈45 s vs the
Python engine's minutes on a cold run — same live queries, no on-disk cache needed).

## Connection

Reads the same `.env` the legacy `spoke-cli` uses (kept in this folder):

```
KNOWLEDGE_GRAPH_URI=bolt://spokedev.cgl.ucsf.edu:7687
KNOWLEDGE_GRAPH_USERNAME=neo4j
KNOWLEDGE_GRAPH_PASSWORD=…
KNOWLEDGE_GRAPH_DATABASE=spoke
```

`.env` is git-ignored — keep it to trusted local/agent runtimes.

### Safety & operations

- **Read-only by design (F1).** `query` refuses any Cypher containing a write/DDL
  clause (`CREATE`/`MERGE`/`DELETE`/`DETACH`/`SET`/`REMOVE`/`DROP`/`FOREACH`/`LOAD CSV`);
  the fabric only ever *reads* SPOKE. This guard is enforced at the client before a
  statement is sent.
- **Prefer a least-privilege user (F2).** The SPOKE dev instance is Neo4j Community
  Edition with no RBAC, so `neo4j` is effectively a superuser and the client-side
  read-only guard is the only write barrier there. On any instance that supports
  roles, point `KNOWLEDGE_GRAPH_USERNAME` at a dedicated **read-only** role for
  defense-in-depth.
- **Cache freshness (F6).** Disk-cache (`runs/.spoke_cache`) entries expire after
  `SPOKE_FABRIC_CACHE_TTL_SECS` (default `604800` = 7 days; `0` = never expire). Set
  `SPOKE_FABRIC_NO_CACHE=1` in CI to always hit live SPOKE.
- **Recall is a floor (F7).** Deterministic (no-LLM) recall is weakest for molecule-
  and measurement-heavy KBs (`Molecule`, `BiomedicalMeasure`, `Phenotype`). For those
  domains, run the LLM review tier (`--curation` / `--adjudications`) — the
  protein-description recovery and reviewed tier lift many nodes a raw deterministic
  sweep leaves `not_found`.

## Build

```
cargo build --release      # -> target/release/spoke-fabric
```

## Subcommands

| command | purpose |
|---|---|
| `test-connection` | check the native Bolt connection |
| `query "<cypher>"` | run **read-only** Cypher against SPOKE, print JSON rows (the connection subtool; write/DDL clauses are refused — see Safety) |
| `pipeline <export> --out <o> --report <r> [--curation <c>]` | **fail-closed** run: every step gated + a manifest written |
| `audit --out <o> --manifest <m>` | independent end-to-end audit (re-queries live SPOKE) |
| `link <export> --out <o> --report <r> [--adjudications <a>] [--canonicalize] [--validate]` | ungated annotate |
| `validate <annotated>` | check an annotated graph against the contract invariants |
| `report <report.json>` | print a human summary |
| `lookup <name> --type <T> [--subtype <s>]` | resolve a single (type, name) |
| `selftest` | link a tiny in-repo fixture and assert invariants |

### Example — the canonical run

```
target/release/spoke-fabric pipeline runs/benchmark-biored/biookf-export.json \
  --out   runs/benchmark-biored/rust-annotated.json \
  --report runs/benchmark-biored/rust-report.json \
  --curation data/curation_overrides.json
target/release/spoke-fabric audit \
  --out runs/benchmark-biored/rust-annotated.json \
  --manifest runs/benchmark-biored/rust-annotated.manifest.json
```

Set `SPOKE_FABRIC_TIMING=1` to print per-phase / per-tier timings to stderr.

## The pipeline (fail-closed, gated)

`preflight → load_export → resolve_nodes → apply_curation → verify_node_ids → canonicalize →
check_edges → validate_invariants`. Each step has a pre/post-condition **gate** that aborts on
violation (never silently skipped); a `.manifest.json` records that every required step ran and
passed; `audit` re-derives every check from the output file and re-queries live SPOKE, so a clean
run can't be faked. `verify_node_ids` is unconditional — every mapped `(label, identifier)` must
exist in SPOKE, catching hallucinated/stale ids from any tier (including curation).

## Node resolution — the trust-ordered waterfall

`exact_identifier → exact_name → exact_name (case-insensitive) → synonym (guarded) → fuzzy (gated)
→ reviewed (curation overlay)`, else `ambiguous` / `not_found`. Tiers are ordered most-trusted first
and deterministic exact matches always outrank the review overlay. How a node matched is recorded as
`match_method`; **no numeric confidence is stored** — for a `reviewed` (LLM/curator) node the harness
gate is structural (the reviewer chooses a candidate; `verify_node_ids` confirms it exists in SPOKE),
not a threshold on a self-reported score. Crosswalks live in `data/type_crosswalk.yaml` and
`data/predicate_crosswalk.yaml`.

## Edge verification

A BioOKF node is an entity → resolved **1-to-1** to one SPOKE node. A BioOKF edge is an *assertion*
→ **not** mapped 1-to-1; it is verified against **every** relationship SPOKE holds between the two
resolved endpoints, yielding one `support_status`
(`supported`/`contradicted`/`related`/`different_relation`/`unsupported`/`not_evaluated`) plus all
relevant relationships as `evidence`. Every edge annotation is self-describing even when
`not_evaluated`: `endpoints` always reports `source_mapped`/`target_mapped` (+ ids) and `notes` says
which endpoint failed, so a reader always sees the fabric ran.

## Display placement (consumer contract)

Both annotations are always present (on **every** node and edge). A viewer MUST render the SPOKE
annotation at one fixed position in the record: **immediately after the record's mandatory attributes
and immediately before its optional attributes** (per `SCHEMA.md` → *Required fields*):

- **node** — mandatory = `type`, `identifier`; the SPOKE card is the **first body section**, before
  any optional attribute (`subtype`, `xref`, `synonyms`, …).
- **edge** — mandatory = `predicate`, `object`, `knowledge_level`, `agent_type`, `primary_source`
  (the provenance triplet); the SPOKE card follows the **provenance triplet**, before optional
  attributes (`direction`, `stats`, `qualifiers`, …).

The reference viewer (`app/studio`) hard-codes exactly this (see `spokeSectionHtml` /
`spokeEdgeSectionHtml`), and marks the section with one consistent square colour + identical
styling on nodes and edges.

## Speed — where it comes from

- **One pooled Bolt connection** for the whole run (no per-query subprocess spawn).
- **Concurrent** independent queries (`buffer_unordered`): edge checks, id verification, and the
  cross-label fulltext gather all run in parallel.
- **Index-friendly Cypher**: case-insensitive Compound canonicalization uses per-variant
  `MATCH (n {name: v})` index seeks instead of a `toLower(n.name)` full-label scan (276 s → 0.3 s).

Concurrency only parallelizes *independent* queries — the node-matching order and every result are
unchanged, so output stays identical to the sequential/Python reference.

## Layout

```
src/config.rs      .env + pooled Bolt connection
src/normalize.rs   name normalization / Lucene sanitization
src/crosswalk.rs   type + predicate crosswalk loaders (data/*.yaml)
src/client.rs      native Bolt query layer + cypher helpers + batch methods
src/resolver.rs    the waterfall (NodeResolver) + edge verification (EdgeChecker)
src/annotate.rs    load / curation overlay / canonicalize / report
src/pipeline.rs    fail-closed pipeline + gates + manifest + audit + validate
src/main.rs        clap CLI
```
