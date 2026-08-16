//! BioOKF Studio — the Tauri desktop app. A thin front-end: every command
//! delegates to `bokf-core`, so the GUI is a pure visualizer/dashboard over the
//! same backend the CLI and MCP server use.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
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
fn get_export_bundle(id: String) -> Result<serde_json::Value, String> {
    let path = resolve(&id).ok_or_else(|| format!("unknown bundle: {id}"))?;
    bokf_core::export::studio_bundle_doc(&path, None).map_err(|e| e.to_string())
}

#[tauri::command]
async fn network_metrics(
    id: String,
    exclude_provenance: bool,
) -> Result<serde_json::Value, String> {
    let path = resolve(&id).ok_or_else(|| format!("unknown bundle: {id}"))?;
    tauri::async_runtime::spawn_blocking(move || {
        let bundle = bokf_core::open_bundle(&path).map_err(|e| e.to_string())?;
        let report = bokf_core::network_metrics::analyze(
            &bundle,
            bokf_core::network_metrics::NetworkOptions { exclude_provenance },
        )?;
        serde_json::to_value(report).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("network metrics task failed: {e}"))?
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
    // Path-traversal containment for `source_id` lives in one place: the core
    // `bokf_core::export::source_info`, which canonicalizes the resolved path and
    // confines it under `raw/` (AUDIT C8). We deliberately do NOT add a second,
    // string-based guard here — two divergent guards is the inconsistency AUDIT
    // M10 flagged (a string check can accept/reject differently than canonicalize).
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

/// Open a registered bundle's own folder in the OS file manager.
///
/// Distinct from `reveal_in_finder`, which selects a FILE inside the bundle and
/// is deliberately restricted to `knowledge/` documents and root text files.
/// The sidebar's "Open folder" wants the bundle root itself, so it resolves the
/// registered id rather than accepting a caller-supplied path — there is no path
/// to escape from.
#[tauri::command]
fn open_base_folder(base: String) -> Result<(), String> {
    let root = resolve(&base).ok_or_else(|| format!("unknown bundle: {base}"))?;
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    if !root.is_dir() {
        return Err(format!("not a directory: {}", root.display()));
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&root)
            .status()
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&root)
            .status()
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&root)
            .status()
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Err("opening a folder is not supported on this platform".into())
    }
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

fn normalize_export_html_path(path: &str) -> Result<PathBuf, String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("export path is empty".into());
    }
    let mut out = PathBuf::from(trimmed);
    if out.file_name().is_none() {
        return Err("export path must include a file name".into());
    }
    match out.extension().and_then(|e| e.to_str()) {
        Some(ext) if ext.eq_ignore_ascii_case("html") || ext.eq_ignore_ascii_case("htm") => {}
        None => {
            out.set_extension("html");
        }
        _ => return Err("export file must use .html or .htm".into()),
    }
    Ok(out)
}

#[tauri::command]
fn write_export_html(path: String, html: String) -> Result<String, String> {
    let out = normalize_export_html_path(&path)?;
    if html.len() > 128 * 1024 * 1024 {
        return Err("export is unexpectedly large".into());
    }
    let parent = out
        .parent()
        .ok_or_else(|| "export path must include a parent directory".to_string())?;
    if !parent.is_dir() {
        return Err("export directory does not exist".into());
    }
    std::fs::write(&out, html).map_err(|e| e.to_string())?;
    Ok(out.to_string_lossy().to_string())
}

#[tauri::command]
fn write_network_metrics_json(path: String, content: String) -> Result<String, String> {
    if content.len() > 64 * 1024 * 1024 {
        return Err("metrics export is unexpectedly large".into());
    }
    let mut out = PathBuf::from(path);
    if out.extension().and_then(|value| value.to_str()) != Some("json") {
        out.set_extension("json");
    }
    let parent = out
        .parent()
        .ok_or_else(|| "export path must include a parent directory".to_string())?;
    if !parent.is_dir() {
        return Err("export directory does not exist".into());
    }
    std::fs::write(&out, content).map_err(|e| e.to_string())?;
    Ok(out.to_string_lossy().to_string())
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
/// Lock the session map, recovering from a poisoned mutex instead of panicking
/// (AUDIT M9). A panic in one terminal's reader thread must not permanently
/// wedge the whole terminal feature; the map holds independent `PtySession`
/// handles, so a poisoned guard's contents remain safe to use.
fn sessions_lock() -> std::sync::MutexGuard<'static, HashMap<String, PtySession>> {
    sessions().lock().unwrap_or_else(|e| e.into_inner())
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
    sessions_lock().insert(
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
    let mut s = sessions_lock();
    let sess = s.get_mut(&id).ok_or("no such terminal")?;
    sess.writer
        .write_all(data.as_bytes())
        .map_err(|e| e.to_string())?;
    sess.writer.flush().map_err(|e| e.to_string())
}

/// Resize the PTY to match the front-end grid.
#[tauri::command]
fn term_resize(id: String, rows: u16, cols: u16) -> Result<(), String> {
    let s = sessions_lock();
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
    if let Some(mut sess) = sessions_lock().remove(&id) {
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

// --- Paperclip → BioOKF generator (machine-local Studio integration) -------

static PAPERCLIP_GENERATION_RUNNING: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct PaperclipGenerateRequest {
    query: String,
    sources: Vec<String>,
    limit: u8,
    kb_name: String,
    provider: String,
    model: Option<String>,
    year_min: Option<u16>,
    year_max: Option<u16>,
    since: Option<String>,
}

fn paperclip_harness_binary() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("PAPERCLIP2BIOOKF_BIN") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Some(path);
        }
    }
    tool_on_path(if cfg!(windows) {
        "pc-biookf.exe"
    } else {
        "pc-biookf"
    })
    .map(PathBuf::from)
}

fn paperclip_workspace() -> PathBuf {
    std::env::var_os("PAPERCLIP2BIOOKF_WORKSPACE")
        .map(PathBuf::from)
        .unwrap_or_else(|| config_root().join("paperclip2biookf"))
}

fn paperclip_child_path() -> String {
    let mut paths = Vec::<PathBuf>::new();
    if let Some(home) = std::env::var_os("HOME") {
        let home = PathBuf::from(home);
        paths.push(home.join(".local/bin"));
        paths.push(home.join(".cargo/bin"));
    }
    paths.extend(
        [
            "/opt/homebrew/bin",
            "/usr/local/bin",
            "/usr/bin",
            "/bin",
            "/usr/sbin",
            "/sbin",
        ]
        .into_iter()
        .map(PathBuf::from),
    );
    if let Some(inherited) = std::env::var_os("PATH") {
        paths.extend(std::env::split_paths(&inherited));
    }
    std::env::join_paths(paths)
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}

fn paperclip_model_catalog() -> serde_json::Value {
    let mut codex = Vec::new();
    if let Some(home) = std::env::var_os("HOME") {
        let cache = PathBuf::from(home).join(".codex/models_cache.json");
        if let Ok(raw) = std::fs::read_to_string(cache) {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) {
                if let Some(models) = value.get("models").and_then(|v| v.as_array()) {
                    for model in models {
                        if model.get("visibility").and_then(|v| v.as_str()) != Some("list") {
                            continue;
                        }
                        if let Some(id) = model.get("slug").and_then(|v| v.as_str()) {
                            codex.push(serde_json::json!({
                                "id": id,
                                "label": model.get("display_name").and_then(|v| v.as_str()).unwrap_or(id)
                            }));
                        }
                    }
                }
            }
        }
    }
    serde_json::json!({
        "codex": codex,
        "claude": [
            {"id": "sonnet", "label": "Claude Sonnet (latest)"},
            {"id": "opus", "label": "Claude Opus (latest)"},
            {"id": "fable", "label": "Claude Fable (latest)"}
        ]
    })
}

#[tauri::command]
fn paperclip_generator_status() -> serde_json::Value {
    let binary = paperclip_harness_binary();
    let doctor = binary.as_ref().and_then(|path| {
        let output = std::process::Command::new(path)
            .arg("doctor")
            .env("PATH", paperclip_child_path())
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        serde_json::from_slice::<serde_json::Value>(&output.stdout).ok()
    });
    serde_json::json!({
        "installed": binary.is_some(),
        "binary": binary.map(|p| p.to_string_lossy().to_string()),
        "workspace": paperclip_workspace().to_string_lossy(),
        "doctor": doctor,
        "models": paperclip_model_catalog(),
        "running": PAPERCLIP_GENERATION_RUNNING.load(Ordering::SeqCst),
        "standard": "BioOKF v0.5"
    })
}

fn validate_paperclip_request(request: &PaperclipGenerateRequest) -> Result<(), String> {
    const SOURCES: [&str; 13] = [
        "pmc",
        "biorxiv",
        "medrxiv",
        "arxiv",
        "abstracts",
        "fda/us",
        "fda/eu",
        "fda/jp",
        "trials/us",
        "trials/eu",
        "trials/jp",
        "trials/cn",
        "trials",
    ];
    if request.query.trim().is_empty() {
        return Err("Enter a Paperclip search query".into());
    }
    if request.kb_name.trim().is_empty() {
        return Err("Enter a knowledge-base name".into());
    }
    if request.sources.is_empty()
        || request
            .sources
            .iter()
            .any(|s| !SOURCES.contains(&s.as_str()))
    {
        return Err("Select at least one supported Paperclip source".into());
    }
    if !(1..=25).contains(&request.limit) {
        return Err("Papers per source must be between 1 and 25".into());
    }
    if !matches!(request.provider.as_str(), "codex" | "claude") {
        return Err("Choose Codex or Claude".into());
    }
    if let (Some(start), Some(end)) = (request.year_min, request.year_max) {
        if start > end {
            return Err("Start year must not exceed end year".into());
        }
    }
    Ok(())
}

