# BioOKF: Biomedical Open Knowledge Format

## Abstract

The adoption of AI agents has reimagined how biomedical knowledge is structured, typed, and sourced.
We present the Biomedical Open Knowledge Format (BioOKF), a knowledge harness defining a closed,
molecularly grounded vocabulary of entities (genes, variants, molecules) and relationships, with
provenance required on every claim. Within it, an agent reading unstructured sources (manuscripts,
slide decks, posts) ingests concepts and relationships in a knowledge base that follows one standard
across curators, models, and topics, and these knowledge bases can be merged with others to give
agents persistent, well-sourced context. BioOKF extends Google's Open Knowledge Format and ships as
an open-source stack, an MCP server, a command-line tool, and a desktop interface, linking curation,
querying, linting, and visualization to the agent. In benchmarks holding the model and source text
fixed, curating under BioOKF captured more biologically meaningful entities and relationships,
represented them more accurately, and yielded more explainable bases than a bare model or generic
OKF. With BioOKF, we aim to build a periodic table of biomedical knowledge that grows collaboratively
in the generative-AI era.

## Overview

**BioOKF is a periodic table for agentic biomedical research: a closed, controlled vocabulary of 28
entity types and 35 relationship predicates that maps any biomedical source (a paper, preprint,
bench note, slide deck, CSV, figure, or tweet) into a structured, interlinked, version-controlled
knowledge base. It does not invent new concepts or new relationships; it gives biomedical knowledge a
fixed organizing system, a knowledge harness that you and an AI agent can curate and explore together
as a graph, live, in a desktop Studio.**

It is a biomedical *profile* of Google Cloud's [Open Knowledge Format
(OKF)](https://github.com/GoogleCloudPlatform/knowledge-catalog), itself a formalization of
Andrej Karpathy's [LLM Wiki](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f)
pattern. BioOKF keeps OKF's portable substrate (a Git-shippable tree of Markdown with YAML
frontmatter) and adds the one thing OKF leaves open: a closed, controlled universe of meaning, 28
node types and 35 edge predicates with node-based provenance on every claim.

This repository contains both the format (the [spec](SPEC.md)) and the toolchain that implements it:
a Rust core, a CLI (`bokf`), an MCP server (`bokf-mcp`), and the **BioOKF Studio** desktop
visualizer, distributed as a notarized desktop app plus Claude Code and Codex plugin manifests.

