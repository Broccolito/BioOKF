//! Filesystem operations behind the MCP tools, with path-safety guards. These
//! are the thin idempotent primitives the AI agent drives; all heavy logic lives
//! in `bokf-core`.

use bokf_core::parse::parse_node;
use std::path::{Path, PathBuf};

/// Validate a page path for WRITING: under `knowledge/` or a reserved root file;
/// never `raw/` (immutable); never `..` traversal.
fn writable_path(bundle: &Path, page: &str) -> Result<PathBuf, String> {
    if page.contains("..") {
        return Err("path traversal ('..') is not allowed".into());
    }
    let allowed = page.starts_with("knowledge/")
        || matches!(page, "index.md" | "log.md" | "schema.md" | "README.md");
    if !allowed {
        return Err("page must be under knowledge/ or one of index.md/log.md/schema.md".into());
    }
    Ok(bundle.join(page))
}

/// Validate a page path for READING: anything in the bundle except `..`.
fn readable_path(bundle: &Path, page: &str) -> Result<PathBuf, String> {
    if page.contains("..") {
        return Err("path traversal ('..') is not allowed".into());
    }
    Ok(bundle.join(page))
}

pub fn write_page(bundle: &Path, page: &str, content: &str) -> Result<String, String> {
    let full = writable_path(bundle, page)?;
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let tmp = full.with_extension("md.tmp");
    std::fs::write(&tmp, content).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, &full).map_err(|e| e.to_string())?;

    // If it's a concept doc, parse to validate and report back.
    if page.starts_with("knowledge/") {
        match parse_node(content, Path::new(page)) {
            Ok(n) => Ok(format!(
                "wrote {page}; parsed OK: type={}, identifier={:?}, {} edge(s){}",
                n.node_type.as_str(),
                n.identifier,
                n.edges.len(),
                if n.node_type.is_valid() { "" } else { "  [WARNING: invalid type]" }
            )),
            Err(e) => Ok(format!(
                "wrote {page}, but it does NOT parse as a valid concept document: {e}. Fix and rewrite."
            )),
        }
    } else {
        Ok(format!("wrote {page}"))
    }
}

pub fn read_page(bundle: &Path, page: &str) -> Result<String, String> {
    let full = readable_path(bundle, page)?;
    std::fs::read_to_string(&full).map_err(|e| format!("cannot read {page}: {e}"))
}

pub fn list_pages(bundle: &Path) -> Result<Vec<String>, String> {
    let knowledge = bundle.join("knowledge");
    let mut out = Vec::new();
    fn walk(dir: &Path, root: &Path, out: &mut Vec<String>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for e in entries.flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(&p, root, out);
                } else if p.extension().and_then(|x| x.to_str()) == Some("md") {
                    if let Ok(rel) = p.strip_prefix(root) {
                        out.push(rel.to_string_lossy().to_string());
                    }
                }
            }
        }
    }
    walk(&knowledge, bundle, &mut out);
    out.sort();
    Ok(out)
}

pub fn validate_page(content: &str) -> Result<String, String> {
    match parse_node(content, Path::new("knowledge/_check.md")) {
        Ok(n) => {
            let mut notes = Vec::new();
            if !n.node_type.is_valid() {
                notes.push(format!("type `{}` is NOT one of the 28 controlled types", n.raw_type));
            }
            if n.subtype.is_none() {
                notes.push("no subtype (expected, agent-coined)".into());
            }
            for e in &n.edges {
                if !e.predicate.is_valid() {
                    notes.push(format!("predicate `{}` is NOT one of the 23", e.raw_predicate));
                }
                if e.knowledge_level.is_none() || e.agent_type.is_none() || e.primary_source.is_none() {
                    notes.push(format!("edge `{} -> {}` is missing part of the provenance triplet", e.predicate.as_str(), e.object));
                }
            }
            Ok(format!(
                "VALID concept document: type={}, identifier={:?}, {} edge(s).{}",
                n.node_type.as_str(),
                n.identifier,
                n.edges.len(),
                if notes.is_empty() { String::new() } else { format!("\nNotes:\n - {}", notes.join("\n - ")) }
            ))
        }
        Err(e) => Err(format!("INVALID: {e}")),
    }
}

pub fn scaffold(bundle: &Path, name: &str) -> Result<String, String> {
    std::fs::create_dir_all(bundle.join("raw")).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(bundle.join("knowledge")).map_err(|e| e.to_string())?;
    let write_absent = |rel: &str, content: String| -> Result<(), String> {
        let p = bundle.join(rel);
        if !p.exists() {
            std::fs::write(&p, content).map_err(|e| e.to_string())?;
        }
        Ok(())
    };
    write_absent(
        "index.md",
        format!("# {name}\n\nokf_version: 0.5\nbiookf_version: 0.5\n\n> Catalog of concept pages.\n"),
    )?;
    write_absent("log.md", format!("# Change log: {name}\n"))?;
    write_absent(
        "schema.md",
        "# BioOKF operating schema (v0.5)\n\n28 node types, 24 forward-only predicates. See the canonical schema.md.\n".to_string(),
    )?;

    // version-track + register + activate the new bundle (mirror the CLI).
    let repo = bokf_core::git::GitRepo::open(bundle);
    if repo.ensure_repo().is_ok() {
        let _ = repo.commit_all(bokf_core::git::ChangeKind::Manual, &format!("create knowledge base {name}"), None);
    }
    if let (Some(id), Some(root)) = (bundle.file_name().map(|s| s.to_string_lossy().to_lowercase()), bundle.parent()) {
        if bokf_core::registry::validate_kb_id(&id).is_ok() {
            let abs = std::fs::canonicalize(bundle).unwrap_or_else(|_| bundle.to_path_buf());
            let _ = bokf_core::registry::register(root, &id, &abs.to_string_lossy());
            let _ = bokf_core::active::set_active(root, Some(&id));
        }
    }
    Ok(format!("scaffolded BioOKF bundle '{name}' at {}", bundle.display()))
}

pub fn list_bases(root: &Path) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(root) {
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() && (p.join("knowledge").is_dir() || p.join("index.md").is_file()) {
                out.push(p.file_name().unwrap_or_default().to_string_lossy().to_string());
            }
        }
    }
    out.sort();
    Ok(out)
}

pub fn append_log(bundle: &Path, date: &str, entry: &str) -> Result<String, String> {
    let path = bundle.join("log.md");
    let existing = std::fs::read_to_string(&path).unwrap_or_else(|_| "# Change log\n".to_string());
    // newest-first: insert the dated section right after the title line.
    let block = format!("\n## {date}\n\n{entry}\n");
    let new = match existing.find('\n') {
        Some(i) => format!("{}{}{}", &existing[..=i], block, &existing[i + 1..]),
        None => format!("{existing}{block}"),
    };
    std::fs::write(&path, new).map_err(|e| e.to_string())?;
    Ok(format!("appended log entry for {date}"))
}
