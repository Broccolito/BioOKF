//! BioOKF Studio — the Tauri desktop app. A thin front-end: every command
//! delegates to `bokf-core`, so the GUI is a pure visualizer/dashboard over the
//! same backend the CLI and MCP server use.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use tauri::{AppHandle, Emitter};

/// Root that contains the bundles (env `OKF_ROOT`, else the BioOKF repo root).
fn repo_root() -> PathBuf {
    std::env::var("OKF_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.."))
}

/// Candidate bundle directories: the canonical `examples/` plus every dir under
/// `studio/test-kb/`.
fn candidate_bundles() -> Vec<PathBuf> {
    let root = repo_root();
    let mut v = vec![root.join("examples")];
    if let Ok(rd) = std::fs::read_dir(root.join("studio/test-kb")) {
        for e in rd.flatten() {
            if e.path().is_dir() {
                v.push(e.path());
            }
        }
    }
    v.into_iter()
        .filter(|p| p.join("knowledge").is_dir() || p.join("index.md").is_file())
        .collect()
}

fn resolve(id: &str) -> Option<PathBuf> {
    candidate_bundles()
        .into_iter()
        .find(|p| p.file_name().map(|n| n.to_string_lossy() == id).unwrap_or(false))
}

#[tauri::command]
fn list_bases() -> Result<serde_json::Value, String> {
    let mut out = Vec::new();
    for p in candidate_bundles() {
        if let Ok(info) = bokf_core::export::base_info(&p) {
            out.push(info);
        }
    }
    Ok(serde_json::Value::Array(out))
}

#[tauri::command]
fn get_bundle(id: String) -> Result<serde_json::Value, String> {
    let path = resolve(&id).ok_or_else(|| format!("unknown bundle: {id}"))?;
    bokf_core::export::bundle_doc(&path, None).map_err(|e| e.to_string())
}

#[tauri::command]
fn lint_bundle(id: String) -> Result<serde_json::Value, String> {
    let path = resolve(&id).ok_or_else(|| format!("unknown bundle: {id}"))?;
    let bundle = bokf_core::open_bundle(&path).map_err(|e| e.to_string())?;
    let report = bokf_core::lint(&bundle);
    serde_json::to_value(&report).map_err(|e| e.to_string())
}

#[tauri::command]
fn search_bundle(id: String, query: String, limit: Option<usize>) -> Result<serde_json::Value, String> {
    let path = resolve(&id).ok_or_else(|| format!("unknown bundle: {id}"))?;
    let bundle = bokf_core::open_bundle(&path).map_err(|e| e.to_string())?;
    let index = bokf_core::SearchIndex::build(&bundle);
    let hits = index.search(&query, limit.unwrap_or(10));
    serde_json::to_value(&hits).map_err(|e| e.to_string())
}

/// On-disk provenance for one raw source (`raw/<source_id>/meta.yaml`): source
/// type, credibility, figures, identifiers. Additive read-only connector so the
/// GUI can render figures / credibility / source type.
#[tauri::command]
fn source_info(base: String, source_id: String) -> Result<serde_json::Value, String> {
    let root = resolve(&base).ok_or_else(|| format!("unknown bundle: {base}"))?;
    // Guard the source id against path traversal: it is a single dir name under `raw/`.
    if source_id.is_empty()
        || source_id.contains('/')
        || source_id.contains('\\')
        || source_id.contains("..")
    {
        return Err("invalid source id".into());
    }
    bokf_core::export::source_info(&root, &source_id)
}

/// Replace the markdown body of a node file while preserving its YAML
/// frontmatter (the leading `---` … `---` block) verbatim. If the file has no
/// frontmatter, the whole file becomes the new body. Pure string transform so
/// it is trivially testable.
fn replace_body(existing: &str, new_body: &str) -> String {
    let body = new_body.trim_end_matches('\n');
    let mut iter = existing.lines();
    if iter.next().map(|l| l.trim_end()) == Some("---") {
        let mut fm = String::from("---\n");
        let mut closed = false;
        for line in iter {
            fm.push_str(line);
            fm.push('\n');
            if line.trim_end() == "---" {
                closed = true;
                break;
            }
        }
        if closed {
            return format!("{fm}\n{body}\n");
        }
    }
    format!("{body}\n")
}

/// The text after the closing `---` of the frontmatter (blank line + body),
/// preserved verbatim. `None` when there is no frontmatter block.
fn body_after_frontmatter(content: &str) -> Option<String> {
    if content.lines().next().map(|l| l.trim_end()) != Some("---") {
        return None;
    }
    let mut offset = 0usize;
    let mut first = true;
    for line in content.split_inclusive('\n') {
        offset += line.len();
        if first {
            first = false;
            continue;
        }
        if line.trim_end_matches(['\r', '\n']) == "---" {
            return Some(content[offset..].to_string());
        }
    }
    None
}

