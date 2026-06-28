//! Document conversion: pdf / html / docx / pptx / csv / xlsx / zip / folder → raw
//! Markdown, written under `raw/<source-id>/` with a content-derived, human-readable
//! source id. Most office formats are zip containers, so the dependency footprint is
//! `zip` (docx/pptx/generic-zip) + `calamine` (spreadsheets) + `html2md` (html).

use crate::git::today_iso;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};

/// Marker prepended to a `source.md` that still needs a faithful LLM conversion (unknown
/// format, scanned PDF, or failed extraction). The agent removes it once the rendering is
/// complete; `bokf lint`/`bokf verify` flag any source still carrying it.
pub const NEEDS_CONVERSION_MARKER: &str = "<!-- bokf:needs-conversion -->";

/// The result of converting one source's bytes to Markdown.
#[derive(Debug, Clone)]
pub struct Converted {
    pub markdown: String,
    pub title: String,
    /// Short format tag (e.g. "pdf", "html", "csv").
    pub format: String,
    /// True when faithful text could not be extracted (e.g. a scanned PDF) and an
    /// OCR / LLM pass is needed downstream.
    pub needs_llm_fallback: bool,
}

/// What gets ingested.
pub enum SourceInput {
    Path(PathBuf),
    Text { text: String, title: Option<String> },
}

/// On-disk inventory entry for one extracted figure under `raw/<id>/figures/`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FigureMeta {
    /// Path relative to `raw/<id>/`, e.g. "figures/kaplan-meier-by-arm.png".
    pub file: String,
    /// True until the figure is named by content.
    pub provisional: bool,
    /// True once `source.md` carries a non-empty description for this figure.
    pub described: bool,
    /// Where the figure came from: "word/media/image1.png", "data-uri", "folder:figure3.png", etc.
    pub origin: String,
}

/// On-disk provenance for a stored raw source.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SourceMeta {
    pub id: String,
    pub title: String,
    pub sha256: String,
    pub format: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_filename: Option<String>,
    pub ingested_at: String,
    pub needs_llm_fallback: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub figures: Vec<FigureMeta>,
}

/// A stored raw source after ingestion.
#[derive(Debug, Clone, Serialize)]
pub struct SourceRecord {
    pub source_id: String,
    pub source_md_path: String,
    pub meta_path: String,
    pub title: String,
    pub needs_llm_fallback: bool,
    pub reused: bool,
}

// ---------------------------------------------------------------------------
// Conversion dispatch
// ---------------------------------------------------------------------------

/// True when `ext` names a raster or vector image format we copy verbatim as a figure.
pub fn is_image_ext(ext: &str) -> bool {
    matches!(ext.to_ascii_lowercase().as_str(), "png" | "jpg" | "jpeg" | "gif" | "webp" | "tiff" | "tif" | "bmp" | "svg")
}

/// One image pulled out of a source during conversion.
#[derive(Debug, Clone)]
pub struct ExtractedFigure {
    /// Provisional name (e.g. "fig-001.png") until renamed by content.
    pub provisional_name: String,
    /// The image bytes, copied verbatim.
    pub bytes: Vec<u8>,
    /// Where the figure came from inside the source.
    pub origin: String,
}

/// Map a `data:image/<subtype>` MIME subtype to a file extension.
fn mime_subtype_to_ext(sub: &str) -> String {
    match sub {
        "jpeg" => "jpg".to_string(),
        "svg+xml" => "svg".to_string(),
        other => other.to_string(),
    }
}

