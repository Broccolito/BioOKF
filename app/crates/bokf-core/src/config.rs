//! Canonical config dir for BioOKF: holds `registry.yaml` + `.active-kb`.
//! Resolution order (highest precedence first):
//!   1. `BIOOKF_CONFIG_DIR` env
//!   2. `$XDG_CONFIG_HOME/biookf-studio` if `XDG_CONFIG_HOME` set (unix)
//!   3. `~/.config/biookf-studio` (unix) / `%APPDATA%\biookf-studio` (windows)
//!
//! Centralizing this is what stops `registry.yaml`/`.active-kb` from scattering
//! onto the Desktop: every caller (CLI, MCP, Studio) resolves the same dir here.
//! `OKF_ROOT` is deliberately *not* a config-dir fallback anymore; it is only
//! consulted as a legacy migration source by `ensure_config_dir`.

use std::path::PathBuf;

const APP_DIR: &str = "biookf-studio";

/// Resolve the config dir without creating it.
pub fn config_dir() -> PathBuf {
    if let Some(p) = std::env::var_os("BIOOKF_CONFIG_DIR") {
        if !p.is_empty() {
            return PathBuf::from(p);
        }
    }
    if cfg!(windows) {
        if let Some(p) = dirs::config_dir() {
            return p.join(APP_DIR);
        }
    } else {
        if let Some(x) = std::env::var_os("XDG_CONFIG_HOME") {
            let x = PathBuf::from(x);
            if !x.as_os_str().is_empty() {
                return x.join(APP_DIR);
            }
        }
        if let Some(home) = dirs::home_dir() {
            return home.join(".config").join(APP_DIR);
        }
    }
    // Last resort: a relative dir (keeps behavior deterministic in odd envs).
    PathBuf::from(".biookf-studio")
}

/// Create the config dir (idempotent) and run the one-time legacy migration.
pub fn ensure_config_dir() -> Result<PathBuf, String> {
    let dir = config_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    migrate_legacy(&dir);
    Ok(dir)
}

/// Seed `<dir>/registry.yaml` from a legacy location if the config dir has none.
/// Idempotent: never overwrites an existing config-dir registry.
fn migrate_legacy(dir: &std::path::Path) {
    let target = dir.join("registry.yaml");
    if target.exists() {
        return;
    }
    // Candidate legacy roots: the current working dir, and the dir given by
    // OKF_ROOT (the old default). First one that has a registry wins.
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd);
    }
    if let Some(p) = std::env::var_os("OKF_ROOT") {
        if !p.is_empty() {
            candidates.push(PathBuf::from(p));
        }
    }
    for c in candidates {
        let legacy = c.join("registry.yaml");
        if legacy.exists() && legacy != target {
            if std::fs::copy(&legacy, &target).is_ok() {
                let la = c.join(".active-kb");
                let ta = dir.join(".active-kb");
                if la.exists() && !ta.exists() {
                    let _ = std::fs::copy(&la, &ta);
                }
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Env is process-global; serialize these tests.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn clear() {
        std::env::remove_var("BIOOKF_CONFIG_DIR");
        std::env::remove_var("OKF_ROOT");
        std::env::remove_var("XDG_CONFIG_HOME");
    }

    #[test]
    fn explicit_override_wins() {
        let _g = ENV_LOCK.lock().unwrap();
        clear();
        std::env::set_var("BIOOKF_CONFIG_DIR", "/tmp/bokf-cfg-test");
        assert_eq!(config_dir(), PathBuf::from("/tmp/bokf-cfg-test"));
        clear();
    }

    #[test]
    fn xdg_is_respected_on_unix() {
        let _g = ENV_LOCK.lock().unwrap();
        clear();
        if cfg!(windows) {
            return;
        }
        std::env::set_var("XDG_CONFIG_HOME", "/tmp/xdg");
        assert_eq!(config_dir(), PathBuf::from("/tmp/xdg/biookf-studio"));
        clear();
    }

    #[test]
    fn okf_root_does_not_override_global_config_location() {
        let _g = ENV_LOCK.lock().unwrap();
        clear();
        if cfg!(windows) {
            return;
        }
        std::env::set_var("XDG_CONFIG_HOME", "/tmp/xdg-global");
        std::env::set_var("OKF_ROOT", "/tmp/project-local-kb-root");
        assert_eq!(config_dir(), PathBuf::from("/tmp/xdg-global/biookf-studio"));
        clear();
    }

    #[test]
    fn ensure_creates_and_migrates() {
        let _g = ENV_LOCK.lock().unwrap();
        clear();
        let tmp = std::env::temp_dir().join(format!("bokf-cfg-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        let cfg = tmp.join("cfg");
        let legacy = tmp.join("legacy");
        std::fs::create_dir_all(&legacy).unwrap();
        std::fs::write(
            legacy.join("registry.yaml"),
            "bases:\n- id: x\n  path: /x\n",
        )
        .unwrap();
        // BIOOKF_CONFIG_DIR points at the new config dir and takes precedence;
        // OKF_ROOT names the legacy dir to migrate from.
        std::env::set_var("BIOOKF_CONFIG_DIR", &cfg);
        std::env::set_var("OKF_ROOT", &legacy);
        let got = ensure_config_dir().unwrap();
        assert_eq!(got, cfg);
        let seeded = std::fs::read_to_string(cfg.join("registry.yaml")).unwrap();
        assert!(seeded.contains("id: x"));
        clear();
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
