# BioOKF Studio: standalone, notarized macOS app — design

Date: 2026-06-28
Status: Approved

## Goal

Ship **BioOKF Studio** as a standalone macOS app that users download as a notarized `.dmg`
from a GitHub Release and install on their Mac. After install the Studio:

1. Puts a single source of truth for its config under `~/.config/biookf-studio/`
   (no more `registry.yaml` / `.active-kb` scattered onto the Desktop).
2. 5 seconds after launch, if the `bokf` CLI is not on the user's PATH, shows an
   in-app popup that one-click installs it to `/usr/local/bin`.
3. Bundles the `bokf` (+ `bokf-mcp`) binaries inside the `.app` so the CLI and the
   in-Studio terminal work without a separate download.

The intended user flow: **install Studio (DMG) → Studio installs the `bokf` CLI →
install the `biookf` Claude Code "cloud plugin" separately** (the primary way users
drive the Studio via Claude over MCP).

## Decisions (locked)

- CLI install mechanism: copy bundled `bokf` → `/usr/local/bin/bokf` via a single
  `osascript ... with administrator privileges` prompt.
- DMG architecture: Apple Silicon (`aarch64`) only.
- Signing + notarization: **locally** on this Mac (UCSF Developer ID cert already in
  the keychain), then `gh release upload`. CI continues to build the other unsigned
  platform bundles unchanged.
- Release: new **v0.2.0** (keeps v0.1.0 intact).
- Bundle **both** `bokf` and `bokf-mcp` inside the `.app`.
- "Don't ask again" popup flag persists in `~/.config/biookf-studio/`.

## Root cause of the "scattered files" bug

`registry.rs` / `active.rs` write to `<root>/registry.yaml` and `<root>/.active-kb`.
The callers disagree about `<root>`:

- CLI scaffold autoregister (`bokf-cli/src/main.rs` ~L684) and MCP ops autoregister
  (`bokf-mcp/src/ops.rs` ~L141) pass `path.parent()` — so scaffolding a KB at
  `~/Desktop/my-kb` writes `registry.yaml`/`.active-kb` to `~/Desktop/`.
- Studio `repo_root()` (`studio/src-tauri/src/main.rs` L16-20) uses `OKF_ROOT` or a
  compile-time repo path.

Fix: one shared `config_dir()` used by every caller.

## Workstreams

### 1. `bokf-core::config` — canonical config dir

- New module `crates/bokf-core/src/config.rs`:
  - `config_dir() -> PathBuf` resolution order:
    1. `BIOOKF_CONFIG_DIR` env (new; primarily for tests + power users)
    2. `OKF_ROOT` env (kept for back-compat)
    3. `$XDG_CONFIG_HOME/biookf-studio` if `XDG_CONFIG_HOME` set
    4. `~/.config/biookf-studio` (macOS/Linux) / `%APPDATA%\biookf-studio` (Windows)
  - `ensure_config_dir() -> Result<PathBuf>` creates it (mkdir -p) and runs the
    one-time migration below.
- Migration: if `<config>/registry.yaml` is absent, seed it by merging any
  `registry.yaml` found at the legacy locations (the old `OKF_ROOT`/compiled repo
  root, and the current working dir). Idempotent — never overwrites an existing
  config-dir registry.
- `registry.rs` / `active.rs` are unchanged (already `root: &Path`).
- Add the `dirs` crate (or equivalent home-dir resolution) as a workspace dep.

### 2. Call-site changes

- `bokf-cli`: scaffold autoregister and the `set-active`/`register`/`get-active`
  commands resolve `root` from `bokf_core::config::config_dir()` (not `path.parent()`,
  not CWD).
- `bokf-mcp`: ops autoregister + the active/registry handlers use `config_dir()`.
  The `OKF_ROOT` plumbing to the spawned Studio stays but defaults to `config_dir()`.
- `studio/src-tauri/src/main.rs`: `repo_root()` → `config_dir()` (rename to
  `config_root()`).