/// Decode a standard base64 string (no whitespace) into bytes. Returns None on any
/// invalid character or malformed padding. No new dependency.
fn b64_decode(s: &str) -> Option<Vec<u8>> {
    fn val(c: u8) -> Option<u32> {
        match c {
            b'A'..=b'Z' => Some((c - b'A') as u32),
            b'a'..=b'z' => Some((c - b'a' + 26) as u32),
            b'0'..=b'9' => Some((c - b'0' + 52) as u32),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    }
    let bytes: Vec<u8> = s.bytes().filter(|&c| c != b'\n' && c != b'\r').collect();
    let mut out = Vec::new();
    let mut chunk = [0u32; 4];
    let mut n = 0usize;
    let mut pad = 0usize;
    for &c in &bytes {
        if c == b'=' {
            chunk[n] = 0;
            pad += 1;
            n += 1;
        } else {
            chunk[n] = val(c)?;
            n += 1;
        }
        if n == 4 {
            let triple = (chunk[0] << 18) | (chunk[1] << 12) | (chunk[2] << 6) | chunk[3];
            out.push((triple >> 16) as u8);
            if pad < 2 {
                out.push((triple >> 8) as u8);
            }
            if pad < 1 {
                out.push(triple as u8);
            }
            n = 0;
            pad = 0;
        }
    }
    if n != 0 {
        return None;
    }
    Some(out)
}

/// Extract embedded figures from a source. Returns the figures plus an optionally
/// rewritten Markdown (None for zip-media formats; for html/md the data URIs are
/// rewritten to local figure references).
pub fn extract_figures(ext: &str, bytes: &[u8], markdown: &str) -> (Vec<ExtractedFigure>, Option<String>) {
    let ext = ext.to_ascii_lowercase();
    // html/md: decode inline base64 data URIs into figures and rewrite the references.
    if matches!(ext.as_str(), "html" | "htm" | "md" | "markdown") {
        let re = regex::Regex::new(r"data:image/([a-z0-9.+-]+);base64,([A-Za-z0-9+/=]+)").unwrap();
        let mut figs = Vec::new();
        let mut rewritten = String::new();
        let mut last = 0usize;
        for cap in re.captures_iter(markdown) {
            let whole = cap.get(0).unwrap();
            let sub = cap.get(1).unwrap().as_str();
            let b64 = cap.get(2).unwrap().as_str();
            let data = match b64_decode(b64) {
                Some(d) => d,
                None => continue,
            };
            let fext = mime_subtype_to_ext(sub);
            let name = format!("fig-{:03}.{}", figs.len() + 1, fext);
            rewritten.push_str(&markdown[last..whole.start()]);
            rewritten.push_str(&format!("figures/{name}"));
            last = whole.end();
            figs.push(ExtractedFigure { provisional_name: name, bytes: data, origin: "data-uri".into() });
        }
        if figs.is_empty() {
            return (figs, None);
        }
        rewritten.push_str(&markdown[last..]);
        return (figs, Some(rewritten));
    }
    // docx/pptx/xlsx/ods: copy image members out of the zip media folder.
    let media_prefix = match ext.as_str() {
        "docx" => "word/media/",
        "pptx" => "ppt/media/",
        "xlsx" | "ods" => "xl/media/",
        _ => return (vec![], None),
    };
    let mut figs = Vec::new();
    if let Ok(mut zip) = zip::ZipArchive::new(Cursor::new(bytes.to_vec())) {
        let mut names: Vec<String> = zip
            .file_names()
            .filter(|n| n.starts_with(media_prefix) && is_image_ext(&ext_of(n)))
            .map(|s| s.to_string())
            .collect();
        names.sort();
        for (i, name) in names.iter().enumerate() {
            let mut buf = Vec::new();
            match zip.by_name(name) {
                Ok(mut f) => {
                    if f.read_to_end(&mut buf).is_err() {
                        continue;
                    }
                }
                Err(_) => continue,
            }
            let ie = ext_of(name).to_ascii_lowercase();
            figs.push(ExtractedFigure {
                provisional_name: format!("fig-{:03}.{}", i + 1, ie),
                bytes: buf,
                origin: name.clone(),
            });
        }
    }
    (figs, None)
}

/// Convert a single file's bytes to Markdown, dispatching on extension.
pub fn convert_bytes(ext: &str, filename: &str, bytes: &[u8]) -> Converted {
    let ext = ext.to_ascii_lowercase();
    match ext.as_str() {
        e if is_image_ext(e) => Converted {
            markdown: format!("![{filename}](figures/{filename})\n"),
            title: String::new(),
            format: "image".into(),
            needs_llm_fallback: true,
        },
        "md" | "markdown" | "txt" | "text" | "" | "xml" | "yaml" | "yml" | "rst" | "log" | "tex" | "org" => passthrough(bytes, "text"),
        "json" => {
            let body = String::from_utf8_lossy(bytes);
            Converted { markdown: format!("```json\n{}\n```\n", body.trim_end()), title: String::new(), format: "json".into(), needs_llm_fallback: false }
        }
        "csv" | "tsv" => Converted { markdown: csv_to_md(bytes, if ext == "tsv" { '\t' } else { ',' }), title: String::new(), format: "csv".into(), needs_llm_fallback: false },
        "html" | "htm" => {
            let s = String::from_utf8_lossy(bytes);
            Converted { markdown: html2md::parse_html(&s), title: String::new(), format: "html".into(), needs_llm_fallback: false }
        }
        "xlsx" | "xls" | "ods" => match xlsx_to_md(bytes) {
            Ok(md) => Converted { markdown: md, title: String::new(), format: "spreadsheet".into(), needs_llm_fallback: false },
            Err(e) => Converted { markdown: format!("> spreadsheet conversion failed: {e}\n"), title: String::new(), format: "spreadsheet".into(), needs_llm_fallback: true },
        },
        "docx" => office_to_md(bytes, "word/document.xml", "</w:p>", None).unwrap_or_else(|e| Converted { markdown: format!("> docx conversion failed: {e}\n"), title: String::new(), format: "docx".into(), needs_llm_fallback: true }),
        "pptx" => pptx_to_md(bytes).unwrap_or_else(|e| Converted { markdown: format!("> pptx conversion failed: {e}\n"), title: String::new(), format: "pptx".into(), needs_llm_fallback: true }),
        "pdf" => Converted {
            markdown: format!("> **PDF source: needs OCR/LLM extraction.** `{filename}` was stored verbatim under `raw/`; produce a faithful Markdown rendering in the next step.\n"),
            title: String::new(),
            format: "pdf".into(),
            needs_llm_fallback: true,
        },
        _ => {
            // Unknown format: keep a best-effort text extract, but ALWAYS flag for the LLM to
            // render the original faithfully; by the end every source must be complete Markdown.
            let lossy = String::from_utf8_lossy(bytes);
            let snippet: String = lossy.chars().take(8000).collect();
            Converted {
                markdown: format!(
                    "Unknown format `.{ext}` (`{filename}`): best-effort text extract below; the agent must read `original.*` and render ALL content faithfully to Markdown.\n\n```\n{}\n```\n",
                    snippet.trim_end()
                ),
                title: String::new(),
                format: if ext.is_empty() { "unknown".into() } else { ext },
                needs_llm_fallback: true,
            }
        }
    }
}

fn passthrough(bytes: &[u8], format: &str) -> Converted {
    Converted { markdown: String::from_utf8_lossy(bytes).to_string(), title: String::new(), format: format.into(), needs_llm_fallback: false }
}

fn csv_to_md(bytes: &[u8], delim: char) -> String {
    let text = String::from_utf8_lossy(bytes);
    let mut out = String::new();
    let mut rows = text.lines().filter(|l| !l.trim().is_empty());
    if let Some(header) = rows.next() {
        let cells = split_delimited(header, delim);
        out.push_str("| ");
        out.push_str(&cells.join(" | "));
        out.push_str(" |\n|");
        for _ in &cells {
            out.push_str(" --- |");
        }
        out.push('\n');
        for (i, row) in rows.enumerate() {
            if i >= 500 {
                out.push_str("\n_(truncated at 500 rows)_\n");
                break;
            }
            out.push_str("| ");
            out.push_str(&split_delimited(row, delim).join(" | "));
            out.push_str(" |\n");
        }
    }
    out
}

/// Minimal quote-aware delimited-field split.
fn split_delimited(line: &str, delim: char) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_q = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if in_q {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    cur.push('"');
                    chars.next();
                } else {
                    in_q = false;
                }
            } else {
                cur.push(c);
            }
        } else if c == '"' {
            in_q = true;
        } else if c == delim {
            out.push(cur.trim().replace('|', "\\|"));
            cur = String::new();
        } else {
            cur.push(c);
        }
    }
    out.push(cur.trim().replace('|', "\\|"));
    out
}

