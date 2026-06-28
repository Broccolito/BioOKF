# BioOKF Raw Conversion v2: Figures, URL Ingestion, and Source Provenance (SP7)

**Goal:** Extend `bokf convert` so that (1) figures in every supported format are extracted into the raw store, committed, named by content, and fed to the LLM for both transcription and node/edge extraction; (2) a list of URLs can be ingested by downloading their content; and (3) every ingested source is classified for origin and credibility (peer-reviewed, preprint archive, database, grey literature, or web) using a deterministic resolver chain modeled on BioRouter.

**Architecture:** All new behavior lives in `bokf-core` and surfaces through `bokf` (CLI) and `bokf-mcp` (MCP tools), keeping the GUI and skills as thin clients. Figure extraction is offline and deterministic. URL download and provenance resolution are Rust-native (synchronous `reqwest` blocking client plus the free, no-auth Crossref and OpenAlex APIs). The LLM remains in the loop only for what deterministic code cannot do: describing figures, extracting concepts and edges from figure content, and rendering scanned or paywalled PDFs.

**Tech Stack:** Rust workspace at `studio/`. Existing: `zip`, `calamine`, `html2md`, `sha2`, `serde`/`serde_yaml`/`serde_json`, `regex`. New: `reqwest` (blocking, rustls-tls, json) in `bokf-core`. No new image-processing dependency in this sub-project: extracted images are already-encoded bytes that we copy verbatim. PDF page rasterization (pdfium) is deferred to a follow-on slice.

## Global Constraints

- Rust edition 2021; workspace versions from `studio/Cargo.toml`. New dep: `reqwest = { version = "0.12", default-features = false, features = ["blocking", "rustls-tls", "json"] }` in `bokf-core` (blocking client so `bokf-core` stays synchronous and tokio-free).
- `raw/<id>/original.*` remains the only immutable, gitignored artifact. Everything else under `raw/<id>/` (including `figures/`) is committed by default; the current `.gitignore` rule `raw/**/original.*` already achieves this with no change.
- Prose in code, docs, skills, and tool messages uses plain language: no em-dashes, no AI-sounding filler. Controlled vocabulary, CURIEs, predicate names, and node-type names are never altered.
- Source ids and figure names are content-derived and human-readable, never a bare hash or uuid (the existing `looks_machine_generated` guard applies to both).
- Network access is best-effort. Every resolver and download path fails soft: on timeout or non-2xx, the pipeline records what it has and flags the rest for the agent, never aborting a batch.

## Decisions captured from design review

- Figures are committed to git by default (they are small relative to a graph and are needed by the GUI and reviewers).
- During ingestion the LLM receives the figures, not only their captions, so figure content yields nodes and edges.
- A folder or zip is treated as one logical source set: all loose image files attach as figures of the primary document in that folder (the largest or sole document). Standalone images dropped on their own become their own image-source.
- Markdown filenames and image filenames must reflect the true content of the source, including inside `raw/`.
- URL networking and Crossref/OpenAlex resolution are Rust-native inside `bokf`.
- `source_type` (origin) and `credibility` (trust) are separate fields, fixing BioRouter's conflation of the two.

---

## Slice A: Figures across all formats (offline, no new deps)

### A.1 Raw layout

```
raw/<id>/
  original.<ext>      # immutable bytes, gitignored (unchanged)
  figures/            # extracted images, committed
    <content-slug>.png
  source.md           # references figures with ![desc](figures/<slug>.png)
  meta.yaml           # gains a figures inventory
```

`SourceMeta` (in `convert.rs`) gains:

```rust
pub figures: Vec<FigureMeta>,   // default empty, skip_serializing_if empty

pub struct FigureMeta {
    pub file: String,            // relative to raw/<id>/, e.g. "figures/kaplan-meier-by-arm.png"
    pub provisional: bool,       // true until named by content
    pub described: bool,         // true once source.md carries a non-empty description
    pub origin: String,          // "word/media/image1.png", "data-uri", "folder:figure3.png", etc.
}
```

### A.2 Extraction by format

A new `extract_figures(ext, bytes) -> Vec<(provisional_name, image_bytes, origin)>` runs alongside text conversion in `store()`. Image bytes are written under `raw/<id>/figures/` with provisional names (`fig-001.png` or the original media basename), and a `![fig-001](figures/fig-001.png)` placeholder plus the needs-conversion marker is appended to `source.md`.

