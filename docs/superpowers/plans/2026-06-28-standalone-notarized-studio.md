# Standalone Notarized BioOKF Studio — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship BioOKF Studio as a standalone, notarized macOS `.dmg` that centralizes all config under `~/.config/biookf-studio/`, bundles the `bokf`/`bokf-mcp` binaries, and pops up a CLI-install prompt 5s after launch when `bokf` is not on PATH.

**Architecture:** Add a single `bokf-core::config::config_dir()` used by core, CLI, MCP, and Studio so `registry.yaml`/`.active-kb` stop scattering. Bundle the CLI binaries inside the `.app` and add Tauri commands (`cli_status`, `install_cli`) plus a front-end modal. Sign + notarize locally with the UCSF Developer ID cert and publish the DMG to a new `v0.2.0` release.

**Tech Stack:** Rust (workspace: `bokf-core`, `bokf-cli`, `bokf-mcp`, Tauri v2 `biookf-studio`), Tauri CLI, plain HTML/JS front-end (`app/studio/dist`), `osascript`, Apple `notarytool`/`stapler`, `gh`.

## Global Constraints

- Use **Rust** for all implementation/tooling; only throwaway drivers may be another language.
- Naming: always `bokf-*` / `bokf_*` / `biookf-*`; never `okf-xxx`. The CLI binary is `bokf`.
- Config dir: `~/.config/biookf-studio` (respect `$XDG_CONFIG_HOME` → `$XDG_CONFIG_HOME/biookf-studio`; Windows `%APPDATA%\biookf-studio`). Override env (highest first): `BIOOKF_CONFIG_DIR`, then `OKF_ROOT`.
- Copy/UI: no em dashes, no AI-tell wording; squared/flat minimalism; verify UI visually.
- DMG: Apple Silicon (`aarch64-apple-darwin`) only. Sign with `Developer ID Application: University of California at San Francisco (F3YYBXAFJ8)`. Notarize + staple locally.
- Release: new tag `v0.2.0`; version `0.2.0` across all manifests.
- Secrets live in `notarization/` and MUST be gitignored; never committed.
- Run all `cargo` commands from `app/` (the workspace root).

## File Structure

- Create `app/crates/bokf-core/src/config.rs` — config dir resolution + ensure + one-time migration.
- Modify `app/crates/bokf-core/src/lib.rs` — `pub mod config;`.
- Modify `app/Cargo.toml` — add `dirs` workspace dep.
- Modify `app/crates/bokf-core/Cargo.toml` — depend on `dirs`.
- Modify `app/crates/bokf-cli/src/main.rs` — scaffold autoregister + set-active/register/get-active use `config_dir()`.
- Modify `app/crates/bokf-mcp/src/ops.rs` — scaffold autoregister uses `config_dir()`.
- Modify `app/crates/bokf-mcp/src/main.rs` — active/registry handlers + `spawn_studio` default root to `config_dir()`.
- Modify `app/studio/src-tauri/src/main.rs` — `repo_root()`→`config_root()`; add `cli_status`/`install_cli` commands; PTY PATH; register commands.
- Modify `app/studio/src-tauri/Cargo.toml` — depend on `bokf-core` (already?) and `dirs` if needed.
- Modify `app/studio/src-tauri/tauri.conf.json` — version `0.2.0`, `bundle.resources` (bin), `bundle.macOS` signing/dmg/entitlements.
- Create `app/studio/src-tauri/bin/.gitkeep` — staging dir for bundled binaries (binaries themselves gitignored).
- Create `app/studio/src-tauri/entitlements.plist` — hardened-runtime entitlements.
- Modify `app/studio/dist/index.html`, `app/studio/dist/app.js` (+ inline CSS) — CLI-install modal.
- Modify `app/studio/tests/visual.spec.mjs` — popup visual test.
- Create `notarization/` — credentials env, `.p12`, entitlements, `notarization.md` (gitignored).
- Modify `.gitignore` — add `notarization/` and `app/studio/src-tauri/bin/bokf*`.
- Modify `app/Cargo.toml` (workspace version) + `plugins/biookf/.claude-plugin/plugin.json` + `app/.claude-plugin/plugin.json` — version `0.2.0`.
- Modify `README.md` — new install flow.

---

### Task 1: `bokf-core::config` module (config dir + migration)

**Files:**
- Create: `app/crates/bokf-core/src/config.rs`
- Modify: `app/crates/bokf-core/src/lib.rs` (add `pub mod config;`)
- Modify: `app/Cargo.toml` (workspace dep), `app/crates/bokf-core/Cargo.toml` (use dep)
- Test: inline `#[cfg(test)]` in `config.rs`

**Interfaces:**
- Produces: `bokf_core::config::config_dir() -> std::path::PathBuf`, `bokf_core::config::ensure_config_dir() -> Result<std::path::PathBuf, String>`. `ensure_config_dir` creates the dir and runs one-time migration; `config_dir` is pure resolution (no side effects).

- [ ] **Step 1: Add the `dirs` dependency**

In `app/Cargo.toml` under `[workspace.dependencies]` add:
```toml
dirs = "5"
```
In `app/crates/bokf-core/Cargo.toml` under `[dependencies]` add:
```toml
dirs = { workspace = true }
```