fn xlsx_to_md(bytes: &[u8]) -> Result<String, String> {
    use calamine::{open_workbook_auto_from_rs, Reader};
    let mut wb = open_workbook_auto_from_rs(Cursor::new(bytes.to_vec())).map_err(|e| e.to_string())?;
    let mut out = String::new();
    let names = wb.sheet_names().to_owned();
    for name in names {
        let range = match wb.worksheet_range(&name) {
            Ok(r) => r,
            Err(_) => continue,
        };
        if range.is_empty() {
            continue;
        }
        out.push_str(&format!("## {name}\n\n"));
        let mut first = true;
        for (i, row) in range.rows().enumerate() {
            if i >= 500 {
                out.push_str("\n_(truncated at 500 rows)_\n");
                break;
            }
            let cells: Vec<String> = row.iter().map(|c| c.to_string().trim().replace('|', "\\|")).collect();
            out.push_str("| ");
            out.push_str(&cells.join(" | "));
            out.push_str(" |\n");
            if first {
                out.push('|');
                for _ in &cells {
                    out.push_str(" --- |");
                }
                out.push('\n');
                first = false;
            }
        }
        out.push('\n');
    }
    Ok(out)
}

/// Extract text from a single XML part inside a zip (docx). `para_close` marks
/// paragraph boundaries; `title` is an optional section heading.
fn office_to_md(bytes: &[u8], part: &str, para_close: &str, title: Option<&str>) -> Result<Converted, String> {
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes.to_vec())).map_err(|e| e.to_string())?;
    let mut xml = String::new();
    zip.by_name(part).map_err(|e| e.to_string())?.read_to_string(&mut xml).map_err(|e| e.to_string())?;
    let mut md = String::new();
    if let Some(t) = title {
        md.push_str(&format!("## {t}\n\n"));
    }
    md.push_str(&xml_text(&xml, para_close));
    Ok(Converted { markdown: md, title: String::new(), format: "docx".into(), needs_llm_fallback: false })
}

