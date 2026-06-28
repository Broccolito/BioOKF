# BioOKF Raw Conversion v2 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend `bokf convert` to extract figures from every supported format into the committed raw store, name them by content, and feed them to the LLM; and to ingest a list of URLs while classifying each source's origin and credibility with a deterministic Crossref/OpenAlex resolver chain.

**Architecture:** All logic lives in `bokf-core` and surfaces through `bokf` (CLI) and `bokf-mcp`. Slice A (figures) is offline and adds no dependency. Slice B (URL + provenance) adds a synchronous `reqwest` blocking client and a new `credibility/` module with a deterministic-first waterfall. The LLM stays in the loop only for figure description, figure-derived extraction, and scanned/paywalled PDFs.

**Tech Stack:** Rust workspace at `studio/`. Existing: `zip`, `calamine`, `html2md`, `sha2`, `serde`/`serde_yaml`/`serde_json`, `regex`. New (Slice B only): `reqwest` blocking client.

## Global Constraints

- Rust edition 2021; workspace versions from `studio/Cargo.toml`. New dep (Slice B): `reqwest = { version = "0.12", default-features = false, features = ["blocking", "rustls-tls", "json"] }` in `bokf-core`.
- `raw/<id>/original.*` is the only immutable, gitignored artifact; the `.gitignore` rule `raw/**/original.*` is unchanged, so `figures/` is committed by default.
- Prose in code, docs, skills, tool messages uses plain language: no em-dashes, no AI-sounding filler. Controlled vocabulary, CURIEs, predicate names, node-type names unchanged.
- Source ids and figure names are content-derived and human-readable, never a bare hash/uuid (`looks_machine_generated` guards both).
- Network access is best-effort: every resolver/download path fails soft, never aborting a batch.
- `source_type` (origin) and `credibility` (trust) are separate fields.
- Tests use fixture JSON only; any test making a live network call is `#[ignore]`.
- After every task: `cd studio && cargo test` is green, then commit.

## File Structure

- `crates/bokf-core/src/convert.rs` (modify): add `FigureMeta`, the image-format class, `extract_figures`, figure writes in `store`, folder/zip primary-doc grouping, the URL fetch + ingest path, and provenance fields on `SourceMeta`.
- `crates/bokf-core/src/figures.rs` (create): `name_figure` (rename file + rewrite `source.md` + update `meta.yaml`).
- `crates/bokf-core/src/credibility/mod.rs` (create): `classify` waterfall + the `SourceType`/`CredibilityTier`/`Credibility`/`SourceIds` types.
- `crates/bokf-core/src/credibility/{identifiers,host_patterns,allowlist,crossref,openalex,text_signal}.rs` (create): one resolver each, all pure functions plus (crossref/openalex) a network fetch behind `#[ignore]` tests.
- `crates/bokf-core/src/lint.rs` (modify): `source.figure_undescribed`, `source.figure_unnamed`, `source.not_scholarly`, `source.retracted`.
- `crates/bokf-core/src/lib.rs` (modify): `pub mod figures; pub mod credibility;` and re-exports.
- `crates/bokf-core/Cargo.toml` (modify, Slice B): add `reqwest`.
- `crates/bokf-cli/src/main.rs` (modify): `--url`/`--urls` on `Convert`; new `NameFigure` subcommand.
- `crates/bokf-mcp/src/{ops.rs,main.rs}` (modify): `bokf_name_figure`; `url`/`urls` args on `bokf_convert`.
- `studio/skills/biookf-convert/SKILL.md`, `studio/skills/biookf-ingest/SKILL.md` (modify).
- `SCHEMA.md`, `SPEC.md` (modify, Slice B): document `source_type`/`credibility` and URL ingestion.

---

# Slice A: Figures across all formats (offline, no new deps)

### Task A1: `FigureMeta` type and `SourceMeta.figures`

**Files:** Modify `crates/bokf-core/src/convert.rs` (the `SourceMeta` struct near line 36).

**Interfaces:**
- Produces: `pub struct FigureMeta { pub file: String, pub provisional: bool, pub described: bool, pub origin: String }` (derives `Debug, Clone, Serialize, Deserialize, PartialEq`); `SourceMeta` gains `pub figures: Vec<FigureMeta>` with `#[serde(default, skip_serializing_if = "Vec::is_empty")]`.

- [ ] **Step 1: Write the failing test** (append to the `tests` module in `convert.rs`):

```rust
#[test]
fn source_meta_roundtrips_figures() {
    let m = SourceMeta {
        id: "x-abc123".into(), title: "X".into(), sha256: "deadbeef".into(),
        format: "docx".into(), original_filename: Some("x.docx".into()),
        ingested_at: "2026-06-27".into(), needs_llm_fallback: false,
        figures: vec![FigureMeta { file: "figures/fig-001.png".into(), provisional: true, described: false, origin: "word/media/image1.png".into() }],
    };
    let y = serde_yaml::to_string(&m).unwrap();
    let back: SourceMeta = serde_yaml::from_str(&y).unwrap();
    assert_eq!(m, back);
    // empty figures is omitted from yaml
    let mut m2 = m.clone(); m2.figures.clear();
    assert!(!serde_yaml::to_string(&m2).unwrap().contains("figures"));
}
```

- [ ] **Step 2: Run, expect FAIL** — `cd studio && cargo test -p bokf-core source_meta_roundtrips_figures` (compile error: no field `figures`).
- [ ] **Step 3: Implement** — add the `FigureMeta` struct above `SourceMeta`, add the `figures` field with the serde attribute, and add `figures: vec![]` everywhere `SourceMeta { .. }` is constructed in `store` (around line 470).
- [ ] **Step 4: Run, expect PASS** — `cargo test -p bokf-core source_meta_roundtrips_figures`.
- [ ] **Step 5: Commit** — `git add -A && git commit -m "feat(convert): add FigureMeta and SourceMeta.figures"`.

### Task A2: Image-format class in `convert_bytes`

**Files:** Modify `crates/bokf-core/src/convert.rs` (the `match ext` in `convert_bytes`, around line 66).

