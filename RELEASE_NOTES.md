# BioOKF Release Notes

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
