# BioOKF Studio — test plan & results

BioOKF Studio is, first, an **agentic backend** (an MCP server + CLI over `okf-core`) and,
second, a **Tauri visualizer** front-end over that backend. The tests cover both, plus the
three agentic loops (ingest / lint / query) exercised on the real `reviews/` corpus.

## How to run

```bash
cd studio
cargo test -p okf-core -p okf-cli      # backend unit + integration tests
cargo build                            # builds all 4 crates incl. the Tauri app
./target/debug/okf-mcp                 # MCP server over stdio (drive from Claude/Codex)
# frontend (same code as the Tauri webview):
cd app/dist && python3 -m http.server 8754   # then open http://localhost:8754
npx playwright test app/tests/visual.spec.mjs # frontend assertions (needs @playwright/test)
```

## 1. Backend — `okf-core` (library)  ✅ 7/7

`cargo test -p okf-core`:
- `parses_frontmatter_split`, `parses_a_node_with_edges_and_normalizes_legacy`
  (title+id→identifier merge; inverse predicate `caused_by`→`causes` + reversed flag),
  `node_type_palette_is_complete` (28 types / 23 predicates, all colored).
- Integration `tests/examples.rs` on the real `examples/` bundle: opens it (self-healing YAML
  repairs the malformed `sider.md`), derives a graph whose every edge endpoint resolves,
  lints (produces findings, never panics), and BM25-searches ("interleukin").

## 2. Backend — `okf` CLI  ✅ 1/1 + manual

`cargo test -p okf-cli` (`tests/cli.rs`): scaffold → author valid source + 2 concept docs with a
provenance-stamped edge → `okf lint --json` is clean (0 errors, exit 0) → `okf graph` contains the
BRAF→Melanoma edge → `okf search "kinase"` finds BRAF → introduce an invalid `type` + bad
`predicate` → lint flags `type.invalid` + `predicate.invalid` and exits non-zero.
Manual: `okf validate <file>` (valid/invalid + issue list), `okf get <bundle> <id>` (exact lookup),
`okf stats`, `okf export`.

## 3. Backend — `okf-mcp` (MCP server)  ✅ verified

A real MCP stdio handshake (`initialize` → `tools/list` → `tools/call`) confirms the server
advertises **11 tools** (`okf_scaffold`, `okf_list_pages`, `okf_read_page`, `okf_write_page`,
`okf_validate_page`, `okf_append_log`, `okf_lint`, `okf_graph`, `okf_search`, `okf_stats`,
`okf_list_bases`) and returns correct results for `okf_stats` and `okf_search` on the examples
bundle. `get_info()` ships the agent-facing operating brief + the ingest/query/lint procedures.

## 4. Agentic loops on the real corpus  ✅ 4/4 clean

Four review articles were ingested by AI sub-agents acting as the MCP/CLI client
(ingest → self-lint/fix → query). Final `okf lint`:

| Knowledge base | nodes | edges | lint |
|---|---:|---:|---|
| Drug resistance in cancer | 24 | 56 | 0 err · 0 warn · 0 info |
| Type 2 diabetes | 29 | 92 | 0 err · 0 warn · 0 info |
| Microbiota & colorectal cancer | 28 | 88 | 0 err · 0 warn · 0 info |
| Fungal impacts on ecosystems | 31 | 71 | 0 err · 0 warn · 0 info |

The agents authored correct v0.5 (28 types, 23 forward-only predicates, node-based
`primary_source`, `raw_source`-anchored sources) and reached **zero findings**. Issues they flagged
about the loops/spec are recorded in [ISSUES.md](ISSUES.md).

## 5. Frontend — the visualizer  ✅ verified in-browser

The frontend (`app/dist`, identical to the Tauri webview) was driven with Playwright against
**real `okf-core` exports**:
- Loads the 5-base sidebar; selecting a base renders the typed, color-coded graph (verified on the
  examples bundle and the agentic Type-2-diabetes KB: 29 nodes / 92 edges).
- **Node click** → glass detail panel with the real frontmatter, synonyms + xref CURIE chips, and
  edges grouped by predicate (COVID-19, Tocilizumab).
- **Edge click** → provenance triplet (node-based `primary_source`) + quantitative attributes
  ("Saturated fat intake → regulates → Hepatic gluconeogenesis").
- Directional tapered edges, faint dashed **synthesized provenance** edges, dashed **external**
  stubs, search dimming, collapse, pan/zoom, zoom controls. Screenshots in `mockups/screens/` and
  `okf-app-*.png`. Repeatable assertions in `app/tests/visual.spec.mjs`.

## 6. Tauri integration  ✅ compiles

`cargo build -p biookf-studio` compiles the desktop app (tauri 2.11): commands `list_bases`,
`get_bundle`, `lint_bundle`, `search_bundle` delegate to `okf-core`; `tauri-plugin-mcp` is wired
behind the `debug-mcp` feature for agent-driven webview debugging. The GUI is not launched in CI
(headless has no window server); the webview content is the tested `app/dist` frontend.