**Interfaces:**
- Produces: `pub fn is_image_ext(ext: &str) -> bool` (png, jpg, jpeg, gif, webp, tiff, tif, bmp, svg). An image extension yields `Converted { markdown: "![<filename>](figures/<filename>)\n", title: "", format: "image", needs_llm_fallback: true }`.

- [ ] **Step 1: Write the failing test:**

```rust
#[test]
fn image_file_becomes_image_source() {
    assert!(is_image_ext("PNG"));
    let c = convert_bytes("png", "Figure_3.png", &[0x89, b'P', b'N', b'G']);
    assert_eq!(c.format, "image");
    assert!(c.needs_llm_fallback);
    assert!(c.markdown.contains("![Figure_3.png](figures/Figure_3.png)"), "{}", c.markdown);
}
```

- [ ] **Step 2: Run, expect FAIL** — `cargo test -p bokf-core image_file_becomes_image_source`.
- [ ] **Step 3: Implement** — add `pub fn is_image_ext(ext: &str) -> bool { matches!(ext.to_ascii_lowercase().as_str(), "png"|"jpg"|"jpeg"|"gif"|"webp"|"tiff"|"tif"|"bmp"|"svg") }`. In `convert_bytes`, before the `_ =>` arm, add an arm that calls `is_image_ext(&ext)`:

```rust
e if is_image_ext(e) => Converted {
    markdown: format!("![{filename}](figures/{filename})\n"),
    title: String::new(), format: "image".into(), needs_llm_fallback: true,
},
```

(Use a guard: `ext if is_image_ext(ext) => ...` placed before `_ =>`.)
- [ ] **Step 4: Run, expect PASS.**
- [ ] **Step 5: Commit** — `git commit -am "feat(convert): recognize image files as image-sources"`.

### Task A3: `extract_figures` for zip-media formats (docx/pptx/xlsx)

**Files:** Modify `crates/bokf-core/src/convert.rs`.

**Interfaces:**
- Produces: `pub struct ExtractedFigure { pub provisional_name: String, pub bytes: Vec<u8>, pub origin: String }`; `pub fn extract_figures(ext: &str, bytes: &[u8], markdown: &str) -> (Vec<ExtractedFigure>, Option<String>)`. The second tuple element is an optionally rewritten markdown (None for zip-media; used by A4). For docx/pptx/xlsx it pulls every member under `word/media/`, `ppt/media/`, `xl/media/` whose extension is an image, naming each `fig-001.<ext>` in order, `origin` = the zip member path.

- [ ] **Step 1: Write the failing test** (build a synthetic zip with a media image):

```rust
#[test]
fn extract_figures_pulls_office_media() {
    use std::io::Write;
    let mut buf = Vec::new();
    {
        let mut zw = zip::ZipWriter::new(Cursor::new(&mut buf));
        let opts: zip::write::FileOptions<()> = zip::write::FileOptions::default();
        zw.start_file("word/document.xml", opts).unwrap();
        zw.write_all(b"<w:p><w:t>hi</w:t></w:p>").unwrap();
        zw.start_file("word/media/image1.png", opts).unwrap();
        zw.write_all(&[0x89, b'P', b'N', b'G', 1, 2, 3]).unwrap();
        zw.finish().unwrap();
    }
    let (figs, _md) = extract_figures("docx", &buf, "");
    assert_eq!(figs.len(), 1);
    assert_eq!(figs[0].provisional_name, "fig-001.png");
    assert_eq!(figs[0].origin, "word/media/image1.png");
    assert_eq!(figs[0].bytes, vec![0x89, b'P', b'N', b'G', 1, 2, 3]);
}
```

