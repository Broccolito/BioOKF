//! Content-true figure naming: rename a provisional figure to a content slug and keep
//! every reference consistent (the file under `raw/<id>/figures/`, the reference in
//! `source.md`, and the `FigureMeta` in `meta.yaml`). Runs through `bokf`, so it never
//! trips the raw-guard hook.

use crate::convert::{ext_of, slug, SourceMeta};
use std::path::Path;

/// Rename a provisional figure to a content slug and rewrite every reference.
///
/// `current_rel` is the figure's current path relative to `raw/<id>/`, e.g.
/// "figures/fig-001.png". Returns the new relative figure path.
pub fn name_figure(bundle_root: &Path, source_id: &str, current_rel: &str, caption: &str) -> Result<String, String> {
    let src_dir = bundle_root.join("raw").join(source_id);
    let cur_path = src_dir.join(current_rel);
    if !cur_path.exists() {
        return Err(format!("figure not found: {current_rel}"));
    }
    let ext = ext_of(current_rel);
    let base = slug(caption);
    let base = if base.is_empty() { "figure".to_string() } else { base };
    let new_name = if ext.is_empty() { base } else { format!("{base}.{ext}") };
    let new_rel = format!("figures/{new_name}");
    let new_path = src_dir.join(&new_rel);

    // Move the file (git-friendly rename).
    if new_path != cur_path {
        if let Some(parent) = new_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::rename(&cur_path, &new_path).map_err(|e| e.to_string())?;
    }

    // Rewrite the matching reference in source.md.
    let md_path = src_dir.join("source.md");
    if let Ok(md) = std::fs::read_to_string(&md_path) {
        let rewritten = md.replace(current_rel, &new_rel);
        if rewritten != md {
            std::fs::write(&md_path, rewritten).map_err(|e| e.to_string())?;
        }
    }

    // Update the matching FigureMeta in meta.yaml.
    let meta_path = src_dir.join("meta.yaml");
    let txt = std::fs::read_to_string(&meta_path).map_err(|e| e.to_string())?;
    let mut meta: SourceMeta = serde_yaml::from_str(&txt).map_err(|e| e.to_string())?;
    for f in meta.figures.iter_mut() {
        if f.file == current_rel {
            f.file = new_rel.clone();
            f.provisional = false;
        }
    }
    std::fs::write(&meta_path, serde_yaml::to_string(&meta).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;

    Ok(new_rel)
}

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
            ..Default::default()
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
