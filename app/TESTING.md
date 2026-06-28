# BioOKF Studio: test plan & results

BioOKF Studio is, first, an **agentic backend** (an MCP server + CLI over `bokf-core`) and,
second, a **Tauri visualizer** front-end over that backend. The tests cover both, plus the
three agentic loops (ingest / lint / query) exercised on the real `reviews/` corpus.


## How to run

```bash
cd studio
cargo test -p bokf-core -p bokf-cli      # backend unit + integration tests
cargo build                            # builds all 4 crates incl. the Tauri app
./target/debug/bokf-mcp                 # MCP server over stdio (drive from Claude/Codex)
# frontend (same code as the Tauri webview):
cd app/dist && python3 -m http.server 8754   # then open http://localhost:8754
npx playwright test app/tests/visual.spec.mjs # frontend assertions (needs @playwright/test)
```

The live-control plane ships in normal builds (default `control` feature); the socket
server + guest-inject only attach when `BIOOKF_STUDIO_CONTROL=1`, which `bokf_studio_open`
sets automatically, so no special feature flag or build is required.

## 1. Backend: `bokf-core` (library)  ✅ 40+ unit + 4 integration suites

`cargo test -p bokf-core` runs ~40+ unit tests plus the integration suites
`tests/active.rs`, `tests/cli.rs`, `tests/examples.rs`, `tests/version.rs`. Highlights:
- `parses_frontmatter_split`, `parses_a_node_with_edges_and_normalizes_legacy`
  (title+id→identifier merge; inverse predicate `caused_by`→`causes` + reversed flag),
  `node_type_palette_is_complete` (28 types / 35 predicates = 24 positive + 11 `not_<X>`
  negatives, all colored), plus `not_<X>` validation/range checks.
- Integration `tests/examples.rs` on the real `examples/` bundle: opens it (self-healing YAML
  repairs the malformed `sider.md`), derives a graph whose every edge endpoint resolves,
  lints (produces findings, never panics), and BM25-searches ("interleukin cytokine").

## 2. Backend: `bokf` CLI  ✅ integration + manual

`cargo test -p bokf-cli` (`tests/cli.rs`): scaffold → author valid source + 2 concept docs with a
provenance-stamped edge → `bokf lint --json` is clean (0 errors, exit 0) → `bokf graph` contains the
BRAF→Melanoma edge → `bokf search "kinase"` finds BRAF → introduce an invalid `type` + bad
`predicate` → lint flags `type.invalid` + `predicate.invalid` and exits non-zero.

The CLI surface is now large: `scaffold`, `validate`, `get`, `export`, `lint`, `graph`, `search`,
`stats`, `predicates`, `log`, `log-sync`, `commit`, `restore`, `set-active`, `get-active`,
`register`, `verify`, `convert`, `install-pdfium`, `name-figure`, `index`, `merge-raw`,
`merge-snapshot`.
Manual: `bokf validate <file>` (valid/invalid + issue list), `bokf get <bundle> <id>` (exact lookup),
`bokf stats`, `bokf export`.

## 3. Backend: `bokf-mcp` (MCP server)  ✅ verified

A real MCP stdio handshake (`initialize` → `tools/list` → `tools/call`) confirms the server
advertises **33 tools** and returns correct results for `bokf_stats` and `bokf_search` on the
examples bundle. `get_info()` ships the agent-facing operating brief + the ingest/query/lint
procedures. The 33 tools, grouped:

- **Core curation:** `bokf_list_bases`, `bokf_scaffold`, `bokf_set_active`, `bokf_get_active`,
  `bokf_list_pages`, `bokf_read_page`, `bokf_write_page`, `bokf_validate_page`, `bokf_append_log`,
  `bokf_log_sync`, `bokf_log`, `bokf_restore`, `bokf_convert`, `bokf_name_figure`, `bokf_index`,
  `bokf_merge_raw`, `bokf_merge_snapshot`.
- **Analysis:** `bokf_lint`, `bokf_verify`, `bokf_graph`, `bokf_search`, `bokf_stats`,
  `bokf_predicates`.