/// Replace the YAML frontmatter (between the `---` fences) while preserving the
/// document body verbatim. `new_fm` is the YAML without fences.
fn replace_frontmatter(existing: &str, new_fm: &str) -> String {
    let fm = new_fm.trim_matches('\n');
    match body_after_frontmatter(existing) {
        Some(rest) => format!("---\n{fm}\n---\n{rest}"),
        None => format!("---\n{fm}\n---\n\n{}", existing.trim_start_matches('\n')),
    }
}

/// Resolve `rel` inside the bundle for `base`, rejecting path traversal. The file
/// must already exist (it is canonicalized and checked to stay under the root).
fn safe_bundle_path(base: &str, rel: &str) -> Result<PathBuf, String> {
    let root = resolve(base).ok_or_else(|| format!("unknown bundle: {base}"))?;
    let r = std::path::Path::new(rel);
    if r.is_absolute()
        || r.components()
            .any(|c| matches!(c, std::path::Component::ParentDir | std::path::Component::Prefix(_)))
    {
        return Err("invalid path".into());
    }
    let root_c = root.canonicalize().map_err(|e| e.to_string())?;
    let full_c = root.join(r).canonicalize().map_err(|e| e.to_string())?;
    if !full_c.starts_with(&root_c) {
        return Err("path escapes bundle".into());
    }
    Ok(full_c)
}

/// Persist a user edit to a node's document body, preserving its frontmatter.
#[tauri::command]
fn save_node_body(base: String, path: String, body: String) -> Result<(), String> {
    let full = safe_bundle_path(&base, &path)?;
    let existing = std::fs::read_to_string(&full).map_err(|e| e.to_string())?;
    std::fs::write(&full, replace_body(&existing, &body)).map_err(|e| e.to_string())?;
    Ok(())
}

/// Read a file inside a bundle as text (e.g. the raw source paper, a node `.md`).
#[tauri::command]
fn read_bundle_file(base: String, path: String) -> Result<String, String> {
    let full = safe_bundle_path(&base, &path)?;
    std::fs::read_to_string(&full).map_err(|e| e.to_string())
}

/// Standard base64 (RFC 4648) encode, std-only — used to ship binary bundle files
/// (e.g. `raw/<id>/figures/*.png`) to the webview as data URIs.
fn base64_encode(bytes: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((bytes.len() + 2) / 3 * 4);
    for c in bytes.chunks(3) {
        let b = [c[0], *c.get(1).unwrap_or(&0), *c.get(2).unwrap_or(&0)];
        let n = (b[0] as usize) << 16 | (b[1] as usize) << 8 | b[2] as usize;
        out.push(T[(n >> 18) & 63] as char);
        out.push(T[(n >> 12) & 63] as char);
        out.push(if c.len() > 1 { T[(n >> 6) & 63] as char } else { '=' });
        out.push(if c.len() > 2 { T[n & 63] as char } else { '=' });
    }
    out
}

/// Read a binary file inside a bundle (path-guarded) and return it base64-encoded,
/// so figures and other non-text assets can be inlined as data URIs in the webview.
#[tauri::command]
fn read_bundle_bytes(base: String, path: String) -> Result<String, String> {
    let full = safe_bundle_path(&base, &path)?;
    let bytes = std::fs::read(&full).map_err(|e| e.to_string())?;
    Ok(base64_encode(&bytes))
}

/// Persist an edited frontmatter block, preserving the document body verbatim.
#[tauri::command]
fn save_node_frontmatter(
    base: String,
    path: String,
    frontmatter: String,
    label: String,
    date: String,
) -> Result<(), String> {
    let full = safe_bundle_path(&base, &path)?;
    let existing = std::fs::read_to_string(&full).map_err(|e| e.to_string())?;
    std::fs::write(&full, replace_frontmatter(&existing, &frontmatter)).map_err(|e| e.to_string())?;
    append_log_entry(&base, &date, &format!("- Edited frontmatter of `{}`", label))?;
    Ok(())
}

/// The YAML frontmatter text (between the first `---` line and its closing `---`
/// line), without the fences. `None` when the document has no frontmatter block.
fn frontmatter_yaml(content: &str) -> Option<String> {
    if content.lines().next().map(|l| l.trim_end()) != Some("---") {
        return None;
    }
    let mut iter = content.split_inclusive('\n');
    iter.next(); // skip the opening `---`
    let mut fm = String::new();
    for line in iter {
        if line.trim_end_matches(['\r', '\n']) == "---" {
            return Some(fm);
        }
        fm.push_str(line);
    }
    None
}

