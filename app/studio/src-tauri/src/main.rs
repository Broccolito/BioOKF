//! BioOKF Studio — the Tauri desktop app. A thin front-end: every command
//! delegates to `bokf-core`, so the GUI is a pure visualizer/dashboard over the
//! same backend the CLI and MCP server use.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use tauri::{AppHandle, Emitter, Manager};

/// Canonical config dir holding `registry.yaml` + `.active-kb`
/// (`~/.config/biookf-studio`), shared with the CLI and MCP server. This is what
/// keeps the GUI's KB list from scattering across whatever dir it was opened in.
fn config_root() -> PathBuf {
    bokf_core::config::ensure_config_dir().unwrap_or_else(|_| bokf_core::config::config_dir())
}

/// Registered bundles, the source of truth for discovery: every `Base` in
/// `<root>/registry.yaml` mapped to (registered-id, path), keeping only those
/// whose folder still exists and looks like a bundle. A KB whose folder was
/// deleted or moved simply isn't returned.
fn registered_bundles() -> Vec<(String, PathBuf)> {
    bokf_core::registry::list(&config_root())
        .into_iter()
        .map(|b| (b.id, PathBuf::from(b.path)))
        .filter(|(_, p)| p.join("knowledge").is_dir() || p.join("index.md").is_file())
        .collect()
}

fn resolve(id: &str) -> Option<PathBuf> {
    bokf_core::registry::resolve(&config_root(), id)
        .map(PathBuf::from)
        .filter(|p| p.is_dir())
}

/// JSON object for one bundle in the sidebar: `base_info(p)` with `"id"` set to
/// the REGISTERED kb-id (not the dir name) and `"path"` inserted.
fn base_entry(id: &str, p: &std::path::Path) -> Result<serde_json::Value, String> {
    let mut info = bokf_core::export::base_info(p).map_err(|e| e.to_string())?;
    if let Some(obj) = info.as_object_mut() {
        obj.insert("id".into(), serde_json::Value::String(id.to_string()));
        obj.insert(
            "path".into(),
            serde_json::Value::String(p.to_string_lossy().to_string()),
        );
    }
    Ok(info)
}

#[tauri::command]
fn list_bases() -> Result<serde_json::Value, String> {
    let mut out = Vec::new();
    for (id, p) in registered_bundles() {
        if let Ok(info) = base_entry(&id, &p) {
            out.push(info);
        }
    }
    Ok(serde_json::Value::Array(out))
}

/// Set the active KB pointer (`<root>/.active-kb`) to `id`.
#[tauri::command]
fn set_active_kb(id: String) -> Result<(), String> {
    bokf_core::active::set_active(&config_root(), Some(&id))
}

/// Read the active KB pointer, `None` when unset.
#[tauri::command]
fn get_active_kb() -> Result<Option<String>, String> {
    Ok(bokf_core::active::get_active(&config_root()))
}

/// Derive a kb-id from a folder name: lowercase, non-`[a-z0-9-]` → `-`, with
/// runs collapsed and leading/trailing `-` stripped (so it passes
/// `validate_kb_id`). Empty input yields `"base"`.
fn kb_id_from_dir_name(name: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;
    for ch in name.chars() {
        let c = ch.to_ascii_lowercase();
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            out.push(c);
            prev_dash = false;
        } else if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }
    let trimmed = out.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "base".to_string()
    } else {
        trimmed
    }
}

/// Validate a folder is a real BioOKF bundle and register it. Returns the same
/// shape as `list_bases` entries so the frontend can add it to the sidebar.
#[tauri::command]
fn add_base(path: String) -> Result<serde_json::Value, String> {
    let p = std::path::Path::new(&path)
        .canonicalize()
        .map_err(|e| format!("Not a valid BioOKF knowledge base: {e}"))?;
    if !p.is_dir() {
        return Err("Not a valid BioOKF knowledge base: not a directory".into());
    }
    if !(p.join("knowledge").is_dir() || p.join("index.md").is_file()) {
        return Err(
            "Not a valid BioOKF knowledge base: missing `knowledge/` directory or `index.md`"
                .into(),
        );
    }
    // It must parse as a bundle (lint errors are tolerated — only structure matters).
    bokf_core::open_bundle(&p).map_err(|e| format!("Not a valid BioOKF knowledge base: {e}"))?;

    let root = config_root();
    let dir_name = p
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let base_id = kb_id_from_dir_name(&dir_name);

    // If this exact path is already registered, return its existing entry.
    let already: Vec<bokf_core::registry::Base> = bokf_core::registry::list(&root);
    let path_str = p.to_string_lossy().to_string();
    if let Some(b) = already.iter().find(|b| b.path == path_str) {
        return base_entry(&b.id, &p);
    }

    // Pick an id that isn't taken: `base_id`, then `base_id-2`, `base_id-3`, …
    let taken: std::collections::HashSet<&str> = already.iter().map(|b| b.id.as_str()).collect();
    let mut id = base_id.clone();
    let mut n = 2;
    while taken.contains(id.as_str()) {
        id = format!("{base_id}-{n}");
        n += 1;
    }
    bokf_core::registry::validate_kb_id(&id)?;
    bokf_core::registry::register(&root, &id, &path_str)?;
    base_entry(&id, &p)
}

