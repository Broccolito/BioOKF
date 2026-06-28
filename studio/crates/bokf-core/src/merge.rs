//! Merge primitives. `merge_raw` relocates a Secondary KB's `raw/` into the Main KB's
//! `raw/` (dedup by content hash, rename on collision) and returns the id remapping so the
//! caller can rewrite `raw_source` references. `snapshot`/`verify_snapshot` capture the MKB
//! identifier→path set before a merge and confirm it stayed canonical afterward.

use crate::bundle::Bundle;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct MergeRawResult {
    pub moved: Vec<String>,
    pub renamed: Vec<(String, String)>,
    pub dropped_duplicates: Vec<String>,
    /// SKB source-id → final MKB source-id (rewrite `raw_source: [raw/<old>/…]` accordingly).
    pub id_map: BTreeMap<String, String>,
}

fn read_meta_sha(dir: &Path) -> Option<String> {
    let txt = std::fs::read_to_string(dir.join("meta.yaml")).ok()?;
    let m: crate::convert::SourceMeta = serde_yaml::from_str(&txt).ok()?;
    Some(m.sha256)
}

fn raw_sha_map(raw: &Path) -> BTreeMap<String, String> {
    let mut m = BTreeMap::new();
    if let Ok(entries) = std::fs::read_dir(raw) {
        for e in entries.flatten() {
            if e.path().is_dir() {
                if let Some(sha) = read_meta_sha(&e.path()) {
                    m.insert(sha, e.file_name().to_string_lossy().to_string());
                }
            }
        }
    }
    m
}

fn move_dir(from: &Path, to: &Path) -> Result<(), String> {
    if std::fs::rename(from, to).is_ok() {
        return Ok(());
    }
    // cross-filesystem fallback: recursive copy then remove
    copy_dir(from, to)?;
    std::fs::remove_dir_all(from).map_err(|e| e.to_string())
}

