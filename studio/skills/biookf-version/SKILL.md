---
name: biookf-version
description: Use to inspect or operate the git-backed version tracking — review history, keep log.md<->commit parity, forward-only restore.
---

# Skill: biookf-version

Every curation step is committed through **`bokf log-sync <bundle> --kind <K> --summary S
[--delta D] [--counts]`** — it appends a dated `## YYYY-MM-DD` block to `log.md` AND git-commits,
atomically. It is the **sole step-committer**; do not hand-commit per tool. `--kind` is one of
`ingest | convert | link | merge | lint | index | restore | manual`; `--counts` auto-fills
node/edge/source counts from `bokf stats`.

## Operations
- `bokf log <bundle> [--limit N] [--json]` — history, newest-first (`sha`, `kind`, `summary`,
  `delta`, `timestamp`).
- `bokf restore <bundle> <sha> [--summary S]` — **forward-only**: reproduces the old tree as a NEW
  `[restore]` commit on top of HEAD; it never rewinds history.
- **Parity:** because `log-sync` couples the `log.md` block and the commit, they cannot drift;
  `bokf verify` surfaces any anomaly.
- `bokf commit <bundle> --kind K --summary S` is for **non-logged lifecycle** commits only (scaffold
  init, transaction squash). Normal curation always uses `log-sync`.

`bokf scaffold` git-inits the bundle, registers it, and sets it active; a pre-existing un-versioned
bundle gets a repo on its first `log-sync` (init-on-first-use).