/// Insert or replace a top-level `# Notes` section at the END of a document body.
/// Pure string transform: frontmatter is the caller's concern.
///
/// * Empty `notes`: drop an existing `# Notes` section (from the exact `# Notes`
///   line through the next top-level `# ` heading or EOF), leaving the rest
///   unchanged (trailing blank lines collapsed to one).
/// * Otherwise: replace the content of an existing `# Notes` section, or append a
///   new one. Exactly one blank line precedes `# Notes`.
fn upsert_notes_section(body: &str, notes: &str) -> String {
    let lines: Vec<&str> = body.lines().collect();
    // Locate an existing top-level `# Notes` heading.
    let start = lines.iter().position(|l| *l == "# Notes");

    if notes.trim().is_empty() {
        let Some(start) = start else {
            // No section to remove; return body unchanged.
            return body.to_string();
        };
        // Find the end of the section: next top-level `# ` heading, or EOF.
        let end = lines[start + 1..]
            .iter()
            .position(|l| l.starts_with("# "))
            .map(|i| start + 1 + i)
            .unwrap_or(lines.len());
        let mut kept: Vec<&str> = Vec::new();
        kept.extend_from_slice(&lines[..start]);
        kept.extend_from_slice(&lines[end..]);
        let joined = kept.join("\n");
        let trimmed = joined.trim_end_matches('\n');
        return if trimmed.is_empty() { String::new() } else { format!("{trimmed}\n") };
    }

    let notes_body = notes.trim_end_matches('\n');
    match start {
        Some(start) => {
            // Replace the existing section's content, keeping everything after the
            // section (the next `# ` heading onward) verbatim.
            let end = lines[start + 1..]
                .iter()
                .position(|l| l.starts_with("# "))
                .map(|i| start + 1 + i)
                .unwrap_or(lines.len());
            let before = lines[..start].join("\n");
            let before = before.trim_end_matches('\n');
            let after = lines[end..].join("\n");
            let mut out = String::new();
            if !before.is_empty() {
                out.push_str(before);
                out.push_str("\n\n");
            }
            out.push_str("# Notes\n\n");
            out.push_str(notes_body);
            out.push('\n');
            if !after.is_empty() {
                out.push('\n');
                out.push_str(after.trim_end_matches('\n'));
                out.push('\n');
            }
            out
        }
        None => {
            // Append a fresh `# Notes` section at the end of the body.
            let trimmed = body.trim_end_matches('\n');
            if trimmed.is_empty() {
                format!("# Notes\n\n{notes_body}\n")
            } else {
                format!("{trimmed}\n\n# Notes\n\n{notes_body}\n")
            }
        }
    }
}

/// Write/update a top-level `# Notes` section at the end of a node file's body,
/// preserving its frontmatter and existing body content. Logs the change.
#[tauri::command]
fn save_node_notes(
    base: String,
    path: String,
    notes: String,
    label: String,
    date: String,
) -> Result<(), String> {
    let full = safe_bundle_path(&base, &path)?;
    let existing = std::fs::read_to_string(&full).map_err(|e| e.to_string())?;
    let body = body_after_frontmatter(&existing).unwrap_or_else(|| existing.clone());
    let body_trimmed = body.trim_start_matches('\n');
    let new_body = upsert_notes_section(body_trimmed, &notes);
    std::fs::write(&full, replace_body(&existing, &new_body)).map_err(|e| e.to_string())?;
    append_log_entry(
        &base,
        &date,
        &format!(
            "- {} notes on `{}`",
            if notes.trim().is_empty() { "Cleared" } else { "Updated" },
            label
        ),
    )?;
    Ok(())
}

