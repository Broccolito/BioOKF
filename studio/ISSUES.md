# BioOKF Studio — flagged issues (agentic loops & spec/schema)

> Per the build goal: surface problems in the ingestion / querying / linting loops, and
> candidate spec/schema changes. **The spec & schema are NOT edited** — these are notes for
> the maintainer. Loops are perfected in code; spec questions are only flagged.
>
> Evidence base: 4 real review articles were ingested by AI sub-agents acting as the MCP/CLI
> client (ingest → self-lint/fix → query). All 4 produced bundles that lint **0 errors / 0
> warnings / 0 infos** (24–31 nodes, 56–92 edges). The notes below are the issues those agents
> and the maintainer hit along the way.

## A. Tooling / loop issues — FIXED in our code

- **[FIXED] Self-healing YAML frontmatter.** `examples/knowledge/dataset/sider.md` had an
  unquoted `": "` in a `description:` value (the #1 LLM-authoring mistake), which strict
  `serde_yaml` rejects. `okf-core::parse` now retries with a repair pass that quotes such
  scalars. (The ingest agents independently learned to quote these — both belt and suspenders.)
- **[FIXED] CLI ↔ skill tool parity.** ~5 ingest agents noted the skills reference MCP tools
  (`okf_validate_page`, `okf_write_page`, `okf_search`, `okf_append_log`) but the `okf` CLI
  lacked a validate-before-write command. Added **`okf validate <file>`** (single-doc validation
  sharing `okf-core::validate`) and **`okf get <bundle> <identifier>`** (exact lookup to enforce
  "reuse, never fork"). The MCP server already exposed `okf_validate_page` / `okf_write_page` /
  `okf_append_log`; the surfaces now match.
- **[FIXED] Provenance source nodes appeared as graph orphans.** Source nodes referenced only via
  `primary_source` (HGNC, MONDO, …) had no `reported_in` edge, so they floated disconnected and
  the linter flagged 10 orphans on the examples bundle. The graph now **synthesizes implicit
  `reported_in` edges from `primary_source`** (rendered faint/dashed), so provenance is visible and
  orphan-source noise is gone.

## B. Loop improvements to consider (code, not spec)

- **Query loop: BM25 over a full natural-language question misses entity nodes.** Searching
  *"molecular mechanisms of drug resistance in cancer"* did **not** surface BRAF/KRAS or the
  RAS-RAF-MEK-ERK / PI3K-AKT-mTOR pathway nodes, because the question text doesn't contain those
  names. Graph traversal itself had no gaps (bundles are single connected components). Mitigations:
  (1) the query skill now must search **broadly then traverse edges** (strengthened wording);
  (2) consider entity-name extraction or multi-query fan-out before ranking; (3) consider boosting
  identifier/synonym field matches in BM25.
- **Lint domain/range coverage is partial.** Only `treats`/`prevents`/`has_phenotype`/`encodes`/
  `reported_in` are range-checked, so a clean lint does **not** prove full domain/range conformance
  (e.g. `located_in` with a `Disease` subject passed silently; `encodes` Gene→Variant correctly
  warned). Expanding the domain/range table (as warnings) would tighten this.
- **Lint has no positive/summary output.** A clean bundle prints only `0 errors / 0 warnings`.
  A `--summary` reporting edges-resolved, provenance-complete %, and per-type counts would give
  curators confidence. (`okf stats` partially covers this.)
- **`effect_metric` is not validated.** Agents coined values outside the spec's documented
  `effect_metric` enum (e.g. `percent_change`). This is *correct* under the current rules (only
  `type`/`predicate`/`knowledge_level`/`agent_type` are closed universes) — noting it in case the
  maintainer wants `effect_metric` checked (see C).

## C. Candidate spec/schema questions (FLAG ONLY — do not change the spec)

- **No forward predicate for "produces / secretes / activates".** Multiple agents reached for
  `produces` (β-cell → Insulin) or `activates` (DAG → PKCε) and had to fall back to
  `catalyzes`/`participates_in`/`regulates`, which don't fit cleanly. The 23 forward predicates may
  have a genuine gap for a generic "gives rise to / secretes" relation.
- **`affects_response_to` domain/range is undocumented** in schema.md's domain/range notes; agents
  guessed `Variant`/`Gene`/`BiomedicalMeasure → Molecule` (pharmacogenomics). Worth documenting.
- **Provenance granularity.** A review cites many primary studies (e.g. *Head et al. 2022*), but
  ingesting "the review" naturally yields one `primary_source` (the review) for every edge, so
  claims can't be cross-validated. Should the spec encourage per-citation source nodes when the
  review names the primary source? (Granularity guidance, not a rule change.)
- **`effect_metric` enum: advisory or enforced?** The spec lists an enum but it is not in the
  closed-universe set. Decide whether lint should warn on out-of-enum values.
- **Dual-facet entities** (`BiologicalPathway` vs `BiologicalFunction`, `Disease` vs `Phenotype`):
  handled fine via §5.C/§5.D, but agents spent effort deciding — the boundary tests work as intended.

## D. Environment / harness notes (not product issues)

- The Tauri **GUI cannot run headless** here (no window server for the webview); the crate
  **compiles** and the **frontend is verified in a browser** (identical code to the webview).
- `tauri-plugin-mcp` is a **git dependency**; wired behind the `debug-mcp` cargo feature so the
  default build stays offline-clean. Enable with `cargo run -p biookf-studio --features debug-mcp`.