- [ ] **Step 2: Write the failing tests**

Create `app/crates/bokf-core/src/config.rs`:
```rust
//! Canonical config dir for BioOKF: holds `registry.yaml` + `.active-kb`.
//! Resolution order (highest precedence first):
//!   1. `BIOOKF_CONFIG_DIR` env
//!   2. `OKF_ROOT` env (back-compat)
//!   3. `$XDG_CONFIG_HOME/biookf-studio` if `XDG_CONFIG_HOME` set
//!   4. `~/.config/biookf-studio` (unix) / `%APPDATA%\biookf-studio` (windows)

use std::path::PathBuf;

const APP_DIR: &str = "biookf-studio";

/// Resolve the config dir without creating it.
pub fn config_dir() -> PathBuf {
    if let Some(p) = std::env::var_os("BIOOKF_CONFIG_DIR") {
        return PathBuf::from(p);
    }
    if let Some(p) = std::env::var_os("OKF_ROOT") {
        return PathBuf::from(p);
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
    // Candidate legacy roots: current working dir, and the dir given by OKF_ROOT
    // (the old default). First one that has a registry wins.
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd);
    }
    if let Some(p) = std::env::var_os("OKF_ROOT") {
        candidates.push(PathBuf::from(p));
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
    fn ensure_creates_and_migrates() {
        let _g = ENV_LOCK.lock().unwrap();
        clear();
        let tmp = std::env::temp_dir().join(format!("bokf-cfg-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        let cfg = tmp.join("cfg");
        let legacy = tmp.join("legacy");
        std::fs::create_dir_all(&legacy).unwrap();
        std::fs::write(legacy.join("registry.yaml"), "bases:\n- id: x\n  path: /x\n").unwrap();
        std::env::set_var("BIOOKF_CONFIG_DIR", &cfg);
        std::env::set_var("OKF_ROOT", &legacy); // used as a migration candidate
        // OKF_ROOT would normally override config_dir; BIOOKF_CONFIG_DIR takes precedence.
        let got = ensure_config_dir().unwrap();
        assert_eq!(got, cfg);
        let seeded = std::fs::read_to_string(cfg.join("registry.yaml")).unwrap();
        assert!(seeded.contains("id: x"));
        clear();
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
```

- [ ] **Step 3: Register the module** — add to `app/crates/bokf-core/src/lib.rs`:
```rust
pub mod config;
```
(place alongside the other `pub mod` lines).

- [ ] **Step 4: Run tests, expect FAIL then PASS**

Run: `cd app && cargo test -p bokf-core config::`
Expected first run before lib.rs wiring: compile error / not found; after Step 3: PASS (3 tests).

- [ ] **Step 5: Commit**
```bash
git add app/Cargo.toml app/crates/bokf-core/Cargo.toml app/crates/bokf-core/src/config.rs app/crates/bokf-core/src/lib.rs
git commit -m "feat(core): canonical config_dir() under ~/.config/biookf-studio with legacy migration"
```

---

### Task 2: CLI uses the config dir (fix the scatter bug)

**Files:**
- Modify: `app/crates/bokf-cli/src/main.rs` (scaffold autoregister ~L685; `SetActive`/`GetActive`/`Register` defs ~L105-115 + dispatch ~L216-218 + fns ~L414-453)
- Test: `app/crates/bokf-cli/tests/cli.rs`

**Interfaces:**
- Consumes: `bokf_core::config::ensure_config_dir`.
- Produces: scaffolding a KB registers into `config_dir()`, never `path.parent()`. `set-active`/`register`/`get-active` default `--root` to `config_dir()`.

- [ ] **Step 1: Write the failing CLI test**

Append to `app/crates/bokf-cli/tests/cli.rs`:
```rust
#[test]
fn scaffold_registers_into_config_dir_not_parent() {
    let tmp = std::env::temp_dir().join(format!("bokf-cli-scaffold-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    let cfg = tmp.join("cfg");
    let workdir = tmp.join("work");
    std::fs::create_dir_all(&workdir).unwrap();
    let kb = workdir.join("my-kb");

    let status = std::process::Command::new(env!("CARGO_BIN_EXE_bokf"))
        .args(["scaffold", kb.to_str().unwrap(), "--name", "My KB"])
        .env("BIOOKF_CONFIG_DIR", &cfg)
        .status()
        .unwrap();
    assert!(status.success());

    // Registry + active pointer land in the config dir...
    assert!(cfg.join("registry.yaml").exists(), "registry.yaml should be in config dir");
    // ...and NOT scattered next to the new KB.
    assert!(!workdir.join("registry.yaml").exists(), "must not scatter to parent");
    assert!(!workdir.join(".active-kb").exists(), "must not scatter .active-kb to parent");

    let reg = std::fs::read_to_string(cfg.join("registry.yaml")).unwrap();
    assert!(reg.contains("my-kb"));
    let _ = std::fs::remove_dir_all(&tmp);
}
```
(If `cli.rs` has no imports header needed, this is self-contained. The bin name env is `CARGO_BIN_EXE_bokf`.)