- [ ] **Step 2: Run, expect FAIL.**
- [ ] **Step 3: Implement** — add `ExtractedFigure` and `extract_figures`. For `docx|pptx|xlsx|ods`, open the zip, collect member names starting with the right media prefix (`word/media/`, `ppt/media/`, `xl/media/`) and ending in an image ext (`is_image_ext` on the member's extension), sort by name, read each, emit `fig-{i:03}.<ext>`. Return `(figs, None)`. For other exts return `(vec![], None)` for now.
- [ ] **Step 4: Run, expect PASS.**
- [ ] **Step 5: Commit** — `git commit -am "feat(convert): extract embedded media from docx/pptx/xlsx"`.

### Task A4: `extract_figures` data-URI handling for html/md

**Files:** Modify `crates/bokf-core/src/convert.rs` (extend `extract_figures`).

**Interfaces:**
- Consumes: `extract_figures` from A3.
- Produces: for `html|htm|md|markdown`, decode each `data:image/<t>;base64,<B>` occurrence in `markdown` into an `ExtractedFigure` (`provisional_name = fig-{i:03}.<t>`, `origin = "data-uri"`), and return a rewritten markdown where each data URI is replaced by `figures/fig-{i:03}.<t>`.

- [ ] **Step 1: Write the failing test:**

```rust
#[test]
fn extract_figures_decodes_data_uris() {
    // "AAEC" base64 decodes to bytes [0,1,2]
    let md = "text ![x](data:image/png;base64,AAEC) more";
    let (figs, rewritten) = extract_figures("md", b"", md);
    assert_eq!(figs.len(), 1);
    assert_eq!(figs[0].provisional_name, "fig-001.png");
    assert_eq!(figs[0].bytes, vec![0u8, 1, 2]);
    assert!(rewritten.unwrap().contains("figures/fig-001.png"));
}
```

- [ ] **Step 2: Run, expect FAIL.**
- [ ] **Step 3: Implement** — add a base64 decoder (small standalone fn `fn b64_decode(s: &str) -> Option<Vec<u8>>`, no new dep) and a regex `data:image/([a-z]+);base64,([A-Za-z0-9+/=]+)` over the markdown; for each match emit a figure and build the rewritten string. Map mime subtype to ext (`jpeg`->`jpg`, `svg+xml`->`svg`, else the subtype).
- [ ] **Step 4: Run, expect PASS.**
- [ ] **Step 5: Commit** — `git commit -am "feat(convert): decode html/md data-URI images into figures"`.

### Task A5: `store` writes figures + references + FigureMeta

**Files:** Modify `crates/bokf-core/src/convert.rs` (`store`, around line 441).

**Interfaces:**
- Consumes: `extract_figures` (A3/A4), `FigureMeta` (A1).
- Produces: after writing `source.md`, `store` writes each extracted figure to `raw/<id>/figures/<provisional_name>`, appends a `![<name>](figures/<name>)` reference for any figure not already referenced in the (possibly rewritten) markdown, and records a `FigureMeta { provisional: true, described: false, .. }` per figure. `needs_llm_fallback` becomes true whenever figures exist.

- [ ] **Step 1: Write the failing test:**

```rust
#[test]
fn store_writes_figures_and_meta() {
    use std::io::Write;
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::create_dir_all(root.join("raw")).unwrap();
    let mut buf = Vec::new();
    {
        let mut zw = zip::ZipWriter::new(Cursor::new(&mut buf));
        let opts: zip::write::FileOptions<()> = zip::write::FileOptions::default();
        zw.start_file("ppt/slides/slide1.xml", opts).unwrap();
        zw.write_all(b"<a:p><a:t>Slide</a:t></a:p>").unwrap();
        zw.start_file("ppt/media/image1.png", opts).unwrap();
        zw.write_all(&[0x89, b'P', b'N', b'G']).unwrap();
        zw.finish().unwrap();
    }
    let rec = store(root, "deck.pptx", &buf).unwrap();
    let figdir = root.join("raw").join(&rec.source_id).join("figures");
    assert!(figdir.join("fig-001.png").exists());
    let src = std::fs::read_to_string(root.join(&rec.source_md_path)).unwrap();
    assert!(src.contains("figures/fig-001.png"));
    let meta: SourceMeta = serde_yaml::from_str(&std::fs::read_to_string(root.join(&rec.meta_path)).unwrap()).unwrap();
    assert_eq!(meta.figures.len(), 1);
    assert!(meta.figures[0].provisional && !meta.figures[0].described);
}
```

- [ ] **Step 2: Run, expect FAIL.**
- [ ] **Step 3: Implement** — in `store`, after computing `converted`, call `extract_figures(&ext, bytes, &converted.markdown)`; use the rewritten markdown if `Some`. Create `raw/<id>/figures/`, write each figure's bytes, append references for unreferenced figures, build `Vec<FigureMeta>`, set `needs_llm_fallback |= !figs.is_empty()`, and store `figures` in `SourceMeta`.
- [ ] **Step 4: Run, expect PASS** (and `cargo test -p bokf-core` overall stays green).
- [ ] **Step 5: Commit** — `git commit -am "feat(convert): persist extracted figures + FigureMeta in store"`.

### Task A6: Folder/zip primary-document grouping

**Files:** Modify `crates/bokf-core/src/convert.rs` (`ingest`, around line 492).

**Interfaces:**
- Produces: when `ingest` expands a folder/zip (not `combined`) into members, loose image members attach as figures of the primary document instead of becoming separate sources. Primary document = the sole non-image member, or the largest non-image member by byte length. If there are no non-image members, fall back to one image-source per image (current per-member behavior).
- New helper: `fn store_with_extra_figures(bundle_root, filename, bytes, extra: Vec<(String, Vec<u8>)>) -> Result<SourceRecord, String>` where `extra` is `(member_name, image_bytes)`; the images are written to the doc's `figures/` with `origin = format!("folder:{member_name}")`.

- [ ] **Step 1: Write the failing test:**

```rust
#[test]
fn folder_attaches_loose_images_to_primary_doc() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::create_dir_all(root.join("raw")).unwrap();
    let folder = tempfile::tempdir().unwrap();
    std::fs::write(folder.path().join("paper.md"), "# Paper\n\nBody with two figures.").unwrap();
    std::fs::write(folder.path().join("figure1.png"), [0x89, b'P', b'N', b'G', 1]).unwrap();
    std::fs::write(folder.path().join("figure2.png"), [0x89, b'P', b'N', b'G', 2]).unwrap();
    let recs = ingest(root, SourceInput::Path(folder.path().to_path_buf()), false).unwrap();
    assert_eq!(recs.len(), 1, "one doc-source, images attached: {recs:?}");
    let id = &recs[0].source_id;
    let figdir = root.join("raw").join(id).join("figures");
    assert!(figdir.join("fig-001.png").exists() || figdir.read_dir().unwrap().count() == 2);
    let meta: SourceMeta = serde_yaml::from_str(&std::fs::read_to_string(root.join(&recs[0].meta_path)).unwrap()).unwrap();
    assert_eq!(meta.figures.len(), 2);
}
```

- [ ] **Step 2: Run, expect FAIL.**
- [ ] **Step 3: Implement** — in `ingest`, after resolving `members`, if `!combined` and `members.len() > 1`: partition into `docs` (ext not an image) and `images` (ext is an image). If `docs` is non-empty, pick the primary (sole, else max by `bytes.len()`), call `store_with_extra_figures(root, primary_name, primary_bytes, images)`, and `store` each remaining non-primary doc normally; return all records. If `docs` is empty, keep the current per-member loop. Add `store_with_extra_figures` (factor the figure-writing block out of A5 so both call it; the extra images are appended with `fig-{n:03}.<ext>` numbering continuing after the doc's own figures and `origin = "folder:<member>"`).
- [ ] **Step 4: Run, expect PASS.**
- [ ] **Step 5: Commit** — `git commit -am "feat(convert): attach loose folder/zip images to the primary document"`.

### Task A7: `name_figure` core (rename + rewrite source.md + update meta)

**Files:** Create `crates/bokf-core/src/figures.rs`; modify `lib.rs` (`pub mod figures;`).

**Interfaces:**
- Produces: `pub fn name_figure(bundle_root: &Path, source_id: &str, current_rel: &str, caption: &str) -> Result<String, String>` returning the new relative figure path. It slugifies `caption` (reuse `convert::slug` — make `slug` `pub(crate)`), preserves the original extension, moves `raw/<id>/figures/<current>` to `raw/<id>/figures/<slug>.<ext>`, rewrites the matching reference in `raw/<id>/source.md`, sets the matching `FigureMeta.provisional = false` and `file` to the new path, and writes `meta.yaml`.

- [ ] **Step 1: Write the failing test** (in `figures.rs`):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_figure_renames_and_rewrites() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let figs = root.join("raw/x-abc123/figures");
        std::fs::create_dir_all(&figs).unwrap();
        std::fs::write(figs.join("fig-001.png"), b"img").unwrap();
        std::fs::write(root.join("raw/x-abc123/source.md"), "see ![fig-001](figures/fig-001.png)").unwrap();
        let meta = crate::convert::SourceMeta {
            id: "x-abc123".into(), title: "X".into(), sha256: "d".into(), format: "image".into(),
            original_filename: None, ingested_at: "2026-06-27".into(), needs_llm_fallback: true,
            figures: vec![crate::convert::FigureMeta { file: "figures/fig-001.png".into(), provisional: true, described: false, origin: "data-uri".into() }],
        };
        std::fs::write(root.join("raw/x-abc123/meta.yaml"), serde_yaml::to_string(&meta).unwrap()).unwrap();
        let newp = name_figure(root, "x-abc123", "figures/fig-001.png", "Kaplan-Meier by arm").unwrap();
        assert_eq!(newp, "figures/kaplan-meier-by-arm.png");
        assert!(figs.join("kaplan-meier-by-arm.png").exists());
        assert!(!figs.join("fig-001.png").exists());
        let src = std::fs::read_to_string(root.join("raw/x-abc123/source.md")).unwrap();
        assert!(src.contains("figures/kaplan-meier-by-arm.png"));
        let back: crate::convert::SourceMeta = serde_yaml::from_str(&std::fs::read_to_string(root.join("raw/x-abc123/meta.yaml")).unwrap()).unwrap();
        assert!(!back.figures[0].provisional);
        assert_eq!(back.figures[0].file, "figures/kaplan-meier-by-arm.png");
    }
}
```

- [ ] **Step 2: Run, expect FAIL** — `cargo test -p bokf-core name_figure_renames`.
- [ ] **Step 3: Implement** `figures.rs` per the interface; make `convert::slug` and `convert::ext_of` `pub(crate)`.
- [ ] **Step 4: Run, expect PASS.**
- [ ] **Step 5: Commit** — `git commit -am "feat(figures): bokf name-figure core (rename + rewrite + meta)"`.

### Task A8: `name-figure` CLI + MCP

**Files:** Modify `crates/bokf-cli/src/main.rs` (new `NameFigure` subcommand); `crates/bokf-mcp/src/{ops.rs,main.rs}` (`bokf_name_figure`).

**Interfaces:**
- Consumes: `figures::name_figure`.
- Produces CLI: `bokf name-figure <bundle> --source <id> --figure <figures/...> --as "<caption>" [--json]`. Produces MCP tool `bokf_name_figure { bundle, source, figure, caption }`.

- [ ] **Step 1: Write the failing test** (CLI integration test under `crates/bokf-cli/tests/` or assert via a smoke test calling `Cmd`): add a doc-comment smoke test that builds the binary is overkill; instead unit-test the wiring by calling `bokf_core::figures::name_figure` from a CLI helper `fn cmd_name_figure(...)`. Write a test in `figures.rs` already covers core; here assert the CLI parses. Minimal: add a `#[test]` in `main.rs` that constructs `Cmd::NameFigure { .. }` via `clap` parse:

```rust
#[test]
fn cli_parses_name_figure() {
    use clap::Parser;
    let c = Cli::try_parse_from(["bokf","name-figure","kb","--source","x-1","--figure","figures/fig-001.png","--as","A B"]).unwrap();
    matches!(c.cmd, Cmd::NameFigure { .. });
}
```

- [ ] **Step 2: Run, expect FAIL.**
- [ ] **Step 3: Implement** — add the `NameFigure { bundle: PathBuf, #[arg(long)] source: String, #[arg(long)] figure: String, #[arg(long="as")] caption: String, #[arg(long)] json: bool }` variant, a `cmd_name_figure` that calls `name_figure` and prints the new path (or JSON), and the dispatch arm. Then mirror as `bokf_name_figure` in the MCP crate following the existing tool pattern in `ops.rs`/`main.rs`.
- [ ] **Step 4: Run, expect PASS** — `cargo test -p bokf-cli` and `cargo build -p bokf-mcp`.
- [ ] **Step 5: Commit** — `git commit -am "feat(cli,mcp): name-figure command + bokf_name_figure tool"`.

### Task A9: figure lints (`source.figure_undescribed`, `source.figure_unnamed`)

**Files:** Modify `crates/bokf-core/src/lint.rs`.

**Interfaces:**
- Produces: for each `raw/<id>/meta.yaml`, a `source.figure_unnamed` Warn per `FigureMeta` with `provisional == true`, and a `source.figure_undescribed` Warn per `FigureMeta` whose reference in `source.md` has an empty description (the `![]` alt text is empty or the figure file is not referenced at all).

- [ ] **Step 1: Write the failing test** (in `lint.rs` tests, build a tiny bundle with a provisional figure):

```rust
#[test]
fn lints_flag_unnamed_and_undescribed_figures() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::create_dir_all(root.join("raw/x-1/figures")).unwrap();
    std::fs::create_dir_all(root.join("knowledge")).unwrap();
    std::fs::write(root.join("raw/x-1/figures/fig-001.png"), b"i").unwrap();
    std::fs::write(root.join("raw/x-1/source.md"), "![](figures/fig-001.png)").unwrap();
    let meta = crate::convert::SourceMeta { id:"x-1".into(), title:"X".into(), sha256:"d".into(), format:"image".into(), original_filename:None, ingested_at:"2026-06-27".into(), needs_llm_fallback:true, figures: vec![crate::convert::FigureMeta{ file:"figures/fig-001.png".into(), provisional:true, described:false, origin:"data-uri".into() }] };
    std::fs::write(root.join("raw/x-1/meta.yaml"), serde_yaml::to_string(&meta).unwrap()).unwrap();
    let rep = lint(&crate::bundle::Bundle::open(root).unwrap());
    assert!(rep.findings.iter().any(|f| f.rule == "source.figure_unnamed"));
    assert!(rep.findings.iter().any(|f| f.rule == "source.figure_undescribed"));
}
```