/// Unquote a YAML scalar value: strip a single pair of surrounding single or
/// double quotes (for matching purposes only — does not decode escapes).
fn unquote_yaml(v: &str) -> &str {
    let v = v.trim();
    if v.len() >= 2 {
        let bytes = v.as_bytes();
        if (bytes[0] == b'"' && bytes[v.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[v.len() - 1] == b'\'')
        {
            return &v[1..v.len() - 1];
        }
    }
    v
}

/// Encode `v` as a YAML double-quoted scalar (collapsing newlines to `\n`).
fn yaml_double_quote(v: &str) -> String {
    let mut s = String::from("\"");
    for ch in v.chars() {
        match ch {
            '\\' => s.push_str("\\\\"),
            '"' => s.push_str("\\\""),
            '\n' => s.push_str("\\n"),
            '\r' => {} // drop carriage returns; the newline handling covers line breaks
            c => s.push(c),
        }
    }
    s.push('"');
    s
}

/// Set (or, when `note` is blank, clear) the `note:` field on the one `edges:`
/// entry whose `predicate:` and `object:` both match. Targeted line edit that
/// preserves key order and every other line verbatim. Pure string transform.
fn set_edge_note_in_fm(
    fm_yaml: &str,
    predicate: &str,
    object: &str,
    note: &str,
) -> Result<String, String> {
    // Each entry in the `edges:` list begins with a `- ` line. We track the lines
    // belonging to the current entry and, once we know it matches, edit them.
    let lines: Vec<&str> = fm_yaml.lines().collect();
    let pred_target = predicate.trim();
    let obj_target = object.trim();
    let clear = note.trim().is_empty();

    // Identify entry boundaries inside the `edges:` block. An entry starts at a
    // line whose trimmed text begins with `- ` and that line carries a
    // `predicate:` field (either inline `- predicate:` or the first field of the
    // dash item). We treat any `- ` dash within the edges indentation as a new
    // item boundary.
    let mut entry_starts: Vec<usize> = Vec::new();
    let mut in_edges = false;
    let mut edges_indent: usize = 0;
    for (i, raw) in lines.iter().enumerate() {
        let trimmed = raw.trim_start();
        let indent = raw.len() - trimmed.len();
        if trimmed.starts_with("edges:") && indent == 0 {
            in_edges = true;
            continue;
        }
        if in_edges {
            // A new top-level key (indent 0, not a dash) ends the edges block.
            if indent == 0 && !trimmed.starts_with('-') && !trimmed.is_empty() {
                in_edges = false;
                continue;
            }
            if trimmed.starts_with("- ") || trimmed == "-" {
                if entry_starts.is_empty() {
                    edges_indent = indent;
                }
                if indent == edges_indent {
                    entry_starts.push(i);
                }
            }
        }
    }

    if entry_starts.is_empty() {
        return Err("edge not found".into());
    }

    // For each entry, compute its line range [start, end).
    let line_count = lines.len();
    let mut matched: Option<(usize, usize)> = None;
    for (idx, &start) in entry_starts.iter().enumerate() {
        let end = entry_starts.get(idx + 1).copied().unwrap_or_else(|| {
            // Extend to the end of the edges block: stop at the first line that
            // dedents back to (or past) a top-level non-dash key.
            let mut e = start + 1;
            while e < line_count {
                let trimmed = lines[e].trim_start();
                let indent = lines[e].len() - trimmed.len();
                if indent == 0 && !trimmed.starts_with('-') && !trimmed.is_empty() {
                    break;
                }
                e += 1;
            }
            e
        });
        // Within this entry, read its predicate and object field values.
        let mut pred_val: Option<String> = None;
        let mut obj_val: Option<String> = None;
        for line in &lines[start..end] {
            let t = line.trim_start();
            // The dash item's first field may be inline: `- predicate: foo`.
            let body = t.strip_prefix("- ").unwrap_or(t);
            if let Some(rest) = body.strip_prefix("predicate:") {
                pred_val = Some(unquote_yaml(rest).to_string());
            } else if let Some(rest) = body.strip_prefix("object:") {
                obj_val = Some(unquote_yaml(rest).to_string());
            }
        }
        if pred_val.as_deref() == Some(pred_target) && obj_val.as_deref() == Some(obj_target) {
            matched = Some((start, end));
            break;
        }
    }

    let (start, end) = matched.ok_or_else(|| "edge not found".to_string())?;

    // Determine the field indentation used inside this entry (the indent of the
    // entry's fields — i.e. the columns after `- `, or the indent of subsequent
    // field lines).
    let field_indent: String = {
        // Prefer a non-dash field line's indentation; fall back to dash-indent + 2.
        let mut found: Option<String> = None;
        for line in &lines[start..end] {
            let trimmed = line.trim_start();
            if trimmed.is_empty() {
                continue;
            }
            if trimmed.starts_with("- ") {
                continue;
            }
            let indent = line.len() - trimmed.len();
            found = Some(" ".repeat(indent));
            break;
        }
        found.unwrap_or_else(|| {
            let dash_indent = lines[start].len() - lines[start].trim_start().len();
            " ".repeat(dash_indent + 2)
        })
    };

    // Locate an existing `note:` line within the entry and the `object:` line.
    let mut note_line: Option<usize> = None;
    let mut object_line: Option<usize> = None;
    for (off, line) in lines[start..end].iter().enumerate() {
        let i = start + off;
        let t = line.trim_start();
        let body = t.strip_prefix("- ").unwrap_or(t);
        if body.starts_with("note:") {
            note_line = Some(i);
        }
        if body.starts_with("object:") {
            object_line = Some(i);
        }
    }

    let mut out_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();

    if clear {
        if let Some(i) = note_line {
            out_lines.remove(i);
        }
        // If there was no note, nothing changes.
    } else {
        let new_line = format!("{field_indent}note: {}", yaml_double_quote(note));
        match note_line {
            Some(i) => out_lines[i] = new_line,
            None => {
                let insert_at = object_line.map(|i| i + 1).unwrap_or(end);
                out_lines.insert(insert_at, new_line);
            }
        }
    }

    Ok(out_lines.join("\n"))
}

/// Set/clear the `note:` on a single edge inside a node file's frontmatter,
/// matched by predicate + object. Preserves all other frontmatter and the body.
#[tauri::command]
#[allow(clippy::too_many_arguments)]
fn save_edge_note(
    base: String,
    path: String,
    predicate: String,
    object: String,
    note: String,
    label: String,
    date: String,
) -> Result<(), String> {
    let full = safe_bundle_path(&base, &path)?;
    let existing = std::fs::read_to_string(&full).map_err(|e| e.to_string())?;
    let fm = frontmatter_yaml(&existing).ok_or_else(|| "no frontmatter".to_string())?;
    let edited_fm = set_edge_note_in_fm(&fm, &predicate, &object, &note)?;
    std::fs::write(&full, replace_frontmatter(&existing, &edited_fm)).map_err(|e| e.to_string())?;
    append_log_entry(
        &base,
        &date,
        &format!(
            "- {} note on edge `{}`",
            if note.trim().is_empty() { "Cleared" } else { "Set" },
            label
        ),
    )?;
    Ok(())
}

/// Append a `- ...` bullet entry to the bundle's `log.md`, grouping same-day
/// edits under one `## YYYY-MM-DD` H2 section (newest-first). Internal helper.
fn append_log_entry(base: &str, date: &str, entry: &str) -> Result<(), String> {
    let bundle = resolve(base).ok_or_else(|| format!("unknown bundle: {base}"))?;
    let log = bundle.join("log.md");
    let existing = std::fs::read_to_string(&log).unwrap_or_else(|_| "# Change log\n".to_string());

    let date_heading = format!("## {date}");
    let lines: Vec<&str> = existing.lines().collect();

    let new_content = if let Some(sec) = lines.iter().position(|l| l.trim_end() == date_heading) {
        // Append the bullet to the end of this date section's content (just after the
        // last non-blank line, with a single blank line before the next section).
        let end = lines[sec + 1..]
            .iter()
            .position(|l| l.starts_with("## "))
            .map(|i| sec + 1 + i)
            .unwrap_or(lines.len());
        let mut content_end = end;
        while content_end > sec + 1 && lines[content_end - 1].trim().is_empty() {
            content_end -= 1;
        }
        let mut out: Vec<String> = Vec::new();
        out.extend(lines[..content_end].iter().map(|s| s.to_string()));
        if content_end == sec + 1 {
            out.push(String::new()); // empty section: blank line after the heading
        }
        out.push(entry.to_string());
        if end < lines.len() {
            out.push(String::new()); // blank line before the next section
        }
        out.extend(lines[end..].iter().map(|s| s.to_string()));
        format!("{}\n", out.join("\n").trim_end_matches('\n'))
    } else {
        // Insert a new section immediately after the first line (the title).
        let mut out: Vec<String> = Vec::new();
        if let Some(first) = lines.first() {
            out.push(first.to_string());
        }
        out.push(String::new());
        out.push(date_heading);
        out.push(String::new());
        out.push(entry.to_string());
        let mut rest = 1;
        while rest < lines.len() && lines[rest].trim().is_empty() {
            rest += 1;
        }
        if rest < lines.len() {
            out.push(String::new());
            out.extend(lines[rest..].iter().map(|s| s.to_string()));
        }
        format!("{}\n", out.join("\n").trim_end_matches('\n'))
    };

    std::fs::write(&log, new_content).map_err(|e| e.to_string())?;
    Ok(())
}

/// Reveal a bundle file in the macOS Finder (selecting it).
#[tauri::command]
fn reveal_in_finder(base: String, path: String) -> Result<(), String> {
    let full = safe_bundle_path(&base, &path)?;
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .args(["-R", &full.to_string_lossy()])
            .status()
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = full;
        Err("only supported on macOS".into())
    }
}