/// Remove a knowledge base from the global Studio/CLI/MCP registry. This is
/// intentionally non-destructive: the bundle folder stays on disk.
#[tauri::command]
fn remove_base(id: String) -> Result<(), String> {
    let root = config_root();
    let registered = bokf_core::registry::list(&root);
    if registered.iter().any(|b| b.id == id) {
        bokf_core::registry::unregister(&root, &id)?;
    }
    if bokf_core::active::get_active(&root).as_deref() == Some(id.as_str()) {
        bokf_core::active::set_active(&root, None)?;
    }
    Ok(())
}

#[tauri::command]
fn get_bundle(id: String) -> Result<serde_json::Value, String> {
    let path = resolve(&id).ok_or_else(|| format!("unknown bundle: {id}"))?;
    bokf_core::export::studio_graph_doc(&path, None).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_node_file(base: String, path: String) -> Result<serde_json::Value, String> {
    let rel = clean_bundle_rel(&path)?;
    if !is_knowledge_markdown(&rel) {
        return Err("Studio node details are limited to knowledge/*.md files".into());
    }
    let full = safe_existing_bundle_path(&base, &path)?;
    let text = std::fs::read_to_string(&full).map_err(|e| e.to_string())?;
    let node = bokf_core::parse_node(&text, &rel).map_err(|e| e.to_string())?;
    serde_json::to_value(node).map_err(|e| e.to_string())
}

#[tauri::command]
fn lint_bundle(id: String) -> Result<serde_json::Value, String> {
    let path = resolve(&id).ok_or_else(|| format!("unknown bundle: {id}"))?;
    let bundle = bokf_core::open_bundle(&path).map_err(|e| e.to_string())?;
    let report = bokf_core::lint_fast(&bundle);
    serde_json::to_value(&report).map_err(|e| e.to_string())
}

#[tauri::command]
fn search_bundle(
    id: String,
    query: String,
    limit: Option<usize>,
) -> Result<serde_json::Value, String> {
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
        || source_id.starts_with('.')
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

const ROOT_TEXT_FILES: [&str; 4] = ["index.md", "log.md", "SCHEMA.md", "README.md"];

fn clean_bundle_rel(rel: &str) -> Result<PathBuf, String> {
    let r = std::path::Path::new(rel);
    if r.is_absolute()
        || r.components().any(|c| {
            matches!(
                c,
                std::path::Component::ParentDir | std::path::Component::Prefix(_)
            )
        })
    {
        return Err("invalid path".into());
    }
    if r.components().any(|c| match c {
        std::path::Component::Normal(s) => s.to_string_lossy().starts_with('.'),
        _ => false,
    }) {
        return Err("hidden files and directories are not addressable from Studio".into());
    }
    Ok(r.to_path_buf())
}

fn is_root_text_file(rel: &std::path::Path) -> bool {
    rel.components().count() == 1
        && rel
            .to_str()
            .map(|s| ROOT_TEXT_FILES.contains(&s))
            .unwrap_or(false)
}

fn is_knowledge_markdown(rel: &std::path::Path) -> bool {
    rel.starts_with("knowledge") && rel.extension().and_then(|e| e.to_str()) == Some("md")
}

fn is_readable_bundle_content(rel: &std::path::Path) -> bool {
    is_knowledge_markdown(rel) || is_root_text_file(rel)
}

/// Resolve an existing file inside the bundle for `base`, after the caller has
/// checked which BioOKF content areas are allowed for the operation.
fn safe_existing_bundle_path(base: &str, rel: &str) -> Result<PathBuf, String> {
    let root = resolve(base).ok_or_else(|| format!("unknown bundle: {base}"))?;
    let r = clean_bundle_rel(rel)?;
    let root_c = root.canonicalize().map_err(|e| e.to_string())?;
    let full_c = root.join(&r).canonicalize().map_err(|e| e.to_string())?;
    if !full_c.starts_with(&root_c) {
        return Err("path escapes bundle".into());
    }
    Ok(full_c)
}

fn safe_read_bundle_path(base: &str, rel: &str) -> Result<PathBuf, String> {
    let r = clean_bundle_rel(rel)?;
    if !is_readable_bundle_content(&r) {
        return Err("path must be under knowledge/ or a BioOKF root text file".into());
    }
    safe_existing_bundle_path(base, rel)
}

fn safe_write_node_path(base: &str, rel: &str) -> Result<PathBuf, String> {
    let r = clean_bundle_rel(rel)?;
    if !is_knowledge_markdown(&r) {
        return Err("Studio edits are limited to existing knowledge/*.md files".into());
    }
    safe_existing_bundle_path(base, rel)
}

/// Persist a user edit to a node's document body, preserving its frontmatter.
#[tauri::command]
fn save_node_body(base: String, path: String, body: String) -> Result<(), String> {
    let full = safe_write_node_path(&base, &path)?;
    let existing = std::fs::read_to_string(&full).map_err(|e| e.to_string())?;
    std::fs::write(&full, replace_body(&existing, &body)).map_err(|e| e.to_string())?;
    Ok(())
}

/// Read a text file inside a bundle. Studio deliberately excludes `raw/` here:
/// raw papers, PDFs, and extracted images can be large and are not part of graph rendering.
#[tauri::command]
fn read_bundle_file(base: String, path: String) -> Result<String, String> {
    let full = safe_read_bundle_path(&base, &path)?;
    std::fs::read_to_string(&full).map_err(|e| e.to_string())
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
    let full = safe_write_node_path(&base, &path)?;
    let existing = std::fs::read_to_string(&full).map_err(|e| e.to_string())?;
    std::fs::write(&full, replace_frontmatter(&existing, &frontmatter))
        .map_err(|e| e.to_string())?;
    append_log_entry(
        &base,
        &date,
        &format!("- Edited frontmatter of `{}`", label),
    )?;
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
        return if trimmed.is_empty() {
            String::new()
        } else {
            format!("{trimmed}\n")
        };
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
    let full = safe_write_node_path(&base, &path)?;
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
            if notes.trim().is_empty() {
                "Cleared"
            } else {
                "Updated"
            },
            label
        ),
    )?;
    Ok(())
}