- **Studio GUI control:** `bokf_studio_open`, `bokf_studio_close`, `bokf_studio_status`,
  `bokf_studio_state`, `bokf_studio_graph`, `bokf_studio_select`, `bokf_studio_reload`,
  `bokf_studio_search`, `bokf_studio_screenshot`, `bokf_studio_narrate`.

### MCP ↔ Studio GUI control

The `bokf_studio_*` tools drive and observe the *running* GUI over a Unix control socket via the
`studio_client` bridge (newline-delimited JSON) at `/tmp/biookf-tauri-mcp.sock` (override with
`BIOOKF_STUDIO_IPC`). `bokf_studio_open` spawns `biookf-studio` with `BIOOKF_STUDIO_CONTROL=1` and
waits for the webview to come up; `bokf_studio_state` returns the complete GUI status as structured
JSON (base, counts, query, selection, panel/sidebar/terminal flags, lint, lastAgentAction, bases[]),
so agents read status WITHOUT screenshots. Each agent action is shown live in an in-app "AI agent"
banner so a human watching sees what the agent is doing.

## 4. Agentic loops on the real corpus  ✅ 4/4 clean

Four review articles were ingested by AI sub-agents acting as the MCP/CLI client
(ingest → self-lint/fix → query). Final `bokf lint`:

| Knowledge base | nodes | edges | lint |
|---|---:|---:|---|
| Drug resistance in cancer | 24 | 56 | 0 err · 0 warn · 0 info |
| Type 2 diabetes | 29 | 92 | 0 err · 0 warn · 0 info |
| Microbiota & colorectal cancer | 28 | 88 | 0 err · 0 warn · 0 info |
| Fungal impacts on ecosystems | 31 | 71 | 0 err · 0 warn · 0 info |

The agents authored correct v0.5 (28 types, 24 positive + 11 `not_<X>` = 35 predicates, node-based
`primary_source`, `raw_source`-anchored sources) and reached **zero findings**. Issues they flagged
about the loops/spec are recorded in [ISSUES.md](ISSUES.md).

## 5. Frontend: the visualizer  ✅ verified in-browser

The frontend (`app/dist`, identical to the Tauri webview) was driven with Playwright against
**real `bokf-core` exports**:
- Loads the 6-base sidebar; selecting a base renders the typed, color-coded graph (verified on the
  examples bundle and the agentic Type-2-diabetes KB: 29 nodes / 92 edges). The sidebar is now
  **registry-driven**: bundles are registered by absolute path (in `registry.yaml`) and can live
  ANYWHERE on disk. The example/corpus bundles were moved out of the repo onto the user's Desktop
  and are registry-linked. Broken/missing registry paths auto-drop from the sidebar, and newly
  registered ones auto-appear without a restart (a live re-sync poll). `app/test-kb` no longer
  exists.
- **Node click** → glass detail panel with the real frontmatter, synonyms + xref CURIE chips, and
  edges grouped by predicate (COVID-19, Tocilizumab).
- **Edge click** → provenance triplet (node-based `primary_source`) + quantitative attributes
  ("Saturated fat intake → regulates → Hepatic gluconeogenesis").
- Directional tapered edges, faint dashed **synthesized provenance** edges, dashed **external**
  stubs, search dimming, collapse, pan/zoom, zoom controls. The in-repo screenshot is
  `app/studio/screens/native-01.png`. Repeatable assertions in `app/tests/visual.spec.mjs`.

## 6. Tauri integration  ✅ compiles

`cargo build -p biookf-studio` compiles the desktop app (tauri 2). The backend now exposes many
commands beyond the original four `list_bases` / `get_bundle` / `lint_bundle` / `search_bundle`
(which delegate to `bokf-core`), e.g. `add_base`, `set_active_kb` / `get_active_kb`, `source_info`,
the `term_*` PTY terminal commands (`term_open` / `term_write` / `term_resize` / `term_close`), and
the node/edge save commands. The live-control plane is wired through the **default `control`
feature** (`tauri-plugin-mcp`; `debug-mcp` is kept only as a back-compat alias), so it ships in
normal builds. The socket server + guest-inject only attach when `BIOOKF_STUDIO_CONTROL=1` (set
automatically by `bokf_studio_open`). The GUI is not launched in CI (headless has no window server);
the webview content is the tested `app/dist` frontend.
