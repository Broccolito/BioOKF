---
name: biookf
description: Use when curating, querying, validating, or visualizing BioOKF knowledge bases through the bokf MCP tools, CLI, and BioOKF Studio from Codex.
---

# Skill: biookf

Use the `biookf` MCP server first when available. It exposes the `bokf_*` tools for scaffold,
conversion, curation, indexing, linting, verification, graph export, search, statistics, and live
BioOKF Studio control. The same operations are available through the `bokf` CLI.

Core rules:

- Work in a BioOKF bundle: `raw/`, `knowledge/<type>/<slug>.md`, `index.md`, `log.md`, and `SCHEMA.md`.
- Treat `raw/` as immutable. Put source bytes there with `bokf_convert`; never hand-edit raw files.
- Reuse existing identifiers before creating new concept pages. Validate drafts with
  `bokf_validate_page` before writing them with `bokf_write_page`.
- Every concept `type` must be one of the 28 BioOKF node types; every edge predicate must be one of
  the controlled predicates, including `not_<predicate>` only for explicitly negated effect claims.
- Every edge needs `knowledge_level`, `agent_type`, and `primary_source`; `primary_source` names a
  Publication, Study, Dataset, or Agent source node, not a CURIE.
- Track curation steps with `bokf_log_sync --kind <kind> --summary <summary> --delta <delta>`.
  There is no `--counts` flag; use `bokf_stats` first when you want counts in the delta.
- Finish ingest or merge work with `bokf_verify`; fix every error before calling the bundle clean.

For visual QA, use `bokf_studio_open` and prefer `bokf_studio_state` / `bokf_studio_graph` over
screenshots unless the user explicitly asks for a visual screenshot.