/// Persist an entire node/edge `.md` file in a single write, taking the full-file
/// editor's text verbatim (frontmatter + body). The path is limited to an existing
/// `knowledge/*.md` file, which is correct here since the editor only edits nodes.
/// Logs the change.
#[tauri::command]
fn save_node_file(
    base: String,
    path: String,
    content: String,
    label: String,
    date: String,
) -> Result<(), String> {
    let full = safe_write_node_path(&base, &path)?;
    std::fs::write(&full, content).map_err(|e| e.to_string())?;
    append_log_entry(&base, &date, &format!("- Edited `{}`", label))?;
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
    let full = safe_write_node_path(&base, &path)?;
    let existing = std::fs::read_to_string(&full).map_err(|e| e.to_string())?;
    let fm = frontmatter_yaml(&existing).ok_or_else(|| "no frontmatter".to_string())?;
    let edited_fm = set_edge_note_in_fm(&fm, &predicate, &object, &note)?;
    std::fs::write(&full, replace_frontmatter(&existing, &edited_fm)).map_err(|e| e.to_string())?;
    append_log_entry(
        &base,
        &date,
        &format!(
            "- {} note on edge `{}`",
            if note.trim().is_empty() {
                "Cleared"
            } else {
                "Set"
            },
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
    let full = safe_read_bundle_path(&base, &path)?;
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
        .openpty(PtySize {
            rows: rows.max(1),
            cols: cols.max(1),
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| e.to_string())?;
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
    let mut cmd = CommandBuilder::new(shell);
    cmd.env("TERM", "xterm-256color");
    // Make the Studio-bundled `bokf`/`bokf-mcp` available in the integrated
    // terminal even before the user installs the CLI system-wide.
    if let Some(bin) = bundled_bin_dir(&app) {
        let existing = std::env::var("PATH").unwrap_or_default();
        cmd.env("PATH", format!("{}:{}", bin.display(), existing));
    }
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
                    let _ = eapp.emit(
                        "term-output",
                        serde_json::json!({ "id": eid, "data": data }),
                    );
                }
            }
        }
    });
    sessions().lock().unwrap().insert(
        id.clone(),
        PtySession {
            master: pair.master,
            writer,
            child,
        },
    );
    Ok(id)
}

/// Forward user keystrokes to the PTY.
#[tauri::command]
fn term_write(id: String, data: String) -> Result<(), String> {
    let mut s = sessions().lock().unwrap();
    let sess = s.get_mut(&id).ok_or("no such terminal")?;
    sess.writer
        .write_all(data.as_bytes())
        .map_err(|e| e.to_string())?;
    sess.writer.flush().map_err(|e| e.to_string())
}