fn copy_dir(from: &Path, to: &Path) -> Result<(), String> {
    std::fs::create_dir_all(to).map_err(|e| e.to_string())?;
    for e in std::fs::read_dir(from).map_err(|e| e.to_string())?.flatten() {
        let p = e.path();
        let dest = to.join(e.file_name());
        if p.is_dir() {
            copy_dir(&p, &dest)?;
        } else {
            std::fs::copy(&p, &dest).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

/// Move `skb/raw/<id>/` dirs into `mkb/raw/`: drop true duplicates (same sha256), rename on
/// id collision, and record the id remapping.
pub fn merge_raw(mkb: &Path, skb: &Path) -> Result<MergeRawResult, String> {
    let skb_raw = skb.join("raw");
    let mkb_raw = mkb.join("raw");
    std::fs::create_dir_all(&mkb_raw).map_err(|e| e.to_string())?;
    let mut res = MergeRawResult { moved: vec![], renamed: vec![], dropped_duplicates: vec![], id_map: BTreeMap::new() };
    let mkb_by_sha = raw_sha_map(&mkb_raw);

    let entries = match std::fs::read_dir(&skb_raw) {
        Ok(e) => e,
        Err(_) => return Ok(res),
    };
    for e in entries.flatten() {
        let p = e.path();
        if !p.is_dir() {
            continue;
        }
        let id = e.file_name().to_string_lossy().to_string();
        // duplicate by content?
        if let Some(sha) = read_meta_sha(&p) {
            if let Some(existing) = mkb_by_sha.get(&sha) {
                res.dropped_duplicates.push(id.clone());
                res.id_map.insert(id, existing.clone());
                continue;
            }
        }
        // id collision → rename
        let mut target = id.clone();
        let mut n = 2;
        while mkb_raw.join(&target).exists() {
            target = format!("{id}-{n}");
            n += 1;
        }
        move_dir(&p, &mkb_raw.join(&target))?;
        if target != id {
            res.renamed.push((id.clone(), target.clone()));
        }
        res.moved.push(target.clone());
        res.id_map.insert(id, target);
    }
    Ok(res)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Snapshot {
    /// identifier → relative path, captured before a merge.
    pub identifiers: BTreeMap<String, String>,
}

const SNAPSHOT_FILE: &str = ".bokf-premerge.json";

pub fn snapshot(bundle: &Bundle) -> Snapshot {
    Snapshot {
        identifiers: bundle.nodes.iter().map(|n| (n.identifier.clone(), n.path.to_string_lossy().to_string())).collect(),
    }
}

pub fn write_snapshot(root: &Path, snap: &Snapshot) -> Result<(), String> {
    std::fs::write(root.join(SNAPSHOT_FILE), serde_json::to_string_pretty(snap).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}

/// Compare the current MKB against a pre-merge snapshot; report any MKB identifier that was
/// removed/renamed or whose path changed (the MKB must stay canonical through a merge).
pub fn verify_snapshot(root: &Path, current: &Bundle) -> Result<Vec<String>, String> {
    let txt = std::fs::read_to_string(root.join(SNAPSHOT_FILE)).map_err(|e| format!("no snapshot: {e}"))?;
    let snap: Snapshot = serde_json::from_str(&txt).map_err(|e| e.to_string())?;
    let cur: BTreeMap<String, String> = current.nodes.iter().map(|n| (n.identifier.clone(), n.path.to_string_lossy().to_string())).collect();
    let mut issues = Vec::new();
    for (id, path) in &snap.identifiers {
        match cur.get(id) {
            None => issues.push(format!("MKB identifier `{id}` was removed/renamed — not allowed in a merge")),
            Some(p) if p != path => issues.push(format!("MKB identifier `{id}` moved `{path}` → `{p}` — MKB paths must stay stable")),
            _ => {}
        }
    }
    Ok(issues)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::convert::{ingest, SourceInput};

    #[test]
    fn merge_raw_dedups_shared_and_moves_skb_only() {
        let dir = tempfile::tempdir().unwrap();
        let mkb = dir.path().join("mkb");
        let skb = dir.path().join("skb");
        std::fs::create_dir_all(mkb.join("raw")).unwrap();
        std::fs::create_dir_all(skb.join("raw")).unwrap();
        let shared = tempfile::tempdir().unwrap();
        std::fs::write(shared.path().join("shared.md"), "# Shared Source\nbody").unwrap();
        ingest(&mkb, SourceInput::Path(shared.path().join("shared.md")), false).unwrap();
        ingest(&skb, SourceInput::Path(shared.path().join("shared.md")), false).unwrap();
        let only = tempfile::tempdir().unwrap();
        std::fs::write(only.path().join("only.md"), "# Only In SKB\nbody").unwrap();
        ingest(&skb, SourceInput::Path(only.path().join("only.md")), false).unwrap();

        let res = merge_raw(&mkb, &skb).unwrap();
        assert_eq!(res.dropped_duplicates.len(), 1, "{res:?}");
        assert_eq!(res.moved.len(), 1, "{res:?}");
        assert!(mkb.join("raw").join(&res.moved[0]).join("source.md").exists());
    }

    #[test]
    fn snapshot_flags_removed_mkb_identifier() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join("knowledge/gene")).unwrap();
        std::fs::write(root.join("knowledge/gene/braf.md"), "---\ntype: Gene\nidentifier: BRAF\nsubtype: protein_coding\n---\n# BRAF\n").unwrap();
        let b = crate::bundle::Bundle::open(root).unwrap();
        write_snapshot(root, &snapshot(&b)).unwrap();
        assert!(verify_snapshot(root, &b).unwrap().is_empty());
        std::fs::remove_file(root.join("knowledge/gene/braf.md")).unwrap();
        let b2 = crate::bundle::Bundle::open(root).unwrap();
        assert!(verify_snapshot(root, &b2).unwrap().iter().any(|i| i.contains("BRAF")));
    }
}