fn pptx_to_md(bytes: &[u8]) -> Result<Converted, String> {
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes.to_vec())).map_err(|e| e.to_string())?;
    let mut slides: Vec<String> = zip
        .file_names()
        .filter(|n| n.starts_with("ppt/slides/slide") && n.ends_with(".xml"))
        .map(|s| s.to_string())
        .collect();
    slides.sort_by(|a, b| slide_num(a).cmp(&slide_num(b)));
    let mut md = String::new();
    for (i, name) in slides.iter().enumerate() {
        let mut xml = String::new();
        if zip.by_name(name).and_then(|mut f| Ok(f.read_to_string(&mut xml))).is_err() {
            continue;
        }
        let text = xml_text(&xml, "</a:p>");
        if !text.trim().is_empty() {
            md.push_str(&format!("## Slide {}\n\n{}\n\n", i + 1, text.trim()));
        }
    }
    Ok(Converted { markdown: md, title: String::new(), format: "pptx".into(), needs_llm_fallback: false })
}

fn slide_num(name: &str) -> u32 {
    name.trim_start_matches("ppt/slides/slide").trim_end_matches(".xml").parse().unwrap_or(0)
}

/// Strip XML tags, inserting a paragraph break at each `para_close`, and decode the
/// five predefined XML entities.
fn xml_text(xml: &str, para_close: &str) -> String {
    let with_breaks = xml.replace(para_close, "\n\n");
    let mut out = String::new();
    let mut in_tag = false;
    for c in with_breaks.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    let out = out
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'");
    // collapse 3+ newlines
    let mut collapsed = String::new();
    let mut nl = 0;
    for c in out.chars() {
        if c == '\n' {
            nl += 1;
            if nl <= 2 {
                collapsed.push(c);
            }
        } else {
            nl = 0;
            collapsed.push(c);
        }
    }
    collapsed.trim().to_string()
}

// ---------------------------------------------------------------------------
// Archive / folder expansion
// ---------------------------------------------------------------------------

const MAX_MEMBERS: usize = 200;

/// Expand a `.zip` into `(member_filename, bytes)` pairs, skipping junk + directories.
pub fn expand_zip(bytes: &[u8]) -> Result<Vec<(String, Vec<u8>)>, String> {
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes.to_vec())).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for i in 0..zip.len().min(MAX_MEMBERS) {
        let mut f = zip.by_index(i).map_err(|e| e.to_string())?;
        if f.is_dir() {
            continue;
        }
        let name = f.name().to_string();
        if is_junk(&name) {
            continue;
        }
        let mut buf = Vec::new();
        if f.read_to_end(&mut buf).is_ok() {
            out.push((name, buf));
        }
    }
    Ok(out)
}