/// Resize the PTY to match the front-end grid.
#[tauri::command]
fn term_resize(id: String, rows: u16, cols: u16) -> Result<(), String> {
    let s = sessions().lock().unwrap();
    let sess = s.get(&id).ok_or("no such terminal")?;
    sess.master
        .resize(PtySize {
            rows: rows.max(1),
            cols: cols.max(1),
            pixel_width: 0,
            pixel_height: 0,
        })
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

// --- bundled CLI: detect + install ------------------------------------------

fn bokf_exe_name() -> &'static str {
    if cfg!(windows) {
        "bokf.exe"
    } else {
        "bokf"
    }
}

fn bokf_mcp_exe_name() -> &'static str {
    if cfg!(windows) {
        "bokf-mcp.exe"
    } else {
        "bokf-mcp"
    }
}

/// Directory inside the app bundle that holds the shipped `bokf`/`bokf-mcp`.
/// In a packaged `.app` this is `Contents/Resources/bin`; under `cargo run` it
/// falls back to the workspace target dir next to the studio exe.
fn bundled_bin_dir(app: &AppHandle) -> Option<PathBuf> {
    if let Ok(res) = app.path().resource_dir() {
        let p = res.join("bin");
        if p.join(bokf_exe_name()).exists() {
            return Some(p);
        }
    }
    // Dev fallback: binaries sit next to the studio exe in target/<profile>.
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            if dir.join(bokf_exe_name()).exists() {
                return Some(dir.to_path_buf());
            }
        }
    }
    None
}

/// The install path of a bundled BioOKF tool on the user's PATH (or the standard
/// install location), if any.
fn tool_on_path(exe_name: &str) -> Option<String> {
    let mut candidates = vec![
        Path::new("/usr/local/bin").join(exe_name),
        Path::new("/opt/homebrew/bin").join(exe_name),
    ];
    if let Some(home) = std::env::var_os("HOME") {
        candidates.push(PathBuf::from(home).join(".cargo/bin").join(exe_name));
    }
    for cand in candidates {
        if cand.exists() {
            return Some(cand.display().to_string());
        }
    }
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let cand = dir.join(exe_name);
        if cand.exists() {
            return Some(cand.display().to_string());
        }
    }
    None
}

fn bokf_on_path() -> Option<String> {
    tool_on_path(bokf_exe_name())
}

fn bokf_mcp_on_path() -> Option<String> {
    tool_on_path(bokf_mcp_exe_name())
}