fn paperclip_generate_args(request: &PaperclipGenerateRequest, workspace: &Path) -> Vec<String> {
    let mut args = vec![
        "--workspace".into(),
        workspace.to_string_lossy().to_string(),
        "run".into(),
        "--query".into(),
        request.query.trim().into(),
        "--limit".into(),
        request.limit.to_string(),
        "--kb-name".into(),
        request.kb_name.trim().into(),
        "--agent".into(),
        request.provider.clone(),
        "--register".into(),
    ];
    for source in &request.sources {
        args.push("--source".into());
        args.push(source.clone());
    }
    if let Some(model) = request
        .model
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        args.extend(["--model".into(), model.into()]);
    }
    if let Some(year) = request.year_min {
        args.extend(["--year-min".into(), year.to_string()]);
    }
    if let Some(year) = request.year_max {
        args.extend(["--year-max".into(), year.to_string()]);
    }
    if let Some(since) = request
        .since
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        args.extend(["--since".into(), since.into()]);
    }
    args
}

fn run_paperclip_generation(
    app: AppHandle,
    request: PaperclipGenerateRequest,
) -> Result<serde_json::Value, String> {
    validate_paperclip_request(&request)?;
    let binary = paperclip_harness_binary().ok_or_else(|| {
        "paperclip2bioOKF was not found; install `pc-biookf` or set PAPERCLIP2BIOOKF_BIN"
            .to_string()
    })?;
    let workspace = paperclip_workspace();
    std::fs::create_dir_all(&workspace)
        .map_err(|e| format!("cannot create Paperclip workspace: {e}"))?;
    let args = paperclip_generate_args(&request, &workspace);
    let mut command = std::process::Command::new(&binary);
    command.args(&args).env("PATH", paperclip_child_path());
    let mut child = subscription_only_environment(&mut command)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to launch {}: {e}", binary.display()))?;
    let stderr = child
        .stderr
        .take()
        .ok_or("failed to capture generator progress")?;
    let mut progress = Vec::new();
    for line in BufReader::new(stderr).lines() {
        let line = line.map_err(|e| format!("failed reading generator progress: {e}"))?;
        let _ = app.emit(
            "paperclip2biookf-progress",
            serde_json::json!({"message": line}),
        );
        progress.push(line);
    }
    let mut stdout = String::new();
    child
        .stdout
        .take()
        .ok_or("failed to capture generator output")?
        .read_to_string(&mut stdout)
        .map_err(|e| format!("failed reading generator output: {e}"))?;
    let status = child
        .wait()
        .map_err(|e| format!("generator wait failed: {e}"))?;
    if !status.success() {
        return Err(progress
            .last()
            .cloned()
            .unwrap_or_else(|| format!("paperclip2bioOKF exited with status {status}")));
    }
    serde_json::from_str(&stdout).map_err(|e| format!("generator returned invalid JSON: {e}"))
}

#[tauri::command]
async fn paperclip_generate_base(
    app: AppHandle,
    request: PaperclipGenerateRequest,
) -> Result<serde_json::Value, String> {
    require_connection("paperclip")?;
    require_subscription(&request.provider)?;
    if PAPERCLIP_GENERATION_RUNNING
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Err("A Paperclip generation is already running".into());
    }
    let joined =
        tauri::async_runtime::spawn_blocking(move || run_paperclip_generation(app, request)).await;
    PAPERCLIP_GENERATION_RUNNING.store(false, Ordering::SeqCst);
    joined.map_err(|e| format!("Paperclip generation task failed: {e}"))?
}

// --- Local subscription connections + agent workflows --------------------

static BIOOKF_AGENT_WORKFLOW_RUNNING: AtomicBool = AtomicBool::new(false);
static BIOOKF_AGENT_TEMP_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
struct LocalConnections {
    codex: bool,
    claude: bool,
    paperclip: bool,
}

impl Default for LocalConnections {
    fn default() -> Self {
        Self {
            codex: true,
            claude: true,
            paperclip: true,
        }
    }
}

fn connections_path() -> PathBuf {
    config_root().join("connections.json")
}