/// Walk a folder into `(relative_path, bytes)` pairs.
pub fn expand_folder(dir: &Path) -> Vec<(String, Vec<u8>)> {
    let mut out = Vec::new();
    walk(dir, dir, &mut out);
    out
}

fn walk(base: &Path, dir: &Path, out: &mut Vec<(String, Vec<u8>)>) {
    if out.len() >= MAX_MEMBERS {
        return;
    }
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for e in entries.flatten() {
        let p = e.path();
        let name = e.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        if p.is_dir() {
            walk(base, &p, out);
        } else if let Ok(bytes) = std::fs::read(&p) {
            let rel = p.strip_prefix(base).unwrap_or(&p).to_string_lossy().to_string();
            if !is_junk(&rel) {
                out.push((rel, bytes));
            }
        }
    }
}

fn is_junk(name: &str) -> bool {
    let base = name.rsplit('/').next().unwrap_or(name);
    name.contains("__MACOSX") || base == ".DS_Store" || base.starts_with("._")
}

// ---------------------------------------------------------------------------
// Naming + storage
// ---------------------------------------------------------------------------

fn ext_of(name: &str) -> String {
    Path::new(name).extension().and_then(|e| e.to_str()).unwrap_or("").to_string()
}

/// A human-readable slug from a title (lowercase, non-alnum → `-`, ≤40 chars).
fn slug(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') {
            out.push('-');
        }
        if out.trim_matches('-').len() >= 40 {
            break;
        }
    }
    out.trim_matches('-').to_string()
}

/// True when a candidate title is machine-generated noise (all-hex / uuid-ish) and
/// should NOT be used as a human-readable name.
pub fn looks_machine_generated(s: &str) -> bool {
    let t = s.trim().replace('-', "");
    t.len() >= 12 && t.chars().all(|c| c.is_ascii_hexdigit())
}

/// Pick a human-readable title: first Markdown heading → first substantial line →
/// cleaned filename → "Untitled source". Never a hash/uuid.
pub fn title_from(markdown: &str, filename: &str, explicit: Option<&str>) -> String {
    if let Some(t) = explicit {
        if !t.trim().is_empty() && !looks_machine_generated(t) {
            return t.trim().to_string();
        }
    }
    for line in markdown.lines() {
        let l = line.trim();
        if let Some(h) = l.strip_prefix('#') {
            let h = h.trim_start_matches('#').trim();
            if !h.is_empty() && !looks_machine_generated(h) {
                return h.to_string();
            }
        }
    }
    for line in markdown.lines() {
        let l = line.trim();
        if l.len() >= 8 && !looks_machine_generated(l) {
            return l.chars().take(80).collect();
        }
    }
    let stem = Path::new(filename).file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let cleaned = stem.replace(['_', '-'], " ");
    if !cleaned.trim().is_empty() && !looks_machine_generated(&cleaned) {
        return cleaned.trim().to_string();
    }
    "Untitled source".to_string()
}

fn hash_bytes(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    format!("{:x}", h.finalize())
}

/// `<title-slug>-<6 hex>`: content-derived and human-readable, never a bare hash.
fn new_source_id(title: &str, sha: &str) -> String {
    let s = slug(title);
    let base = if s.is_empty() { "source".to_string() } else { s };
    format!("{base}-{}", &sha[..6])
}

/// Find an already-ingested source with the same content hash (dedup).
fn find_existing(bundle_root: &Path, sha: &str) -> Option<String> {
    let raw = bundle_root.join("raw");
    let entries = std::fs::read_dir(&raw).ok()?;
    for e in entries.flatten() {
        let meta = e.path().join("meta.yaml");
        if let Ok(txt) = std::fs::read_to_string(&meta) {
            if let Ok(m) = serde_yaml::from_str::<SourceMeta>(&txt) {
                if m.sha256 == sha {
                    return Some(m.id);
                }
            }
        }
    }
    None
}