fn bokf_version(bin: &std::path::Path) -> Option<String> {
    let out = std::process::Command::new(bin)
        .arg("--version")
        .output()
        .ok()?;
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

fn current_studio_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

fn sh_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

fn applescript_string(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

fn version_parts(v: &str) -> Vec<u64> {
    v.trim()
        .trim_start_matches('v')
        .split(|c: char| !c.is_ascii_digit())
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse::<u64>().ok())
        .collect()
}

fn version_newer(latest: &str, current: &str) -> bool {
    let mut a = version_parts(latest);
    let mut b = version_parts(current);
    let n = a.len().max(b.len());
    a.resize(n, 0);
    b.resize(n, 0);
    a > b
}

#[derive(Debug, serde::Deserialize)]
struct GhRelease {
    tag_name: String,
    html_url: Option<String>,
    assets: Vec<GhAsset>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct GhAsset {
    name: String,
    browser_download_url: String,
}

fn fetch_latest_release() -> Result<GhRelease, String> {
    if let Ok(raw) = std::env::var("BIOOKF_UPDATE_RELEASE_JSON") {
        return serde_json::from_str(&raw).map_err(|e| format!("bad BIOOKF_UPDATE_RELEASE_JSON: {e}"));
    }
    let url = std::env::var("BIOOKF_UPDATE_API_URL")
        .unwrap_or_else(|_| "https://api.github.com/repos/Broccolito/BioOKF/releases/latest".to_string());
    let out = std::process::Command::new("curl")
        .arg("-fsSL")
        .arg("--connect-timeout")
        .arg("5")
        .arg("--max-time")
        .arg("15")
        .arg("-H")
        .arg("Accept: application/vnd.github+json")
        .arg("-H")
        .arg(format!("User-Agent: BioOKF-Studio/{}", current_studio_version()))
        .arg(&url)
        .output()
        .map_err(|e| format!("failed to run curl for release check: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "release check failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    serde_json::from_slice(&out.stdout).map_err(|e| format!("bad release response: {e}"))
}

fn current_platform_tokens() -> (&'static str, &'static [&'static str]) {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => ("macos-arm64", &["aarch64", "arm64", "macos-arm64"]),
        ("macos", "x86_64") => ("macos-x64", &["x86_64", "x64", "macos-x64"]),
        ("linux", "x86_64") => ("linux-x64", &["linux-x64", "x86_64"]),
        ("windows", "x86_64") => ("windows-x64", &["windows-x64", "x64"]),
        _ => ("unsupported", &[]),
    }
}

fn asset_for_current_platform(release: &GhRelease) -> Option<GhAsset> {
    let (platform, tokens) = current_platform_tokens();
    let is_archive = |name: &str| {
        let n = name.to_ascii_lowercase();
        n.ends_with(".dmg") || n.ends_with(".tar.gz") || n.ends_with(".tgz") || n.ends_with(".zip")
    };
    release
        .assets
        .iter()
        .find(|a| {
            let n = a.name.to_ascii_lowercase();
            is_archive(&n) && tokens.iter().any(|t| n.contains(t))
        })
        .or_else(|| {
            release.assets.iter().find(|a| {
                let n = a.name.to_ascii_lowercase();
                is_archive(&n) && n.contains(platform)
            })
        })
        .or_else(|| {
            if std::env::consts::OS == "macos" {
                release.assets.iter().find(|a| a.name.to_ascii_lowercase().ends_with(".dmg"))
            } else {
                None
            }
        })
        .cloned()
}

fn install_supported_for_asset(asset_name: &str) -> bool {
    if std::env::consts::OS != "macos" {
        return false;
    }
    let n = asset_name.to_ascii_lowercase();
    n.ends_with(".dmg") || n.ends_with(".tar.gz") || n.ends_with(".tgz")
}

fn app_bundle_path() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    for ancestor in exe.ancestors() {
        if ancestor.extension().and_then(|s| s.to_str()) == Some("app") {
            return Some(ancestor.to_path_buf());
        }
    }
    None
}

fn download_asset(asset: &GhAsset) -> Result<PathBuf, String> {
    let mut dir = std::env::temp_dir();
    dir.push(format!("biookf-update-{}", std::process::id()));
    std::fs::create_dir_all(&dir).map_err(|e| format!("failed to create update temp dir: {e}"))?;
    let safe_name = asset.name.replace('/', "_");
    let dest = dir.join(safe_name);
    let out = std::process::Command::new("curl")
        .arg("-fL")
        .arg("--connect-timeout")
        .arg("10")
        .arg("--max-time")
        .arg("300")
        .arg("-H")
        .arg(format!("User-Agent: BioOKF-Studio/{}", current_studio_version()))
        .arg("-o")
        .arg(&dest)
        .arg(&asset.browser_download_url)
        .output()
        .map_err(|e| format!("failed to run curl for update download: {e}"))?;
    if out.status.success() {
        Ok(dest)
    } else {
        Err(format!(
            "update download failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ))
    }
}

fn write_macos_relauncher(asset: &Path, dest_app: &Path) -> Result<PathBuf, String> {
    let mut script = std::env::temp_dir();
    script.push(format!("biookf-relaunch-{}.sh", std::process::id()));
    let body = format!(
        r#"#!/bin/bash
set -euo pipefail
PID={pid}
ASSET={asset}
DEST={dest}
APP_NAME="BioOKF Studio.app"
LOG="$HOME/Library/Logs/BioOKF Studio Updater.log"
mkdir -p "$(dirname "$LOG")"
exec >> "$LOG" 2>&1
echo "$(date): starting BioOKF update from $ASSET"
while kill -0 "$PID" 2>/dev/null; do sleep 0.2; done
WORK="$(mktemp -d /tmp/biookf-update.XXXXXX)"
cleanup() {{
  if [ -n "${{MOUNT:-}}" ]; then hdiutil detach "$MOUNT" -quiet || true; fi
  rm -rf "$WORK"
}}
trap cleanup EXIT
SRC=""
case "$ASSET" in
  *.dmg)
    MOUNT="$WORK/mount"
    mkdir -p "$MOUNT"
    hdiutil attach "$ASSET" -nobrowse -quiet -mountpoint "$MOUNT"
    SRC="$(find "$MOUNT" -maxdepth 3 -name "$APP_NAME" -type d -print -quit)"
    ;;
  *.tar.gz|*.tgz)
    mkdir -p "$WORK/extract"
    tar -xzf "$ASSET" -C "$WORK/extract"
    SRC="$(find "$WORK/extract" -maxdepth 5 -name "$APP_NAME" -type d -print -quit)"
    ;;
esac
if [ -z "$SRC" ] || [ ! -d "$SRC" ]; then
  echo "BioOKF update failed: BioOKF Studio.app not found in downloaded asset"
  open "$DEST" || true
  exit 1
fi
codesign --verify --deep --strict "$SRC"
spctl --assess --type execute "$SRC"
install_cmd="rm -rf $(printf '%q' "$DEST") && ditto $(printf '%q' "$SRC") $(printf '%q' "$DEST") && mkdir -p /usr/local/bin"
for tool in bokf bokf-mcp; do
  bundled="$DEST/Contents/Resources/bin/$tool"
  target="/usr/local/bin/$tool"
  install_cmd="$install_cmd && if [ -x $(printf '%q' "$bundled") ]; then cp $(printf '%q' "$bundled") $(printf '%q' "$target") && chmod 755 $(printf '%q' "$target"); fi"
done
escaped="${{install_cmd//\\/\\\\}}"
escaped="${{escaped//\"/\\\"}}"
osascript -e "do shell script \"$escaped\" with administrator privileges"
echo "$(date): BioOKF update installed; reopening $DEST"
open "$DEST"
"#,
        pid = std::process::id(),
        asset = sh_quote(&asset.to_string_lossy()),
        dest = sh_quote(&dest_app.to_string_lossy()),
    );
    std::fs::write(&script, body).map_err(|e| format!("failed to write relauncher: {e}"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&script)
            .map_err(|e| e.to_string())?
            .permissions();
        perms.set_mode(0o700);
        std::fs::set_permissions(&script, perms).map_err(|e| e.to_string())?;
    }
    Ok(script)
}

/// Report whether the `bokf` CLI is installed on PATH, plus version info. The
/// front-end uses `installed == false` to decide whether to show the install
/// popup 5 seconds after launch.
#[tauri::command]
fn cli_status(app: AppHandle) -> serde_json::Value {
    let installed_path = bokf_on_path();
    let installed_mcp_path = bokf_mcp_on_path();
    let bundled = bundled_bin_dir(&app).map(|d| d.join(bokf_exe_name()));
    let bundled_mcp = bundled_bin_dir(&app)
        .map(|d| d.join(bokf_mcp_exe_name()).to_string_lossy().to_string());
    let bundled_version = bundled.as_deref().and_then(bokf_version);
    let installed_version = installed_path
        .as_deref()
        .map(std::path::Path::new)
        .and_then(bokf_version);
    serde_json::json!({
        "installed": installed_path.is_some(),
        "path": installed_path,
        "mcpInstalled": installed_mcp_path.is_some(),
        "mcpPath": installed_mcp_path,
        "version": installed_version,
        "bundledVersion": bundled_version,
        "bundledMcpPath": bundled_mcp,
    })
}

/// Copy the bundled `bokf` and `bokf-mcp` to `/usr/local/bin` with one admin prompt.
#[tauri::command]
fn install_cli(app: AppHandle) -> Result<String, String> {
    let dir = bundled_bin_dir(&app).ok_or("bundled bokf binary not found")?;
    let src_cli = dir.join(bokf_exe_name());
    let src_mcp = dir.join(bokf_mcp_exe_name());
    if !src_cli.exists() {
        return Err(format!("bundled bokf not found at {}", src_cli.display()));
    }
    if !src_mcp.exists() {
        return Err(format!("bundled bokf-mcp not found at {}", src_mcp.display()));
    }
    let dest_cli = "/usr/local/bin/bokf";
    let dest_mcp = "/usr/local/bin/bokf-mcp";
    // One admin prompt: ensure /usr/local/bin exists, copy both tools, mark executable.
    let shell = format!(
        "mkdir -p {} && cp {} {} && cp {} {} && chmod 755 {} {}",
        sh_quote("/usr/local/bin"),
        sh_quote(&src_cli.to_string_lossy()),
        sh_quote(dest_cli),
        sh_quote(&src_mcp.to_string_lossy()),
        sh_quote(dest_mcp),
        sh_quote(dest_cli),
        sh_quote(dest_mcp)
    );
    let script = format!(
        "do shell script {} with administrator privileges",
        applescript_string(&shell)
    );
    let out = std::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| format!("failed to launch osascript: {e}"))?;
    if out.status.success() {
        Ok(format!("{dest_cli} and {dest_mcp}"))
    } else {
        let err = String::from_utf8_lossy(&out.stderr);
        if err.contains("-128") || err.to_lowercase().contains("cancel") {
            Err("install cancelled".to_string())
        } else {
            Err(format!("install failed: {}", err.trim()))
        }
    }
}

#[tauri::command]
fn update_status() -> serde_json::Value {
    let current = current_studio_version();
    match fetch_latest_release() {
        Ok(release) => {
            let asset = asset_for_current_platform(&release);
            let latest = release.tag_name.trim_start_matches('v').to_string();
            let update_available = version_newer(&release.tag_name, current);
            serde_json::json!({
                "ok": true,
                "currentVersion": current,
                "latestVersion": latest,
                "latestTag": release.tag_name,
                "releaseUrl": release.html_url,
                "updateAvailable": update_available,
                "platform": current_platform_tokens().0,
                "assetName": asset.as_ref().map(|a| a.name.clone()),
                "assetUrl": asset.as_ref().map(|a| a.browser_download_url.clone()),
                "installSupported": asset.as_ref().map(|a| install_supported_for_asset(&a.name)).unwrap_or(false),
            })
        }
        Err(e) => serde_json::json!({
            "ok": false,
            "currentVersion": current,
            "updateAvailable": false,
            "error": e,
        }),
    }
}

#[tauri::command]
fn install_update(app: AppHandle) -> Result<String, String> {
    let current = current_studio_version();
    let release = fetch_latest_release()?;
    if !version_newer(&release.tag_name, current) {
        return Ok("BioOKF Studio is already up to date.".to_string());
    }
    let asset = asset_for_current_platform(&release)
        .ok_or_else(|| format!("no release asset found for {}", current_platform_tokens().0))?;
    if !install_supported_for_asset(&asset.name) {
        return Err(format!(
            "automatic install is not supported for this asset yet: {}",
            asset.name
        ));
    }
    let downloaded = download_asset(&asset)?;
    let dest_app = app_bundle_path().unwrap_or_else(|| PathBuf::from("/Applications/BioOKF Studio.app"));
    let relauncher = write_macos_relauncher(&downloaded, &dest_app)?;
    std::process::Command::new("/bin/bash")
        .arg(&relauncher)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("failed to start updater: {e}"))?;
    let app_for_exit = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(250));
        app_for_exit.exit(0);
    });
    Ok(format!(
        "Installing BioOKF {} from {}; Studio will restart.",
        release.tag_name, asset.name
    ))
}

