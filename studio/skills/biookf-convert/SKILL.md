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
3. **Vision rendering (the important part).** **PDFs are always rendered by your vision**, not a
   deterministic parser: pure-Rust PDF text extraction silently corrupts mathematical formulas and
   special characters and misses figure content, so `bokf` does not attempt it. Unknown or scanned
   formats and failed extractions also return `needs_llm_fallback: true` with a
   `<!-- bokf:needs-conversion -->` marker in `source.md`. **You must finish each one:** open
   `raw/<id>/original.*` and read every page with your vision, then overwrite `raw/<id>/source.md`
   (removing the marker) with a faithful rendering:
   - **Text and tables:** transcribe everything exactly, preserving structure and reading order.
   - **Formulas:** render every equation and inline expression correctly as LaTeX (`$...$` /
     `$$...$$`); never approximate, drop, or guess symbols, subscripts, or superscripts.
   - **Figures:** for each figure, chart, diagram, gel, micrograph, or plot, describe what it shows
     AND transcribe its axes, labels, legend, and data values. Key findings often live in the
     figures, not only the text. Do not summarize away detail.
4. **Figures pass (office, html, md sources).** For docx/pptx/xlsx media, html/md data URIs, and
   loose folder/zip images, `bokf convert` pulls every embedded image into `raw/<id>/figures/` with
   a provisional name like `fig-001.png`, references each in `source.md`, and records them in
   `meta.yaml`. (PDF figures are NOT extracted to files; you read them directly from `original.pdf`
   with your vision in step 3 and mine them for nodes/edges in biookf-ingest.) For each figure:
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
- **PDF page rendering (no setup needed):** PDFs always convert, because you read them with vision.
  When the optional PDFium library is installed (one command, `bokf install-pdfium`), `bokf convert`
  also writes high-resolution page images to `raw/<id>/pages/page-NNN.jpg` for you to read; when it
  is not, read `raw/<id>/original.pdf` directly. The `pages/` cache is local-only (gitignored).
- Then proceed to **biookf-ingest** (Step 2+): extract typed nodes and provenance-stamped edges.