/* ---------- integrated terminal (real pseudo-terminal) ---------- */
struct PtySession {
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    child: Box<dyn Child + Send + Sync>,
}
fn sessions() -> &'static Mutex<HashMap<String, PtySession>> {
    static S: OnceLock<Mutex<HashMap<String, PtySession>>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(HashMap::new()))
}
static TERM_SEQ: AtomicU64 = AtomicU64::new(1);

/// Open a PTY running the user's `$SHELL`. Output streams to the frontend as
/// `term-output` events ({id, data}); `term-exit` (id) fires when it ends.
#[tauri::command]
fn term_open(app: AppHandle, rows: u16, cols: u16) -> Result<String, String> {
    let pair = native_pty_system()
        .openpty(PtySize { rows: rows.max(1), cols: cols.max(1), pixel_width: 0, pixel_height: 0 })
        .map_err(|e| e.to_string())?;
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
    let mut cmd = CommandBuilder::new(shell);
    cmd.env("TERM", "xterm-256color");
    if let Some(home) = std::env::var_os("HOME") {
        cmd.cwd(home);
    }
    let child = pair.slave.spawn_command(cmd).map_err(|e| e.to_string())?;
    drop(pair.slave); // parent doesn't need the slave handle
    let mut reader = pair.master.try_clone_reader().map_err(|e| e.to_string())?;
    let writer = pair.master.take_writer().map_err(|e| e.to_string())?;
    let id = format!("t{}", TERM_SEQ.fetch_add(1, Ordering::Relaxed));
    let (eid, eapp) = (id.clone(), app.clone());
    std::thread::spawn(move || {
        let mut buf = [0u8; 8192];
        loop {
            match reader.read(&mut buf) {
                Ok(0) | Err(_) => {
                    let _ = eapp.emit("term-exit", &eid);
                    break;
                }
                Ok(n) => {
                    let data = String::from_utf8_lossy(&buf[..n]).to_string();
                    let _ = eapp.emit("term-output", serde_json::json!({ "id": eid, "data": data }));
                }
            }
        }
    });
    sessions()
        .lock()
        .unwrap()
        .insert(id.clone(), PtySession { master: pair.master, writer, child });
    Ok(id)
}