- [ ] **Step 2: Run test, expect FAIL**

Run: `cd app && cargo test -p bokf-cli scaffold_registers_into_config_dir_not_parent`
Expected: FAIL (registry.yaml lands in `workdir`, assertion fails).

- [ ] **Step 3: Fix the scaffold autoregister**

In `app/crates/bokf-cli/src/main.rs`, replace the autoregister block (currently using `path.parent()`):
```rust
    let kb_id = path.file_name().map(|s| s.to_string_lossy().to_lowercase());
    if let (Some(id), Some(root)) = (kb_id, path.parent()) {
        if bokf_core::registry::validate_kb_id(&id).is_ok() {
            let abs = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
            let _ = bokf_core::registry::register(root, &id, &abs.to_string_lossy());
            let _ = bokf_core::active::set_active(root, Some(&id));
        }
    }
```
with:
```rust
    let kb_id = path.file_name().map(|s| s.to_string_lossy().to_lowercase());
    if let (Some(id), Ok(root)) = (kb_id, bokf_core::config::ensure_config_dir()) {
        if bokf_core::registry::validate_kb_id(&id).is_ok() {
            let abs = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
            let _ = bokf_core::registry::register(&root, &id, &abs.to_string_lossy());
            let _ = bokf_core::active::set_active(&root, Some(&id));
        }
    }
```

- [ ] **Step 4: Default the explicit-root commands to config_dir**

Change the `Cmd` variants to make `root` optional. In the enum:
```rust
    /// Set the active KB id (defaults to the config dir).
    SetActive {
        #[arg(long)]
        root: Option<PathBuf>,
        kb_id: String,
    },
    /// Print the active KB id + resolved path.
    GetActive {
        #[arg(long)]
        root: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
    /// Register/list/unregister a known bundle.
    Register {
        #[arg(long)]
        root: Option<PathBuf>,
        kb_id: Option<String>,
        #[arg(long)]
        path: Option<PathBuf>,
        #[arg(long)]
        list: bool,
        #[arg(long)]
        unregister: Option<String>,
    },
```
Update dispatch (the three arms) to resolve root:
```rust
        Cmd::SetActive { root, kb_id } => cmd_set_active(resolve_root(root), kb_id),
        Cmd::GetActive { root, json } => cmd_get_active(resolve_root(root), json),
        Cmd::Register { root, kb_id, path, list, unregister } => cmd_register(resolve_root(root), kb_id, path, list, unregister),
```
Add the helper near the top of `main.rs`:
```rust
/// The bundle/config root for registry + active-pointer ops: an explicit
/// `--root` if given, else the canonical config dir.
fn resolve_root(root: Option<PathBuf>) -> PathBuf {
    root.unwrap_or_else(|| bokf_core::config::ensure_config_dir().unwrap_or_else(|_| bokf_core::config::config_dir()))
}
```
(`cmd_set_active`/`cmd_get_active`/`cmd_register` keep their `root: PathBuf` signatures.)

- [ ] **Step 5: Run tests, expect PASS**

Run: `cd app && cargo test -p bokf-cli`
Expected: PASS (existing tests + the new one).

- [ ] **Step 6: Commit**
```bash
git add app/crates/bokf-cli/src/main.rs app/crates/bokf-cli/tests/cli.rs
git commit -m "fix(cli): scaffold + active/registry ops use config_dir, not the KB's parent"
```

---

### Task 3: MCP uses the config dir

**Files:**
- Modify: `app/crates/bokf-mcp/src/ops.rs` (scaffold autoregister ~L141)
- Modify: `app/crates/bokf-mcp/src/main.rs` (`set_active`/`get_active` handlers ~L291-307; `spawn_studio` ~L593-606; `RootParam`/`RootKbParam` if needed)

**Interfaces:**
- Consumes: `bokf_core::config::ensure_config_dir`.
- Produces: MCP scaffold registers into `config_dir()`; `bokf_set_active`/`bokf_get_active` default a missing/empty `root` to `config_dir()`; spawned Studio inherits the config dir (no `OKF_ROOT` override needed).

- [ ] **Step 1: Fix the ops scaffold autoregister**

In `app/crates/bokf-mcp/src/ops.rs`, replace:
```rust
    if let (Some(id), Some(root)) = (bundle.file_name().map(|s| s.to_string_lossy().to_lowercase()), bundle.parent()) {
        if bokf_core::registry::validate_kb_id(&id).is_ok() {
            let abs = std::fs::canonicalize(bundle).unwrap_or_else(|_| bundle.to_path_buf());
            let _ = bokf_core::registry::register(root, &id, &abs.to_string_lossy());
            let _ = bokf_core::active::set_active(root, Some(&id));
        }
    }
```
with:
```rust
    if let (Some(id), Ok(root)) = (bundle.file_name().map(|s| s.to_string_lossy().to_lowercase()), bokf_core::config::ensure_config_dir()) {
        if bokf_core::registry::validate_kb_id(&id).is_ok() {
            let abs = std::fs::canonicalize(bundle).unwrap_or_else(|_| bundle.to_path_buf());
            let _ = bokf_core::registry::register(&root, &id, &abs.to_string_lossy());
            let _ = bokf_core::active::set_active(&root, Some(&id));
        }
    }
```