fn main() {
    let builder = tauri::Builder::default()
        // Native folder picker for the "+ New base" dialog (a normal feature).
        .plugin(tauri_plugin_dialog::init())
        .setup(|_app| {
            // Native macOS vibrancy: the whole window becomes translucent frosted
            // glass (preserving the rounded window corners), so the canvas shows the
            // blurred desktop and the app's own surfaces layer on top.
            #[cfg(target_os = "macos")]
            {
                use window_vibrancy::{
                    apply_vibrancy, NSVisualEffectMaterial, NSVisualEffectState,
                };
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
            set_active_kb,
            get_active_kb,
            add_base,
            remove_base,
            get_bundle,
            get_node_file,
            lint_bundle,
            search_bundle,
            save_node_body,
            read_bundle_file,
            save_node_frontmatter,
            save_node_notes,
            save_node_file,
            save_edge_note,
            reveal_in_finder,
            term_open,
            term_write,
            term_resize,
            term_close,
            source_info,
            cli_status,
            install_cli,
            update_status,
            install_update
        ])
        // Native menu so macOS actually delivers Cmd+K: WKWebView swallows Cmd-key
        // combos as key equivalents before they reach the webview's JS keydown, so the
        // shortcut needs a real accelerator. "Go ▸ Search ⌘K" emits a `menu-search`
        // event the frontend focuses the search box on.
        .menu(|app| {
            let menu = tauri::menu::Menu::default(app)?;
            let search = tauri::menu::MenuItemBuilder::with_id("search", "Search")
                .accelerator("CmdOrCtrl+K")
                .build(app)?;
            let go = tauri::menu::SubmenuBuilder::new(app, "Go")
                .item(&search)
                .build()?;
            menu.append(&go)?;
            Ok(menu)
        })
        .on_menu_event(|app, event| {
            if event.id().0.as_str() == "search" {
                let _ = app.emit("menu-search", ());
            }
        });

    // Live-control plane: compiled in by default, but the socket server and the
    // guest-inject plugin are only ATTACHED when BIOOKF_STUDIO_CONTROL is set (the
    // MCP `bokf_studio_open` sets it when launching). A normal build/run leaves the
    // socket closed and injects nothing.
    #[cfg(feature = "control")]
    let builder = if std::env::var_os("BIOOKF_STUDIO_CONTROL").is_some() {
        // Expose the webview to AI agents over the socket (drive/inspect/screenshot).
        let builder = builder.plugin(tauri_plugin_mcp::init_with_config(
            tauri_plugin_mcp::PluginConfig::new("BioOKF Studio".to_string())
                .start_socket_server(true)
                .socket_path(std::path::PathBuf::from("/tmp/biookf-tauri-mcp.sock")),
        ));

        // Inject the tauri-plugin-mcp guest listeners so the webview answers the
        // JS/DOM tools (execute_js, get_dom, get_page_map, manage_storage, selector
        // clicks/typing, wait_for). Our no-bundler vanilla frontend can't `import`
        // the npm guest bindings, so we eval a prebuilt IIFE on every page load.
        builder.plugin(
            tauri::plugin::Builder::<tauri::Wry>::new("biookf-control-guest")
                .on_page_load(|webview, _payload| {
                    // on_page_load fires more than once per navigation; guard so the
                    // guest registers its execute-js listener exactly once (otherwise
                    // every execute_js evals N times — once per duplicate registration).
                    let js = concat!(
                        "if(!window.__bokfGuestReady){window.__bokfGuestReady=1;\n",
                        include_str!("mcp_guest.js"),
                        "\n}"
                    );
                    if let Err(e) = webview.eval(js) {
                        eprintln!(
                            "[biookf-control-guest] failed to inject MCP guest listeners: {e}"
                        );
                    }
                })
                .build(),
        )
    } else {
        builder
    };

    builder
        .run(tauri::generate_context!())
        .expect("error while running BioOKF Studio");
}

#[cfg(test)]
mod tests {
    use super::replace_body;

    #[test]
    fn update_version_compare_handles_v_prefixes() {
        assert!(super::version_newer("v0.2.3", "0.2.2"));
        assert!(super::version_newer("0.10.0", "0.9.9"));
        assert!(!super::version_newer("v0.2.2", "0.2.2"));
        assert!(!super::version_newer("v0.2.1", "0.2.2"));
    }

    #[test]
    fn update_asset_selection_finds_current_macos_dmg() {
        let release = super::GhRelease {
            tag_name: "v0.2.3".into(),
            html_url: None,
            assets: vec![super::GhAsset {
                name: "BioOKF.Studio_0.2.3_aarch64.dmg".into(),
                browser_download_url: "https://example.invalid/BioOKF.dmg".into(),
            }],
        };
        let asset = super::asset_for_current_platform(&release);
        if std::env::consts::OS == "macos" && std::env::consts::ARCH == "aarch64" {
            assert_eq!(asset.unwrap().name, "BioOKF.Studio_0.2.3_aarch64.dmg");
        }
    }

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
    // throwaway bundle under a temp BIOOKF_CONFIG_DIR, so no real knowledge file is touched.
    #[test]
    fn save_node_body_writes_file_and_guards_traversal() {
        let tmp = std::env::temp_dir().join(format!("bokf-save-test-{}", std::process::id()));
        let cfg = tmp.join("cfg");
        let base = tmp.join("mybase");
        std::fs::create_dir_all(base.join("knowledge/gene")).unwrap();
        std::fs::create_dir_all(&cfg).unwrap();
        let file = base.join("knowledge/gene/x.md");
        std::fs::write(
            &file,
            "---\ntype: Gene\nidentifier: X\n---\n\n# X\n\nold body\n",
        )
        .unwrap();

        std::env::set_var("BIOOKF_CONFIG_DIR", &cfg);
        // Registry is the source of truth now: register the temp bundle so
        // `resolve("mybase")` finds it.
        bokf_core::registry::register(&cfg, "mybase", &base.to_string_lossy()).unwrap();
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
        assert!(
            super::save_node_body("mybase".into(), "../../escape.md".into(), "x".into()).is_err()
        );
        std::fs::create_dir_all(base.join("raw/s")).unwrap();
        std::fs::write(base.join("raw/s/source.md"), "# raw\n").unwrap();
        std::fs::create_dir_all(base.join(".git")).unwrap();
        std::fs::write(base.join(".git/config"), "[core]\n").unwrap();
        assert!(super::safe_read_bundle_path("mybase", "raw/s/source.md").is_err());
        assert!(super::safe_read_bundle_path("mybase", ".git/config").is_err());
        assert!(super::safe_write_node_path("mybase", "raw/s/source.md").is_err());
        std::env::remove_var("BIOOKF_CONFIG_DIR");

        std::env::remove_var("OKF_ROOT");
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn replace_frontmatter_preserves_body_and_round_trips() {
        let original = "---\ntype: Gene\nidentifier: BRAF\n---\n\n# BRAF\n\nBody stays.\n";
        // unchanged frontmatter round-trips byte-for-byte
        assert_eq!(
            super::replace_frontmatter(original, "type: Gene\nidentifier: BRAF"),
            original
        );
        // edited frontmatter, body untouched
        let out = super::replace_frontmatter(original, "type: Gene\nidentifier: BRAF\nnote: mine");
        assert_eq!(
            out,
            "---\ntype: Gene\nidentifier: BRAF\nnote: mine\n---\n\n# BRAF\n\nBody stays.\n"
        );
    }

    use super::{set_edge_note_in_fm, upsert_notes_section};

    #[test]
    fn upsert_notes_appends_when_absent() {
        let body = "# BRAF\n\nProse about BRAF.\n";
        let out = upsert_notes_section(body, "My first note.");
        assert_eq!(
            out,
            "# BRAF\n\nProse about BRAF.\n\n# Notes\n\nMy first note.\n"
        );
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
        assert_eq!(
            out,
            "# BRAF\n\nIntro.\n\n# Notes\n\nUpdated.\n\n# References\n\nstuff\n"
        );
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
        let err =
            set_edge_note_in_fm(EDGES_FM, "predisposes_to", "Nonexistent object", "x").unwrap_err();
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