- [ ] **Step 2: Run, expect FAIL.**
- [ ] **Step 3: Implement** — add a `lint_figures` pass that walks `raw/*/meta.yaml`, parses `SourceMeta`, reads the sibling `source.md`, and pushes the two Warns. A reference is "described" when `source.md` contains `[<non-empty>](<file>)`.
- [ ] **Step 4: Run, expect PASS.**
- [ ] **Step 5: Commit** — `git commit -am "feat(lint): figure_unnamed + figure_undescribed warnings"`.

### Task A10: skill updates (biookf-convert, biookf-ingest)

**Files:** Modify `studio/skills/biookf-convert/SKILL.md`, `studio/skills/biookf-ingest/SKILL.md`.

- [ ] **Step 1:** Add to `biookf-convert` a figures pass: after `bokf convert`, view each `raw/<id>/figures/*`, write a faithful description beside its reference in `source.md`, and run `bokf name-figure` to give it a content name; clear `source.figure_unnamed`/`source.figure_undescribed` before finishing. No em-dashes.
- [ ] **Step 2:** Add to `biookf-ingest` an explicit instruction to view `raw/<id>/figures/*` and extract typed nodes and provenance-stamped edges from figure content (a survival curve yields outcome edges; a pathway diagram yields `regulates`/`activates` edges).
- [ ] **Step 3: Commit** — `git commit -am "docs(skills): figures pass in convert + figure-derived extraction in ingest"`.

---

# Slice B: URL ingestion and source provenance (adds reqwest)

### Task B1: provenance types + `reqwest` dep + `SourceMeta` fields

**Files:** Modify `crates/bokf-core/Cargo.toml`; create `crates/bokf-core/src/credibility/mod.rs`; modify `lib.rs`, `convert.rs`.

**Interfaces:**
- Produces (in `credibility/mod.rs`): `SourceType` enum {JournalArticle, Preprint, Review, Book, Dataset, Database, ClinicalGuideline, GovReport, WebPage, Personal, Unknown}; `CredibilityTier` {PeerReviewed, Preprint, Archive, GrayLit, Web, Unknown}; `Credibility { tier, confidence: f32, retracted: bool, venue: Option<String>, publisher: Option<String>, reasoning: String, classifier_version: u32 }`; `SourceIds { doi, pmid, pmcid, arxiv, isbn: Option<String> }`. All `Serialize, Deserialize, PartialEq, Clone, Debug`; enums serialize snake_case. `SourceMeta` gains `url: Option<String>`, `final_url: Option<String>`, `#[serde(default)] source_type: SourceType` (Default = Unknown), `#[serde(default)] credibility: Credibility` (Default = Unknown tier, conf 0.3, version 0), `#[serde(default)] ids: SourceIds`.

- [ ] **Step 1: Write the failing test** (in `credibility/mod.rs`):

```rust
#[test]
fn types_roundtrip_snake_case() {
    let c = Credibility { tier: CredibilityTier::PeerReviewed, confidence: 0.9, retracted: false,
        venue: Some("Nature".into()), publisher: Some("Springer Nature".into()),
        reasoning: "crossref journal-article".into(), classifier_version: 1 };
    let y = serde_yaml::to_string(&c).unwrap();
    assert!(y.contains("tier: peer_reviewed"));
    let back: Credibility = serde_yaml::from_str(&y).unwrap();
    assert_eq!(c, back);
}
```

- [ ] **Step 2: Run, expect FAIL.**
- [ ] **Step 3: Implement** — add `reqwest = { version = "0.12", default-features = false, features = ["blocking", "rustls-tls", "json"] }` to `bokf-core/Cargo.toml`; create `credibility/mod.rs` with the types + `#[serde(rename_all = "snake_case")]` on the enums and `Default` impls; add `pub mod credibility;` to `lib.rs`; extend `SourceMeta` with the new `#[serde(default)]` fields and add them to the `SourceMeta { .. }` literals in `convert.rs`.
- [ ] **Step 4: Run, expect PASS** — `cargo test -p bokf-core types_roundtrip` (this also proves `reqwest` resolves).
- [ ] **Step 5: Commit** — `git commit -am "feat(credibility): provenance types + reqwest dep + SourceMeta fields"`.

### Task B2: `identifiers.rs` (DOI/arXiv/PMID/PMCID/ISBN + bioRxiv prefix)

**Files:** Create `crates/bokf-core/src/credibility/identifiers.rs`; add `pub mod identifiers;` to `credibility/mod.rs`.

**Interfaces:**
- Produces: `pub fn extract(text: &str) -> SourceIds`; `pub fn is_biorxiv_doi(doi: &str) -> bool` (true when the DOI starts with `10.1101/`).

- [ ] **Step 1: Write the failing test:**

```rust
#[test]
fn extracts_identifiers_and_biorxiv() {
    let t = "See https://doi.org/10.1101/2020.01.02.123456 and PMID: 31234567 and arXiv:2003.01234";
    let ids = extract(t);
    assert_eq!(ids.doi.as_deref(), Some("10.1101/2020.01.02.123456"));
    assert_eq!(ids.pmid.as_deref(), Some("31234567"));
    assert_eq!(ids.arxiv.as_deref(), Some("2003.01234"));
    assert!(is_biorxiv_doi(ids.doi.as_deref().unwrap()));
}
```

- [ ] **Step 2: Run, expect FAIL.**
- [ ] **Step 3: Implement** — `regex` patterns: DOI `(?i)\b10\.\d{4,9}/[-._;()/:a-z0-9]+\b` (lowercase the capture), arXiv `(?i)arxiv:(\d{4}\.\d{4,5})|arxiv\.org/(?:abs|pdf)/(\d{4}\.\d{4,5})`, PMID `(?i)\bPMID[: ]\s*(\d{6,9})\b|pubmed\.ncbi\.nlm\.nih\.gov/(\d{6,9})`, PMCID `(?i)\bPMC(\d{5,9})\b`, ISBN as in spec. Compile once with `once_cell`-style `std::sync::OnceLock` (no new dep) or `regex::Regex::new` per call (acceptable). `is_biorxiv_doi` is a prefix check.
- [ ] **Step 4: Run, expect PASS.**
- [ ] **Step 5: Commit** — `git commit -am "feat(credibility): identifier extraction (+ bioRxiv DOI prefix)"`.

