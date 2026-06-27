---
name: about-biookf
description: Use for any question about BioOKF Studio itself — what it is, the format, the tools, and how the pieces fit together.
---

# Skill: about-biookf

**BioOKF Studio** is an MCP server + CLI + visualizer for **BioOKF** knowledge bases. The agentic layer (the MCP tools + these skills) is the backbone; the Tauri GUI is just a front-end that visualizes the same data.

## The format (BioOKF v0.5)
A bundle is a Git-shippable tree of Markdown files: `raw/` (immutable sources), `knowledge/<type>/<slug>.md` (typed concept docs = the graph), `index.md` (catalog), `log.md` (history). Each concept doc = YAML frontmatter + Markdown body. **28** controlled node types, **23** forward-only edge predicates, node-based provenance (`primary_source` names a source node). BioOKF is the strict biomedical profile of the Open Knowledge Format (OKF).

## The pieces
- **okf-core** — the library: parse, normalize (accepts v0.4/legacy and emits v0.5), derive graph, lint, BM25 search.
- **okf** (CLI) — `okf lint|graph|search|stats|scaffold <bundle>`.
- **okf-mcp** — the stdio MCP server an AI (Claude/Codex) drives: `okf_scaffold`, `okf_list_pages`, `okf_read_page`, `okf_write_page`, `okf_validate_page`, `okf_append_log`, `okf_lint`, `okf_graph`, `okf_search`, `okf_stats`, `okf_list_bases`.
- **BioOKF Studio** (Tauri app) — sidebar of bundles + a glassy infinite canvas: nodes colored by the 28 types, directional edges (tapered toward the object), clickable nodes/edges → a Markdown detail panel.

## How to work
Use the **biookf-ingest**, **biookf-query**, and **biookf-lint** skills for the three loops. Always `okf_validate_page` before `okf_write_page`, and `okf_lint` after a batch of edits.
