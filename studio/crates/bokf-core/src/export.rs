//! Build a self-contained bundle document (graph + per-node detail + lint
//! summary): the JSON the GUI consumes. Shared by the CLI (`bokf export`) and
//! the Tauri commands so both surfaces return identical shapes.

use crate::{lint, Bundle, Graph};
use std::path::Path;

/// Read and parse the on-disk provenance for one raw source: `<bundle_root>/raw/<source_id>/meta.yaml`.
/// Returns the full [`crate::convert::SourceMeta`] (source type, credibility, figures, ids …) as JSON
/// so the GUI can render figures / credibility / source type. Additive — does not touch the spec.
pub fn source_info(bundle_root: &Path, source_id: &str) -> Result<serde_json::Value, String> {
    let meta_path = bundle_root.join("raw").join(source_id).join("meta.yaml");
    let txt = std::fs::read_to_string(&meta_path)
        .map_err(|e| format!("cannot read {}: {e}", meta_path.display()))?;
    let meta: crate::convert::SourceMeta =
        serde_yaml::from_str(&txt).map_err(|e| format!("invalid meta.yaml for source `{source_id}`: {e}"))?;
    serde_json::to_value(&meta).map_err(|e| e.to_string())
}

/// List the raw source ids present under `<bundle_root>/raw/` (each subdir with a `meta.yaml`).
pub fn list_sources(bundle_root: &Path) -> Vec<String> {
    let mut ids = Vec::new();
    if let Ok(rd) = std::fs::read_dir(bundle_root.join("raw")) {
        for e in rd.flatten() {
            if e.path().join("meta.yaml").is_file() {
                if let Some(name) = e.file_name().to_str() {
                    ids.push(name.to_string());
                }
            }
        }
    }
    ids.sort();
    ids
}

/// The most recent change date for a bundle: the newest `## YYYY-MM-DD` heading in
/// `log.md` (the convention is newest-first), else `None`.
pub fn last_updated(root: impl AsRef<Path>) -> Option<String> {
    let log = std::fs::read_to_string(root.as_ref().join("log.md")).ok()?;
    for line in log.lines() {
        if let Some(rest) = line.trim_start().strip_prefix("## ") {
            let d = rest.trim();
            if !d.is_empty() {
                return Some(d.to_string());
            }
        }
    }
    None
}

/// Raw `log.md` content (the change history), or empty string if absent.
pub fn change_log(root: impl AsRef<Path>) -> String {
    std::fs::read_to_string(root.as_ref().join("log.md")).unwrap_or_default()
}

pub fn bundle_doc(root: impl AsRef<Path>, name: Option<String>) -> std::io::Result<serde_json::Value> {
    let root = root.as_ref();
    let bundle = Bundle::open(root)?;
    let graph = Graph::from_bundle(&bundle);
    let report = lint(&bundle);
    let id = root
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "bundle".to_string());
    let name = name.unwrap_or_else(|| id.clone());

    let mut pages = serde_json::Map::new();
    for n in &bundle.nodes {
        if let Ok(v) = serde_json::to_value(n) {
            pages.insert(n.identifier.clone(), v);
        }
    }

    Ok(serde_json::json!({
        "id": id,
        "name": name,
        "node_count": bundle.nodes.len(),
        "edge_count": graph.edges.iter().filter(|e| !e.synthesized).count(),
        "updated": last_updated(root),
        "log": change_log(root),
        "lint": { "errors": report.errors(), "warnings": report.warnings(), "infos": report.infos() },
        "graph": graph.to_json(),
        "pages": pages,
    }))
}

/// A lightweight index entry for the sidebar (no graph/pages payload).
pub fn base_info(root: impl AsRef<Path>) -> std::io::Result<serde_json::Value> {
    let root = root.as_ref();
    let bundle = Bundle::open(root)?;
    let graph = Graph::from_bundle(&bundle);
    let report = lint(&bundle);
    let id = root.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    Ok(serde_json::json!({
        "id": id,
        "name": id,
        "node_count": bundle.nodes.len(),
        "edge_count": graph.edges.iter().filter(|e| !e.synthesized).count(),
        "updated": last_updated(root),
        "lint": { "errors": report.errors(), "warnings": report.warnings(), "infos": report.infos() },
    }))
}
