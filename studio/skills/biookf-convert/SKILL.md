---
name: biookf-convert
description: Use to turn a dropped source (pdf/html/docx/pptx/csv/xlsx/zip/folder/url/unknown) into immutable raw bytes + faithful raw Markdown under raw/, before extracting nodes. Ingestion Step 1.
---

# Skill: biookf-convert

Ingestion **Step 1**: get every source into `raw/` as BOTH its original bytes and a faithful
Markdown rendering. By the end, **every source is complete `.md` with the most raw information
preserved.** Never hand-edit `raw/` originals.

## Loop
1. Make sure a KB is active (`bokf get-active <root>`); convert into it with `--into <bundle>`.
2. Run **`bokf convert <path | --text "…">  --into <bundle> [--combined] --json`**. It writes
   `raw/<id>/{original.*, source.md, meta.yaml}` with a human-readable, **content-derived** id
   (title → slug; never a bare hash). A `.zip`/folder expands to **one source per member** (use
   `--combined` to merge into one). Identical bytes are de-duplicated (`reused: true`).
3. **LLM fallback (the important part).** Known formats convert deterministically; anything else
   (an unknown extension like `random_file.xyz`, a scanned PDF, or a failed extraction) comes back
   with `needs_llm_fallback: true` and leaves a `<!-- bokf:needs-conversion -->` marker in
   `source.md`. **You must finish it:** read `raw/<id>/original.*`, render **all** content (text,
   tables, figure/caption text) faithfully to Markdown, and overwrite `raw/<id>/source.md`,
   **removing the marker**. Preserve raw detail; do not summarize.
4. Re-check: `bokf verify <bundle>` (or `bokf lint`) flags `source.needs_conversion` for any source
   still carrying the marker; clear them all.
5. Record it: `bokf log-sync <bundle> --kind convert --summary "converted N sources"`.

## Rules
- `raw/` originals are immutable; only `source.md` (the rendering) is yours to (re)write.
- Everything ends as faithful Markdown: the deterministic converter handles what it can, you handle
  the rest. Nothing is left binary-only.
- Then proceed to **biookf-ingest** (Step 2+): extract typed nodes and provenance-stamped edges.
