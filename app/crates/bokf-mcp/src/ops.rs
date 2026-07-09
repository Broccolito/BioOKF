//! Filesystem operations behind the MCP tools, with path-safety guards. These
//! are the thin idempotent primitives the AI agent drives; all heavy logic lives
//! in `bokf-core`.

use bokf_core::parse::parse_node;
use std::path::{Component, Path, PathBuf};

/// Reserved plain-text files that live at the bundle root and may be read/written.
const ROOT_TEXT_FILES: [&str; 4] = ["index.md", "log.md", "SCHEMA.md", "README.md"];

/// String-level sanitization of a caller-supplied relative page path: reject
/// absolute paths, `..` traversal, drive prefixes, and hidden (dot) segments.
/// This is only the first gate; real containment is enforced by canonicalizing
/// against the bundle root (see [`contained_read_path`]/[`contained_write_path`]).
fn clean_bundle_rel(page: &str) -> Result<PathBuf, String> {
    let p = Path::new(page);
    if p.is_absolute()
        || p.components().any(|c| {
            matches!(c, Component::ParentDir | Component::Prefix(_) | Component::RootDir)
        })
    {
        return Err("path not permitted".into());
    }
    if p.components().any(|c| match c {
        Component::Normal(s) => s.to_string_lossy().starts_with('.'),
        _ => false,
    }) {
        return Err("path not permitted".into());
    }
    Ok(p.to_path_buf())
}

fn is_root_text_file(rel: &Path) -> bool {
    rel.components().count() == 1
        && rel
            .to_str()
            .map(|s| ROOT_TEXT_FILES.contains(&s))
            .unwrap_or(false)
}

fn is_knowledge_markdown(rel: &Path) -> bool {
    rel.starts_with("knowledge") && rel.extension().and_then(|e| e.to_str()) == Some("md")
}

/// Allow-list for pages that may be READ: knowledge/*.md + the root text files.
fn is_readable_bundle_content(rel: &Path) -> bool {
    is_knowledge_markdown(rel) || is_root_text_file(rel)
}

/// Resolve an EXISTING file inside the bundle, defeating symlink escapes by
/// canonicalizing both the bundle root and the target and requiring the target
/// to stay under the root. Mirrors the Studio's `safe_existing_bundle_path`.
fn safe_existing_bundle_path(bundle: &Path, rel: &Path) -> Result<PathBuf, String> {
    let root_c = bundle.canonicalize().map_err(|_| "path not permitted".to_string())?;
    let full_c = bundle
        .join(rel)
        .canonicalize()
        .map_err(|_| "path not permitted".to_string())?;
    if !full_c.starts_with(&root_c) {
        return Err("path not permitted".into());
    }
    Ok(full_c)
}

/// Resolve a WRITE target that may not exist yet. The file's parent directory is
/// canonicalized (creating it under the bundle if needed) and required to stay
/// inside the canonicalized bundle root, so a symlinked parent cannot escape.
/// Returns the concrete path to write (canonical parent joined with the file name).
fn safe_write_bundle_path(bundle: &Path, rel: &Path) -> Result<PathBuf, String> {
    let root_c = bundle.canonicalize().map_err(|_| "path not permitted".to_string())?;
    let file_name = rel
        .file_name()
        .ok_or_else(|| "path not permitted".to_string())?;
    let parent_rel = rel.parent().unwrap_or_else(|| Path::new(""));
    let parent = bundle.join(parent_rel);
    std::fs::create_dir_all(&parent).map_err(|_| "path not permitted".to_string())?;
    let parent_c = parent.canonicalize().map_err(|_| "path not permitted".to_string())?;
    if !parent_c.starts_with(&root_c) {
        return Err("path not permitted".into());
    }
    Ok(parent_c.join(file_name))
}

pub fn write_page(bundle: &Path, page: &str, content: &str) -> Result<String, String> {
    let rel = clean_bundle_rel(page)?;
    let is_concept = is_knowledge_markdown(&rel);
    if !(is_concept || is_root_text_file(&rel)) {
        return Err("path not permitted".into());
    }

    // Validate concept documents BEFORE touching disk: a bad write must never
    // land on (and corrupt) the node. Non-concept root files are written as-is.
    let report = if is_concept {
        match parse_node(content, &rel) {
            Ok(n) => format!(
                "wrote {page}; parsed OK: type={}, identifier={:?}, {} edge(s){}",
                n.node_type.as_str(),
                n.identifier,
                n.edges.len(),
                if n.node_type.is_valid() { "" } else { "  [WARNING: invalid type]" }
            ),
            Err(e) => {
                return Err(format!(
                    "not written: content does NOT parse as a valid concept document: {e}. Fix and rewrite."
                ))
            }
        }
    } else {
        format!("wrote {page}")
    };

    let full = safe_write_bundle_path(bundle, &rel)?;
    // Temp sibling with a distinct, collision-resistant name, then atomic rename.
    let mut tmp_name = full
        .file_name()
        .map(|s| s.to_os_string())
        .ok_or_else(|| "cannot write page".to_string())?;
    tmp_name.push(".tmp");
    let tmp = full.with_file_name(tmp_name);
    std::fs::write(&tmp, content).map_err(|_| "cannot write page".to_string())?;
    if std::fs::rename(&tmp, &full).is_err() {
        let _ = std::fs::remove_file(&tmp);
        return Err("cannot write page".into());
    }

    Ok(report)
}