- [ ] **Step 2: Default the handler root to config_dir**

In `app/crates/bokf-mcp/src/main.rs`, add a helper:
```rust
/// Resolve the MCP root: the caller-provided path if non-empty, else the config dir.
fn mcp_root(provided: &str) -> std::path::PathBuf {
    if provided.trim().is_empty() {
        bokf_core::config::ensure_config_dir().unwrap_or_else(|_| bokf_core::config::config_dir())
    } else {
        std::path::PathBuf::from(provided)
    }
}
```
In `set_active`, replace `Path::new(&p.0.root)` with `&mcp_root(&p.0.root)`:
```rust
        match bokf_core::active::set_active(&mcp_root(&p.0.root), Some(&p.0.kb_id)) {
```
In `get_active`, replace `let root = Path::new(&p.0.root);` with:
```rust
        let root = mcp_root(&p.0.root);
        let root = root.as_path();
```

- [ ] **Step 3: Default the spawned Studio to the config dir**

In `spawn_studio`, change so that when `root` is `None`, the Studio still points at the config dir (Studio also defaults to config_dir on its own, so this is belt-and-suspenders). Replace:
```rust
    if let Some(r) = root {
        cmd.env("OKF_ROOT", r);
    }
```
with:
```rust
    let r = root
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| bokf_core::config::ensure_config_dir().unwrap_or_else(|_| bokf_core::config::config_dir()));
    cmd.env("OKF_ROOT", r);
```

- [ ] **Step 4: Build, expect PASS**

Run: `cd app && cargo build -p bokf-mcp && cargo test -p bokf-mcp`
Expected: builds; tests pass.

- [ ] **Step 5: Commit**
```bash
git add app/crates/bokf-mcp/src/ops.rs app/crates/bokf-mcp/src/main.rs
git commit -m "fix(mcp): scaffold + active/registry default to the config dir"
```

---

### Task 4: Studio config_root + CLI bundle/install commands + terminal PATH

**Files:**
- Modify: `app/studio/src-tauri/src/main.rs`
- Modify: `app/studio/src-tauri/Cargo.toml` (ensure `bokf-core` dep)

**Interfaces:**
- Consumes: `bokf_core::config::config_dir`/`ensure_config_dir`; Tauri v2 `app.path().resource_dir()`.
- Produces: Tauri commands `cli_status() -> serde_json::Value` (`{installed, path, version, bundledVersion}`) and `install_cli() -> Result<String, String>`. The embedded PTY has the bundled bin dir on its PATH.

- [ ] **Step 1: Replace `repo_root()` with `config_root()`**

In `app/studio/src-tauri/src/main.rs` replace:
```rust
/// Root that contains the bundles (env `OKF_ROOT`, else the BioOKF repo root).
fn repo_root() -> PathBuf {
    std::env::var("OKF_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.."))
}
```
with:
```rust
/// Canonical config dir holding `registry.yaml` + `.active-kb`.
fn config_root() -> PathBuf {
    bokf_core::config::ensure_config_dir().unwrap_or_else(|_| bokf_core::config::config_dir())
}
```
Then replace every `repo_root()` call in the file with `config_root()` (3 sites: `registered_bundles`, `resolve`, `set_active_kb`, `get_active_kb`).

- [ ] **Step 2: Add the bundled-bin resolver + `cli_status` + `install_cli` commands**

Add near the top of `main.rs`:
```rust
/// Directory inside the app bundle that holds the shipped `bokf`/`bokf-mcp`.
/// In a packaged `.app` this is `Contents/Resources/bin`; in `cargo run` dev it
/// falls back to the workspace `target/<profile>` dir next to the studio exe.
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

fn bokf_exe_name() -> &'static str {
    if cfg!(windows) { "bokf.exe" } else { "bokf" }
}

/// `true` if `bokf` resolves on the user's PATH (or the standard install path).
fn bokf_on_path() -> Option<String> {
    // Standard install target first.
    let std_path = std::path::Path::new("/usr/local/bin/bokf");
    if std_path.exists() {
        return Some(std_path.display().to_string());
    }
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let cand = dir.join(bokf_exe_name());
        if cand.exists() {
            return Some(cand.display().to_string());
        }
    }
    None
}

fn bokf_version(bin: &std::path::Path) -> Option<String> {
    let out = std::process::Command::new(bin).arg("--version").output().ok()?;
    let s = String::from_utf8_lossy(&out.stdout);
    Some(s.trim().to_string()).filter(|s| !s.is_empty())
}

#[tauri::command]
fn cli_status(app: AppHandle) -> serde_json::Value {
    let installed_path = bokf_on_path();
    let bundled = bundled_bin_dir(&app).map(|d| d.join(bokf_exe_name()));
    let bundled_version = bundled.as_deref().and_then(bokf_version);
    let installed_version = installed_path.as_deref().map(std::path::Path::new).and_then(bokf_version);
    serde_json::json!({
        "installed": installed_path.is_some(),
        "path": installed_path,
        "version": installed_version,
        "bundledVersion": bundled_version,
    })
}

#[tauri::command]
fn install_cli(app: AppHandle) -> Result<String, String> {
    let dir = bundled_bin_dir(&app).ok_or("bundled bokf binary not found")?;
    let src = dir.join(bokf_exe_name());
    if !src.exists() {
        return Err(format!("bundled bokf not found at {}", src.display()));
    }
    let dest = "/usr/local/bin/bokf";
    // One admin prompt: ensure /usr/local/bin exists, copy, chmod +x.
    let script = format!(
        "do shell script \"mkdir -p /usr/local/bin && cp '{}' '{}' && chmod 755 '{}'\" with administrator privileges",
        src.display(), dest, dest
    );
    let out = std::process::Command::new("osascript")
        .arg("-e").arg(&script)
        .output()
        .map_err(|e| format!("failed to launch osascript: {e}"))?;
    if out.status.success() {
        Ok(dest.to_string())
    } else {
        let err = String::from_utf8_lossy(&out.stderr);
        if err.contains("-128") || err.to_lowercase().contains("cancel") {
            Err("install cancelled".to_string())
        } else {
            Err(format!("install failed: {}", err.trim()))
        }
    }
}
```
Ensure `use tauri::Manager;` is present (needed for `app.path()`).

