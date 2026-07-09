# BioOKF Release Notes

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