pub fn read_page(bundle: &Path, page: &str) -> Result<String, String> {
    let rel = clean_bundle_rel(page)?;
    if !is_readable_bundle_content(&rel) {
        return Err("path not permitted".into());
    }
    let full = safe_existing_bundle_path(bundle, &rel)?;
    std::fs::read_to_string(&full).map_err(|_| "cannot read page".to_string())
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
                notes.push(format!(
                    "type `{}` is NOT one of the 28 controlled types",
                    n.raw_type
                ));
            }
            if n.subtype.is_none() {
                notes.push("no subtype (expected, agent-coined)".into());
            }
            for e in &n.edges {
                if !e.predicate.is_valid() {
                    notes.push(format!(
                        "predicate `{}` is NOT one of the 35",
                        e.raw_predicate
                    ));
                }
                if e.knowledge_level.is_none()
                    || e.agent_type.is_none()
                    || e.primary_source.is_none()
                {
                    notes.push(format!(
                        "edge `{} -> {}` is missing part of the provenance triplet",
                        e.predicate.as_str(),
                        e.object
                    ));
                }
            }
            Ok(format!(
                "VALID concept document: type={}, identifier={:?}, {} edge(s).{}",
                n.node_type.as_str(),
                n.identifier,
                n.edges.len(),
                if notes.is_empty() {
                    String::new()
                } else {
                    format!("\nNotes:\n - {}", notes.join("\n - "))
                }
            ))
        }
        Err(e) => Err(format!("INVALID: {e}")),
    }
}

/// Validate a caller-supplied bundle location for scaffolding a NEW bundle.
/// The bundle dir itself may not exist yet, but its parent must, and the path
/// must not contain `..` traversal. Errors are generic (no path leaking).
fn validate_scaffold_bundle(bundle: &Path) -> Result<(), String> {
    if bundle.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err("path not permitted".into());
    }
    let parent = bundle.parent().unwrap_or_else(|| Path::new("."));
    let parent = if parent.as_os_str().is_empty() {
        Path::new(".")
    } else {
        parent
    };
    if !parent.is_dir() {
        return Err("path not permitted".into());
    }
    Ok(())
}

pub fn scaffold(bundle: &Path, name: &str) -> Result<String, String> {
    validate_scaffold_bundle(bundle)?;
    std::fs::create_dir_all(bundle.join("raw")).map_err(|_| "cannot create bundle".to_string())?;
    std::fs::create_dir_all(bundle.join("knowledge"))
        .map_err(|_| "cannot create bundle".to_string())?;
    let write_absent = |rel: &str, content: String| -> Result<(), String> {
        let p = bundle.join(rel);
        if !p.exists() {
            std::fs::write(&p, content).map_err(|_| "cannot create bundle".to_string())?;
        }
        Ok(())
    };
    write_absent(
        "index.md",
        format!(
            "# {name}\n\nokf_version: 0.5\nbiookf_version: 0.5\n\n> Catalog of concept pages.\n"
        ),
    )?;
    write_absent("log.md", format!("# Change log: {name}\n"))?;
    write_absent(
        "SCHEMA.md",
        "# BioOKF operating schema (v0.5)\n\n28 node types, 35 forward-only predicates (24 positive + 11 negative). See the canonical SCHEMA.md.\n".to_string(),
    )?;

    // version-track + register + activate the new bundle (mirror the CLI).
    let repo = bokf_core::git::GitRepo::open(bundle);
    if repo.ensure_repo().is_ok() {
        let _ = repo.commit_all(
            bokf_core::git::ChangeKind::Manual,
            &format!("create knowledge base {name}"),
            None,
        );
    }
    if let (Some(id), Ok(root)) = (
        bundle
            .file_name()
            .map(|s| s.to_string_lossy().to_lowercase()),
        bokf_core::config::ensure_config_dir(),
    ) {
        if bokf_core::registry::validate_kb_id(&id).is_ok() {
            let abs = std::fs::canonicalize(bundle).unwrap_or_else(|_| bundle.to_path_buf());
            let _ = bokf_core::registry::register(&root, &id, &abs.to_string_lossy());
            let _ = bokf_core::active::set_active(&root, Some(&id));
        }
    }
    Ok(format!(
        "scaffolded BioOKF bundle '{name}' at {}",
        bundle.display()
    ))
}

pub fn list_bases(root: &Path) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(root) {
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() && (p.join("knowledge").is_dir() || p.join("index.md").is_file()) {
                out.push(
                    p.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                );
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
    std::fs::write(&path, new).map_err(|_| "cannot write log".to_string())?;
    Ok(format!("appended log entry for {date}"))
}