- [ ] **Step 3: Register the commands**

In the `tauri::generate_handler!` / `invoke_handler` list, add `cli_status, install_cli` alongside `set_active_kb, get_active_kb` etc.

- [ ] **Step 4: Put the bundled bin dir on the embedded terminal's PATH**

Find the PTY `CommandBuilder` setup (where the shell is spawned). After building the command, prepend the bundled bin dir:
```rust
if let Some(bin) = bundled_bin_dir(&app_handle) {
    let existing = std::env::var("PATH").unwrap_or_default();
    let joined = format!("{}:{}", bin.display(), existing);
    cmd_builder.env("PATH", joined);
}
```
(Use whatever `AppHandle` is in scope at the PTY spawn site; if none, capture it when the terminal command is set up.)

- [ ] **Step 5: Build the Studio**

Run: `cd app/studio/src-tauri && cargo build`
Expected: compiles. (Functional verification happens in Task 8 manual run.)

- [ ] **Step 6: Commit**
```bash
git add app/studio/src-tauri/src/main.rs app/studio/src-tauri/Cargo.toml
git commit -m "feat(studio): config_root + cli_status/install_cli commands + terminal PATH"
```

---

### Task 5: Front-end CLI-install modal

**Files:**
- Modify: `app/studio/dist/index.html` (modal markup + styles)
- Modify: `app/studio/dist/app.js` (5s timer, invoke `cli_status`/`install_cli`, dismiss/never flags)
- Test: `app/studio/tests/visual.spec.mjs`

**Interfaces:**
- Consumes: Tauri commands `cli_status`, `install_cli` via `window.__TAURI__.core.invoke` (withGlobalTauri is on).
- Produces: a modal `#cli-modal` shown 5s after load when `cli_status().installed === false`, unless a `?forceCliPopup=1` query param forces it on.

- [ ] **Step 1: Add the modal markup + styles to `index.html`**