/// Forward user keystrokes to the PTY.
#[tauri::command]
fn term_write(id: String, data: String) -> Result<(), String> {
    let mut s = sessions().lock().unwrap();
    let sess = s.get_mut(&id).ok_or("no such terminal")?;
    sess.writer.write_all(data.as_bytes()).map_err(|e| e.to_string())?;
    sess.writer.flush().map_err(|e| e.to_string())
}

/// Resize the PTY to match the front-end grid.
#[tauri::command]
fn term_resize(id: String, rows: u16, cols: u16) -> Result<(), String> {
    let s = sessions().lock().unwrap();
    let sess = s.get(&id).ok_or("no such terminal")?;
    sess.master
        .resize(PtySize { rows: rows.max(1), cols: cols.max(1), pixel_width: 0, pixel_height: 0 })
        .map_err(|e| e.to_string())
}

/// Kill the shell and drop the session.
#[tauri::command]
fn term_close(id: String) -> Result<(), String> {
    if let Some(mut sess) = sessions().lock().unwrap().remove(&id) {
        let _ = sess.child.kill();
    }
    Ok(())
}

fn main() {
    let builder = tauri::Builder::default()
        .setup(|_app| {
            // Native macOS vibrancy: the whole window becomes translucent frosted
            // glass (preserving the rounded window corners), so the canvas shows the
            // blurred desktop and the app's own surfaces layer on top.
            #[cfg(target_os = "macos")]
            {
                use tauri::Manager;
                use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial, NSVisualEffectState};
                if let Some(win) = _app.get_webview_window("main") {
                    let _ = apply_vibrancy(
                        &win,
                        NSVisualEffectMaterial::Sidebar,
                        Some(NSVisualEffectState::Active),
                        None,
                    );
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_bases,
            get_bundle,
            lint_bundle,
            search_bundle,
            save_node_body,
            read_bundle_file,
            read_bundle_bytes,
            save_node_frontmatter,
            save_node_notes,
            save_edge_note,
            reveal_in_finder,
            term_open,
            term_write,
            term_resize,
            term_close,
            source_info
        ]);

    // Debug-only: expose the webview to AI agents over MCP (drive/inspect/screenshot).
    #[cfg(feature = "debug-mcp")]
    let builder = builder.plugin(tauri_plugin_mcp::init_with_config(
        tauri_plugin_mcp::PluginConfig::new("BioOKF Studio".to_string())
            .start_socket_server(true)
            .socket_path(std::path::PathBuf::from("/tmp/biookf-tauri-mcp.sock")),
    ));

    // Debug-only: inject the tauri-plugin-mcp guest-js listeners so the JS/DOM tools
    // (execute_js, get_dom, get_page_state, ...) work. The frontend is no-bundler vanilla
    // JS, so the guest-js is pre-bundled into mcp_guest.js and eval'd on every page load.
    // Compiled out of release builds.
    #[cfg(feature = "debug-mcp")]
    let builder = builder.plugin(
        tauri::plugin::Builder::<tauri::Wry>::new("biookf-debug-guest")
            .on_page_load(|webview, _payload| {
                if let Err(e) = webview.eval(include_str!("mcp_guest.js")) {
                    eprintln!("[biookf-debug-guest] failed to inject MCP guest listeners: {e}");
                }
            })
            .build(),
    );

    builder
        .run(tauri::generate_context!())
        .expect("error while running BioOKF Studio");
}

#[cfg(test)]
mod tests {
    use super::replace_body;

    #[test]
    fn preserves_frontmatter_replaces_body() {
        let original = "---\ntype: Gene\nidentifier: BRAF\n---\n\n# BRAF\n\nOld body.\n";
        let out = replace_body(original, "# BRAF\n\nNew body with my additions.\n");
        assert_eq!(
            out,
            "---\ntype: Gene\nidentifier: BRAF\n---\n\n# BRAF\n\nNew body with my additions.\n"
        );
        // frontmatter is byte-identical
        assert!(out.starts_with("---\ntype: Gene\nidentifier: BRAF\n---\n"));
    }

    #[test]
    fn no_frontmatter_writes_whole_body() {
        let out = replace_body("# Note\n\njust prose\n", "# Note\n\nedited\n");
        assert_eq!(out, "# Note\n\nedited\n");
    }

    #[test]
    fn trims_trailing_blank_lines_to_single_newline() {
        let out = replace_body("---\nx: 1\n---\nbody\n", "new body\n\n\n");
        assert_eq!(out, "---\nx: 1\n---\n\nnew body\n");
    }

    // End-to-end exercise of the command (resolve + path guard + write) against a
    // throwaway bundle under a temp OKF_ROOT, so no real knowledge file is touched.
    #[test]
    fn save_node_body_writes_file_and_guards_traversal() {
        let tmp = std::env::temp_dir().join(format!("bokf-save-test-{}", std::process::id()));
        let base = tmp.join("studio/test-kb/mybase");
        std::fs::create_dir_all(base.join("knowledge/gene")).unwrap();
        let file = base.join("knowledge/gene/x.md");
        std::fs::write(&file, "---\ntype: Gene\nidentifier: X\n---\n\n# X\n\nold body\n").unwrap();

        std::env::set_var("OKF_ROOT", &tmp);
        super::save_node_body(
            "mybase".into(),
            "knowledge/gene/x.md".into(),
            "# X\n\nedited body with additions\n".into(),
        )
        .unwrap();
        let got = std::fs::read_to_string(&file).unwrap();
        assert_eq!(
            got,
            "---\ntype: Gene\nidentifier: X\n---\n\n# X\n\nedited body with additions\n"
        );

        // a path that tries to escape the bundle is rejected
        assert!(super::save_node_body("mybase".into(), "../../escape.md".into(), "x".into()).is_err());

        std::env::remove_var("OKF_ROOT");
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn replace_frontmatter_preserves_body_and_round_trips() {
        let original = "---\ntype: Gene\nidentifier: BRAF\n---\n\n# BRAF\n\nBody stays.\n";
        // unchanged frontmatter round-trips byte-for-byte
        assert_eq!(super::replace_frontmatter(original, "type: Gene\nidentifier: BRAF"), original);
        // edited frontmatter, body untouched
        let out = super::replace_frontmatter(original, "type: Gene\nidentifier: BRAF\nnote: mine");
        assert_eq!(out, "---\ntype: Gene\nidentifier: BRAF\nnote: mine\n---\n\n# BRAF\n\nBody stays.\n");
    }

    use super::{set_edge_note_in_fm, upsert_notes_section};

    #[test]
    fn upsert_notes_appends_when_absent() {
        let body = "# BRAF\n\nProse about BRAF.\n";
        let out = upsert_notes_section(body, "My first note.");
        assert_eq!(out, "# BRAF\n\nProse about BRAF.\n\n# Notes\n\nMy first note.\n");
    }

    #[test]
    fn upsert_notes_appends_to_empty_body() {
        let out = upsert_notes_section("", "Lonely note.");
        assert_eq!(out, "# Notes\n\nLonely note.\n");
    }

    #[test]
    fn upsert_notes_replaces_when_present() {
        let body = "# BRAF\n\nProse.\n\n# Notes\n\nOld note.\n";
        let out = upsert_notes_section(body, "Brand new note.");
        assert_eq!(out, "# BRAF\n\nProse.\n\n# Notes\n\nBrand new note.\n");
    }

    #[test]
    fn upsert_notes_replaces_and_keeps_trailing_section() {
        // A `# Notes` section in the middle, with another top-level heading after it.
        let body = "# BRAF\n\nIntro.\n\n# Notes\n\nOld.\n\n# References\n\nstuff\n";
        let out = upsert_notes_section(body, "Updated.");
        assert_eq!(out, "# BRAF\n\nIntro.\n\n# Notes\n\nUpdated.\n\n# References\n\nstuff\n");
    }

    #[test]
    fn upsert_notes_removes_when_blank() {
        let body = "# BRAF\n\nProse.\n\n# Notes\n\nGoodbye.\n";
        let out = upsert_notes_section(body, "   ");
        assert_eq!(out, "# BRAF\n\nProse.\n");
    }

    #[test]
    fn upsert_notes_removes_middle_section_keeping_rest() {
        let body = "# BRAF\n\nIntro.\n\n# Notes\n\nGone.\n\n# References\n\nkeep\n";
        let out = upsert_notes_section(body, "");
        assert_eq!(out, "# BRAF\n\nIntro.\n\n# References\n\nkeep\n");
    }

    #[test]
    fn upsert_notes_blank_when_no_section_is_noop() {
        let body = "# BRAF\n\nNothing to remove.\n";
        let out = upsert_notes_section(body, "");
        assert_eq!(out, "# BRAF\n\nNothing to remove.\n");
    }

    // Realistic indented YAML (line-continuation `\` would strip the indent, so
    // the newlines are written out explicitly).
    const EDGES_FM: &str = "type: Gene\nidentifier: BRAF\nedges:\n  - predicate: predisposes_to\n    object: Cancer drug resistance\n    knowledge_level: knowledge_assertion\n  - predicate: participates_in\n    object: RAS-RAF-MEK-ERK signaling pathway\n";

    #[test]
    fn set_edge_note_inserts_on_matching_edge_only() {
        let out = set_edge_note_in_fm(
            EDGES_FM,
            "predisposes_to",
            "Cancer drug resistance",
            "Strong clinical evidence.",
        )
        .unwrap();
        let expected = "type: Gene\nidentifier: BRAF\nedges:\n  - predicate: predisposes_to\n    object: Cancer drug resistance\n    note: \"Strong clinical evidence.\"\n    knowledge_level: knowledge_assertion\n  - predicate: participates_in\n    object: RAS-RAF-MEK-ERK signaling pathway";
        assert_eq!(out, expected);
        // The other edge is untouched (no note added).
        assert_eq!(out.matches("note:").count(), 1);
    }

    #[test]
    fn set_edge_note_replaces_existing_note() {
        let fm = "type: Gene\nidentifier: BRAF\nedges:\n  - predicate: predisposes_to\n    object: Cancer drug resistance\n    note: \"Old.\"\n    knowledge_level: knowledge_assertion\n  - predicate: participates_in\n    object: RAS-RAF-MEK-ERK signaling pathway\n";
        let out = set_edge_note_in_fm(fm, "predisposes_to", "Cancer drug resistance", "New value.")
            .unwrap();
        assert!(out.contains("    note: \"New value.\"\n"));
        assert!(!out.contains("Old."));
        assert_eq!(out.matches("note:").count(), 1);
    }

    #[test]
    fn set_edge_note_targets_second_edge() {
        let out = set_edge_note_in_fm(
            EDGES_FM,
            "participates_in",
            "RAS-RAF-MEK-ERK signaling pathway",
            "Canonical pathway.",
        )
        .unwrap();
        // First edge stays exactly as it was.
        assert!(out.contains(
            "  - predicate: predisposes_to\n    object: Cancer drug resistance\n    knowledge_level: knowledge_assertion\n"
        ));
        // Note landed right after the second edge's object line.
        assert!(out.ends_with(
            "  - predicate: participates_in\n    object: RAS-RAF-MEK-ERK signaling pathway\n    note: \"Canonical pathway.\""
        ));
    }

    #[test]
    fn set_edge_note_removes_on_blank() {
        let fm = "type: Gene\nidentifier: BRAF\nedges:\n  - predicate: predisposes_to\n    object: Cancer drug resistance\n    note: \"Remove me.\"\n    knowledge_level: knowledge_assertion\n  - predicate: participates_in\n    object: RAS-RAF-MEK-ERK signaling pathway\n";
        let out =
            set_edge_note_in_fm(fm, "predisposes_to", "Cancer drug resistance", "  ").unwrap();
        assert!(!out.contains("note:"));
        assert!(out.contains("    knowledge_level: knowledge_assertion"));
    }

    #[test]
    fn set_edge_note_errors_when_not_found() {
        let err = set_edge_note_in_fm(EDGES_FM, "predisposes_to", "Nonexistent object", "x")
            .unwrap_err();
        assert_eq!(err, "edge not found");
    }

    #[test]
    fn set_edge_note_escapes_quotes_and_newlines() {
        let out = set_edge_note_in_fm(
            EDGES_FM,
            "predisposes_to",
            "Cancer drug resistance",
            "Line \"one\"\nline two",
        )
        .unwrap();
        assert!(out.contains("    note: \"Line \\\"one\\\"\\nline two\"\n"));
    }
}
