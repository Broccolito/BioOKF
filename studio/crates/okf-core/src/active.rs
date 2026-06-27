//! Active-KB pointer: `<root>/.active-kb` = plaintext kb-id of the active graph.
//! Single global pointer (per-session scoping is a deferred extension).

use std::path::{Path, PathBuf};
use std::sync::Mutex;

static LOCK: Mutex<()> = Mutex::new(());

fn path_of(root: &Path) -> PathBuf {
    root.join(".active-kb")
}

pub fn get_active(root: &Path) -> Option<String> {
    let _g = LOCK.lock();
    std::fs::read_to_string(path_of(root))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

pub fn set_active(root: &Path, id: Option<&str>) -> Result<(), String> {
    let _g = LOCK.lock();
    let p = path_of(root);
    match id {
        Some(id) => {
            crate::registry::validate_kb_id(id)?;
            let tmp = root.join(".active-kb.tmp");
            std::fs::write(&tmp, id).map_err(|e| e.to_string())?;
            std::fs::rename(&tmp, &p).map_err(|e| e.to_string())
        }
        None => {
            let _ = std::fs::remove_file(&p);
            Ok(())
        }
    }
}