### 3. Studio: bundle CLI, popup, terminal PATH

- tauri.conf.json `bundle.resources`: include `bin/bokf` and `bin/bokf-mcp`
  (a `src-tauri/bin/` dir populated from `app/target/release/` before the Tauri build).
- New Tauri commands:
  - `cli_status() -> { installed, path, version, bundledVersion }` — checks PATH /
    `/usr/local/bin/bokf`.
  - `install_cli() -> Result<()>` — resolves the bundled `bokf` via
    `resource_dir()`, copies to `/usr/local/bin/bokf` with one admin prompt
    (`osascript -e 'do shell script "cp ..." with administrator privileges'`),
    `chmod +x`.
- Embedded PTY (xterm) terminal: prepend the bundled bin dir to its `PATH`.
- Frontend (`dist/index.html` + `dist/app.js` + CSS): modal shown 5s after load when
  `cli_status().installed == false`. Buttons: **Install CLI**, **Later**,
  **Don't ask again** (writes a flag file in the config dir). Styled to the Studio's
  squared/flat minimalism. A hidden `?forceCliPopup=1` query param forces it on for
  visual testing.

### 4. Notarization assets (relocated + gitignored)

- New `notarization/` in the repo: `notarization_credentials.env`,
  `UCSF-AppleDeveloper-Main_Application.p12`, `entitlements.plist`, Tauri-adapted
  `notarization.md`.
- `.gitignore` gains `notarization/`.

### 5. DMG build + sign + notarize (local runbook)

tauri.conf.json `bundle.macOS`: `signingIdentity` = "Developer ID Application:
University of California at San Francisco (F3YYBXAFJ8)", `entitlements`, hardened
runtime, `dmg` target. Build:

```
export APPLE_ID=...                # from notarization/notarization_credentials.env
export APPLE_PASSWORD=...          # app-specific password
export APPLE_TEAM_ID=F3YYBXAFJ8
export APPLE_SIGNING_IDENTITY="Developer ID Application: University of California at San Francisco (F3YYBXAFJ8)"
cargo build --release -p bokf-cli -p bokf-mcp
cp app/target/release/bokf app/target/release/bokf-mcp app/studio/src-tauri/bin/
cargo tauri build --bundles dmg     # signs + notarizes (notarytool) + staples
```

Verify: `spctl --assess --type open --context context:primary-signature -v <dmg>`
and `stapler validate <dmg>`. Output:
`app/target/release/bundle/dmg/BioOKF Studio_0.2.0_aarch64.dmg`.

### 6. Release v0.2.0

- Bump version to `0.2.0`: workspace `Cargo.toml`, `tauri.conf.json`,
  `plugins/biookf/.claude-plugin/plugin.json`, `app/.claude-plugin/plugin.json`.
- Tag `v0.2.0` (CI builds the unsigned multi-platform bundles as today).
- `gh release upload v0.2.0 "<notarized dmg>"`.
- README / docs: document the install flow (DMG → CLI → cloud plugin).

## Error handling

- `install_cli` admin-cancel → friendly error, popup stays open, nothing written.
- Missing bundled binary → surfaced error (not silent).
- Config-dir creation failure → error, never silently scatter to CWD/parent.

## Testing

- `bokf-core` unit tests: `config_dir` resolution per tier; migration seeding
  (absent → seeded; present → untouched).
- CLI tests (`bokf-cli/tests/cli.rs`): scaffold registers into the config dir, never
  `path.parent()`; driven via `BIOOKF_CONFIG_DIR` → temp dir.
- Playwright visual test: `?forceCliPopup=1` renders the popup; screenshot captured.
- Manual integration on this Mac: install DMG → popup → Install CLI → `bokf` in
  Terminal.app; new KB registers into `~/.config/biookf-studio`, zero Desktop scatter;
  `spctl`/`stapler` pass.

## Out of scope (YAGNI)

Intel/universal DMG, CI-side notarization, cloud-plugin reuse of the Studio's bundled
`bokf-mcp` instead of downloading from Releases.