fn load_connections() -> LocalConnections {
    std::fs::read_to_string(connections_path())
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

fn write_connections(value: &LocalConnections) -> Result<(), String> {
    let path = connections_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let raw = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    std::fs::write(path, raw + "\n").map_err(|e| e.to_string())
}

fn require_connection(name: &str) -> Result<(), String> {
    let value = load_connections();
    let enabled = match name {
        "codex" => value.codex,
        "claude" => value.claude,
        "paperclip" => value.paperclip,
        _ => false,
    };
    if enabled {
        Ok(())
    } else {
        Err(format!("{name} is disabled in BioOKF Studio Connections"))
    }
}

fn require_subscription(provider: &str) -> Result<(), String> {
    require_connection(provider)?;
    let status = paperclip_generator_status();
    let agent = status
        .get("doctor")
        .and_then(|value| value.get("agents"))
        .and_then(|value| value.get(provider))
        .ok_or_else(|| format!("could not inspect {provider} authentication"))?;
    let authenticated = agent.get("authenticated").and_then(|value| value.as_bool()) == Some(true);
    let method = agent
        .get("auth_method")
        .and_then(|value| value.as_str())
        .unwrap_or("");
    let subscription = match provider {
        "codex" => method == "ChatGPT subscription",
        "claude" => {
            method == "claude.ai"
                && agent
                    .get("subscription_type")
                    .and_then(|value| value.as_str())
                    .is_some_and(|value| !value.is_empty())
        }
        _ => false,
    };
    if authenticated && subscription {
        Ok(())
    } else {
        Err(format!(
            "{provider} is not authenticated with a supported local subscription; sign in with the native CLI, then refresh Connections"
        ))
    }
}

fn subscription_only_environment(
    command: &mut std::process::Command,
) -> &mut std::process::Command {
    for name in [
        "OPENAI_API_KEY",
        "CODEX_API_KEY",
        "ANTHROPIC_API_KEY",
        "CLAUDE_CODE_USE_BEDROCK",
        "CLAUDE_CODE_USE_VERTEX",
        "CLAUDE_CODE_USE_FOUNDRY",
        "AWS_BEARER_TOKEN_BEDROCK",
        "ANTHROPIC_VERTEX_PROJECT_ID",
    ] {
        command.env_remove(name);
    }
    command
}

#[tauri::command]
fn local_connections_status() -> serde_json::Value {
    let configured = load_connections();
    let generator = paperclip_generator_status();
    serde_json::json!({
        "configured": configured,
        "detected": generator.get("doctor").cloned().unwrap_or(serde_json::Value::Null),
        "models": paperclip_model_catalog(),
        "settingsPath": connections_path().to_string_lossy(),
    })
}

#[tauri::command]
fn save_local_connections(connections: LocalConnections) -> Result<serde_json::Value, String> {
    write_connections(&connections)?;
    Ok(local_connections_status())
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct KnowledgeChatRequest {
    base_id: String,
    provider: String,
    model: Option<String>,
    question: String,
    #[serde(default)]
    history: Vec<ChatMessage>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct DoctorKnowledgeRequest {
    base_id: String,
    provider: String,
    model: Option<String>,
    instruction: String,
    #[serde(default)]
    history: Vec<ChatMessage>,
}

fn validate_agent(provider: &str, model: Option<&str>) -> Result<(), String> {
    if !matches!(provider, "codex" | "claude") {
        return Err("Choose Codex or Claude".into());
    }
    require_subscription(provider)?;
    if model.is_some_and(|value| value.len() > 160) {
        return Err("Model identifier is too long".into());
    }
    Ok(())
}

fn chat_context(base_id: &str, question: &str) -> Result<(PathBuf, String, usize), String> {
    let path = resolve(base_id).ok_or_else(|| format!("unknown bundle: {base_id}"))?;
    let bundle = bokf_core::open_bundle(&path).map_err(|e| e.to_string())?;
    let hits = bokf_core::SearchIndex::build(&bundle).search(question, 14);
    let mut wanted = std::collections::HashSet::new();
    for hit in &hits {
        wanted.insert(hit.identifier.clone());
    }
    if wanted.is_empty() {
        wanted.extend(
            bundle
                .nodes
                .iter()
                .take(10)
                .map(|node| node.identifier.clone()),
        );
    }
    let first_hop: Vec<String> = wanted.iter().cloned().collect();
    for identifier in first_hop {
        if let Some(index) = bundle.by_identifier.get(&identifier) {
            for edge in bundle.nodes[*index].edges.iter().take(16) {
                if wanted.len() >= 36 {
                    break;
                }
                wanted.insert(edge.object.clone());
                if let Some(source) = &edge.primary_source {
                    wanted.insert(source.clone());
                }
            }
        }
    }
    let selected: Vec<&bokf_core::Node> = bundle
        .nodes
        .iter()
        .filter(|node| wanted.contains(&node.identifier))
        .take(36)
        .collect();
    let value = serde_json::json!({
        "knowledge_base": base_id,
        "retrieval_query": question,
        "retrieval_hits": hits,
        "nodes": selected,
    });
    let mut raw = serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?;
    if raw.len() > 300_000 {
        raw.truncate(300_000);
    }
    Ok((path, raw, selected.len()))
}

fn subscription_prompt(
    provider: &str,
    model: Option<&str>,
    cwd: &Path,
    prompt: &str,
) -> Result<String, String> {
    validate_agent(provider, model)?;
    if provider == "codex" {
        let id = BIOOKF_AGENT_TEMP_ID.fetch_add(1, Ordering::SeqCst);
        let output =
            std::env::temp_dir().join(format!("biookf-chat-{}-{id}.txt", std::process::id()));
        let mut command = std::process::Command::new("codex");
        command.args([
            "exec",
            "--ephemeral",
            "--skip-git-repo-check",
            "--ignore-user-config",
            "--ignore-rules",
            "--sandbox",
            "read-only",
            "--cd",
        ]);
        command
            .arg(cwd)
            .arg("--output-last-message")
            .arg(&output)
            .args(["--color", "never"]);
        if let Some(model) = model.filter(|value| !value.trim().is_empty()) {
            command.arg("--model").arg(model);
        }
        command.arg(prompt).env("PATH", paperclip_child_path());
        let completed = subscription_only_environment(&mut command)
            .output()
            .map_err(|e| format!("failed to launch Codex: {e}"))?;
        if !completed.status.success() {
            return Err(format!(
                "Codex failed: {}",
                String::from_utf8_lossy(&completed.stderr).trim()
            ));
        }
        let answer = std::fs::read_to_string(&output)
            .map_err(|e| format!("Codex returned no answer: {e}"))?;
        let _ = std::fs::remove_file(output);
        return Ok(answer.trim().to_string());
    }

    let mut command = std::process::Command::new("claude");
    command.args([
        "--print",
        "--no-session-persistence",
        "--safe-mode",
        "--permission-mode",
        "dontAsk",
        "--tools",
        "",
        "--output-format",
        "text",
    ]);
    if let Some(model) = model.filter(|value| !value.trim().is_empty()) {
        command.arg("--model").arg(model);
    }
    command.current_dir(cwd).env("PATH", paperclip_child_path());
    let mut child = subscription_only_environment(&mut command)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to launch Claude: {e}"))?;
    child
        .stdin
        .take()
        .ok_or("failed to open Claude input")?
        .write_all(prompt.as_bytes())
        .map_err(|e| format!("failed to send prompt to Claude: {e}"))?;
    let completed = child.wait_with_output().map_err(|e| e.to_string())?;
    if !completed.status.success() {
        return Err(format!(
            "Claude failed: {}",
            String::from_utf8_lossy(&completed.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&completed.stdout)
        .trim()
        .to_string())
}

fn run_knowledge_chat(request: KnowledgeChatRequest) -> Result<serde_json::Value, String> {
    if request.question.trim().is_empty() {
        return Err("Enter a question for the selected knowledge base".into());
    }
    if request.question.len() > 12_000 {
        return Err("Question is too long".into());
    }
    validate_agent(&request.provider, request.model.as_deref())?;
    let (path, context, context_nodes) = chat_context(&request.base_id, &request.question)?;
    let history = request
        .history
        .iter()
        .rev()
        .take(8)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .map(|message| format!("{}: {}", message.role, message.content))
        .collect::<Vec<_>>()
        .join("\n");
    let prompt = format!(
        "You are BioOKF Studio's knowledge-base analyst. Answer only from the supplied retrieved BioOKF nodes and their edges. Distinguish explicit evidence, statistical association, prediction and absence of evidence. Preserve negative findings and contradictions. Cite supporting node identifiers in square brackets and, when present, include evidence_url values. If the retrieved context is insufficient, say exactly what is missing. Do not use external knowledge or access the network.\n\nCONVERSATION\n{history}\n\nQUESTION\n{}\n\nRETRIEVED BIOOKF CONTEXT\n{context}",
        request.question.trim()
    );
    let answer = subscription_prompt(&request.provider, request.model.as_deref(), &path, &prompt)?;
    Ok(serde_json::json!({
        "answer": answer,
        "baseId": request.base_id,
        "provider": request.provider,
        "model": request.model.unwrap_or_else(|| "subscription default".into()),
        "contextNodes": context_nodes,
    }))
}

#[tauri::command]
async fn chat_with_knowledge_base(
    request: KnowledgeChatRequest,
) -> Result<serde_json::Value, String> {
    tauri::async_runtime::spawn_blocking(move || run_knowledge_chat(request))
        .await
        .map_err(|e| format!("knowledge chat task failed: {e}"))?
}

#[tauri::command]
async fn doctor_knowledge_base(
    app: AppHandle,
    request: DoctorKnowledgeRequest,
) -> Result<serde_json::Value, String> {
    let instruction = request.instruction.trim();
    if instruction.is_empty() {
        return Err("Tell Doctor what to inspect or revise".into());
    }
    if instruction.len() > 12_000 {
        return Err("Doctor instruction is too long".into());
    }
    validate_agent(&request.provider, request.model.as_deref())?;
    let bundle =
        resolve(&request.base_id).ok_or_else(|| format!("unknown bundle: {}", request.base_id))?;
    if BIOOKF_AGENT_WORKFLOW_RUNNING
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Err("Another BioOKF agent workflow is already running".into());
    }
    let history = request
        .history
        .iter()
        .rev()
        .take(6)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .map(|message| format!("{}: {}", message.role, message.content))
        .collect::<Vec<_>>()
        .join("\n");
    let contextual_instruction = if history.is_empty() {
        instruction.to_string()
    } else {
        format!(
            "Prior Doctor conversation (context only):\n{history}\n\nCurrent revision request:\n{instruction}"
        )
    };
    let mut args = vec![
        "doctor".into(),
        "--bundle".into(),
        bundle.to_string_lossy().to_string(),
        "--workspace".into(),
        paperclip_workspace().to_string_lossy().to_string(),
        "--instruction".into(),
        contextual_instruction,
        "--provider".into(),
        request.provider,
    ];
    if let Some(model) = request.model.filter(|value| !value.trim().is_empty()) {
        args.extend(["--model".into(), model]);
    }
    let joined = tauri::async_runtime::spawn_blocking(move || run_agent_workflow(app, args)).await;
    BIOOKF_AGENT_WORKFLOW_RUNNING.store(false, Ordering::SeqCst);
    joined.map_err(|e| format!("Doctor workflow failed: {e}"))?
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct LocalKnowledgeRequest {
    source_path: String,
    kb_name: String,
    provider: String,
    model: Option<String>,
    max_files: Option<u8>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct MergeKnowledgeRequest {
    base_ids: Vec<String>,
    kb_name: String,
    provider: String,
    model: Option<String>,
}

fn workflow_helper(app: &AppHandle) -> Result<PathBuf, String> {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/agent_workflows.py");
    if source.is_file() {
        return Ok(source);
    }
    let resource = app
        .path()
        .resource_dir()
        .map_err(|e| format!("cannot resolve Studio resources: {e}"))?;
    for candidate in [
        resource.join("resources/agent_workflows.py"),
        resource.join("agent_workflows.py"),
    ] {
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err("BioOKF local workflow helper is missing from this build".into())
}

fn workflow_python() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("PAPERCLIP2BIOOKF_PYTHON") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Some(path);
        }
    }
    if let Some(harness) = paperclip_harness_binary() {
        if let Ok(content) = std::fs::read_to_string(harness) {
            if let Some(interpreter) = content
                .lines()
                .next()
                .and_then(|line| line.strip_prefix("#!"))
                .map(str::trim)
                .filter(|value| !value.is_empty())
            {
                let path = PathBuf::from(interpreter);
                if path.is_file() {
                    return Some(path);
                }
            }
        }
    }
    tool_on_path(if cfg!(windows) {
        "python.exe"
    } else {
        "python3"
    })
    .map(PathBuf::from)
}

fn run_agent_workflow(app: AppHandle, args: Vec<String>) -> Result<serde_json::Value, String> {
    let helper = workflow_helper(&app)?;
    let python = workflow_python().ok_or_else(|| {
        "subscription workflow Python is missing; set PAPERCLIP2BIOOKF_PYTHON or install pc-biookf"
            .to_string()
    })?;
    let mut command = std::process::Command::new(&python);
    command
        .arg(helper)
        .args(args)
        .env("PATH", paperclip_child_path());
    let mut child = subscription_only_environment(&mut command)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to launch BioOKF agent workflow: {e}"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or("failed to capture workflow progress")?;
    let mut progress_lines = Vec::new();
    for line in BufReader::new(stderr).lines() {
        let line = line.map_err(|e| format!("failed reading workflow progress: {e}"))?;
        let _ = app.emit(
            "biookf-agent-progress",
            serde_json::json!({"message": line}),
        );
        progress_lines.push(line);
    }
    let mut stdout = String::new();
    child
        .stdout
        .take()
        .ok_or("failed to capture workflow result")?
        .read_to_string(&mut stdout)
        .map_err(|e| e.to_string())?;
    let status = child.wait().map_err(|e| e.to_string())?;
    let value: serde_json::Value = serde_json::from_str(stdout.trim())
        .map_err(|e| format!("workflow returned invalid JSON: {e}"))?;
    if !status.success() || value.get("ok").and_then(|v| v.as_bool()) != Some(true) {
        return Err(value
            .get("error")
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .or_else(|| progress_lines.last().cloned())
            .unwrap_or_else(|| format!("workflow exited with {status}")));
    }
    Ok(value)
}

fn validate_kb_name(value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err("Enter a knowledge-base name".into())
    } else if value.len() > 160 {
        Err("Knowledge-base name is too long".into())
    } else {
        Ok(())
    }
}

#[tauri::command]
async fn create_local_knowledge_base(
    app: AppHandle,
    request: LocalKnowledgeRequest,
) -> Result<serde_json::Value, String> {
    validate_kb_name(&request.kb_name)?;
    validate_agent(&request.provider, request.model.as_deref())?;
    let source = PathBuf::from(&request.source_path)
        .canonicalize()
        .map_err(|e| format!("invalid local papers folder: {e}"))?;
    if !source.is_dir() {
        return Err("Select a local papers folder".into());
    }
    if BIOOKF_AGENT_WORKFLOW_RUNNING
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Err("Another BioOKF agent workflow is already running".into());
    }
    let args = vec![
        "local".into(),
        "--source".into(),
        source.to_string_lossy().to_string(),
        "--workspace".into(),
        paperclip_workspace().to_string_lossy().to_string(),
        "--name".into(),
        request.kb_name,
        "--provider".into(),
        request.provider,
        "--max-files".into(),
        request.max_files.unwrap_or(25).clamp(1, 50).to_string(),
    ];
    let mut args = args;
    if let Some(model) = request.model.filter(|value| !value.trim().is_empty()) {
        args.extend(["--model".into(), model]);
    }
    let joined = tauri::async_runtime::spawn_blocking(move || run_agent_workflow(app, args)).await;
    BIOOKF_AGENT_WORKFLOW_RUNNING.store(false, Ordering::SeqCst);
    joined.map_err(|e| format!("local knowledge workflow failed: {e}"))?
}

#[tauri::command]
async fn merge_knowledge_bases(
    app: AppHandle,
    request: MergeKnowledgeRequest,
) -> Result<serde_json::Value, String> {
    validate_kb_name(&request.kb_name)?;
    validate_agent(&request.provider, request.model.as_deref())?;
    if request.base_ids.len() < 2 {
        return Err("Select at least two knowledge bases to merge".into());
    }
    let mut paths = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for id in &request.base_ids {
        if !seen.insert(id) {
            continue;
        }
        paths.push(resolve(id).ok_or_else(|| format!("unknown bundle: {id}"))?);
    }
    if paths.len() < 2 {
        return Err("Select at least two distinct knowledge bases".into());
    }
    if BIOOKF_AGENT_WORKFLOW_RUNNING
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Err("Another BioOKF agent workflow is already running".into());
    }
    let mut args = vec![
        "merge".into(),
        "--workspace".into(),
        paperclip_workspace().to_string_lossy().to_string(),
        "--name".into(),
        request.kb_name,
        "--provider".into(),
        request.provider,
    ];
    if let Some(model) = request.model.filter(|value| !value.trim().is_empty()) {
        args.extend(["--model".into(), model]);
    }
    for path in paths {
        args.extend(["--input".into(), path.to_string_lossy().to_string()]);
    }
    let joined = tauri::async_runtime::spawn_blocking(move || run_agent_workflow(app, args)).await;
    BIOOKF_AGENT_WORKFLOW_RUNNING.store(false, Ordering::SeqCst);
    joined.map_err(|e| format!("merge workflow failed: {e}"))?
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
        return serde_json::from_str(&raw)
            .map_err(|e| format!("bad BIOOKF_UPDATE_RELEASE_JSON: {e}"));
    }
    let url = std::env::var("BIOOKF_UPDATE_API_URL").unwrap_or_else(|_| {
        "https://api.github.com/repos/Broccolito/BioOKF/releases/latest".to_string()
    });
    let out = std::process::Command::new("curl")
        .arg("-fsSL")
        .arg("--connect-timeout")
        .arg("5")
        .arg("--max-time")
        .arg("15")
        .arg("-H")
        .arg("Accept: application/vnd.github+json")
        .arg("-H")
        .arg(format!(
            "User-Agent: BioOKF-Studio/{}",
            current_studio_version()
        ))
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

/// Install preference for an asset, lowest wins; `None` means "not an archive".
///
/// A macOS release carries both a signed `.dmg` and the CI `.tar.gz`, which is
/// built with `--no-sign`. The updater replaces `/Applications/BioOKF Studio.app`
/// with administrator rights, so it must always reach for the signed disk image;
/// picking whichever asset the GitHub API happened to list first is how the
/// updater ended up trying to install an unsigned bundle.
fn asset_kind_rank(name: &str) -> Option<u8> {
    let n = name.to_ascii_lowercase();
    if n.ends_with(".dmg") {
        Some(0)
    } else if n.ends_with(".tar.gz") || n.ends_with(".tgz") {
        Some(1)
    } else if n.ends_with(".zip") {
        Some(2)
    } else {
        None
    }
}

fn asset_for_current_platform(release: &GhRelease) -> Option<GhAsset> {
    let (platform, tokens) = current_platform_tokens();
    // `min_by_key` keeps the first asset of the best rank, so equally-ranked
    // assets still fall back to release order.
    let best = |matches: &dyn Fn(&str) -> bool| -> Option<&GhAsset> {
        release
            .assets
            .iter()
            .filter(|a| matches(&a.name.to_ascii_lowercase()))
            .filter_map(|a| asset_kind_rank(&a.name).map(|rank| (rank, a)))
            .min_by_key(|(rank, _)| *rank)
            .map(|(_, a)| a)
    };
    best(&|n: &str| tokens.iter().any(|t| n.contains(t)))
        .or_else(|| best(&|n: &str| n.contains(platform)))
        .or_else(|| {
            if std::env::consts::OS == "macos" {
                release
                    .assets
                    .iter()
                    .find(|a| a.name.to_ascii_lowercase().ends_with(".dmg"))
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

const STUDIO_APP_NAME: &str = "BioOKF Studio.app";

/// Scratch dir holding the downloaded asset and the unpacked app. The relauncher
/// removes it once the install is done.
fn update_staging_root() -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!("biookf-update-{}", std::process::id()));
    dir
}

fn run_checked<I, S>(program: &str, args: I, what: &str) -> Result<(), String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let out = std::process::Command::new(program)
        .args(args)
        .output()
        .map_err(|e| format!("failed to run {program} while {what}: {e}"))?;
    if out.status.success() {
        return Ok(());
    }
    let err = String::from_utf8_lossy(&out.stderr);
    match err.trim() {
        "" => Err(format!("{what} failed")),
        msg => Err(format!("{what} failed: {msg}")),
    }
}

/// Locate `BioOKF Studio.app` under `root`. Symlinks are skipped on purpose: a
/// mounted `.dmg` usually contains an `/Applications` alias, and following it
/// would "find" the app that is already installed.
fn find_app_bundle(root: &Path, depth: usize) -> Option<PathBuf> {
    if depth == 0 {
        return None;
    }
    let mut subdirs = Vec::new();
    for entry in std::fs::read_dir(root).ok()?.flatten() {
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        if kind.is_symlink() || !kind.is_dir() {
            continue;
        }
        let path = entry.path();
        if path.file_name().and_then(|s| s.to_str()) == Some(STUDIO_APP_NAME) {
            return Some(path);
        }
        subdirs.push(path);
    }
    subdirs
        .into_iter()
        .find_map(|d| find_app_bundle(&d, depth - 1))
}

/// Unpack the asset and copy the app out to a stable staging path. Copying
/// matters for `.dmg`: the mount is detached here, so the relauncher never has
/// to keep a disk image attached across the app's exit.
fn stage_app_from_asset(asset: &Path, root: &Path) -> Result<PathBuf, String> {
    let staging = root.join("staged");
    let _ = std::fs::remove_dir_all(&staging);
    std::fs::create_dir_all(&staging).map_err(|e| format!("failed to create staging dir: {e}"))?;
    let staged_app = staging.join(STUDIO_APP_NAME);
    let missing = || format!("{STUDIO_APP_NAME} not found in {}", asset.display());

    let name = asset
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_ascii_lowercase();

    if name.ends_with(".dmg") {
        let mount = root.join("mount");
        let _ = std::fs::remove_dir_all(&mount);
        std::fs::create_dir_all(&mount).map_err(|e| format!("failed to create mount dir: {e}"))?;
        run_checked(
            "hdiutil",
            [
                std::ffi::OsStr::new("attach"),
                asset.as_os_str(),
                std::ffi::OsStr::new("-nobrowse"),
                std::ffi::OsStr::new("-readonly"),
                std::ffi::OsStr::new("-quiet"),
                std::ffi::OsStr::new("-mountpoint"),
                mount.as_os_str(),
            ],
            "mounting the downloaded disk image",
        )?;
        let copied = match find_app_bundle(&mount, 3) {
            Some(src) => run_checked(
                "ditto",
                [src.as_os_str(), staged_app.as_os_str()],
                "copying the app out of the disk image",
            ),
            None => Err(missing()),
        };
        // Detach before reporting the copy result, or a failed copy leaks the mount.
        let _ = std::process::Command::new("hdiutil")
            .arg("detach")
            .arg(&mount)
            .arg("-quiet")
            .output();
        copied?;
    } else if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
        let extract = root.join("extract");
        let _ = std::fs::remove_dir_all(&extract);
        std::fs::create_dir_all(&extract)
            .map_err(|e| format!("failed to create extract dir: {e}"))?;
        run_checked(
            "tar",
            [
                std::ffi::OsStr::new("-xzf"),
                asset.as_os_str(),
                std::ffi::OsStr::new("-C"),
                extract.as_os_str(),
            ],
            "extracting the downloaded archive",
        )?;
        let src = find_app_bundle(&extract, 5).ok_or_else(missing)?;
        run_checked(
            "ditto",
            [src.as_os_str(), staged_app.as_os_str()],
            "staging the extracted app",
        )?;
    } else {
        return Err(format!("unsupported update asset: {}", asset.display()));
    }

    if !staged_app.is_dir() {
        return Err(missing());
    }
    Ok(staged_app)
}

fn codesign_verify(app: &Path) -> Result<(), String> {
    let out = std::process::Command::new("codesign")
        .arg("--verify")
        .arg("--deep")
        .arg("--strict")
        .arg("--")
        .arg(app)
        .output()
        .map_err(|e| format!("failed to run codesign: {e}"))?;
    if out.status.success() {
        return Ok(());
    }
    Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
}

/// `codesign -dv` prints its bundle summary on stderr; ad-hoc signatures report
/// `TeamIdentifier=not set`, which is not an identity we can pin against.
fn parse_team_identifier(codesign_output: &str) -> Option<String> {
    codesign_output
        .lines()
        .find_map(|l| l.trim().strip_prefix("TeamIdentifier="))
        .map(str::trim)
        .filter(|t| !t.is_empty() && *t != "not set")
        .map(str::to_string)
}

fn codesign_team_id(app: &Path) -> Option<String> {
    let out = std::process::Command::new("codesign")
        .arg("-dv")
        .arg("--verbose=2")
        .arg("--")
        .arg(app)
        .output()
        .ok()?;
    parse_team_identifier(&String::from_utf8_lossy(&out.stderr))
}

/// Gate the update *before* Studio quits. The staged bundle is about to be
/// `ditto`ed into `/Applications` as root, so it has to carry a valid signature
/// from the same Developer ID team as the app that is running.
///
/// `spctl --assess` is deliberately not used: it succeeds unconditionally on
/// machines where Gatekeeper assessments are disabled and rejects merely
/// un-notarized builds elsewhere, so it decides the update's fate based on a
/// setting that has nothing to do with the download.
fn verify_staged_app(staged: &Path, current_app: &Path) -> Result<(), String> {
    codesign_verify(staged).map_err(|e| {
        format!("the downloaded update is not correctly code-signed, so it was not installed: {e}")
    })?;
    check_team_identity(
        codesign_team_id(current_app).as_deref(),
        codesign_team_id(staged).as_deref(),
    )
}

/// An unsigned or ad-hoc-signed running app has no identity to pin against, so
/// nothing is required of the update. Once we do have one, the update must match.
fn check_team_identity(expected: Option<&str>, found: Option<&str>) -> Result<(), String> {
    let Some(expected) = expected else {
        return Ok(());
    };
    match found {
        Some(found) if found == expected => Ok(()),
        Some(found) => Err(format!(
            "the downloaded update is signed by team {found}, but this app is signed by {expected}; refusing to install it"
        )),
        None => Err(
            "the downloaded update carries no Developer ID team identifier; refusing to install it"
                .to_string(),
        ),
    }
}

/// `uid:gid` of the app being replaced, so a root install does not leave the
/// bundle owned by `root:wheel`. Empty when the app is not on disk yet.
fn dest_owner(dest: &Path) -> String {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if let Ok(md) = std::fs::metadata(dest) {
            return format!("{}:{}", md.uid(), md.gid());
        }
    }
    String::new()
}

