# Open Knowledge Format (OKF) — Open-Source Usage & Adaptations Catalog

> Research compiled 2026-06-25. OKF v0.1 was published by Google Cloud (Sam McVeety, Amir Hormati) on **2026-06-12** as a vendor-neutral spec for representing knowledge as a directory of markdown files with YAML frontmatter. The only hard requirement on a concept file is a non-empty `type` frontmatter field. This document catalogs open-source repos and real-world executions that **use, produce, consume, validate, or extend** OKF (and the closely related "markdown + YAML frontmatter knowledge bundle" pattern).

## Spec recap (the baseline everything builds on)

- **Required frontmatter:** `type` (producer-defined string; consumers route/filter/present on it and must tolerate unknown values).
- **Recommended optional:** `title`, `description`, `resource` (URI), `tags` (list), `timestamp` (ISO 8601).
- **Reserved filenames:** `index.md` (directory listing / progressive disclosure), `log.md` (chronological history). Everything else `*.md` is a concept.
- **Cross-links:** ordinary markdown links, absolute (bundle-root `/…`) or relative; relationship type lives in prose, not the link. Broken links must be tolerated.
- **Conformance (v0.1):** parseable YAML frontmatter on every non-reserved `.md`, non-empty `type`, reserved files follow their structure when present. Everything else is guidance.
- **Canonical example `type` values (from spec + GCP blog):** `BigQuery Table`, `BigQuery Dataset`, `API Endpoint`, `Metric`, `Playbook`, `Reference`, plus implied `Dataset`/`Table`/`Runbook`.
- Spec is explicitly **versioned for backward-compatible growth**; users are encouraged to "propose extensions," and consumers must tolerate unknown types/fields, missing indexes, and broken links.
- Source: <https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md>

---

## A. Official reference implementation (GoogleCloudPlatform/knowledge-catalog)

The `okf/` folder of the GCP repo (Apache-2.0, Python, ~3.3K★) is the seed of the ecosystem.

| Component | What it does | Notes |
|---|---|---|
| **Reference / Enrichment Agent** (`src/reference_agent/`) | LLM producer: two passes — (1) extract BigQuery metadata → one OKF doc per table/view; (2) optionally crawl seed doc URLs and enrich concepts with citations, schemas, join paths. | Pure OKF, no custom types. |
| **Static HTML Visualizer** (`viz.html`) | Self-contained, zero-dependency interactive **force-directed graph** of a bundle (Cytoscape.js + marked.js): search, detail panels, backlinks, layouts. | The pattern most third-party tools re-implement. |
| **Sample bundles** (`bundles/`, `samples/`) | Three living conformant bundles: **GA4 e-commerce**, **Stack Overflow**, **Bitcoin/crypto** (FK relationships expressed in prose). Each ships a recipe + generated `viz.html`. | Types: `datasets`, `tables`, `references`. |
| **kcmd** (`toolbox/mdcode`) | TypeScript **CLI + library (`npm i kcmd`) + MCP server** for *bidirectional sync* ("git for metadata") between local YAML/markdown and Google Cloud Knowledge Catalog (ex-Dataplex). MCP tools: `pull`, `push`, `list-entries`, `lookup-entry`, `modify-entry`. | **Extends/adapts** OKF toward a Dataplex catalog model: YAML entries + sidecar `.md` for rich overviews, rather than pure one-file-per-concept. |

---

## B. Coding-agent skills / plugins (author • validate • visualize OKF)

A cluster of near-identical "teach Claude Code / agents to do OKF" plugins appeared within days of launch. Most **vendor the v0.1 spec verbatim** and add a deterministic Python conformance checker — they do **not** extend the format; invented `type` values are left to the end user.