| Format | Image source | Method |
|---|---|---|
| docx | `word/media/*` | existing `zip` reader, copy image members |
| pptx | `ppt/media/*` | existing `zip` reader, copy image members |
| xlsx/ods | `xl/media/*` | existing `zip` reader, copy image members |
| html/htm | `<img src="data:image/...;base64,...">` | decode base64 to bytes; rewrite the reference to the local figure |
| md/markdown | `![](data:...)` and `![](local/path)` | decode data URIs; resolve local paths when ingested from a folder/zip |
| standalone image (png/jpg/jpeg/gif/webp/tiff/bmp/svg) | the file itself | becomes an image-source whose single figure is the image |
| folder / zip | per the grouping rule below | see A.3 |
| pdf | page figures | deferred to Slice C |

Recognized image extensions are added to the dispatch in `convert_bytes` as an `Image` class: the produced `source.md` is `![<filename>](figures/<file>)` and `needs_llm_fallback = true` (the agent must describe and, in Slice B+, extract from it).

### A.3 Folder and zip grouping

`ingest()` already expands a folder or zip to members. New behavior:

1. Partition members into documents (md/html/docx/pptx/pdf/text) and images (the image extensions above).
2. Choose the primary document: the single document if there is one, otherwise the largest by byte length.
3. Attach every loose image in the set as a figure of the primary document's source (copied into its `figures/`, recorded in `meta.figures` with `origin = "folder:<member>"`).
4. If the set has no document (images only), fall back to one image-source per image.

Non-primary documents in the same folder still convert to their own sources; only the loose images join the primary document.

### A.4 Content-true naming

A new command renames a provisional figure to a content slug and keeps every reference consistent:

`bokf name-figure <bundle> raw/<id>/figures/fig-001.png --as "Kaplan-Meier survival by arm"`

It slugifies the caption (reusing `slug`), moves the file (git-friendly), rewrites the matching `![...](figures/fig-001.png)` reference in `source.md` to the new path, sets `FigureMeta.provisional = false`, and updates `meta.yaml`. Because it runs through `bokf`, it does not trip the raw-guard hook. Source folder ids and `source.md` placement are already content-derived; this brings images to the same standard.

### A.5 Lint and verify

New rule `source.figure_undescribed` (Warning): any `figures/*` recorded in `meta.yaml` with `described = false`, or referenced in `source.md` with an empty alt/description, is flagged. A second rule `source.figure_unnamed` (Warning) flags figures still `provisional = true` at verify time. These join the existing `source.needs_conversion` gate so the convert step is not considered complete until figures are both named and described.

### A.6 Skills

`biookf-convert` (Step 1) gains a figures pass: after deterministic extraction, the agent views each `figures/*`, writes a faithful description into `source.md` beside the reference, and runs `bokf name-figure` to give it a content name. `biookf-ingest` (Step 2) gains an explicit instruction to view `raw/<id>/figures/*` and extract typed nodes and provenance-stamped edges from figure content (for example, a survival curve yields outcome edges; a pathway diagram yields `regulates`/`activates` edges).

---

## Slice B: URL ingestion and source provenance (Rust-native networking)

### B.1 Commands

- `bokf convert --url <u> --into <bundle>`: download one URL, convert, classify, store.
- `bokf convert --urls <file|-> --into <bundle>`: one URL per line; ingested with bounded concurrency is not required (sequential is acceptable for v1), with a small in-run DOI cache. Blank lines and `#` comments are skipped.

Download uses a `reqwest::blocking::Client` with a 30 second timeout and a `BioOKF/<version>` user agent, captures the post-redirect final URL and the `Content-Type`, and reconciles MIME by header, extension, and byte sniffing (`%PDF-`, `<html`). The downloaded bytes flow through the same `convert_bytes` plus figure extraction as a local file. Per the decision to keep archival fidelity, the raw downloaded bytes are stored as `original.<ext>` (unlike BioRouter, which discards URL bytes).

### B.2 Provenance model

`SourceMeta` gains:

```rust
pub url: Option<String>,
pub final_url: Option<String>,
pub source_type: SourceType,      // origin/kind
pub credibility: Credibility,     // trust
pub ids: SourceIds,               // doi, pmid, pmcid, arxiv, isbn
```

```rust
pub enum SourceType {            // origin, distinct from trust
    JournalArticle, Preprint, Review, Book, Dataset, Database,
    ClinicalGuideline, GovReport, WebPage, Personal, Unknown,
}

pub struct Credibility {
    pub tier: CredibilityTier,    // PeerReviewed | Preprint | Archive | GrayLit | Web | Unknown
    pub confidence: f32,          // 0.3..0.95
    pub retracted: bool,
    pub venue: Option<String>,
    pub publisher: Option<String>,
    pub reasoning: String,        // human-readable explanation
    pub classifier_version: u32,  // 1 deterministic, 2 agentic
}

pub struct SourceIds { pub doi: Option<String>, pub pmid: Option<String>,
                       pub pmcid: Option<String>, pub arxiv: Option<String>, pub isbn: Option<String> }
```