fn download_asset(asset: &GhAsset) -> Result<PathBuf, String> {
    let dir = update_staging_root();
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
        .arg(format!(
            "User-Agent: BioOKF-Studio/{}",
            current_studio_version()
        ))
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

fn write_script(path: &Path, body: String) -> Result<(), String> {
    std::fs::write(path, body).map_err(|e| format!("failed to write {}: {e}", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(path)
            .map_err(|e| e.to_string())?
            .permissions();
        perms.set_mode(0o700);
        std::fs::set_permissions(path, perms).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// The privileged half of the update, run as root through one `osascript` prompt.
///
/// Ordering is the whole point: the replacement bundle is copied in fully before
/// the running app is moved aside, and every failure puts the old app back. The
/// previous version ran `rm -rf "$DEST" && ditto ...`, which destroys the
/// installed app if anything after the `rm` goes wrong.
fn privileged_installer_body(staged_app: &Path, dest_app: &Path, owner: &str) -> String {
    format!(
        r#"#!/bin/bash
set -uo pipefail
# `do shell script` hands us a minimal environment, and `chown` lives in /usr/sbin.
# Append rather than replace so a caller-provided PATH still wins (used by tests).
export PATH="${{PATH:-}}:/usr/bin:/bin:/usr/sbin:/sbin"
STAGED={staged}
DEST={dest}
OWNER={owner}
BIN_DIR="${{BIOOKF_UPDATE_BIN_DIR:-/usr/local/bin}}"
PARENT="$(dirname "$DEST")"
NEW="$PARENT/.biookf-update-new-$$"
BACKUP="$PARENT/.biookf-update-backup-$$"

rm -rf "$NEW" "$BACKUP"

if ! ditto "$STAGED" "$NEW"; then
  echo "could not copy the new app into $PARENT" >&2
  rm -rf "$NEW"
  exit 1
fi

if [ -e "$DEST" ] && ! mv "$DEST" "$BACKUP"; then
  echo "could not move the existing app aside" >&2
  rm -rf "$NEW"
  exit 1
fi

if ! mv "$NEW" "$DEST"; then
  echo "could not move the new app into place" >&2
  if [ -e "$BACKUP" ]; then mv "$BACKUP" "$DEST"; fi
  rm -rf "$NEW"
  exit 1
fi

if ! codesign --verify --deep --strict -- "$DEST"; then
  echo "the installed app failed signature verification; rolling back" >&2
  rm -rf "$DEST"
  if [ -e "$BACKUP" ]; then mv "$BACKUP" "$DEST"; fi
  exit 1
fi

if [ -n "$OWNER" ]; then chown -R "$OWNER" "$DEST" || true; fi

# Refreshing the command-line tools is best effort: the app is already installed
# and must not be rolled back just because $BIN_DIR is unwritable.
mkdir -p "$BIN_DIR" 2>/dev/null || true
for tool in bokf bokf-mcp; do
  src="$DEST/Contents/Resources/bin/$tool"
  if [ -x "$src" ]; then
    cp "$src" "$BIN_DIR/$tool" 2>/dev/null && chmod 755 "$BIN_DIR/$tool" 2>/dev/null || true
  fi
done

rm -rf "$BACKUP"
exit 0
"#,
        staged = sh_quote(&staged_app.to_string_lossy()),
        dest = sh_quote(&dest_app.to_string_lossy()),
        owner = sh_quote(owner),
    )
}

fn write_privileged_installer(staged_app: &Path, dest_app: &Path) -> Result<PathBuf, String> {
    let mut script = update_staging_root();
    script.push("install.sh");
    let body = privileged_installer_body(staged_app, dest_app, &dest_owner(dest_app));
    write_script(&script, body)?;
    Ok(script)
}

/// The detached half: waits for Studio to exit, runs the privileged installer,
/// and reopens the app.
///
/// Every exit path calls `reopen`, because by the time this script runs Studio
/// has already quit. A bare `set -e` abort here — which is what a failing
/// `codesign` check used to cause — leaves the user staring at a closed app with
/// the only explanation buried in a log file.
fn relauncher_body(
    pid: u32,
    staged_app: &Path,
    dest_app: &Path,
    staging_root: &Path,
    installer: &Path,
) -> String {
    let osascript_cmd = sh_quote(&format!(
        "do shell script {} with administrator privileges",
        applescript_string(&format!(
            "/bin/bash {}",
            sh_quote(&installer.to_string_lossy())
        ))
    ));
    format!(
        r#"#!/bin/bash
set -uo pipefail
export PATH="${{PATH:-}}:/usr/bin:/bin:/usr/sbin:/sbin"
PID={pid}
STAGED={staged}
DEST={dest}
STAGING_ROOT={staging_root}
LOG="$HOME/Library/Logs/BioOKF Studio Updater.log"
mkdir -p "$(dirname "$LOG")"
exec >> "$LOG" 2>&1

log() {{ echo "$(date '+%Y-%m-%d %H:%M:%S') $*"; }}
cleanup() {{ rm -rf "$STAGING_ROOT" "$0" >/dev/null 2>&1 || true; }}
trap cleanup EXIT

reopen() {{
  if [ -d "$DEST" ]; then
    log "reopening $DEST"
    open "$DEST" || log "WARNING: could not reopen $DEST"
  else
    log "ERROR: no app at $DEST to reopen"
  fi
}}

fail() {{ log "update FAILED: $*"; reopen; exit 1; }}

log "starting update: staged=$STAGED dest=$DEST"

waited=0
while kill -0 "$PID" 2>/dev/null; do
  sleep 0.2
  waited=$((waited + 1))
  if [ "$waited" -ge 300 ]; then fail "timed out waiting for Studio (pid $PID) to exit"; fi
done

if [ ! -d "$STAGED" ]; then fail "staged app missing at $STAGED"; fi
if ! codesign --verify --deep --strict -- "$STAGED"; then fail "staged app failed signature verification"; fi

if osascript -e {osascript_cmd}; then
  log "installed $DEST"
else
  fail "privileged install step failed or was cancelled"
fi

reopen
log "update complete"
"#,
        pid = pid,
        staged = sh_quote(&staged_app.to_string_lossy()),
        dest = sh_quote(&dest_app.to_string_lossy()),
        staging_root = sh_quote(&staging_root.to_string_lossy()),
        osascript_cmd = osascript_cmd,
    )
}

fn write_macos_relauncher(
    staged_app: &Path,
    dest_app: &Path,
    installer: &Path,
) -> Result<PathBuf, String> {
    let mut script = std::env::temp_dir();
    script.push(format!("biookf-relaunch-{}.sh", std::process::id()));
    let body = relauncher_body(
        std::process::id(),
        staged_app,
        dest_app,
        &update_staging_root(),
        installer,
    );
    write_script(&script, body)?;
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
    let bundled_mcp =
        bundled_bin_dir(&app).map(|d| d.join(bokf_mcp_exe_name()).to_string_lossy().to_string());
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
        return Err(format!(
            "bundled bokf-mcp not found at {}",
            src_mcp.display()
        ));
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
    let dest_app =
        app_bundle_path().unwrap_or_else(|| PathBuf::from("/Applications/BioOKF Studio.app"));

    // Download, unpack, and signature-check while the window is still up, so a bad
    // asset surfaces in the modal instead of silently killing the app. Studio only
    // quits below, once there is a verified bundle to install.
    let staging = update_staging_root();
    let staged = download_asset(&asset)
        .and_then(|archive| stage_app_from_asset(&archive, &staging))
        .and_then(|app| verify_staged_app(&app, &dest_app).map(|()| app))
        .inspect_err(|_| {
            // Don't strand a rejected download (tens of MB) in the temp dir.
            let _ = std::fs::remove_dir_all(&staging);
        })?;

    let installer = write_privileged_installer(&staged, &dest_app)?;
    let relauncher = write_macos_relauncher(&staged, &dest_app, &installer)?;
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

/// Control-plane socket path (AUDIT M1). Must match `bokf-mcp`'s
/// `studio_client::socket_path()`: `$BIOOKF_STUDIO_IPC`, else
/// `$HOME/.biookf/studio-mcp.sock` (a per-user, 0700 directory).
#[cfg(feature = "control")]
fn biookf_control_socket_path() -> std::path::PathBuf {
    if let Some(p) = std::env::var_os("BIOOKF_STUDIO_IPC") {
        return std::path::PathBuf::from(p);
    }
    let home = std::env::var_os("HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    home.join(".biookf").join("studio-mcp.sock")
}

/// A random per-run auth token for the control socket (AUDIT M1). The plugin
/// writes it to `<socket>.token` (0600); only same-user processes that can read
/// that file may drive the GUI. 32 bytes of `/dev/urandom`, hex-encoded, with a
/// weak pid fallback (the 0700 dir + 0600 file still bound reachability).
#[cfg(feature = "control")]
fn biookf_control_auth_token() -> String {
    use std::io::Read;
    let mut buf = [0u8; 32];
    if let Ok(mut f) = std::fs::File::open("/dev/urandom") {
        if f.read_exact(&mut buf).is_ok() {
            return buf.iter().map(|b| format!("{b:02x}")).collect();
        }
    }
    format!("biookf-{}", std::process::id())
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
            get_export_bundle,
            network_metrics,
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
            open_base_folder,
            write_export_html,
            write_network_metrics_json,
            term_open,
            term_write,
            term_resize,
            term_close,
            source_info,
            paperclip_generator_status,
            paperclip_generate_base,
            local_connections_status,
            save_local_connections,
            chat_with_knowledge_base,
            doctor_knowledge_base,
            create_local_knowledge_base,
            merge_knowledge_bases,
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
        //
        // AUDIT M1: the control socket is a full local-RCE surface (execute_js on
        // the privileged webview). It must NOT live at a fixed, world-reachable
        // /tmp path with no auth. We (a) put it in a per-user 0700 directory so
        // other users can't reach it, and (b) require a random per-run auth token
        // that the plugin writes next to the socket (0600); only same-user
        // processes that can read that file can drive the GUI.
        let sock = biookf_control_socket_path();
        if let Some(dir) = sock.parent() {
            let _ = std::fs::create_dir_all(dir);
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700));
            }
        }
        let builder = builder.plugin(tauri_plugin_mcp::init_with_config(
            tauri_plugin_mcp::PluginConfig::new("BioOKF Studio".to_string())
                .start_socket_server(true)
                .auth_token(biookf_control_auth_token())
                .socket_path(sock),
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
    fn local_subscription_connections_default_to_enabled() {
        let value = super::LocalConnections::default();
        assert!(value.codex && value.claude && value.paperclip);
        let path = super::paperclip_child_path();
        if let Some(home) = std::env::var_os("HOME") {
            let local = std::path::PathBuf::from(home).join(".local/bin");
            assert!(path.contains(local.to_string_lossy().as_ref()));
        }
        assert!(path.contains("/opt/homebrew/bin"));
        assert!(path.contains("/usr/local/bin"));
    }

    #[test]
    fn agent_workflows_require_a_bounded_kb_name() {
        assert!(super::validate_kb_name("Local evidence").is_ok());
        assert!(super::validate_kb_name("  ").is_err());
        assert!(super::validate_kb_name(&"x".repeat(161)).is_err());
    }

    #[test]
    fn paperclip_generation_builds_safe_argv_with_standard_curation() {
        let request = super::PaperclipGenerateRequest {
            query: "tolebrutinib multiple sclerosis".into(),
            sources: vec!["pmc".into(), "trials/us".into()],
            limit: 1,
            kb_name: "Tolebrutinib MS".into(),
            provider: "claude".into(),
            model: Some("sonnet".into()),
            year_min: Some(2020),
            year_max: Some(2026),
            since: None,
        };
        super::validate_paperclip_request(&request).unwrap();
        let args = super::paperclip_generate_args(&request, std::path::Path::new("/tmp/p2b"));
        assert!(args.windows(2).any(|pair| pair == ["--model", "sonnet"]));
        assert_eq!(
            args.iter().filter(|arg| arg.as_str() == "--source").count(),
            2
        );
        assert!(!args.iter().any(|arg| arg == "--prompt"));
        assert!(args.iter().any(|arg| arg == "--register"));
    }

    #[test]
    fn paperclip_generation_rejects_invalid_source_and_year_range() {
        let request = super::PaperclipGenerateRequest {
            query: "x".into(),
            sources: vec!["unknown".into()],
            limit: 1,
            kb_name: "x".into(),
            provider: "codex".into(),
            model: None,
            year_min: Some(2026),
            year_max: Some(2020),
            since: None,
        };
        assert!(super::validate_paperclip_request(&request).is_err());
    }

    #[test]
    fn update_version_compare_handles_v_prefixes() {
        assert!(super::version_newer("v0.3.0", "0.2.3"));
        assert!(super::version_newer("0.10.0", "0.9.9"));
        assert!(!super::version_newer("v0.2.2", "0.2.2"));
        assert!(!super::version_newer("v0.2.1", "0.2.2"));
    }

    use std::path::{Path, PathBuf};
    use std::process::Command;

    fn release_with(names: &[&str]) -> super::GhRelease {
        super::GhRelease {
            tag_name: "v0.3.0".into(),
            html_url: None,
            assets: names
                .iter()
                .map(|n| super::GhAsset {
                    name: (*n).to_string(),
                    browser_download_url: format!("https://example.invalid/{n}"),
                })
                .collect(),
        }
    }

    fn expected_macos_dmg() -> Option<&'static str> {
        match (std::env::consts::OS, std::env::consts::ARCH) {
            ("macos", "aarch64") => Some("BioOKF.Studio_0.3.0_aarch64.dmg"),
            ("macos", "x86_64") => Some("BioOKF.Studio_0.3.0_x64.dmg"),
            _ => None,
        }
    }

    #[test]
    fn update_asset_selection_finds_current_macos_dmg() {
        let release = release_with(&["BioOKF.Studio_0.3.0_aarch64.dmg"]);
        let asset = super::asset_for_current_platform(&release);
        if std::env::consts::OS == "macos" && std::env::consts::ARCH == "aarch64" {
            assert_eq!(asset.unwrap().name, "BioOKF.Studio_0.3.0_aarch64.dmg");
        }
    }

    /// Regression: the v0.3.0 release lists the CI `.tar.gz` (built `--no-sign`)
    /// before the signed `.dmg`, and the old first-match selection installed the
    /// unsigned tarball, which then failed `codesign --verify` in the relauncher.
    #[test]
    fn update_prefers_signed_dmg_over_unsigned_tarball() {
        let release = release_with(&[
            "biookf-macos-arm64.tar.gz",
            "biookf-macos-x64.tar.gz",
            "BioOKF.Studio_0.3.0_aarch64.dmg",
            "BioOKF.Studio_0.3.0_x64.dmg",
        ]);
        let picked = super::asset_for_current_platform(&release).map(|a| a.name);
        if let Some(want) = expected_macos_dmg() {
            assert_eq!(picked.as_deref(), Some(want));
        }
    }

    #[test]
    fn update_asset_selection_falls_back_to_tarball_without_a_dmg() {
        let release = release_with(&["biookf-macos-arm64.tar.gz", "biookf-macos-x64.tar.gz"]);
        let picked = super::asset_for_current_platform(&release).map(|a| a.name);
        match (std::env::consts::OS, std::env::consts::ARCH) {
            ("macos", "aarch64") => {
                assert_eq!(picked.as_deref(), Some("biookf-macos-arm64.tar.gz"))
            }
            ("macos", "x86_64") => assert_eq!(picked.as_deref(), Some("biookf-macos-x64.tar.gz")),
            _ => {}
        }
    }

    #[test]
    fn update_asset_selection_ignores_the_other_architecture() {
        let other = match std::env::consts::ARCH {
            "aarch64" => "BioOKF.Studio_0.3.0_x64.dmg",
            _ => "BioOKF.Studio_0.3.0_aarch64.dmg",
        };
        let release = release_with(&[other, "biookf-sources.zip"]);
        if std::env::consts::OS == "macos" {
            // Only the last-ditch "any macOS dmg" fallback may fire; never the zip.
            let picked = super::asset_for_current_platform(&release).map(|a| a.name);
            assert_ne!(picked.as_deref(), Some("biookf-sources.zip"));
        }
    }

    /// End-to-end through the command the popup actually calls, over the exact
    /// asset list the GitHub API returns for v0.3.0 (tag bumped so an update is
    /// on offer). This is where the unsigned tarball used to win.
    #[test]
    #[cfg(target_os = "macos")]
    fn update_status_offers_the_signed_dmg_for_the_real_release_payload() {
        let payload = r#"{"tag_name":"v9.9.9","html_url":"https://example.invalid/r","assets":[
          {"name":"biookf-macos-arm64.tar.gz","browser_download_url":"https://example.invalid/a.tgz"},
          {"name":"biookf-macos-x64.tar.gz","browser_download_url":"https://example.invalid/b.tgz"},
          {"name":"BioOKF.Studio_0.3.0_aarch64.dmg","browser_download_url":"https://example.invalid/a.dmg"},
          {"name":"BioOKF.Studio_0.3.0_x64.dmg","browser_download_url":"https://example.invalid/b.dmg"}]}"#;
        std::env::set_var("BIOOKF_UPDATE_RELEASE_JSON", payload);
        let status = super::update_status();
        std::env::remove_var("BIOOKF_UPDATE_RELEASE_JSON");

        assert_eq!(status["ok"], true);
        assert_eq!(status["updateAvailable"], true);
        assert_eq!(status["installSupported"], true);
        assert_eq!(status["assetName"].as_str(), expected_macos_dmg());
    }

    /// Real release artifacts, checked without a network fetch. Point it at
    /// archives you already downloaded. Every asset published from v0.3.1 on is
    /// signed — including the tarball — so the unsigned side needs an older
    /// artifact, or a local `--no-sign` build, to stay meaningful:
    ///   BIOOKF_TEST_SIGNED_ASSET=…/BioOKF.Studio_0.3.1_aarch64.dmg \
    ///   BIOOKF_TEST_UNSIGNED_ASSET=…/v0.3.0/biookf-macos-arm64.tar.gz \
    ///   cargo test -p biookf-studio -- --ignored real_release_assets
    ///
    /// Either variable may be omitted to check only one side.
    #[test]
    #[ignore]
    #[cfg(target_os = "macos")]
    fn real_release_assets_install_only_when_signed() {
        let root = scratch("real-assets");
        let mut checked = 0;

        if let Ok(signed) = std::env::var("BIOOKF_TEST_SIGNED_ASSET") {
            let staged = super::stage_app_from_asset(Path::new(&signed), &root.join("ok")).unwrap();
            super::verify_staged_app(&staged, &staged).expect("a signed asset must be installable");
            assert_eq!(
                super::codesign_team_id(&staged).as_deref(),
                Some("F3YYBXAFJ8")
            );
            checked += 1;
        }

        if let Ok(unsigned) = std::env::var("BIOOKF_TEST_UNSIGNED_ASSET") {
            let staged =
                super::stage_app_from_asset(Path::new(&unsigned), &root.join("bad")).unwrap();
            let err = super::verify_staged_app(&staged, &staged)
                .expect_err("an unsigned asset must be rejected before Studio quits");
            assert!(err.contains("not correctly code-signed"), "{err}");
            checked += 1;
        }

        assert!(
            checked > 0,
            "set BIOOKF_TEST_SIGNED_ASSET and/or BIOOKF_TEST_UNSIGNED_ASSET"
        );
    }

    #[test]
    fn asset_kind_rank_prefers_dmg_then_tarball() {
        assert!(super::asset_kind_rank("a.dmg") < super::asset_kind_rank("a.tar.gz"));
        assert!(super::asset_kind_rank("a.tar.gz") < super::asset_kind_rank("a.zip"));
        assert_eq!(
            super::asset_kind_rank("a.tgz"),
            super::asset_kind_rank("a.tar.gz")
        );
        assert_eq!(super::asset_kind_rank("notes.txt"), None);
    }

    #[test]
    fn team_identifier_parsing_ignores_adhoc_signatures() {
        let adhoc = "Identifier=com.biookf.studio\nSignature=adhoc\nTeamIdentifier=not set\n";
        assert_eq!(super::parse_team_identifier(adhoc), None);
        let real = "Identifier=com.biookf.studio\nTeamIdentifier=F3YYBXAFJ8\n";
        assert_eq!(
            super::parse_team_identifier(real).as_deref(),
            Some("F3YYBXAFJ8")
        );
        assert_eq!(super::parse_team_identifier("Identifier=x\n"), None);
    }

    #[test]
    fn team_identity_pins_the_update_to_the_running_apps_team() {
        assert!(super::check_team_identity(None, None).is_ok());
        assert!(super::check_team_identity(None, Some("ANY")).is_ok());
        assert!(super::check_team_identity(Some("F3YYBXAFJ8"), Some("F3YYBXAFJ8")).is_ok());
        assert!(super::check_team_identity(Some("F3YYBXAFJ8"), Some("EVIL")).is_err());
        assert!(super::check_team_identity(Some("F3YYBXAFJ8"), None).is_err());
    }

    // --- macOS bundle fixtures ------------------------------------------------

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("biookf-test-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// A minimal but real `.app`: `codesign` needs a Mach-O main executable, so
    /// borrow `/bin/echo`.
    fn make_app(parent: &Path, name: &str) -> PathBuf {
        let app = parent.join(name);
        std::fs::create_dir_all(app.join("Contents/MacOS")).unwrap();
        std::fs::create_dir_all(app.join("Contents/Resources")).unwrap();
        std::fs::copy("/bin/echo", app.join("Contents/MacOS/app")).unwrap();
        std::fs::write(
            app.join("Contents/Info.plist"),
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleExecutable</key><string>app</string>
<key>CFBundleIdentifier</key><string>com.biookf.studio.test</string>
<key>CFBundleName</key><string>BioOKF Studio</string>
<key>CFBundlePackageType</key><string>APPL</string>
</dict></plist>
"#,
        )
        .unwrap();
        app
    }

    fn adhoc_sign(app: &Path) {
        let out = Command::new("codesign")
            .args(["-s", "-", "--force", "--deep"])
            .arg(app)
            .output()
            .unwrap();
        assert!(out.status.success(), "codesign fixture failed: {out:?}");
    }

    fn bash(script: &Path, env: &[(&str, &str)]) -> std::process::Output {
        let mut cmd = Command::new("/bin/bash");
        cmd.arg(script);
        for (k, v) in env {
            cmd.env(k, v);
        }
        cmd.output().unwrap()
    }

    /// `.app` bundles left over from a failed swap would be re-registered by
    /// LaunchServices, so the installer must leave none behind.
    fn leftovers(parent: &Path) -> Vec<String> {
        std::fs::read_dir(parent)
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|n| n.starts_with(".biookf-update-"))
            .collect()
    }

    #[test]
    fn find_app_bundle_skips_symlinked_directories() {
        let root = scratch("findapp");
        let decoy = root.join("decoy");
        std::fs::create_dir_all(&decoy).unwrap();
        std::fs::create_dir_all(decoy.join(super::STUDIO_APP_NAME)).unwrap();
        let scan = root.join("scan");
        std::fs::create_dir_all(scan.join("real")).unwrap();
        std::fs::create_dir_all(scan.join("real").join(super::STUDIO_APP_NAME)).unwrap();
        std::os::unix::fs::symlink(&decoy, scan.join("link")).unwrap();

        let found = super::find_app_bundle(&scan, 5).unwrap();
        assert_eq!(found, scan.join("real").join(super::STUDIO_APP_NAME));
        assert!(
            super::find_app_bundle(&scan, 1).is_none(),
            "depth is honored"
        );
    }

    // --- the actual regression: an unsigned update must not reach the installer

    #[test]
    #[cfg(target_os = "macos")]
    fn verify_staged_app_rejects_an_unsigned_bundle() {
        let root = scratch("unsigned");
        let app = make_app(&root, super::STUDIO_APP_NAME);
        let err = super::verify_staged_app(&app, &app).unwrap_err();
        assert!(
            err.contains("not correctly code-signed"),
            "unexpected error: {err}"
        );
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn verify_staged_app_accepts_a_validly_signed_bundle() {
        let root = scratch("signed");
        let app = make_app(&root, super::STUDIO_APP_NAME);
        adhoc_sign(&app);
        let current = make_app(&root.join("cur"), super::STUDIO_APP_NAME);
        adhoc_sign(&current);
        super::verify_staged_app(&app, &current).unwrap();
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn staging_extracts_and_locates_the_app_in_a_tarball() {
        let root = scratch("stage-tgz");
        let src = root.join("src");
        std::fs::create_dir_all(&src).unwrap();
        let app = make_app(&src, super::STUDIO_APP_NAME);
        adhoc_sign(&app);
        std::fs::create_dir_all(src.join("bin")).unwrap();
        let tarball = root.join("biookf-macos.tar.gz");
        assert!(Command::new("tar")
            .arg("-czf")
            .arg(&tarball)
            .arg("-C")
            .arg(&src)
            .arg(".")
            .status()
            .unwrap()
            .success());

        let staged = super::stage_app_from_asset(&tarball, &root.join("work")).unwrap();
        assert!(staged.is_dir());
        assert_eq!(
            staged.file_name().and_then(|s| s.to_str()),
            Some(super::STUDIO_APP_NAME)
        );
        // ditto must preserve the signature, or the pre-quit gate would reject it.
        super::codesign_verify(&staged).unwrap();
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn staging_mounts_a_dmg_and_detaches_it() {
        let root = scratch("stage-dmg");
        let src = root.join("src");
        std::fs::create_dir_all(&src).unwrap();
        adhoc_sign(&make_app(&src, super::STUDIO_APP_NAME));
        // Real releases ship an /Applications alias next to the app.
        std::os::unix::fs::symlink("/Applications", src.join("Applications")).unwrap();

        let dmg = root.join("BioOKF.Studio_0.0.0_test.dmg");
        let out = Command::new("hdiutil")
            .args(["create", "-quiet", "-srcfolder"])
            .arg(&src)
            .args(["-volname", "BioOKF Test", "-ov", "-format", "UDZO"])
            .arg(&dmg)
            .output()
            .unwrap();
        assert!(out.status.success(), "hdiutil create failed: {out:?}");

        let staged = super::stage_app_from_asset(&dmg, &root.join("work")).unwrap();
        assert!(staged.is_dir(), "app copied out of the image");
        super::codesign_verify(&staged).unwrap();
        // The staged copy must outlive the mount: nothing may still be attached.
        let mounted = Command::new("hdiutil").args(["info"]).output().unwrap();
        let info = String::from_utf8_lossy(&mounted.stdout);
        assert!(
            !info.contains(
                &root
                    .join("work")
                    .join("mount")
                    .to_string_lossy()
                    .to_string()
            ),
            "disk image left attached"
        );
    }

    // --- the privileged installer, exercised without root -----------------------

    /// Build `staged` + `dest` fixtures and run the generated installer directly.
    /// `chown` to our own uid and a temp `BIN_DIR` keep it root-free.
    fn run_installer(case: &str, sign_staged: bool) -> (std::process::Output, PathBuf, PathBuf) {
        let root = scratch(case);
        let staged = make_app(&root.join("staged"), super::STUDIO_APP_NAME);
        std::fs::create_dir_all(staged.join("Contents/Resources/bin")).unwrap();
        std::fs::write(
            staged.join("Contents/Resources/bin/bokf"),
            "#!/bin/sh\necho new-bokf\n",
        )
        .unwrap();
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(
                staged.join("Contents/Resources/bin/bokf"),
                std::fs::Permissions::from_mode(0o755),
            )
            .unwrap();
        }
        if sign_staged {
            adhoc_sign(&staged);
        }

        let apps = root.join("Applications");
        std::fs::create_dir_all(&apps).unwrap();
        let dest = make_app(&apps, super::STUDIO_APP_NAME);
        std::fs::write(dest.join("Contents/OLD-MARKER"), "old").unwrap();

        let bin_dir = root.join("bin");
        std::fs::create_dir_all(&bin_dir).unwrap();

        let script = root.join("install.sh");
        std::fs::write(
            &script,
            super::privileged_installer_body(&staged, &dest, &super::dest_owner(&dest)),
        )
        .unwrap();
        let bin = bin_dir.to_string_lossy().to_string();
        let out = bash(&script, &[("BIOOKF_UPDATE_BIN_DIR", bin.as_str())]);
        (out, dest, bin_dir)
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn installer_replaces_the_app_and_refreshes_the_cli() {
        let (out, dest, bin_dir) = run_installer("install-ok", true);
        assert!(out.status.success(), "installer failed: {out:?}");
        assert!(dest.is_dir(), "app installed");
        assert!(
            !dest.join("Contents/OLD-MARKER").exists(),
            "old bundle replaced"
        );
        super::codesign_verify(&dest).unwrap();
        assert!(bin_dir.join("bokf").is_file(), "bundled CLI refreshed");
        assert!(
            leftovers(dest.parent().unwrap()).is_empty(),
            "no scratch bundles left behind"
        );
    }

    /// The old installer ran `rm -rf "$DEST" && ditto ...`, so any later failure
    /// left the user with no app at all — which is exactly what shipped.
    #[test]
    #[cfg(target_os = "macos")]
    fn installer_rolls_back_and_keeps_the_old_app_when_the_update_is_unsigned() {
        let (out, dest, bin_dir) = run_installer("install-rollback", false);
        assert!(!out.status.success(), "unsigned update must not install");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains("rolling back"),
            "unexpected stderr: {stderr}"
        );
        assert!(dest.is_dir(), "the previously installed app survives");
        assert!(
            dest.join("Contents/OLD-MARKER").exists(),
            "the original bundle is restored, not a partial copy"
        );
        assert!(!bin_dir.join("bokf").exists(), "CLI untouched on failure");
        assert!(
            leftovers(dest.parent().unwrap()).is_empty(),
            "backup cleaned up"
        );
    }

    /// `osascript`'s `do shell script` runs with a bare PATH. `chown` is in
    /// /usr/sbin, so without an explicit PATH the ownership restore silently
    /// no-ops and the bundle is left owned by root.
    #[test]
    #[cfg(target_os = "macos")]
    fn installer_finds_its_tools_under_a_minimal_path() {
        let root = scratch("install-path");
        let staged = make_app(&root.join("staged"), super::STUDIO_APP_NAME);
        adhoc_sign(&staged);
        let apps = root.join("Applications");
        std::fs::create_dir_all(&apps).unwrap();
        let dest = make_app(&apps, super::STUDIO_APP_NAME);

        let script = root.join("install.sh");
        std::fs::write(
            &script,
            super::privileged_installer_body(&staged, &dest, &super::dest_owner(&dest)),
        )
        .unwrap();

        let out = Command::new("/bin/bash")
            .arg(&script)
            .env("PATH", "") // what `do shell script` effectively gives a bad script
            .env("BIOOKF_UPDATE_BIN_DIR", root.join("bin"))
            .output()
            .unwrap();

        assert!(out.status.success(), "installer failed: {out:?}");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            !stderr.contains("command not found"),
            "installer relies on an inherited PATH: {stderr}"
        );
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn installer_leaves_the_app_alone_when_the_staged_bundle_is_missing() {
        let root = scratch("install-missing");
        let apps = root.join("Applications");
        std::fs::create_dir_all(&apps).unwrap();
        let dest = make_app(&apps, super::STUDIO_APP_NAME);
        std::fs::write(dest.join("Contents/OLD-MARKER"), "old").unwrap();
        let script = root.join("install.sh");
        std::fs::write(
            &script,
            super::privileged_installer_body(&root.join("nope").join("Missing.app"), &dest, ""),
        )
        .unwrap();

        let bin = root.join("bin").to_string_lossy().to_string();
        let out = bash(&script, &[("BIOOKF_UPDATE_BIN_DIR", bin.as_str())]);
        assert!(!out.status.success());
        assert!(dest.join("Contents/OLD-MARKER").exists(), "app untouched");
        assert!(leftovers(&apps).is_empty());
    }

    // --- the relauncher must never strand the user without an app ---------------

    fn write_stub(path: &Path, body: &str) {
        use std::os::unix::fs::PermissionsExt;
        std::fs::write(path, body).unwrap();
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    fn open_stub(dir: &Path, marker: &Path) {
        write_stub(
            &dir.join("open"),
            &format!(
                "#!/bin/sh\nprintf '%s\\n' \"$@\" >> '{}'\n",
                marker.display()
            ),
        );
    }

    /// The shipped relauncher ran under `set -e`, so a failing `codesign` check
    /// aborted it *after* Studio had already quit: no install, no relaunch, and
    /// no message. Whatever happens, the app on disk has to be reopened.
    #[test]
    #[cfg(target_os = "macos")]
    fn relauncher_reopens_the_app_on_success_and_on_failure() {
        for (case, osascript_exit, expect_success, expect_log) in [
            ("relaunch-install-fails", 1, false, "update FAILED"),
            ("relaunch-install-works", 0, true, "update complete"),
        ] {
            let root = scratch(case);
            let home = root.join("home");
            std::fs::create_dir_all(&home).unwrap();
            let staged = make_app(&root.join("staged"), super::STUDIO_APP_NAME);
            adhoc_sign(&staged);
            let dest = make_app(&root.join("Applications"), super::STUDIO_APP_NAME);

            let stub = root.join("stub");
            std::fs::create_dir_all(&stub).unwrap();
            let marker = root.join("open-was-called");
            open_stub(&stub, &marker);
            write_stub(
                &stub.join("osascript"),
                &format!("#!/bin/sh\nexit {osascript_exit}\n"),
            );

            let staging = root.join("staging");
            std::fs::create_dir_all(&staging).unwrap();
            let script = root.join("relaunch.sh");
            std::fs::write(
                &script,
                // u32::MAX is never a live pid, so the wait loop falls straight through.
                super::relauncher_body(
                    u32::MAX,
                    &staged,
                    &dest,
                    &staging,
                    &root.join("install.sh"),
                ),
            )
            .unwrap();

            let out = Command::new("/bin/bash")
                .arg(&script)
                .env("HOME", &home)
                .env("PATH", format!("{}:/usr/bin:/bin", stub.display()))
                .output()
                .unwrap();

            assert_eq!(
                out.status.success(),
                expect_success,
                "{case}: wrong exit status"
            );
            assert!(
                marker.is_file(),
                "{case}: the app was never reopened -- this is the bug users hit"
            );
            let log = std::fs::read_to_string(home.join("Library/Logs/BioOKF Studio Updater.log"))
                .unwrap();
            assert!(log.contains(expect_log), "{case}: log said {log}");
            assert!(log.contains("reopening"), "{case}: log said {log}");
        }
    }

    /// An unsigned staged bundle must be stopped before the root install step,
    /// and must still leave the user with a running app.
    #[test]
    #[cfg(target_os = "macos")]
    fn relauncher_refuses_an_unsigned_staged_bundle_but_still_reopens() {
        let root = scratch("relaunch-unsigned");
        let home = root.join("home");
        std::fs::create_dir_all(&home).unwrap();
        let staged = make_app(&root.join("staged"), super::STUDIO_APP_NAME); // never signed
        let dest = make_app(&root.join("Applications"), super::STUDIO_APP_NAME);

        let stub = root.join("stub");
        std::fs::create_dir_all(&stub).unwrap();
        let marker = root.join("open-was-called");
        open_stub(&stub, &marker);
        let privileged_ran = root.join("PRIVILEGED-RAN");
        write_stub(
            &stub.join("osascript"),
            &format!("#!/bin/sh\ntouch '{}'\n", privileged_ran.display()),
        );

        let staging = root.join("staging");
        std::fs::create_dir_all(&staging).unwrap();
        let script = root.join("relaunch.sh");
        std::fs::write(
            &script,
            super::relauncher_body(u32::MAX, &staged, &dest, &staging, &root.join("install.sh")),
        )
        .unwrap();

        let out = Command::new("/bin/bash")
            .arg(&script)
            .env("HOME", &home)
            .env("PATH", format!("{}:/usr/bin:/bin", stub.display()))
            .output()
            .unwrap();

        assert!(!out.status.success());
        assert!(
            !privileged_ran.exists(),
            "an unsigned bundle reached the root install step"
        );
        assert!(marker.is_file(), "the app must still be reopened");
    }

    // --- generated shell / AppleScript must be syntactically valid ---------------

    fn tricky_paths() -> (PathBuf, PathBuf, PathBuf, PathBuf) {
        (
            PathBuf::from("/tmp/bio okf's stage/BioOKF Studio.app"),
            PathBuf::from("/Applications/BioOKF Studio.app"),
            PathBuf::from("/tmp/bio okf's stage"),
            PathBuf::from("/tmp/bio okf's stage/install.sh"),
        )
    }

    #[test]
    fn generated_scripts_are_valid_bash_even_with_awkward_paths() {
        let (staged, dest, root, installer) = tricky_paths();
        let dir = scratch("syntax");
        for (name, body) in [
            (
                "install.sh",
                super::privileged_installer_body(&staged, &dest, "501:80"),
            ),
            (
                "relaunch.sh",
                super::relauncher_body(1234, &staged, &dest, &root, &installer),
            ),
        ] {
            let p = dir.join(name);
            std::fs::write(&p, &body).unwrap();
            let out = Command::new("bash").arg("-n").arg(&p).output().unwrap();
            assert!(
                out.status.success(),
                "{name} is not valid bash: {}",
                String::from_utf8_lossy(&out.stderr)
            );
        }
    }

    /// The install command crosses bash -> AppleScript -> shell. Pull the
    /// `osascript -e` argument back out through bash and check both that it is
    /// the AppleScript we meant and that `osacompile` accepts it.
    #[test]
    #[cfg(target_os = "macos")]
    fn relauncher_quotes_the_installer_path_through_applescript() {
        let (staged, dest, root, installer) = tricky_paths();
        let body = super::relauncher_body(1234, &staged, &dest, &root, &installer);
        let line = body
            .lines()
            .find(|l| l.trim_start().starts_with("if osascript -e "))
            .expect("osascript invocation present");
        let literal = line.trim_start().trim_start_matches("if osascript -e ");
        let literal = literal.trim_end().trim_end_matches("; then");

        // Layer 1: bash unquotes the `-e` argument back to AppleScript source.
        let out = Command::new("bash")
            .arg("-c")
            .arg(format!("printf '%s' {literal}"))
            .output()
            .unwrap();
        assert!(out.status.success());
        let applescript = String::from_utf8_lossy(&out.stdout).to_string();

        // Layer 2: AppleScript itself unescapes the string literal. Evaluating
        // `return "..."` decodes it without running the shell command.
        let inner = applescript
            .strip_prefix("do shell script ")
            .and_then(|s| s.strip_suffix(" with administrator privileges"))
            .unwrap_or_else(|| panic!("unexpected AppleScript: {applescript}"));
        let decoded = Command::new("osascript")
            .arg("-e")
            .arg(format!("return {inner}"))
            .output()
            .unwrap();
        assert!(
            decoded.status.success(),
            "AppleScript did not parse: {}",
            String::from_utf8_lossy(&decoded.stderr)
        );
        let shell_cmd = String::from_utf8_lossy(&decoded.stdout)
            .trim_end()
            .to_string();
        let quoted_installer = super::sh_quote(&installer.to_string_lossy());
        assert_eq!(shell_cmd, format!("/bin/bash {quoted_installer}"));

        // Layer 3: the shell unquotes that back into the real installer path.
        let unquoted = Command::new("bash")
            .arg("-c")
            .arg(format!("printf '%s' {quoted_installer}"))
            .output()
            .unwrap();
        assert_eq!(
            String::from_utf8_lossy(&unquoted.stdout),
            installer.to_string_lossy()
        );
    }

    #[test]
    fn export_html_path_normalizes_and_guards_extension() {
        let p = super::normalize_export_html_path("/tmp/biookf-export").unwrap();
        assert_eq!(p.extension().and_then(|e| e.to_str()), Some("html"));
        assert!(super::normalize_export_html_path("/tmp/biookf-export.htm").is_ok());
        assert!(super::normalize_export_html_path("/tmp/biookf-export.txt").is_err());
        assert!(super::normalize_export_html_path("   ").is_err());
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