Insert before `</body>` (match the Studio's squared/flat dark styling; adjust colors to existing CSS vars):
```html
<div id="cli-modal" class="cli-modal" hidden>
  <div class="cli-modal__panel">
    <h2 class="cli-modal__title">Install the bokf CLI</h2>
    <p class="cli-modal__body">
      BioOKF Studio ships with the <code>bokf</code> command line tool. Install it to
      <code>/usr/local/bin</code> so it works in any terminal. You will be asked for your
      password once.
    </p>
    <div id="cli-modal__error" class="cli-modal__error" hidden></div>
    <div class="cli-modal__actions">
      <button id="cli-install" class="cli-btn cli-btn--primary">Install CLI</button>
      <button id="cli-later" class="cli-btn">Later</button>
      <button id="cli-never" class="cli-btn cli-btn--ghost">Don't ask again</button>
    </div>
  </div>
</div>
<style>
  .cli-modal { position: fixed; inset: 0; display: flex; align-items: center; justify-content: center;
    background: rgba(0,0,0,.5); z-index: 9999; }
  .cli-modal[hidden] { display: none; }
  .cli-modal__panel { width: 440px; max-width: calc(100vw - 48px); background: #15171c;
    border: 1px solid #2a2e37; border-radius: 4px; padding: 24px; color: #e6e8ec;
    box-shadow: 0 16px 48px rgba(0,0,0,.4); }
  .cli-modal__title { margin: 0 0 8px; font-size: 16px; font-weight: 600; }
  .cli-modal__body { margin: 0 0 16px; font-size: 13px; line-height: 1.5; color: #aeb4bf; }
  .cli-modal__body code { background: #20242c; padding: 1px 5px; border-radius: 3px; font-size: 12px; }
  .cli-modal__error { margin: 0 0 12px; font-size: 12px; color: #ff7a7a; }
  .cli-modal__error[hidden] { display: none; }
  .cli-modal__actions { display: flex; gap: 8px; justify-content: flex-end; }
  .cli-btn { font: inherit; font-size: 13px; padding: 7px 14px; border-radius: 3px;
    border: 1px solid #2a2e37; background: #20242c; color: #e6e8ec; cursor: pointer; }
  .cli-btn:hover { background: #262b34; }
  .cli-btn--primary { background: #3b82f6; border-color: #3b82f6; color: #fff; }
  .cli-btn--primary:hover { background: #2f6fe0; }
  .cli-btn--ghost { background: transparent; color: #8b919c; }
</style>
```

- [ ] **Step 2: Add the popup logic to `app.js`**

Append:
```js
// --- CLI install popup -------------------------------------------------------
(function () {
  const invoke = () => window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.invoke;
  const NEVER_KEY = "bokf.cliPopup.never";
  const params = new URLSearchParams(location.search);
  const forced = params.get("forceCliPopup") === "1";

  function show() {
    const m = document.getElementById("cli-modal");
    if (m) m.hidden = false;
  }
  function hide() {
    const m = document.getElementById("cli-modal");
    if (m) m.hidden = true;
  }
  function setError(msg) {
    const e = document.getElementById("cli-modal__error");
    if (!e) return;
    e.textContent = msg || "";
    e.hidden = !msg;
  }

  async function maybeShow() {
    if (forced) { show(); return; }
    if (localStorage.getItem(NEVER_KEY) === "1") return;
    const inv = invoke();
    if (!inv) return; // not running in Tauri (plain browser preview)
    try {
      const status = await inv("cli_status");
      if (status && status.installed === false) show();
    } catch (_) { /* ignore */ }
  }

  function wire() {
    const installBtn = document.getElementById("cli-install");
    const laterBtn = document.getElementById("cli-later");
    const neverBtn = document.getElementById("cli-never");
    if (installBtn) installBtn.addEventListener("click", async () => {
      setError("");
      installBtn.disabled = true;
      installBtn.textContent = "Installing...";
      const inv = invoke();
      try {
        if (inv) await inv("install_cli");
        hide();
      } catch (e) {
        setError(String(e && e.message ? e.message : e));
      } finally {
        installBtn.disabled = false;
        installBtn.textContent = "Install CLI";
      }
    });
    if (laterBtn) laterBtn.addEventListener("click", hide);
    if (neverBtn) neverBtn.addEventListener("click", () => {
      localStorage.setItem(NEVER_KEY, "1");
      hide();
    });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", () => { wire(); setTimeout(maybeShow, 5000); });
  } else {
    wire();
    setTimeout(maybeShow, 5000);
  }
})();
```

- [ ] **Step 3: Add the visual test**

Append to `app/studio/tests/visual.spec.mjs` (follow the file's existing harness for serving `dist/` and screenshot dir):
```js
test("cli install popup renders when forced", async ({ page }) => {
  await page.goto(baseURL + "/index.html?forceCliPopup=1");
  await page.waitForSelector("#cli-modal:not([hidden])", { timeout: 3000 });
  await expect(page.locator("#cli-modal .cli-modal__title")).toHaveText(/Install the bokf CLI/);
  await page.screenshot({ path: screensDir + "/cli-popup.png" });
});
```
(Match `baseURL`/`screensDir`/`test`/`expect` to the existing imports in that file; if it uses a different server bootstrap, reuse it verbatim.)

- [ ] **Step 4: Run the visual test + capture screenshot**

Run: `cd app/studio && npx playwright test tests/visual.spec.mjs -g "cli install popup"`
Expected: PASS; `cli-popup.png` written. Inspect the screenshot to confirm styling.

- [ ] **Step 5: Commit**
```bash
git add app/studio/dist/index.html app/studio/dist/app.js app/studio/tests/visual.spec.mjs
git commit -m "feat(studio): 5s CLI-install popup (forced-show test + screenshot)"
```

---

### Task 6: Notarization assets + gitignore

**Files:**
- Create: `notarization/notarization_credentials.env`, `notarization/UCSF-AppleDeveloper-Main_Application.p12`, `notarization/entitlements.plist`, `notarization/notarization.md`
- Modify: `.gitignore`

**Interfaces:** none (build inputs).

- [ ] **Step 1: Update `.gitignore` FIRST (before copying any secret)**

Add to `.gitignore`:
```
# macOS notarization secrets (Developer ID cert, app-specific password)
notarization/

# Bundled CLI binaries staged into the Tauri app (rebuilt each release)
app/studio/src-tauri/bin/bokf
app/studio/src-tauri/bin/bokf-mcp
```

- [ ] **Step 2: Verify the ignore takes effect**

Run: `git check-ignore -v notarization/notarization_credentials.env`
Expected: prints a matching `.gitignore` line (ignored). If nothing prints, STOP and fix before copying secrets.

- [ ] **Step 3: Copy the credentials + cert + entitlements**
```bash
mkdir -p notarization
cp /Users/wanjun/Desktop/BioRouter/notarization/notarization_credentials.env notarization/
cp /Users/wanjun/Desktop/bioscratch/notarization/UCSF-AppleDeveloper-Main_Application.p12 notarization/
cp /Users/wanjun/Desktop/bioscratch/notarization/notarization_entitlements.plist notarization/entitlements.plist
```
Then ensure `notarization_credentials.env` has an `APPLE_PASSWORD` alias (Tauri expects `APPLE_PASSWORD`; BioRouter used `APPLE_APP_SPECIFIC_PASSWORD`). If only the latter exists, the runbook exports `APPLE_PASSWORD` from it.

- [ ] **Step 4: Write `notarization/notarization.md`** (Tauri-adapted runbook)

Document: load env, build binaries, stage into `src-tauri/bin`, `cargo tauri build --bundles dmg`, verify with `spctl`/`stapler`, upload with `gh`. (Content mirrors Task 8 + Task 9.)

- [ ] **Step 5: Confirm git sees nothing under `notarization/`**

Run: `git status --porcelain notarization/`
Expected: empty output (all ignored).

- [ ] **Step 6: Commit only the `.gitignore` change**
```bash
git add .gitignore
git commit -m "chore: gitignore notarization secrets + staged CLI binaries"
```

---

### Task 7: tauri.conf.json — version, resources, macOS signing/dmg

**Files:**
- Modify: `app/studio/src-tauri/tauri.conf.json`
- Create: `app/studio/src-tauri/bin/.gitkeep`

**Interfaces:** none (build config).

- [ ] **Step 1: Create the staging dir**
```bash
mkdir -p app/studio/src-tauri/bin && touch app/studio/src-tauri/bin/.gitkeep
```

- [ ] **Step 2: Edit `tauri.conf.json`**

Set `"version": "0.2.0"`. Extend `bundle`:
```json
  "bundle": {
    "active": true,
    "targets": ["app", "dmg"],
    "resources": {
      "bin/bokf": "bin/bokf",
      "bin/bokf-mcp": "bin/bokf-mcp"
    },
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "macOS": {
      "signingIdentity": "Developer ID Application: University of California at San Francisco (F3YYBXAFJ8)",
      "hardenedRuntime": true,
      "entitlements": "entitlements.plist"
    }
  }
```
Create `app/studio/src-tauri/entitlements.plist`:
```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>com.apple.security.cs.allow-jit</key>
  <true/>
  <key>com.apple.security.cs.allow-unsigned-executable-memory</key>
  <true/>
  <key>com.apple.security.cs.disable-library-validation</key>
  <true/>
</dict>
</plist>
```
(`disable-library-validation` lets the signed `.app` ship the separately-built `bokf` binaries as resources.)

- [ ] **Step 3: Sanity-check JSON**

Run: `cd app/studio/src-tauri && python3 -c "import json;json.load(open('tauri.conf.json'));print('ok')"`
Expected: `ok`.

- [ ] **Step 4: Commit**
```bash
git add app/studio/src-tauri/tauri.conf.json app/studio/src-tauri/bin/.gitkeep app/studio/src-tauri/entitlements.plist
git commit -m "build(studio): v0.2.0, bundle bokf binaries, macOS signing + dmg target"
```

---

### Task 8: Build, sign, notarize, verify the DMG (local)

**Files:** none (runbook). Depends on Tasks 1-7.

- [ ] **Step 1: Load credentials**
```bash
cd /Users/wanjun/Desktop/BioOKF
export APPLE_ID=$(grep '^APPLE_ID=' notarization/notarization_credentials.env | cut -d= -f2)
export APPLE_TEAM_ID=$(grep '^APPLE_TEAM_ID=' notarization/notarization_credentials.env | cut -d= -f2)
export APPLE_PASSWORD=$(grep -E '^(APPLE_PASSWORD|APPLE_APP_SPECIFIC_PASSWORD)=' notarization/notarization_credentials.env | head -1 | cut -d= -f2)
export APPLE_SIGNING_IDENTITY="Developer ID Application: University of California at San Francisco (F3YYBXAFJ8)"
echo "id=${APPLE_ID:0:3}… team=$APPLE_TEAM_ID identity set=${APPLE_SIGNING_IDENTITY:+yes} pw set=${APPLE_PASSWORD:+yes}"
```
Expected: team id printed, `identity set=yes`, `pw set=yes`.

- [ ] **Step 2: Build the release binaries + stage them**
```bash
cd app && cargo build --release -p bokf-cli -p bokf-mcp
cp target/release/bokf target/release/bokf-mcp studio/src-tauri/bin/
```
Expected: both binaries copied into `studio/src-tauri/bin/`.

- [ ] **Step 3: Install tauri-cli if missing**
```bash
cargo tauri --version || cargo install tauri-cli --version "^2.0" --locked
```

- [ ] **Step 4: Build + sign + notarize the DMG**
```bash
cd app/studio/src-tauri && cargo tauri build --bundles dmg
```
Expected: signs the `.app`, submits to notarytool, staples, produces `app/target/release/bundle/dmg/BioOKF Studio_0.2.0_aarch64.dmg`. (Notarization can take a few minutes.)

- [ ] **Step 5: Verify signature + notarization**
```bash
DMG="app/target/release/bundle/dmg/BioOKF Studio_0.2.0_aarch64.dmg"
APP="app/target/release/bundle/macos/BioOKF Studio.app"
codesign --verify --deep --strict --verbose=2 "$APP"
spctl --assess --type execute -vv "$APP"
xcrun stapler validate "$DMG"
```
Expected: `accepted`, `source=Notarized Developer ID`, `The validate action worked!`.

- [ ] **Step 6: Manual smoke test on this Mac**

Open the DMG, drag the app to a temp location (or `/Applications`), launch it. Confirm: (a) launches with no Gatekeeper block; (b) after 5s the CLI popup appears (only if `bokf` not already on PATH); (c) clicking **Install CLI** → admin prompt → `which bokf` resolves to `/usr/local/bin/bokf` in a fresh Terminal; (d) creating/scaffolding a KB writes `registry.yaml` into `~/.config/biookf-studio/`, with nothing new dropped on the Desktop.
Record results in the task notes. If any fail, return to the relevant task.

- [ ] **Step 7: (no commit — build artifacts are gitignored)**

---

### Task 9: Release v0.2.0 + docs

**Files:**
- Modify: `app/Cargo.toml` (workspace `version = "0.2.0"`), `plugins/biookf/.claude-plugin/plugin.json`, `app/.claude-plugin/plugin.json`, `README.md`

- [ ] **Step 1: Bump versions**

Set `version = "0.2.0"` in `app/Cargo.toml` `[workspace.package]`; `"version": "0.2.0"` in both `plugin.json` files. (`tauri.conf.json` already set in Task 7.)
Run: `cd app && cargo build` to refresh `Cargo.lock`.

- [ ] **Step 2: Update README install flow**

Document: **1.** download + install the notarized DMG; **2.** the Studio installs the `bokf` CLI (popup); **3.** install the `biookf` Claude Code cloud plugin separately to drive the Studio via Claude. No em dashes / AI-tell phrasing.

- [ ] **Step 3: Commit + open PR (do NOT push to main directly)**
```bash
git add app/Cargo.toml app/Cargo.lock plugins/biookf/.claude-plugin/plugin.json app/.claude-plugin/plugin.json README.md
git commit -m "release: v0.2.0 (standalone notarized Studio, config dir, CLI popup)"
git push -u origin standalone-notarized-studio
gh pr create --fill --base main
```

- [ ] **Step 4: Merge to main, then tag (after user approves the PR)**
```bash
gh pr merge --squash --delete-branch   # or per user's preference
git checkout main && git pull
git tag v0.2.0 && git push origin v0.2.0   # triggers CI multi-platform bundles
```

- [ ] **Step 5: Upload the notarized DMG to the v0.2.0 release**
```bash
cd /Users/wanjun/Desktop/BioOKF
# wait for the release to exist (CI creates it on tag), or create it:
gh release view v0.2.0 >/dev/null 2>&1 || gh release create v0.2.0 --title "BioOKF v0.2.0" --generate-notes
gh release upload v0.2.0 "app/target/release/bundle/dmg/BioOKF Studio_0.2.0_aarch64.dmg" --clobber
```
Expected: DMG asset attached to the v0.2.0 release.

- [ ] **Step 6: Verify the published asset**
```bash
gh release view v0.2.0 --json assets --jq '.assets[].name'
```
Expected: the arm64 DMG listed alongside the CI bundles.

---

## Self-Review

**Spec coverage:**
- Config dir centralization → Tasks 1-4. ✓
- Scatter fix (CLI + MCP scaffold) → Tasks 2, 3. ✓
- Migration of existing scattered config → Task 1 (`migrate_legacy`). ✓
- Bundle CLI in app → Tasks 4, 7. ✓
- `cli_status`/`install_cli` + admin copy to `/usr/local/bin` → Task 4. ✓
- 5s popup, in-app modal, don't-ask-again → Task 5. ✓
- Embedded terminal PATH → Task 4. ✓
- Notarization assets relocated + gitignored → Task 6. ✓
- DMG sign+notarize+verify → Tasks 7, 8. ✓
- Release v0.2.0 + DMG upload + docs → Task 9. ✓

**Placeholder scan:** Task 4 Step 4 and Task 5 Steps 3-4 reference existing in-file harnesses (PTY spawn site, Playwright bootstrap) the executor must read first — flagged explicitly, not silent TODOs. No "TBD"/"handle edge cases" placeholders remain.

**Type consistency:** `config_dir()`/`ensure_config_dir()` signatures consistent across Tasks 1-4. `cli_status`/`install_cli` names consistent between Task 4 (Rust) and Task 5 (JS invoke). `bundled_bin_dir`/`bokf_exe_name`/`bokf_on_path` used consistently within Task 4.

**Known executor caveats (resolve while implementing):**
- Confirm `app/studio/src-tauri/Cargo.toml` already depends on `bokf-core` and `tauri` features include `path`/`Manager`. Add if missing.
- Tauri v2 `resources` map syntax: if `cargo tauri build` rejects the object form, switch to the array form `["bin/bokf", "bin/bokf-mcp"]` and confirm they land in `Contents/Resources/bin/`.
- If notarytool rejects due to the bundled unsigned `bokf` binaries, sign them too (`codesign --options runtime --sign "$APPLE_SIGNING_IDENTITY" bin/bokf bin/bokf-mcp`) before `cargo tauri build`, or add them as `externalBin` so Tauri signs them.