This feeds the graph: an ingested scholarly source becomes a `Publication`/`Study`/`Dataset` node whose `xref` carries the DOI/PMID/PMCID, and the credibility annotation rides with it. Edges already name a `primary_source` node, so the classification enriches exactly the node edges point to.

### B.3 Classifier (new module `credibility/`)

A deterministic-first waterfall, first hit wins, mirroring BioRouter and closing its gaps:

1. `identifiers::extract` over final URL + filename + converted body: DOI (`\b10\.\d{4,9}/[-._;()/:A-Z0-9]+\b`), arXiv, PMID, PMCID, ISBN. New: detect the `10.1101/` DOI prefix as bioRxiv/medRxiv.
2. DOI resolution: `GET api.crossref.org/works/{doi}` then on miss `api.openalex.org/works/doi:{doi}`. Map registry work-type to `source_type` and `tier`: `journal-article|proceedings-article|review-article` to PeerReviewed; `posted-content` to Preprint/Archive; `book*` to Book; `dataset` to Dataset (a tier BioRouter lacks). Read retraction from Crossref `update-to[].type == "retraction"` or OpenAlex `is_retracted`. The publisher allowlist only raises confidence, it never gates the tier.
3. Host patterns for URLs: bioRxiv/medRxiv/arXiv/SSRN/OSF/chemRxiv to Preprint/Archive; `.gov`/`.edu`/WHO/CDC/NIH/FDA/clinicaltrials.gov to GrayLit; any other `http(s)` to Web.
4. Conservative text heuristics when no DOI resolves: preprint fingerprints plus a DOI or journal marker to Preprint; DOI plus at least two journal markers to PeerReviewed.
5. LLM fallback only if nothing deterministic fires (the agent fills `meta.yaml` and sets `classifier_version = 2`).

Networking is polite: a fixed minimum interval between Crossref/OpenAlex calls, one retry with backoff on transient failure, and an in-run DOI to metadata cache so a batch never asks twice. The user agent carries a mailto for the Crossref polite pool.

### B.4 Dedup

By URL first (same `meta.url` reuses and refreshes the source) then by sha256 of the stored bytes, matching the existing content-hash dedup in `store()`.

### B.5 Lint

New rule `source.not_scholarly` (Warning): when a source used as a `primary_source` on any edge has `credibility.tier` in {Web, Unknown}, warn that the edge's evidence base is not a recognized peer-reviewed, preprint-archive, or database source. This is the deterministic form of your "check whether the URL is linked to a peer-reviewed source at all" requirement. `source.retracted` (Warning) flags any source with `retracted = true` still cited as a primary source.

### B.6 Skills

`biookf-convert` documents `--url`/`--urls`, shows the provenance fields it writes, and instructs the agent to inspect low-confidence or `web`/`unknown` verdicts and either correct them or note the weak provenance. `about-biookf` and `SCHEMA.md` describe `source_type` vs `credibility` and how they map onto `Publication`/`Study`/`Dataset` nodes and `xref`.

---

## Slice C (follow-on, not in this plan): PDF figure extraction

PDF figure extraction needs page rasterization and layout, which carries the pdfium native library plus optional ONNX layout/OCR models (the Option A pipeline discussed in review). It is scoped as its own sub-project so this plan ships figures for every other format plus URL ingestion and provenance without taking on that binary and model footprint. Until then, PDFs keep the `needs_llm_fallback` path: the agent reads the original and renders text and figures.

## Testing strategy

- Slice A: unit tests that a synthetic docx/pptx/xlsx with an embedded PNG yields a `figures/` entry and a `source.md` reference; that an HTML/MD data URI is decoded; that a folder of one markdown plus two PNGs attaches both as figures of the markdown source; that a standalone PNG becomes an image-source; that `name-figure` renames the file, rewrites the reference, and updates `meta.yaml`; that `source.figure_undescribed` fires until a description is present.
- Slice B: unit tests for identifier regexes (including `10.1101/` to bioRxiv); for Crossref/OpenAlex JSON-to-tier mapping using fixture JSON (no live network in tests); for host-pattern and text-heuristic classification; for URL and sha256 dedup; for `source.not_scholarly` firing on a web-tier primary source. Live network is exercised only behind an ignored integration test.
- Full workspace `cargo test` and the hook test script stay green.

## Risks and mitigations

- Network flakiness or API changes: every resolver fails soft to the next waterfall stage, ending at the LLM fallback; classification never blocks ingestion.
- Repo growth from committed figures: figures are small and content-named; a future flag can opt a bundle out if needed, but default-on per the decision.
- Over-grouping in folders: the primary-document rule can misattribute when a folder holds several papers; documented behavior is "largest or sole document wins," and a curator can re-run `name-figure`/move a figure if needed.
- New `reqwest` dependency: confined to `bokf-core` behind the blocking client; the GUI and offline commands do not invoke it.