/// Convert + store one source's bytes under `raw/<id>/`. Returns the record (with
/// `reused=true` when an identical source already exists).
fn store(bundle_root: &Path, filename: &str, bytes: &[u8]) -> Result<SourceRecord, String> {
    let sha = hash_bytes(bytes);
    if let Some(id) = find_existing(bundle_root, &sha) {
        return Ok(SourceRecord {
            source_id: id.clone(),
            source_md_path: format!("raw/{id}/source.md"),
            meta_path: format!("raw/{id}/meta.yaml"),
            title: id,
            needs_llm_fallback: false,
            reused: true,
        });
    }
    let ext = ext_of(filename);
    let converted = convert_bytes(&ext, filename, bytes);
    let title = title_from(&converted.markdown, filename, None);
    let id = new_source_id(&title, &sha);
    let dir = bundle_root.join("raw").join(&id);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    // original bytes (immutable; gitignored)
    let orig_ext = if ext.is_empty() { "bin".to_string() } else { ext.clone() };
    std::fs::write(dir.join(format!("original.{orig_ext}")), bytes).map_err(|e| e.to_string())?;
    // derived markdown (prepend a machine-detectable marker when an LLM rendering is still needed)
    let banner = if converted.needs_llm_fallback {
        format!("{NEEDS_CONVERSION_MARKER}\n> ⚠️ Needs faithful Markdown conversion (unknown/binary format or scanned PDF). Read `original.*`, render ALL content to Markdown preserving every detail, then overwrite this file (removing this marker).\n\n")
    } else {
        String::new()
    };
    std::fs::write(dir.join("source.md"), format!("{banner}{}", converted.markdown)).map_err(|e| e.to_string())?;
    // meta
    let meta = SourceMeta {
        id: id.clone(),
        title: title.clone(),
        sha256: sha,
        format: converted.format,
        original_filename: Some(filename.to_string()),
        ingested_at: today_iso(),
        needs_llm_fallback: converted.needs_llm_fallback,
        figures: vec![],
    };
    std::fs::write(dir.join("meta.yaml"), serde_yaml::to_string(&meta).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    Ok(SourceRecord {
        source_id: id.clone(),
        source_md_path: format!("raw/{id}/source.md"),
        meta_path: format!("raw/{id}/meta.yaml"),
        title,
        needs_llm_fallback: meta.needs_llm_fallback,
        reused: false,
    })
}

/// Ingest a source into a bundle's `raw/`. A `.zip` or folder expands to one source
/// per member unless `combined`, which concatenates members into a single source.
pub fn ingest(bundle_root: &Path, input: SourceInput, combined: bool) -> Result<Vec<SourceRecord>, String> {
    // Resolve to (filename, bytes) members.
    let members: Vec<(String, Vec<u8>)> = match input {
        SourceInput::Text { text, title } => {
            let name = title.as_deref().map(|t| format!("{}.md", slug(t))).unwrap_or_else(|| "note.md".into());
            vec![(name, text.into_bytes())]
        }
        SourceInput::Path(p) => {
            if p.is_dir() {
                expand_folder(&p)
            } else {
                let bytes = std::fs::read(&p).map_err(|e| e.to_string())?;
                let name = p.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "source".into());
                if ext_of(&name) == "zip" {
                    expand_zip(&bytes)?
                } else {
                    vec![(name, bytes)]
                }
            }
        }
    };
    if members.is_empty() {
        return Err("no convertible members found".into());
    }

    if combined && members.len() > 1 {
        let mut md = String::new();
        let mut needs = false;
        for (name, bytes) in &members {
            let c = convert_bytes(&ext_of(name), name, bytes);
            needs |= c.needs_llm_fallback;
            md.push_str(&format!("# {}\n\n{}\n\n", name, c.markdown.trim()));
        }
        let combined_bytes = md.clone().into_bytes();
        return Ok(vec![store(bundle_root, "combined.md", &combined_bytes)?]).map(|mut v| {
            if let Some(r) = v.first_mut() {
                r.needs_llm_fallback |= needs;
            }
            v
        });
    }

    let mut records = Vec::new();
    for (name, bytes) in members {
        records.push(store(bundle_root, &name, &bytes)?);
    }
    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_becomes_markdown_table() {
        let md = csv_to_md(b"gene,role\nBRAF,kinase\nTP53,suppressor", ',');
        assert!(md.contains("| gene | role |"));
        assert!(md.contains("| BRAF | kinase |"));
        assert!(md.contains("| --- |"));
    }

    #[test]
    fn title_prefers_heading_rejects_hash() {
        assert_eq!(title_from("# Tocilizumab in COVID-19\n\nbody", "x.md", None), "Tocilizumab in COVID-19");
        // a hex-hash title is rejected; falls back to the filename
        assert_eq!(title_from("body", "the-recovery-trial.pdf", Some("deadbeefcafe1234")), "the recovery trial");
        assert!(looks_machine_generated("deadbeefcafe1234"));
        assert!(!looks_machine_generated("Tocilizumab"));
    }

    #[test]
    fn xml_text_strips_tags_and_breaks_paragraphs() {
        let xml = "<w:p><w:r><w:t>Hello</w:t></w:r></w:p><w:p><w:r><w:t>World &amp; more</w:t></w:r></w:p>";
        let t = xml_text(xml, "</w:p>");
        assert!(t.contains("Hello"));
        assert!(t.contains("World & more"));
    }

    #[test]
    fn ingest_file_and_folder_produce_human_readable_ids() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join("raw")).unwrap();
        // a single markdown source
        let src = tempfile::tempdir().unwrap();
        std::fs::write(src.path().join("recovery-trial.md"), "# RECOVERY Trial\n\nDexamethasone reduced mortality.").unwrap();
        let recs = ingest(root, SourceInput::Path(src.path().join("recovery-trial.md")), false).unwrap();
        assert_eq!(recs.len(), 1);
        let id = &recs[0].source_id;
        assert!(id.starts_with("recovery-trial-"), "id was {id}");
        // id is content-derived, not a bare hash/uuid
        assert!(!looks_machine_generated(id.rsplit('-').next().unwrap()) || id.contains("recovery"));
        assert!(root.join(&recs[0].source_md_path).exists());
        assert!(root.join(&recs[0].meta_path).exists());

        // dedup: ingesting the same bytes again reuses
        let recs2 = ingest(root, SourceInput::Path(src.path().join("recovery-trial.md")), false).unwrap();
        assert!(recs2[0].reused);
        assert_eq!(recs2[0].source_id, *id);
    }

    #[test]
    fn unknown_format_flags_needs_conversion() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join("raw")).unwrap();
        std::fs::create_dir_all(root.join("knowledge")).unwrap();
        let src = tempfile::tempdir().unwrap();
        std::fs::write(src.path().join("weird.xyz"), b"\x00\x01 some bytes that are not a known format").unwrap();
        let recs = ingest(root, SourceInput::Path(src.path().join("weird.xyz")), false).unwrap();
        assert!(recs[0].needs_llm_fallback, "unknown format must flag LLM fallback");
        // the marker is present until the agent renders it
        let report = crate::lint::lint(&crate::bundle::Bundle::open(root).unwrap());
        assert!(report.findings.iter().any(|f| f.rule == "source.needs_conversion"), "{:?}", report.findings);
    }

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

    #[test]
    fn image_file_becomes_image_source() {
        assert!(is_image_ext("PNG"));
        let c = convert_bytes("png", "Figure_3.png", &[0x89, b'P', b'N', b'G']);
        assert_eq!(c.format, "image");
        assert!(c.needs_llm_fallback);
        assert!(c.markdown.contains("![Figure_3.png](figures/Figure_3.png)"), "{}", c.markdown);
    }

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

    #[test]
    fn folder_expands_per_member_with_distinct_names() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join("raw")).unwrap();
        let folder = tempfile::tempdir().unwrap();
        std::fs::write(folder.path().join("alpha.md"), "# Alpha Study\nbody a").unwrap();
        std::fs::write(folder.path().join("beta.csv"), "x,y\n1,2").unwrap();
        std::fs::write(folder.path().join(".DS_Store"), "junk").unwrap();
        let recs = ingest(root, SourceInput::Path(folder.path().to_path_buf()), false).unwrap();
        assert_eq!(recs.len(), 2, "junk should be skipped: {recs:?}");
        let ids: Vec<&str> = recs.iter().map(|r| r.source_id.as_str()).collect();
        assert!(ids.iter().any(|i| i.starts_with("alpha-study-")));
        // every id is human-readable (content-derived), never an all-hex blob
        for r in &recs {
            assert!(!looks_machine_generated(&r.source_id), "{}", r.source_id);
        }
    }
}