### Task B3: `host_patterns.rs`

**Files:** Create `crates/bokf-core/src/credibility/host_patterns.rs`; register the module.

**Interfaces:**
- Produces: `pub fn classify_url(url: &str) -> Option<(SourceType, CredibilityTier, f32)>` — preprint hosts -> (Preprint, Preprint, 0.9); gray-lit hosts -> (GovReport, GrayLit, 0.8); any other http(s) -> (WebPage, Web, 0.6); non-URL -> None. `fn host_of(url: &str) -> Option<String>`.

- [ ] **Step 1: Write the failing test:**

```rust
#[test]
fn classifies_hosts() {
    assert!(matches!(classify_url("https://www.biorxiv.org/content/x"), Some((SourceType::Preprint, CredibilityTier::Preprint, _))));
    assert!(matches!(classify_url("https://www.cdc.gov/x"), Some((_, CredibilityTier::GrayLit, _))));
    assert!(matches!(classify_url("https://example.com/x"), Some((SourceType::WebPage, CredibilityTier::Web, _))));
    assert!(classify_url("not a url").is_none());
}
```

- [ ] **Step 2: Run, expect FAIL.**
- [ ] **Step 3: Implement** — `host_of` splits on `://` then `/`; preprint host set {arxiv.org, biorxiv.org, medrxiv.org, chemrxiv.org, ssrn.com, preprints.org, researchsquare.com, osf.io, psyarxiv.com} (match host or `*.host`); gray-lit: host ends with `.gov`/`.edu` or in {who.int, clinicaltrials.gov, europa.eu, nih.gov, cdc.gov, fda.gov}.
- [ ] **Step 4: Run, expect PASS.**
- [ ] **Step 5: Commit** — `git commit -am "feat(credibility): URL host-pattern classifier"`.

### Task B4: `allowlist.rs`

**Files:** Create `crates/bokf-core/src/credibility/allowlist.rs`; register.

**Interfaces:**
- Produces: `pub fn is_allowlisted(publisher: &str) -> bool` (case-insensitive substring match against a static list of recognized scholarly publishers; confidence booster only).

- [ ] **Step 1: Write the failing test:**

```rust
#[test]
fn allowlist_matches_known_publishers() {
    assert!(is_allowlisted("Springer Nature"));
    assert!(is_allowlisted("ELSEVIER BV"));
    assert!(!is_allowlisted("Random Blog Co"));
}
```

- [ ] **Step 2: Run, expect FAIL.**
- [ ] **Step 3: Implement** — a `const PUBLISHERS: &[&str]` of ~30 lowercase tokens (elsevier, springer, wiley, ieee, plos, mdpi, oxford university press, cambridge university press, nature, lancet, cell press, american chemical society, frontiers, bmj, american medical association, massachusetts medical society, ...); `is_allowlisted` lowercases input and tests substring containment.
- [ ] **Step 4: Run, expect PASS.**
- [ ] **Step 5: Commit** — `git commit -am "feat(credibility): publisher allowlist (confidence booster)"`.

### Task B5: `crossref.rs` JSON->verdict mapping (pure) + fetch (ignored)

**Files:** Create `crates/bokf-core/src/credibility/crossref.rs`; register.

**Interfaces:**
- Produces: `pub fn map_work(v: &serde_json::Value) -> Option<(SourceType, CredibilityTier, Option<String>, Option<String>, bool)>` returning (source_type, tier, venue, publisher, retracted) from a Crossref `message` object; `pub fn fetch(doi: &str) -> Option<serde_json::Value>` (blocking reqwest, returns the `message`). Work-type map: `journal-article|proceedings-article|review-article`->(JournalArticle/Review, PeerReviewed); `posted-content`->(Preprint, Preprint); `book*`->(Book, ...); `dataset`->(Dataset, Web tier? use Archive? -> (Dataset, GrayLit)).

- [ ] **Step 1: Write the failing test** (fixture JSON, no network):

```rust
#[test]
fn maps_crossref_journal_article() {
    let v: serde_json::Value = serde_json::from_str(r#"{
      "type":"journal-article","publisher":"Springer Nature",
      "container-title":["Nature Medicine"],
      "update-to":[{"type":"retraction"}]
    }"#).unwrap();
    let (st, tier, venue, pubr, retracted) = map_work(&v).unwrap();
    assert!(matches!(st, SourceType::JournalArticle));
    assert!(matches!(tier, CredibilityTier::PeerReviewed));
    assert_eq!(venue.as_deref(), Some("Nature Medicine"));
    assert_eq!(pubr.as_deref(), Some("Springer Nature"));
    assert!(retracted);
}

#[test] #[ignore]
fn fetch_crossref_live() { assert!(fetch("10.1038/s41591-020-0968-3").is_some()); }
```

- [ ] **Step 2: Run, expect FAIL.**
- [ ] **Step 3: Implement** — `map_work` reads `type`, `publisher`, `container-title[0]`, and retraction from `update-to[].type == "retraction"`; `fetch` does `reqwest::blocking::Client::builder().timeout(30s).user_agent("BioOKF/0.1 (mailto:wanjun.gu@ucsf.edu)").build()` then `GET https://api.crossref.org/works/{doi}` and returns `json["message"]`.
- [ ] **Step 4: Run, expect PASS** — `cargo test -p bokf-core maps_crossref` (the `#[ignore]` live test is skipped).
- [ ] **Step 5: Commit** — `git commit -am "feat(credibility): Crossref work-type mapping + fetch"`.

### Task B6: `openalex.rs` JSON->verdict mapping (pure) + fetch (ignored)

**Files:** Create `crates/bokf-core/src/credibility/openalex.rs`; register.

**Interfaces:**
- Produces: `pub fn map_work(v: &serde_json::Value) -> Option<(SourceType, CredibilityTier, Option<String>, Option<String>, bool)>`; `pub fn fetch(doi: &str) -> Option<serde_json::Value>` (`GET https://api.openalex.org/works/doi:{doi}`). Reads `type`, `host_venue.display_name`/`publisher`, `is_retracted`.

