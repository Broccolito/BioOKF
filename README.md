# BioOKF: Biomedical Open Knowledge Format

**BioOKF is a format and toolchain for turning any biomedical source (a paper, preprint, bench
note, slide deck, CSV, figure, or tweet) into a structured, interlinked, version-controlled
knowledge base that compounds over time and can be queried as a graph.**

It is a biomedical *profile* of Google Cloud's [Open Knowledge Format
(OKF)](https://github.com/GoogleCloudPlatform/knowledge-catalog), itself a formalization of
Andrej Karpathy's [LLM Wiki](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f)
pattern. BioOKF keeps OKF's portable substrate (a Git-shippable tree of Markdown with YAML
frontmatter) and adds the one thing OKF leaves open: a closed, controlled universe of meaning.
The goal is a format an **AI agent or human curator** can follow to distill entities (nodes) and
relationships (edges) from heterogeneous biomedical sources, and reuse the result as persistent
agent memory and a queryable knowledge graph.

This repository contains both the format (the spec) and `bokf`/`biookf`, a toolchain that
implements it: a Rust core library, a CLI, an MCP server, a desktop visualizer, and a Claude Code
extension of skills plus guardrail hooks.

## The bundle format

A BioOKF **bundle** is a Git-shippable directory tree of Markdown:

| Path | What it holds |
|---|---|
| `raw/` | Immutable ingested sources. You never edit these. |
| `knowledge/<type>/<slug>.md` | The typed concept documents you author. These are the graph. |
| `index.md` | The catalog (identifier registry + by-type list). |
| `log.md` | Newest-first dated change history. |
| `SCHEMA.md` | The operating doc dropped at the bundle root. |

Each concept document is YAML frontmatter plus a Markdown body. Three rules make it BioOKF rather
than plain OKF:

- **28 node types.** Every document's `type` is exactly one of a closed set of 28 (20 biomedical
  entity types plus 8 provenance/context types; `Other` is the closure). An agent-coined
  `subtype` carries finer granularity and is never validated against a fixed list.
- **35 edge predicates.** Relationships are first-class frontmatter `edges:` entries. The
  predicate set is **24 positive predicates** (forward-only, no inverses) plus **11 negative
  `not_<X>` predicates** for the negatable effect predicates, for 35 total. Direction is always
  this-document to object.
- **Node-based provenance.** Every edge carries `knowledge_level`, `agent_type`, and
  `primary_source`. `primary_source` names a source *node* by its `identifier` (a
  `Publication`/`Study`/`Dataset`/`Agent` in the bundle), never a bare CURIE. Ingested-document
  sources anchor to the immutable bytes via `raw_source`; external references record their CURIE
  in `xref`.

Only `type` and `identifier` are mandatory on a node; the `identifier` is a single human-readable,
bundle-unique key (external ontology CURIEs are optional, living in `xref`).

The full normative format is in [SPEC.md](SPEC.md); the agent-facing operating doc (conventions
plus the ingest/query/lint workflow) is in [SCHEMA.md](SCHEMA.md). See [examples/](examples/) for a
small worked bundle.

## Components

Everything ships from the [`studio/`](studio) Cargo workspace plus the Claude Code extension that
wraps it.

| Component | Crate / location | What it is |
|---|---|---|
| `bokf-core` | `studio/crates/bokf-core` | The library: parse, normalize (legacy aliases), build the graph, lint, BM25 search, convert, git/log-sync, merge. Everything else is a thin front-end over it. |
| `bokf` | `studio/crates/bokf-cli` (binary `bokf`) | The command-line tool. The primary terminal surface for curation. |
| `bokf-mcp` | `studio/crates/bokf-mcp` (binary `bokf-mcp`) | A stdio MCP server exposing the `bokf_*` tools, with an embedded operating brief, for AI clients (Claude/Codex). |
| BioOKF Studio | `studio/app/src-tauri` (crate `biookf-studio`) | A Tauri desktop visualizer over `bokf-core`. |
| `biookf` extension | `studio/.claude-plugin`, `studio/skills`, `studio/hooks` | A Claude Code plugin: 8 curation skills plus 4 guardrail hooks over the toolchain. |

### BioOKF Studio (the GUI)

