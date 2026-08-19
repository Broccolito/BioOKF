//! Known-bundle registry: `<root>/registry.yaml` = `{ bases: [ {id, path} ] }`.
//! Plus `validate_kb_id`, the kb-id charset rule shared with the active pointer.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Base {
    pub id: String,
    pub path: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Registry {
    #[serde(default)]
    pub bases: Vec<Base>,
}

fn path_of(root: &Path) -> PathBuf {
    root.join("registry.yaml")
}

pub fn load(root: &Path) -> Registry {
    std::fs::read_to_string(path_of(root))
        .ok()
        .and_then(|s| serde_yaml::from_str(&s).ok())
        .unwrap_or_default()
}

fn save(root: &Path, reg: &Registry) -> Result<(), String> {
    let p = path_of(root);
    let tmp = root.join("registry.yaml.tmp");
    std::fs::write(&tmp, serde_yaml::to_string(reg).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, &p).map_err(|e| e.to_string())
}

/// Validate a base `path` before registering it (AUDIT C6): reject empty, reject
/// a relative path containing a `..` component, and require the path to exist.
fn validate_base_path(path: &str) -> Result<(), String> {
    if path.trim().is_empty() {
        return Err("base path must not be empty".into());
    }
    let p = Path::new(path);
    if p.is_relative()
        && p.components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err(format!("base path `{path}` must not contain `..`"));
    }
    if !p.exists() {
        return Err(format!("base path `{path}` does not exist"));
    }
    Ok(())
}

pub fn register(root: &Path, id: &str, path: &str) -> Result<(), String> {
    validate_kb_id(id)?;
    validate_base_path(path)?;
    // Best-effort against the load->save race (AUDIT C6): the CLI/MCP/Studio share
    // this file, so we load immediately before save (narrowing the window) and
    // save atomically via a temp-file rename. A cross-process advisory lock would
    // be stronger but is intentionally kept simple here.
    let mut reg = load(root);
    if reg.bases.iter().any(|b| b.id == id) {
        return Err(format!("kb-id `{id}` already registered"));
    }
    reg.bases.push(Base {
        id: id.to_string(),
        path: path.to_string(),
    });
    save(root, &reg)
}

pub fn unregister(root: &Path, id: &str) -> Result<(), String> {
    let mut reg = load(root);
    reg.bases.retain(|b| b.id != id);
    save(root, &reg)
}

pub fn list(root: &Path) -> Vec<Base> {
    load(root).bases
}

pub fn resolve(root: &Path, id: &str) -> Option<String> {
    load(root)
        .bases
        .into_iter()
        .find(|b| b.id == id)
        .map(|b| b.path)
}

/// kb-id rule: non-empty, ≤64, `[a-z0-9-]`, no leading/trailing/double `-`.
pub fn validate_kb_id(id: &str) -> Result<(), String> {
    if id.is_empty() || id.len() > 64 {
        return Err("kb-id must be 1..=64 chars".into());
    }
    if !id
        .bytes()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
    {
        return Err("kb-id must be [a-z0-9-]".into());
    }
    if id.starts_with('-') || id.ends_with('-') || id.contains("--") {
        return Err("kb-id must not have leading/trailing/double '-'".into());
    }
    Ok(())
}