- [ ] **Step 1: Write the failing test** (fixture, plus `#[ignore]` live):

```rust
#[test]
fn maps_openalex_posted_content() {
    let v: serde_json::Value = serde_json::from_str(r#"{
      "type":"posted-content","is_retracted":false,
      "host_venue":{"display_name":"bioRxiv","publisher":"Cold Spring Harbor Laboratory"}
    }"#).unwrap();
    let (st, tier, venue, _pubr, retracted) = map_work(&v).unwrap();
    assert!(matches!(st, SourceType::Preprint));
    assert!(matches!(tier, CredibilityTier::Preprint));
    assert_eq!(venue.as_deref(), Some("bioRxiv"));
    assert!(!retracted);
}
```

- [ ] **Step 2: Run, expect FAIL.**
- [ ] **Step 3: Implement** mirroring B5 against the OpenAlex field names.
- [ ] **Step 4: Run, expect PASS.**
- [ ] **Step 5: Commit** — `git commit -am "feat(credibility): OpenAlex work mapping + fetch"`.

### Task B7: `text_signal.rs` heuristic

**Files:** Create `crates/bokf-core/src/credibility/text_signal.rs`; register.

**Interfaces:**
- Produces: `pub fn scholarly_text_signal(text: &str, ids: &SourceIds) -> Option<(SourceType, CredibilityTier, f32)>`. Preprint fingerprints (biorxiv/medrxiv/arxiv/"preprint server") + (a DOI or >=1 journal marker) -> (Preprint, Preprint, 0.7); a DOI + >=2 journal markers (`received:`, `accepted:`, `peer-reviewed`, `corresponding author`, `doi:`, `journal of`, `et al.`, `abstract`) -> (JournalArticle, PeerReviewed, 0.72).

- [ ] **Step 1: Write the failing test:**

```rust
#[test]
fn text_signal_detects_journal() {
    let ids = SourceIds { doi: Some("10.x/y".into()), ..Default::default() };
    let t = "Received: 1 Jan. Accepted: 2 Feb. Corresponding author: x. Journal of Things.";
    assert!(matches!(scholarly_text_signal(t, &ids), Some((_, CredibilityTier::PeerReviewed, _))));
}
```

- [ ] **Step 2: Run, expect FAIL.**
- [ ] **Step 3: Implement** the marker counting (case-insensitive).
- [ ] **Step 4: Run, expect PASS.**
- [ ] **Step 5: Commit** — `git commit -am "feat(credibility): scholarly text-signal heuristic"`.

### Task B8: `classify` waterfall (deterministic, offline)

**Files:** Modify `crates/bokf-core/src/credibility/mod.rs`.

**Interfaces:**
- Consumes: B2-B7.
- Produces: `pub struct ClassifyInput<'a> { pub url: Option<&'a str>, pub filename: Option<&'a str>, pub body: &'a str, pub online: bool }`; `pub fn classify(input: &ClassifyInput) -> (SourceType, Credibility, SourceIds)`. Order: extract ids; if `online` and a DOI, try `crossref::fetch`+`map_work` then `openalex`; else host patterns (if url); else text signal; else default (WebPage/Web 0.6 for url/file, Personal for empty body). `is_allowlisted(publisher)` raises confidence to 0.95. `reasoning` records which branch fired. `classifier_version = 1`.

- [ ] **Step 1: Write the failing test** (offline path, no network):

```rust
#[test]
fn classify_offline_uses_host_then_text() {
    let inp = ClassifyInput { url: Some("https://www.medrxiv.org/content/10.1101/2021.01.01.21249000v1"), filename: None, body: "", online: false };
    let (st, cred, ids) = classify(&inp);
    assert!(matches!(st, SourceType::Preprint));
    assert!(matches!(cred.tier, CredibilityTier::Preprint));
    assert!(ids.doi.is_some());
    assert_eq!(cred.classifier_version, 1);
}
```

- [ ] **Step 2: Run, expect FAIL.**
- [ ] **Step 3: Implement** the waterfall, guarding the network calls behind `input.online`.
- [ ] **Step 4: Run, expect PASS.**
- [ ] **Step 5: Commit** — `git commit -am "feat(credibility): deterministic classify waterfall"`.

### Task B9: URL fetch + ingest path in `convert.rs`

**Files:** Modify `crates/bokf-core/src/convert.rs`.

**Interfaces:**
- Consumes: `credibility::{classify, ClassifyInput}`.
- Produces: `pub fn fetch_url(url: &str) -> Result<(Vec<u8>, String, String), String>` returning `(bytes, final_url, ext)` (ext from final-url path or Content-Type; reqwest blocking, 30s, `BioOKF/<ver>` UA, byte-sniff `%PDF-`/`<html`). `SourceInput` gains a `Url(String)` variant. `ingest` handles `Url`: fetch, store the original bytes as `original.<ext>`, classify with `online = true`, set `url`/`final_url`/`source_type`/`credibility`/`ids` on `SourceMeta`, dedup by `meta.url` first then sha256. Add `pub fn ingest_urls(bundle_root, urls: Vec<String>) -> Vec<Result<SourceRecord, String>>` (sequential, fail-soft).

- [ ] **Step 1: Write the failing test** (no network: test the classify+store integration by storing pre-fetched bytes through a seam). Add `fn store_url(bundle_root, url, final_url, bytes, ext) -> Result<SourceRecord,String>` and test it offline:

```rust
#[test]
fn store_url_records_provenance_offline() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::create_dir_all(root.join("raw")).unwrap();
    let html = b"<html><body><h1>Study</h1><p>doi:10.1101/2020.01.02.123456 received: accepted:</p></body></html>";
    let rec = store_url(root, "https://www.biorxiv.org/x", "https://www.biorxiv.org/x", html, "html").unwrap();
    let meta: SourceMeta = serde_yaml::from_str(&std::fs::read_to_string(root.join(&rec.meta_path)).unwrap()).unwrap();
    assert_eq!(meta.url.as_deref(), Some("https://www.biorxiv.org/x"));
    assert!(matches!(meta.credibility.tier, crate::credibility::CredibilityTier::Preprint));
    assert!(meta.ids.doi.is_some());
    assert!(root.join(format!("raw/{}/original.html", rec.source_id)).exists());
}
```