BioOKF Studio is a Tauri desktop app and a pure visualizer; every command delegates to
`bokf-core`. The window shows a sidebar of bundles and, on selecting one, a canvas that renders
the graph: nodes are colored by their type and edges are drawn as directional links. Clicking a
node opens a Markdown detail panel showing its frontmatter (synonyms, `xref` CURIEs) and its edges
grouped by predicate; clicking an edge shows its provenance triplet and any quantitative
attributes. The Tauri commands are `list_bases`, `get_bundle`, `lint_bundle`, and `search_bundle`.
See [studio/TESTING.md](studio/TESTING.md) for exactly what is verified.

### The `biookf` Claude Code extension

The extension (named `biookf`) packages the curation know-how as a Claude Code plugin:

- **8 skills**: `about-biookf` (what the project is), `biookf-convert` (ingest Step 1: source to
  `raw/`), `biookf-ingest` (the 7-step source-to-concepts loop), `biookf-query`,
  `biookf-lint`, `biookf-verify`, `biookf-merge`, `biookf-version`.
- **4 guardrail hooks** (wired in `studio/hooks/hooks.json`):
  - `session-start.sh` (SessionStart) briefs the agent and surfaces the active KB.
  - `guard-raw.sh` (PreToolUse on Edit/Write/MultiEdit) blocks edits to immutable `raw/`
    originals.
  - `post-write-lint.sh` (PostToolUse on Edit/Write/MultiEdit) surfaces lint errors after a
    concept-doc write (advisory; it cannot block).
  - `stop-verify.sh` (Stop) blocks the stop if the active KB fails `bokf verify`, up to a capped
    number of attempts, so a failed gate becomes another correction pass.

## Build and install

The toolchain is a Cargo workspace. Build all four crates from `studio/`:

```bash
cd studio
cargo build                 # builds bokf-core, bokf (CLI), bokf-mcp, and the Tauri app
cargo test -p bokf-core -p bokf-cli   # backend unit + integration tests
```

After a debug build the binaries land in `studio/target/debug/`:

- `studio/target/debug/bokf` — the CLI
- `studio/target/debug/bokf-mcp` — the MCP server
- `studio/target/debug/biookf-studio` — the desktop app

(Use `cargo build --release` for `studio/target/release/`.)

### Run the MCP server

`bokf-mcp` speaks JSON-RPC over stdio, so an MCP client launches it directly:

```bash
./target/debug/bokf-mcp
```

Point your client (Claude Code, Codex, etc.) at that binary as an MCP stdio server. On
`initialize` it ships an operating brief (the BioOKF rules plus the ingest/query/lint procedures)
and advertises the `bokf_*` tools.

### Install the Claude Code extension

The extension lives in `studio/` as a single-plugin marketplace
(`studio/.claude-plugin/marketplace.json` plus `studio/.claude-plugin/plugin.json`). Add `studio/`
as a marketplace and install the `biookf` plugin from it:

```bash
/plugin marketplace add /absolute/path/to/BioOKF/studio
/plugin install biookf@biookf
```

Installing it makes the 8 `biookf-*` skills and the 4 guardrail hooks active in your Claude Code
session. (Build the `bokf` binary first so the hooks can find it.)

## Usage

There are two ways to drive BioOKF, and most people mix them: let a **coding agent** (Claude Code,
or any MCP client) do the curation in natural language, and reach for the **`bokf` CLI** directly
when you want a precise, scriptable operation. Both surfaces sit on the same `bokf-core`, so they
are interchangeable.

### Curating with Claude Code (or another coding agent)

This is the intended day-to-day path. Install the `biookf` extension (above) and the agent gets
the 8 `biookf-*` skills, the MCP `bokf_*` tools, and the guardrail hooks. You then describe what
you want in plain language; the agent picks the right tools, runs the multi-step ingest loop, keeps
provenance attached, and self-corrects against the `verify` gate (the `stop-verify` hook won't let
it stop on a failing bundle). You stay in the loop by reviewing diffs and the dated `log.md`.

A few things make this work better than hand-running the CLI: the agent reuses existing nodes
instead of forking duplicates (it looks identifiers up first), it stamps every edge with
`knowledge_level` / `agent_type` / `primary_source` automatically, and it batches the 10 to 15
concept pages a single source produces into one coherent pass.

Example prompts that map onto the workflow below:

- **Scaffold:** "Create a new BioOKF bundle at `./mykb` called 'COVID immunology' and make it the
  active KB."
- **Ingest a paper:** "Ingest `./paper.pdf` into `mykb`: convert it to `raw/`, then distill the
  entities and claims into typed concept docs with full provenance, reusing existing nodes where
  they exist."
- **Ingest a quick note:** "Add a bench note to `mykb`: 'IL6 elevation is associated with worse
  COVID-19 outcomes' and wire up the gene/disease nodes and the edge between them."
- **Query:** "What does `mykb` say about IL6, and which sources back each claim?" or "Show me
  everything connected to the Disease node for COVID-19."
- **Maintain:** "Lint `mykb`, fix any errors, then verify and log-sync the result." or "Merge
  `./secondary-kb` into `mykb` and confirm the main KB stayed canonical."
- **Explore the schema:** "What node types and edge predicates can I use, and which predicates are
  negatable?"

The agent translates each of these into the same `bokf` / `bokf_*` operations documented next, so
the CLI walkthrough doubles as a reference for what the agent is doing under the hood.

### The curation workflow (CLI)

A typical end-to-end session, using the `bokf` CLI (the same operations are available as MCP
`bokf_*` tools and as the `biookf-*` skills). Replace `mykb` with your bundle path.

#### 1. Scaffold a bundle

```bash
bokf scaffold ./mykb --name "My knowledge base"
```

This creates `raw/`, `knowledge/`, `index.md`, `log.md`, and `SCHEMA.md`, makes the first git
commit, and registers + activates the bundle so later steps are not blocked by the active-KB
guardrail.

#### 2. Convert a source into `raw/`

`convert` brings a file/folder/zip (pdf, html, docx, pptx, csv, xlsx) or inline text into the
bundle's `raw/` as faithful Markdown, with a content-derived source id. It writes through
`bokf-core`, so the `raw/` guard does not block it.

```bash
bokf convert ./paper.pdf --into ./mykb
bokf convert --into ./mykb --text "IL6 elevation is associated with worse COVID-19 outcomes" --title "bench note"
```

#### 3. Ingest: distill the source into typed concepts

Drive the `biookf-ingest` skill (or the MCP tools) to run the 7-step loop: create a source node
for the document (with `raw_source` pointing at the `raw/` path), then for each entity create or
update its typed concept doc, and for each claim add a provenance-stamped `edges:` entry plus a
`reported_in` edge. A single source typically creates or updates 10 to 15 concept pages. You can
check a draft before writing it, and look up an existing identifier so you reuse rather than fork:

```bash
bokf validate ./mykb/knowledge/gene/il6.md     # validate-before-write
bokf get ./mykb "IL6"                           # exact identifier lookup
```

After writing pages, regenerate the catalog:

```bash
bokf index ./mykb            # rewrite index.md; add --check to only report what's missing
```

#### 4. Version-track every step

`log-sync` is the sole step-committer: it appends a dated block to `log.md` and git-commits,
atomically. Run it after each curation step.

```bash
bokf log-sync ./mykb --kind ingest --summary "ingested COVID-19 / IL6 review" --delta "+12 nodes"
bokf log ./mykb                                  # review history (newest-first)
bokf restore ./mykb <sha>                        # forward-only restore to a prior commit
```

#### 5. Verify (the gate)

`verify` is the deterministic accountability gate: it lints plus runs structure checks and exits
non-zero on any error. Run it at the end of an ingest or merge run. The `stop-verify` hook runs
this automatically for the active KB.

```bash
bokf lint ./mykb                                 # full lint report
bokf verify ./mykb --workflow ingest             # gate: PASS only if zero errors
```

#### 6. Query

Answer questions from the bundle, graph-shaped and provenance-cited:

```bash
bokf search ./mykb "interleukin"                 # BM25 full-text search
bokf graph ./mykb --out graph.json               # nodes + directional edges as JSON
bokf stats ./mykb                                # node/edge counts by type/predicate
```

#### 7. Merge two bundles

Merge a Secondary KB onto a canonical Main KB. Snapshot the Main KB first, relocate the
Secondary's `raw/` (deduplicating by content), then verify the Main KB stayed canonical:

```bash
bokf merge-snapshot ./main-kb                    # snapshot identifiers/paths before merge
bokf merge-raw ./main-kb ./secondary-kb          # relocate raw/, returns the source-id remapping
bokf merge-snapshot ./main-kb --verify           # confirm the Main KB stayed canonical
```

#### Multi-bundle bookkeeping

When you keep several bundles under one root, register them and set the active one (the hooks and
the verify gate operate on the active KB):

```bash
bokf register <root> mykb ./mykb                 # also: --list, --unregister <id>
bokf set-active <root> mykb
bokf get-active <root>
bokf export ./mykb --out mykb.json               # self-contained bundle JSON for the GUI
bokf predicates                                  # print the controlled vocabulary
```

## CLI command reference

| Command | What it does |
|---|---|
| `scaffold` | Create an empty bundle (`raw/`, `knowledge/`, `index.md`, `log.md`, `SCHEMA.md`); commit, register, activate. |
| `convert` | Convert a file/folder/zip or `--text` into raw Markdown under the bundle's `raw/`. |
| `validate` | Validate a single concept-document file without writing it. |
| `get` | Look up a node by exact identifier. |
| `index` | Regenerate `index.md` (or `--check` it). |
| `lint` | Lint a bundle against the v0.5 conformance rules (`--json`). |
| `verify` | Deterministic gate: lint plus structure checks; exits 1 on any error. |
| `graph` | Derive the render-ready graph (nodes + directional edges). |
| `search` | BM25 full-text search over concept documents. |
| `stats` | Summary statistics: node/edge counts by type and predicate. |
| `predicates` | Print the controlled vocabulary (28 types, 24 positive predicates, enums). |
| `log-sync` | Append a dated `log.md` entry AND commit, atomically (the sole step-committer). |
| `commit` | Lower-level stage-all + commit (non-logged lifecycle commit). |
| `log` | Show commit history (newest-first). |
| `restore` | Forward-only restore to a prior commit. |
| `register` | Register/list/unregister a known bundle under a root. |
| `set-active` / `get-active` | Set / read the active KB under a root. |
| `merge-raw` | Relocate a Secondary KB's `raw/` into a Main KB's `raw/` (dedup by content). |
| `merge-snapshot` | Snapshot the Main KB identifier/path set before a merge, or `--verify` after. |
| `export` | Export a self-contained bundle JSON (graph + per-node detail) for the GUI. |

## MCP tool reference

The `bokf-mcp` server exposes these tools (1:1 with the CLI surface):

| Tool | What it does |
|---|---|
| `bokf_list_bases` | List bundles under a root directory. |
| `bokf_scaffold` | Create an empty bundle. |
| `bokf_set_active` / `bokf_get_active` | Set / read the active KB under a root. |
| `bokf_list_pages` | List the concept-document pages under `knowledge/`. |
| `bokf_read_page` | Read one page (concept doc, raw source, or index/log/schema). |
| `bokf_write_page` | Create/overwrite a concept doc; validates concept docs on write. |
| `bokf_validate_page` | Validate a concept-document draft without writing it. |
| `bokf_append_log` | Append a dated entry to `log.md`. |
| `bokf_log_sync` | Append a dated `log.md` entry AND commit, atomically. |
| `bokf_lint` | Lint a bundle; returns a JSON report. |
| `bokf_verify` | Deterministic gate: `ok=true` iff zero lint errors. |
| `bokf_graph` | Return the render-ready graph as JSON. |
| `bokf_search` | BM25 full-text search over concept documents. |
| `bokf_stats` | Node/edge counts by type and predicate. |
| `bokf_predicates` | Print the controlled vocabulary. |
| `bokf_convert` | Convert a file/folder/zip or inline text into raw Markdown under `raw/`. |
| `bokf_index` | Regenerate `index.md` (or `check=true`). |
| `bokf_log` | Show commit history. |
| `bokf_restore` | Forward-only restore to a prior commit. |
| `bokf_merge_raw` | Relocate a Secondary KB's `raw/` into a Main KB's `raw/`. |
| `bokf_merge_snapshot` | Snapshot the Main KB before a merge, or `verify=true` after. |

## Authors

Wanjun Gu (<wanjun.gu@ucsf.edu>), Gianmarco Bellucci, Ilan Ladabaum, James Xue, Jonathan Xue, and
Xi Zheng.

## License

Apache-2.0.