> **🌐 Landing site → [broccolito.github.io/BioOKF](https://broccolito.github.io/BioOKF/)**:
> what BioOKF does, how to install it, and live, interactive [HyperFrames](https://github.com/heygen-com/hyperframes)
> mockups of the Studio UI (graph, inspector, and the live agent loop). Full function reference on the
> [documentation page](https://broccolito.github.io/BioOKF/docs.html). The site's source lives in [`landing/`](landing).

## Install

BioOKF ships as two pieces you install separately: the **Studio desktop app** (which also
installs the `bokf` CLI and `bokf-mcp` server), and an **agent plugin** for Claude Code or Codex,
which wires those tools into your MCP client.

### 1. BioOKF Studio (desktop app + CLI + MCP)

Download the notarized **`BioOKF.Studio_0.2.3_aarch64.dmg`** from the
[latest release](https://github.com/Broccolito/BioOKF/releases/latest), open it, and drag
**BioOKF Studio** into Applications. The DMG is signed and notarized by Apple, so it opens with
no Gatekeeper warning and no quarantine prompt.

The bundle includes `bokf` (CLI) and `bokf-mcp` (MCP server) as signed binaries inside the
`.app`. On startup the Studio checks the latest GitHub Release; when a newer notarized build is
available it offers to install the updated Studio plus the bundled CLI/MCP tools, then restarts
itself. If the app is current but the tools are not on PATH, Studio offers to install **`bokf`**
and **`bokf-mcp`** to `/usr/local/bin` (one admin prompt). The Studio also has a built-in terminal
where both tools are already on the PATH.

All configuration (the knowledge-base registry and the active-KB pointer) lives under
`~/.config/biookf-studio`, shared by the Studio, the CLI, and the MCP server.

(Apple Silicon for now; Intel and other platforms follow from the release pipeline.)

### 2. The BioOKF agent plugin (Claude Code or Codex)

Once the Studio is installed, add the plugin to get the `bokf_*` MCP tools in your agent.

For Claude Code:

```
/plugin marketplace add Broccolito/BioOKF
/plugin install biookf@biookf
```

For Codex, the same published plugin root includes the Codex manifest at
`plugins/biookf/.codex-plugin/plugin.json` and a Codex skill at `plugins/biookf/skills/biookf`.
Install it through the Codex plugin manager from a marketplace entry that points at
`plugins/biookf`:

```
codex plugin add biookf@<marketplace-name>
```

Then restart the client. The `bokf_*` tools are now available, and `bokf_studio_open` launches
the visualizer. Both manifests use the same MCP launcher, which finds and starts the `bokf-mcp`
binary that shipped inside the Studio app; no separate download is needed on macOS.

> **Want your coding agent to do it for you?** Paste this to Claude Code:
>
> > Install the BioOKF plugin from `https://github.com/Broccolito/BioOKF`: run
> > `/plugin marketplace add Broccolito/BioOKF`, then `/plugin install biookf@biookf`, restart,
> > and confirm the `bokf_studio_open` tool works.

**Requirements:** Claude Code or Codex; the BioOKF Studio DMG installed (step 1 above); macOS
Apple Silicon. Intel and other platforms ship as the release pipeline adds them; until a prebuilt
asset exists for your platform you can [build from source](#build-from-source) and point the plugin
at your binary with `BIOOKF_MCP_BIN`.

**How the plugin finds the binaries.** The launcher (`plugins/biookf/scripts/bokf-mcp`) checks
`BIOOKF_MCP_BIN` first (explicit override), then looks for `bokf-mcp` inside a locally installed
`BioOKF Studio.app`, and falls back to downloading `biookf-<platform>.tar.gz` from this repo's
[GitHub Release](https://github.com/Broccolito/BioOKF/releases) and caching it under
`~/.local/share/biookf`. Override the version, cache root, or source repo via `BIOOKF_VERSION`,
`BIOOKF_HOME`, and `BIOOKF_REPO`.

## What you get

| Piece | What it is |
|---|---|
| **BioOKF Studio** | A desktop app that renders a knowledge base as an interactive, type-colored graph, with node/edge detail panels, an integrated terminal, in-app editing, and a registry-driven sidebar of bundles that can live anywhere on disk. |
| **`bokf-mcp`** | A stdio MCP server: **33 tools** for curation, analysis, **and live control of the Studio GUI** (`bokf_studio_*`). It ships an operating brief on `initialize`, so an agent knows the BioOKF rules. |
| **`bokf`** | The same engine as a scriptable CLI (23 subcommands): the precise terminal surface for curation. |
| **The live loop** | The MCP server can open the Studio and **drive/observe it in real time**: an agent searches, selects, and moves around the graph while a human watches each action in an in-app "AI agent" banner, and reads the app's full status as structured JSON instead of taking screenshots. |

## The bundle format

A BioOKF **bundle** is a Git-shippable directory tree of Markdown:

| Path | What it holds |
|---|---|
| `raw/` | Immutable ingested sources. You never edit these. |
| `knowledge/<type>/<slug>.md` | The typed concept documents you author. These are the graph. |
| `index.md` | The catalog (identifier registry + by-type list + subtypes in use). |
| `log.md` | Newest-first dated change history. |
| `SCHEMA.md` | The operating doc dropped at the bundle root. |

Each concept document is YAML frontmatter plus a Markdown body. Three rules make it BioOKF rather
than plain OKF:

- **28 node types.** Every document's `type` is exactly one of a closed set of 28 (20 biomedical
  entity types plus 8 provenance/context types; `Other` is the closure). An agent-coined `subtype`
  carries finer granularity and is never validated against a fixed list.
- **35 edge predicates.** Relationships are first-class frontmatter `edges:` entries: **24 positive
  predicates** (forward-only, no inverses) plus **11 negative `not_<X>` predicates** for the
  negatable effect predicates, 35 total. Direction is always this-document to object; quantitative
  claims go on edges, never in prose.
- **Node-based provenance.** Every edge carries `knowledge_level`
  (`knowledge_assertion` / `statistical_association` / `prediction` / `observation` /
  `not_provided`), `agent_type` (`manual_agent` / `automated_agent` / `text_mining_agent` /
  `data_analysis_pipeline` / `computational_model` / `not_provided`), and `primary_source`.
  `primary_source` names a source *node* by its `identifier` (a `Publication` / `Study` / `Dataset`
  / `Agent` in the bundle), never a bare CURIE, plus a `reported_in` edge. Ingested-document sources
  anchor to the immutable bytes via `raw_source`; external references record their CURIE in `xref`.

Only `type` and `identifier` are mandatory on a node. The full normative format is in
[SPEC.md](SPEC.md); the agent-facing operating doc (conventions plus the ingest/query/lint workflow)
is in [SCHEMA.md](SCHEMA.md).

## BioOKF Studio (the GUI)

Studio is a Tauri desktop app and a pure visualizer; every operation delegates to `bokf-core`.

- **Registry-driven sidebar.** Knowledge bases are tracked by a registry of links, so a bundle can
  live **anywhere on disk**; there is no fixed "knowledge bases" directory. Each entry shows its
  name, node/edge counts, and last-updated date (the full path is the hover tooltip). Delete or move
  a bundle's folder and it drops from the sidebar automatically; register one elsewhere and it
  appears, no restart needed. The **+ New base** button opens a native folder picker that validates the
  folder as a real BioOKF bundle before registering it.
- **Interactive graph canvas.** A force-directed, type-colored graph with pan/zoom/drag,
  fit-to-view, hub emphasis, hover tooltips, and neighbor-focus dimming. Negative `not_<X>` edges
  render struck-through; synthesized provenance edges render faint; symmetric edges are styled
  distinctly. A type-family legend covers all 28 types plus the light "External" swatch for
  referenced-but-undocumented entities.
- **Detail panels.** Click a node for its type badge, frontmatter (subtype, `xref`/synonyms/tags
  chips, `raw_source`, description, notes), outgoing edges grouped by predicate, incoming
  "referenced by" edges, the rendered Markdown body, and, for source nodes, a Source/Provenance
  block with credibility tier, venue, DOI/PMID/arXiv links, and ingested figures. Click an edge for
  its provenance triplet, direction, publications, and quantitative attributes. Citations open a
  side preview of the cited source, including the ingested paper and its figures.
- **In-app editing (desktop).** Edit a concept doc's full Markdown, a per-node notes section, or a
  per-edge note. Each one writes live to disk and appends a dated `log.md` entry. A reveal-in-Finder
  button opens the file in macOS Finder.
- **Integrated terminal.** Multi-tab real PTY (`xterm.js`) running your `$SHELL`, with a resizable
  panel, so you can run `bokf` without leaving the app.
- **Lint pill, search, history.** A toolbar lint pill opens a grouped findings popup; the search box
  filters/highlights the graph live (⌘K / Ctrl-K to focus); the change-log drawer renders `log.md`.

## The live CLI ↔ MCP ↔ GUI integration

The three surfaces are wired together so an AI agent and a human can work on the same knowledge base
at the same time:

- **One active KB, shared.** The CLI, the MCP server, and the GUI all read and write a shared
  `.active-kb` pointer and a `registry.yaml` of bundle links. Selecting a base in the GUI updates
  the pointer for the agent; an agent (or `bokf set-active`) changing the pointer is **mirrored back
  into the GUI**, without yanking the user's view: the agent's active base is shown as a focus
  marker in the sidebar.
- **The agent drives the GUI.** The `bokf_studio_*` tools open the Studio and steer it:
  `bokf_studio_select` / `bokf_studio_search` / `bokf_studio_reload` move around the graph,
  `bokf_studio_state` / `bokf_studio_graph` observe it. The control channel is a Unix socket; it
  ships in normal builds and only listens when `BIOOKF_STUDIO_CONTROL=1` (set automatically when the
  agent opens the app).
- **Status without screenshots.** `bokf_studio_state` returns the GUI's complete status as
  structured JSON: active base, counts, search query, current selection, which panels are open,
  lint summary, and the last agent action. An agent reads what the app is doing instead of
  taking and interpreting screenshots.
- **The human sees the agent work.** Every agent action is narrated in real time in an in-app **"AI
  agent" activity banner** (search, lint, merge, build, query…) and `bokf_studio_narrate` lets the
  agent post a custom status line, so a person watching the Studio always knows what the agent is
  exploring.

## Curating with a coding agent

This is the intended day-to-day path. With the plugin installed, describe what you want in plain
language and the agent picks the right tools, runs the multi-step ingest loop, keeps provenance
attached, reuses existing nodes instead of forking duplicates, and self-corrects against the
`verify` gate. You stay in the loop by reviewing diffs, the dated `log.md`, and the live graph.

Example prompts that map onto the workflow below:

- **Scaffold:** "Create a new BioOKF bundle at `./mykb` called 'COVID immunology' and make it active."
- **Ingest a paper:** "Ingest `./paper.pdf` into `mykb`: convert it to `raw/`, then distill the
  entities and claims into typed concept docs with full provenance, reusing existing nodes."
- **Ingest a quick note:** "Add a note to `mykb`: 'IL6 elevation is associated with worse COVID-19
  outcomes' and wire up the gene/disease nodes and the edge between them."
- **Query:** "What does `mykb` say about IL6, and which sources back each claim?"
- **Watch it work:** "Open the Studio, then walk me through the COVID-19 subgraph node by node."
- **Maintain:** "Lint `mykb`, fix any errors, then verify and log-sync." / "Merge `./secondary-kb`
  into `mykb` and confirm the main KB stayed canonical."
- **Schema:** "What node types and edge predicates can I use, and which predicates are negatable?"

## The curation workflow (CLI)

The same operations the agent runs are available as the `bokf` CLI and as MCP `bokf_*` tools.
Replace `mykb` with your bundle path.

```bash
# 1. Scaffold (creates raw/, knowledge/, index.md, log.md, SCHEMA.md; commits, registers, activates)
bokf scaffold ./mykb --name "My knowledge base"

# 2. Convert a source into raw/ (pdf/html/docx/pptx/csv/xlsx, inline --text, or a --url)
bokf convert ./paper.pdf --into ./mykb
bokf convert --into ./mykb --text "IL6 elevation worsens COVID-19 outcomes" --title "bench note"

# 3. Ingest: distill into typed concepts (validate before write; reuse identifiers, don't fork)
bokf validate ./mykb/knowledge/gene/il6.md
bokf get ./mykb "IL6"
bokf index ./mykb                 # rebuild index.md (--check to only report what's missing)

# 4. Version-track every step (log-sync = append log.md + commit, atomically)
bokf log-sync ./mykb --kind ingest --summary "ingested COVID-19 / IL6 review" --delta "+12 nodes"

# 5. Verify (the gate: lint + structure checks; exits non-zero on any error)
bokf verify ./mykb --workflow ingest

# 6. Query
bokf search ./mykb "interleukin cytokine"
bokf graph ./mykb --out graph.json
bokf stats ./mykb

# 7. Merge a Secondary KB onto a canonical Main KB
bokf merge-snapshot ./main-kb
bokf merge-raw ./main-kb ./secondary-kb
bokf merge-snapshot ./main-kb --verify

# Multi-bundle bookkeeping (a KB can live anywhere on disk; registry + active
# pointer live in ~/.config/biookf-studio, shared by the CLI, MCP, and Studio)
bokf register mykb /abs/path/to/mykb            # also --list, --unregister <id>; --root to override
bokf set-active mykb
bokf predicates                                 # print the controlled vocabulary
```

## CLI command reference

| Command | What it does |
|---|---|
| `scaffold` | Create an empty bundle; commit, register, activate. |
| `convert` | Convert a file/folder/zip, `--text`, or `--url`(s) into raw Markdown under `raw/`. |
| `validate` | Validate a single concept-document file without writing it. |
| `get` | Look up a node by exact identifier. |
| `index` | Regenerate `index.md` (or `--check` it). |
| `lint` | Lint against the BioOKF v0.5 conformance rules (`--json`). |
| `verify` | Deterministic gate: lint + structure checks; exits 1 on any error. |
| `graph` | Derive the render-ready graph (nodes + directional edges). |
| `search` | BM25 full-text search over concept documents. |
| `stats` | Node/edge counts by type and predicate. |
| `predicates` | Print the controlled vocabulary (28 types, 35 predicates, enums). |
| `export` | Export a self-contained bundle JSON (graph + per-node detail) for the GUI. |
| `log-sync` | Append a dated `log.md` entry AND commit, atomically (the sole step-committer). |
| `commit` | Lower-level stage-all + commit (non-logged lifecycle commit). |
| `log` | Show commit history (newest-first). |
| `restore` | Forward-only restore to a prior commit. |
| `register` | Register / `--list` / `--unregister` a known bundle (config dir by default; `--root` to override). |
| `set-active` / `get-active` | Set / read the active KB (config dir by default; `--root` to override). |
| `merge-raw` | Relocate a Secondary KB's `raw/` into a Main KB's `raw/` (dedup by content). |
| `merge-snapshot` | Snapshot the Main KB before a merge, or `--verify` after. |
| `name-figure` | Rename a provisional figure to a content caption and rewrite every reference. |
| `install-pdfium` | Install PDFium so PDF pages render to images for vision (one-time). |

## MCP tool reference

`bokf-mcp` exposes 33 tools in three groups.

**Curation:** `bokf_list_bases`, `bokf_scaffold`, `bokf_set_active`, `bokf_get_active`,
`bokf_list_pages`, `bokf_read_page`, `bokf_write_page` (validates concept docs on write),
`bokf_validate_page`, `bokf_append_log`, `bokf_log_sync`, `bokf_log`, `bokf_restore`,
`bokf_convert`, `bokf_name_figure`, `bokf_index`, `bokf_merge_raw`, `bokf_merge_snapshot`.

**Analysis:** `bokf_lint`, `bokf_verify`, `bokf_graph`, `bokf_search`, `bokf_stats`,
`bokf_predicates`.

**Studio GUI control:** `bokf_studio_open`, `bokf_studio_close`, `bokf_studio_status`,
`bokf_studio_state` (the complete GUI status as JSON; read this instead of a screenshot),
`bokf_studio_graph`, `bokf_studio_select`, `bokf_studio_reload`, `bokf_studio_search`,
`bokf_studio_screenshot`, `bokf_studio_narrate`.

## Build from source

The toolchain is a Cargo workspace under [`app/`](app) (with the Studio desktop source in
`app/studio/`). For development, or for a platform without a prebuilt release:

```bash
cd app
cargo build --release -p bokf-cli -p bokf-mcp     # the CLI + MCP server
cargo install tauri-cli --version "^2"            # one-time, for the Studio app
( cd studio/src-tauri && cargo tauri build --bundles app )   # BioOKF Studio.app

cargo test --workspace                            # backend + integration tests
```

Binaries land in `app/target/release/` (and the app under
`app/target/release/bundle/`). To make the plugin use a local build instead of downloading a
release, set `BIOOKF_MCP_BIN=/path/to/app/target/release/bokf-mcp`.

**Repo dev plugin (skills + guardrails).** The repo also ships a developer-facing Claude Code
extension at [`app/.claude-plugin`](app/.claude-plugin): 8 `biookf-*` curation skills
(convert / ingest / merge / query / lint / verify / version) plus 4 guardrail hooks (a session
brief, a `raw/`-immutability guard, a post-write lint nudge, and a `bokf verify` Stop gate). Add
`app/` as a marketplace to use them while working inside this repository. The published plugin root
under [`plugins/biookf`](plugins/biookf) carries both `.claude-plugin` and `.codex-plugin`
manifests side by side; Codex gets a dedicated `skills/biookf` workflow brief while sharing the
same `bokf-mcp` launcher.

## Authors

Wanjun Gu (<wanjun.gu@ucsf.edu>), Gianmarco Bellucci (<Gianmarco.Bellucci@ucsf.edu>), Ilan Ladabaum
(<Ilan.Ladabaum@ucsf.edu>), James Xue (<jinxuejim@gmail.com>), Jonathan Xue (<jonathanx622@gmail.com>),
and Xi Zheng (<zhx2031558@gmail.com>).

## License

Apache-2.0.
