# BioOKF Release Notes

## v0.3.2 - 2026-08-15

BioOKF 0.3.2 makes the agent plugin use the Studio you already installed. On
0.3.1 the MCP launcher ignored the installed app and always downloaded its own
copy of the binaries, which meant a machine with a notarized Studio in
`/Applications` still ran unsigned tools out of a cache directory.

### The Bug

`plugins/biookf/scripts/bokf-mcp` documented a three-step search — `BIOOKF_MCP_BIN`,
then an installed `BioOKF Studio.app`, then the release tarball — but only ever
implemented the first and third. The middle step did not exist in the script.

On any Mac that had installed the DMG, the consequences were:

- a redundant ~25 MB download of binaries already sitting inside the `.app`;
- the cached copies were the **unsigned** tarball builds (the launcher strips
  `com.apple.quarantine` from them), while the bundle ships Developer
  ID-signed, notarized ones;
- `BIOOKF_STUDIO_BIN` then pointed into the cache, so `bokf_studio_open`
  launched a second, hidden copy of Studio instead of the one in
  `/Applications`.

### Fixes

- The launcher now probes `/Applications/BioOKF Studio.app` and
  `~/Applications/BioOKF Studio.app` before downloading anything, and execs the
  `bokf-mcp` inside the bundle. `BIOOKF_STUDIO_APP` overrides the location.
- `PATH` and `BIOOKF_STUDIO_BIN` are exported from that same bundle, so
  `bokf_studio_open` drives the app you actually installed.
- The pinned release is treated as a floor, not an equality: a bundle newer than
  the pin is used as-is (Studio self-updates ahead of the plugin), while an older
  one is skipped with a note on stderr and the pinned version is downloaded.
- `BIOOKF_MCP_BIN` still takes precedence over everything, and machines with no
  Studio installed keep the unchanged download-and-cache path.
- `bokf_studio_open` now finds the Studio executable inside the shipped `.app`.
  `locate_studio_bin()` only looked *next to* the running `bokf-mcp`, which is
  true of a `cargo build` but not of the DMG layout — `bokf-mcp` ships in
  `Contents/Resources/bin` and `biookf-studio` in `Contents/MacOS` — so opening
  the GUI failed with "biookf-studio not found next to …" for anyone invoking the
  bundled binary directly rather than through the plugin launcher (which sets
  `BIOOKF_STUDIO_BIN` and so masked the bug). A sibling still wins, so a local
  development build is never shadowed by an installed app.

### Studio

- The knowledge-base context menu's **Open folder** item now works. `index.html`
  declared the button but `app.js` never referenced its id, so it rendered,
  highlighted on hover, and did nothing; there was also no command behind it,
  since `reveal_in_finder` is restricted to `knowledge/` documents and root text
  files and cannot open a bundle root. Adds an `open_base_folder` command that
  resolves the registered id (no caller-supplied path, so nothing to escape) and
  wires the menu item to it.
- `app/studio/tests/test-wiring.sh` guards the class of bug rather than the
  instance: the no-bundler frontend has nothing linking `index.html` to `app.js`,
  so it now asserts that every interactive element with an id is referenced by
  `app.js`, and that every `tauriInvoke` target is registered in `main.rs`.

### Release Hygiene

- `scripts/check-versions.sh` asserts that all sixteen places carrying the
  version — the Rust workspace, the Tauri manifest, `Cargo.lock`, both plugin
  manifests, both marketplace fields, the launcher's pinned release, the DMG
  names quoted in the README and landing pages, and these notes — agree with the
  tag being built. `release.yml` runs it before any build step, so a tag can no
  longer ship with a stale version anywhere.

### For Studio Users

- Download `BioOKF.Studio_0.3.2_aarch64.dmg` for Apple Silicon Macs.
- Download `BioOKF.Studio_0.3.2_x64.dmg` for Intel Macs.
- Nothing in the app itself changed in this release; if you are on 0.3.1 you can
  stay there. The fix matters to Claude Code and Codex users.

### For Agent And Plugin Users

- The plugin and marketplace metadata now point at `v0.3.2`, and the launcher
  default `BIOOKF_VERSION` is `v0.3.2`, still overridable through the
  environment.
- Update with `/plugin marketplace update biookf` then
  `/plugin update biookf@biookf`, and restart the client.
- After updating, the launcher logs which binary it chose on stderr
  (`using the installed Studio at ...`). A stale cache under
  `~/.local/share/biookf` is no longer consulted when a Studio is installed and
  can be deleted.
- Existing MCP tool names and CLI commands remain compatible with 0.3.x.

### Verification Targets

- `plugins/biookf/scripts/tests/test-launcher.sh`: 13 assertions over the
  resolution order, run against fake `.app` bundles with an offline `curl` shim,
  covering the bundle probe, the `PATH`/`BIOOKF_STUDIO_BIN` exports, the
  `~/Applications` location, `BIOOKF_MCP_BIN` precedence, newer/older/prerelease/
  unreadable bundle versions, and the unchanged cache fallback.
