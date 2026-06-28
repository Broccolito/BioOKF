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
4. **Figures pass.** `bokf convert` pulls every embedded image (docx/pptx/xlsx media, html/md
   data URIs, loose folder/zip images) into `raw/<id>/figures/` with a provisional name like
   `fig-001.png`, references each in `source.md`, and records them in `meta.yaml`. For each figure:
   - View the image file under `raw/<id>/figures/*` (use `bokf_read_page` on the figure path).
   - Write a faithful description of what the figure shows beside its reference in `source.md`,
     filling in the `![...]` alt text so the reference reads `![<description>](figures/<name>)`.
   - Run `bokf name-figure <bundle> --source <id> --figure figures/fig-001.png --as "<caption>"`
     to give the figure a content name. This renames the file, rewrites the `source.md` reference,
     and updates `meta.yaml`.
   - Repeat until `bokf verify`/`bokf lint` reports no `source.figure_unnamed` and no
     `source.figure_undescribed`.
5. Re-check: `bokf verify <bundle>` (or `bokf lint`) flags `source.needs_conversion` for any source
   still carrying the marker, plus `source.figure_unnamed`/`source.figure_undescribed` for figures
   still provisional or without a description; clear them all.
6. Record it: `bokf log-sync <bundle> --kind convert --summary "converted N sources"`.

## Rules
- `raw/` originals are immutable; only `source.md` (the rendering) is yours to (re)write. Figures
  under `raw/<id>/figures/` are written by `bokf`; rename them with `bokf name-figure`, never by hand.
- Everything ends as faithful Markdown: the deterministic converter handles what it can, you handle
  the rest. Nothing is left binary-only.
- Every figure ends both named by content and described in `source.md`.
- Then proceed to **biookf-ingest** (Step 2+): extract typed nodes and provenance-stamped edges.