| Repo | Lang | What it adds | Invented/used types |
|---|---|---|---|
| **scaccogatto/okf-skills** | Python | Dual install: Claude Code plugin (`.claude-plugin/`) **and** `skills.sh`/`SKILL.md` for 20+ agents (Cursor, Codex, …). Three skills: `/okf:okf` (create/update from templates), `/okf:validate` (deterministic §9 conformance via standalone PyYAML script), `/okf:visualize` (HTML graph). | No invented types; templates illustrate `Service`, `Decision`. |
| **catancs/okf-skill** | (Claude plugin) | Toolkit to "validate, query, lint, create" OKF bundles for coding agents; spec-driven, portable across agent sessions. | No invented types. |
| **xSAVIKx/okf-skills** | **Go** | **Deterministic connectors, no embedded LLM** — turns existing structure (DB schemas, column comments, FKs, file trees, git history) into OKF. 6 connectors (SQLite/MySQL/PostgreSQL/BigQuery + filesystem + git), 3 guidance skills, **MCP server** exposing connectors as tools. Round-trips back to source. | Uses a `ConceptDoc` struct; adds body sections `Data Profile`, `Sample`. |
| **supachai-j/open-knowledge-format-starter** | Python | Fork-and-go **AI-maintained KB starter**. 3-layer arch: `raw/` (immutable sources) → `wiki/` (agent-maintained concepts) → `AGENTS.md` (schema/behavior). Zero-dep `okf-validate.py`, Claude skill (`ingest`/`query`/`author`/`validate`), `concept-template.md`. Recommends BM25+semantic MCP search beyond ~150 pages. | No invented types. |

---

## C. Producers — turn existing docs/content into OKF bundles

| Repo / tool | Lang | Input → output | Extension / invented types |
|---|---|---|---|
| **0dust/OKFy** (`okfy-ai` npm, 32★) | TypeScript | Crawl doc websites **or** import local markdown → OKF bundle (typed concepts, nav, source URLs, internal links + **backlinks**). Ships **MCP server (stdio)**, static HTML Inspector, JSON validation reports. For Claude/Cursor/Codex. | Doesn't standardize types — inherits types from source docs (samples show `type: Guide`). |
| **yzfly/awesome-okf** (Chinese, 9★) | Python | A **suite of 7 zero-dependency producer plugins + `myokf-cli`** umbrella: `feishu-to-okf`, `obsidian-to-okf` (wikilinks→OKF links), `notion-to-okf` (Notion MD export), `github-to-okf` (code symbols), `awesome-to-okf` (curated "awesome-*" lists), `html-to-okf`, plus Claude skills (`okf-creator`, `book-to-okf`, `code-to-okf`, `okf-to-book`, `okf-to-web`). | **Proposes 3 backward-compatible extensions**: i18n (multilingual), **code as first-class** (code symbols), **HTML as first-class** citizen — added without touching mandatory fields. |
| **OKF WordPress plugin** (GPL, free) | PHP | Auto-exports published posts/pages → OKF bundle served at `/okf/`, rebuilt on publish/edit. Read-only (never mutates content). | Cautions that naive auto-export yields one generic `type` and no real links — the value is human-/agent-curated typed concepts (`Service`, `Metric`, `Runbook`). |
| Community conversions cited (no public repo found): **AgentFitech**, **kb.duyet.net** | — | Conformant producers/conversions stood up within days of the spec. | — |

---

## D. Consumers / harnesses / memory systems (serve, query, or persist OKF)

| Repo | Lang | What it does | Types invented |
|---|---|---|---|
| **pumblus/okf-harness** (9★) | TypeScript | **Agent-first local harness / terminal workspace** for OKF-compatible "LLM wikis." Flow: `raw/sources/` → synthesized `wiki/` with citations. Agent-facing CLI `okfh --json`: `init`, `source add`, `ingest plan`, `evidence` (bounded summaries before answering), `read`, `check` (conformance + lint), `graph` (self-contained HTML). For Claude Code / Codex. | None — stays on standard markdown+frontmatter. |
| **psinetron/echoes-vault-opencode** (169★) | TypeScript | **Persistent memory for OpenCode**, Obsidian-style vault, "100% interoperable with standard parsers, renders on GitHub/GitLab." **Strictly enforces `type`** for OKF compliance. | **Invents app types**: `architecture` (shown, e.g. `stack: [nestjs, react]`, `status: active`); structure implies `daily log`, `page`, `asset`. Adds custom frontmatter (`stack`, `status`). |
| **JuneYaooo/lineage-skill** (215★) | Python | Distills videos/PDFs/transcripts/notes into **source-backed Agent Skills** via Capture→Cite→Compress→Connect→Codify→Evaluate. Added "OKF-compatible knowledge packages" output under `references/okf/` (progressive markdown+frontmatter directory). | **Invents capability types**: `diagnostics`, `workflows`, `rubrics`, `templates`, `transfer_rules`, `failure_modes` (in `course_package.json`, mapped into the OKF package). |