- The suite passes under bash 3.2, the macOS system shell.
- A real `initialize` + `tools/list` handshake against an installed 0.3.2 Studio
  with an empty cache: 33 tools, nothing downloaded.
- `scripts/check-versions.sh v0.3.2`.

## v0.3.1 - 2026-07-08

BioOKF 0.3.1 repairs Studio's in-app updater. On 0.3.0, accepting an update quit
Studio and then never brought it back: nothing was installed, and the app did not
reopen. This release fixes that, and is packaged so that Macs already running the
broken 0.3.0 updater can still install it and recover on their own.

### The Bug

Studio picks the first release asset matching the running platform, and the
GitHub API does not return assets in upload order. It therefore chose
`biookf-macos-<arch>.tar.gz` — built in CI with `--no-sign` — rather than the
signed DMG. The updater's relauncher then ran `codesign --verify` under
`set -euo pipefail`, which aborted the script. Because Studio had already quit
before any of this was checked, the app simply vanished, and the only trace was a
line in `~/Library/Logs/BioOKF Studio Updater.log`.

### Fixes

- The updater now prefers the signed `.dmg` over any tarball, instead of taking
  whichever asset GitHub listed first.
- The download is unpacked and signature-checked **before** Studio quits. A bad
  asset now surfaces as an error in the update dialog with Studio still running.
- An update must be signed by the same Developer ID team as the running app.
- Replacing the app is atomic: the new bundle is copied in and verified before
  the old one is moved aside, and any failure restores it. The previous installer
  ran `rm -rf` on the installed app *before* copying the replacement.
- The relauncher reopens Studio on every exit path, including failures.
- Dropped the `spctl --assess` gate, whose result depended on whether the user
  had Gatekeeper assessments enabled rather than on the download.
- The privileged install step no longer depends on an inherited `PATH`, so
  ownership of the installed bundle is restored instead of being left as `root`.

### Packaging

- Both macOS assets — the DMGs **and** the `biookf-<target>.tar.gz` archives the
  plugin installs from — are now signed, notarized, and stapled.
- `.github/workflows/release.yml` no longer publishes release assets. Its runners
  hold no Developer ID certificate, so anything they build is unsigned and cannot
  be installed by the updater. It now runs as a tag-time verification build.

### For Studio Users

- Download `BioOKF.Studio_0.3.1_aarch64.dmg` for Apple Silicon Macs.
- Download `BioOKF.Studio_0.3.1_x64.dmg` for Intel Macs.
- If you are on 0.3.0, the in-app update should now work. If Studio previously
  quit on you during an update, it was never uninstalled — reopen it from
  `/Applications`.

### Verification Targets

- Rust tests for `biookf-studio`, `bokf-cli`, and `bokf-mcp`.
- Updater tests covering asset ranking, signature and team-identity gating,
  atomic install with rollback, relaunch on every failure path, and the
  bash/AppleScript/shell quoting of the privileged install command.
- The v0.3.0 relauncher replayed against both the old unsigned tarball (fails, as
  users experienced) and the new signed tarball (installs and relaunches).
- Signed, notarized, and stapled Apple Silicon and Intel DMGs and tarballs,
  each re-verified after a tar round-trip.

## v0.3.0 - 2026-07-02

BioOKF 0.3.0 is a Studio-focused release. It keeps the CLI, MCP server, and
Tauri desktop app on the same engine, and adds a shareable graph export path for
people who need to review or circulate a knowledge base outside the desktop app.

### Highlights

- Added a Studio graph export button below the zoom and fit controls.
- Exported graphs are self-contained HTML files with embedded data, styling, and
  interaction code. They open directly in a browser without a local server.
- Exported HTML graph views support pan, zoom, search, clickable nodes and
  edges, and a responsive detail drawer for fields, provenance, notes, outgoing
  edges, incoming edges, and rendered document text.
- Polished the Studio canvas control stack so zoom in, zoom out, fit-to-view,
  and export read as a cohesive minimal set.
- Updated release metadata, plugin manifests, marketplace metadata, installer
  documentation, and the public landing page for the 0.3.0 release.

### For Studio Users

- Download `BioOKF.Studio_0.3.0_aarch64.dmg` for Apple Silicon Macs.
- Download `BioOKF.Studio_0.3.0_x64.dmg` for Intel Macs.
- Use the export control in the lower-right graph toolbar to save the active
  graph as a portable HTML file.

### For Agent And Plugin Users

- The bundled Claude Code and Codex plugin metadata now points at `v0.3.0`.
- The plugin launcher default `BIOOKF_VERSION` is now `v0.3.0`, while still
  allowing overrides through the environment.
- Existing MCP tool names and CLI commands remain compatible with 0.2.x.

### Verification Targets

- Rust tests for `biookf-studio`, `bokf-cli`, and `bokf-mcp`.
- Studio HTML export browser checks across desktop and mobile viewports.
- Signed and notarized Apple Silicon and Intel Studio DMGs.
- GitHub Release assets for macOS CLI/MCP archives and Studio DMGs.