(`store_url` classifies with `online = false` in tests via a thread-local or an `online` param; expose `store_url_inner(.., online: bool)` and have `store_url` call it with `true`, the test call a variant with `false`. Simplest: give `store_url` an `online: bool` param and pass `false` in the test, `true` from `ingest`.)

- [ ] **Step 2: Run, expect FAIL.**
- [ ] **Step 3: Implement** `fetch_url`, the `Url` variant, `store_url(.., online)`, `ingest_urls`, and the dedup-by-url lookup (extend `find_existing` or add `find_by_url`).
- [ ] **Step 4: Run, expect PASS** — offline test green; live fetch covered by an `#[ignore]` test.
- [ ] **Step 5: Commit** — `git commit -am "feat(convert): URL fetch + provenance-stamped ingest + dedup"`.

### Task B10: `--url`/`--urls` CLI + MCP

**Files:** Modify `crates/bokf-cli/src/main.rs`; `crates/bokf-mcp/src/{ops.rs,main.rs}`.

**Interfaces:**
- Consumes: `convert::{ingest, ingest_urls, SourceInput}`.
- Produces CLI: `bokf convert --url <u> --into <b>` and `bokf convert --urls <file|-> --into <b>`; MCP: `bokf_convert` gains optional `url` and `urls` args.

- [ ] **Step 1: Write the failing test** (clap parse):

```rust
#[test]
fn cli_parses_convert_url() {
    use clap::Parser;
    let c = Cli::try_parse_from(["bokf","convert","--url","https://x.org/a","--into","kb"]).unwrap();
    if let Cmd::Convert { url, .. } = c.cmd { assert_eq!(url.as_deref(), Some("https://x.org/a")); } else { panic!() }
}
```

- [ ] **Step 2: Run, expect FAIL.**
- [ ] **Step 3: Implement** — add `#[arg(long)] url: Option<String>` and `#[arg(long)] urls: Option<PathBuf>` to `Convert`; in `cmd_convert`, when `url` is set call `ingest` with `SourceInput::Url`, when `urls` is set read the file (one URL per line, skip blanks/`#`) and call `ingest_urls`; mirror the two args on `bokf_convert`.
- [ ] **Step 4: Run, expect PASS** — `cargo test -p bokf-cli`, `cargo build -p bokf-mcp`.
- [ ] **Step 5: Commit** — `git commit -am "feat(cli,mcp): convert --url/--urls"`.

### Task B11: provenance lints (`source.not_scholarly`, `source.retracted`)

**Files:** Modify `crates/bokf-core/src/lint.rs`.

**Interfaces:**
- Produces: for each source node used as a `primary_source` on any edge, if its `raw/<id>/meta.yaml` credibility tier is `Web`/`Unknown`, push `source.not_scholarly` Warn; if `retracted == true`, push `source.retracted` Warn. (Map a source NODE to its `raw_source` -> `raw/<id>/meta.yaml`.)

- [ ] **Step 1: Write the failing test** — build a bundle with one source node whose `meta.yaml` has tier `web`, referenced as a `primary_source`; assert `source.not_scholarly` fires; flip to `retracted: true` and assert `source.retracted` fires.
- [ ] **Step 2: Run, expect FAIL.**
- [ ] **Step 3: Implement** a `lint_provenance` pass: collect `primary_source` ids from edges, resolve each to a source node + its `raw_source` `meta.yaml`, parse `SourceMeta`, push the warns.
- [ ] **Step 4: Run, expect PASS.**
- [ ] **Step 5: Commit** — `git commit -am "feat(lint): not_scholarly + retracted provenance warnings"`.

### Task B12: SCHEMA.md / SPEC.md documentation

**Files:** Modify `SCHEMA.md`, `SPEC.md`.

- [ ] **Step 1:** In `SCHEMA.md`, document URL ingestion (`bokf convert --url/--urls`), the `figures/` subfolder, and the `source_type` vs `credibility` fields on a stored source plus how they map onto `Publication`/`Study`/`Dataset` nodes and `xref`. In `SPEC.md`, add a short subsection on source provenance classification. No em-dashes; do not alter controlled vocab, predicate counts, or node-type names.
- [ ] **Step 2:** Run `cd studio && cargo test` (full suite) to confirm nothing regressed.
- [ ] **Step 3: Commit** — `git commit -am "docs(schema,spec): URL ingestion, figures, source_type vs credibility"`.

---

## Self-Review

**Spec coverage:** Slice A figures (A1-A6 extraction/storage/grouping, A7-A8 naming, A9 lints, A10 skills) and Slice B URL+provenance (B1 types/dep, B2-B8 classifier, B9-B10 fetch/CLI/MCP, B11 lints, B12 docs) cover every Slice A and Slice B requirement in the spec. Excluded by request: the `doc.language_mismatch`/style-outlier heuristic and Slice C (PDF pdfium). Covered elsewhere already: merge style/language harmonization (shipped in the skill + `Merge_KBs_WF.md`).

**Placeholder scan:** No TBD/TODO; every code step shows real code or a precise interface contract with the exact field names used downstream.

**Type consistency:** `FigureMeta`/`SourceMeta.figures` (A1) are used unchanged in A5/A7/A9; `SourceType`/`CredibilityTier`/`Credibility`/`SourceIds` (B1) are used unchanged in B2-B11; `extract_figures` signature (A3) is extended, not changed, in A4 and consumed in A5; `classify`/`ClassifyInput` (B8) are consumed unchanged in B9.

## Notes for the executor

- Run all `cargo` commands from `studio/`. Keep `cargo test` (42 existing + new) green after each task.
- Independent tasks that touch only their own new file can be built in parallel: A3/A4 (after A1), and B2/B3/B4/B5/B6/B7 (after B1). A5, A6, B8, B9 are integration points and must follow their inputs. Slice A must land before Slice B opens (B1 adds the dep).
- Do not edit `raw/**/original.*`; the raw-guard hook blocks it.
- Branch: `biookf-raw-conversion-v2`.