---

## E. Related / parallel patterns (markdown+frontmatter knowledge bundles, not strictly OKF)

| Repo | Relationship | Notes |
|---|---|---|
| **sahil87/fab-kit** (Go, 23★) | Tagged `okf`, but is a **spec-driven dev workflow**, not an OKF producer. 6-stage pipeline (intake→apply→review→hydrate→ship→review-PR) producing persistent typed artifacts: `intake.md`, `plan.md`, `constitution.md`, `code-quality.md`, `code-review.md`. | Shares the "typed markdown artifact" philosophy; no confirmed OKF bundle I/O. |
| **tomazinho/open-ckf-compiled-knowledge-format** | Adjacent/competing format ("Compiled Knowledge Format"). | Treats knowledge as a **compiler problem** (`.ckf.json` with sections/rules/definitions + source traceability) to fight "composition hallucination" in RAG. `ckf-compiler` + `ckf-viewer`. Argues "plain JSON/YAML aren't enough" — i.e., a deliberate counterpoint to OKF's plain-markdown minimalism. No OKF link. |
| **Geeksfino/kb-mcp-server** | **Not OKF.** | txtai/embeddings-based MCP server consuming `tar.gz` KBs; vector-search rather than typed-frontmatter. Listed only to disambiguate. |

---

## Cross-cutting observations

1. **Two dominant repo archetypes:** (a) *agent skills/plugins* that wrap the verbatim spec + a deterministic conformance checker (scaccogatto, catancs, supachai-j, GCP), and (b) *producers/connectors* that mechanically extract structure from an existing source (xSAVIKx connectors, OKFy, awesome-okf, WordPress plugin).
2. **The visualizer is the most-copied component** — almost everyone ships a self-contained HTML force-graph mirroring GCP's `viz.html`.
3. **MCP is the standard serving layer** — kcmd, OKFy, xSAVIKx, and supachai-j's starter all expose OKF via an MCP server so any agent can consume a bundle.
4. **Most tools deliberately do NOT invent types** (they delegate to producers/users), but **memory/skill systems do**: echoes-vault (`architecture`, + `stack`/`status` fields) and lineage-skill (`diagnostics`/`workflows`/`rubrics`/`templates`/`transfer_rules`/`failure_modes`) are the clearest examples of domain-specific type vocabularies layered on OKF.
5. **The one explicit spec-extension proposal** comes from **awesome-okf**: i18n, code-as-first-class, and HTML-as-first-class, all framed as backward-compatible additions that leave the mandatory `type` field untouched.
6. **GCP's own kcmd is the most divergent adaptation**: it bends OKF toward a catalog-entry model (YAML entry + sidecar `.md`) for Dataplex/Knowledge Catalog sync, rather than the pure one-file-per-concept shape.

## Sources

- OKF spec: <https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md>
- GCP blog: <https://cloud.google.com/blog/products/data-analytics/how-the-open-knowledge-format-can-improve-data-sharing>
- GitHub `okf` topic: <https://github.com/topics/okf>
- scaccogatto/okf-skills · catancs/okf-skill · xSAVIKx/okf-skills · supachai-j/open-knowledge-format-starter
- 0dust/OKFy · yzfly/awesome-okf · pumblus/okf-harness · psinetron/echoes-vault-opencode · JuneYaooo/lineage-skill · sahil87/fab-kit
- OKF ecosystem tools: <https://okf.md/tools/>
- Adjacent: tomazinho/open-ckf-compiled-knowledge-format · Geeksfino/kb-mcp-server
