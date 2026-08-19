# BioOKF Release Notes

## v0.4.0 - 2026-08-18

BioOKF 0.4.0 turns Studio from a graph viewer and curator into a complete local
knowledge workflow. It can now discover biomedical evidence through Paperclip,
build reviewable BioOKF bundles with a locally authenticated Codex or Claude
subscription, create bases from local documents, answer grounded questions,
revise evidence with Doctor, merge bases, and inspect network structure without
requiring API keys or sending credentials through Studio.

### Paperclip-to-BioOKF generation

- Added the `paperclip2biookf` integration, including multi-database discovery,
  date and source filters, line-addressable evidence snapshots, subscription-LLM
  extraction, deterministic BioOKF materialization, verification, Studio
  registration, and a local browser UI.
- Added a dedicated **Create from Paperclip** Studio workflow with provider and
  model selection, live progress, evidence previews, generation results, and
  explicit scientific-review status for candidate bundles.
- Preserved source identifiers and evidence URLs while using collision-resistant
  local storage names. Unsafe document identifiers are rejected before any path
  is created, and identifiers that normalize to the same slug no longer
  overwrite one another.
- Added portable Claude Code and Codex integration packages with workflows for
  generation, local-resource creation, grounded chat, Doctor review, semantic
  merge, diagnostics, and network analysis.

### Local knowledge workflows

- Added **Create from local resources** for turning documents and folders into a
  staged BioOKF bundle with provenance and verification.
- Added grounded **Chat** over the active base, including bounded context and
  source-aware answers.
- Added Doctor-backed evidence review and revision, multi-base semantic merge,
  and workflow progress surfaces inside Studio.
- Added CLI and MCP coverage for the same workflows so desktop, terminal, Claude
  Code, and Codex operate on the same registered bases and active-base state.

### Network analysis and evidence timelines

- Added a network-metrics engine covering graph size, density, components,
  degree and weighted-degree rankings, PageRank, betweenness, closeness,
  articulation points, bridges, triangles, transitivity, community structure,
  shortest paths, and source-year distributions.
- Added the Studio **Metrics** panel for topology summaries, rankings,
  communities, evidence timelines, and path exploration.
- Added CLI and MCP interfaces for programmatic network analysis and an
  `analyze-biookf-network` skill for agent-driven interpretation.

### BioOKF metagraph and workflow documentation

- Added a complete BioOKF metagraph example covering the controlled node types
  and representative relationships.
- Added SVG and PNG schema/workflow diagrams, including transparent and compact
  variants suitable for documentation and presentations.
- Archived the non-superseded SPOKE mapping lineage, research corpus, guardrails,
  adjudication tools, and comparison reports under `fabric/spoke/archive`.

### Studio, plugin, and updater fixes

- The main BioOKF plugin now uses `bokf-mcp` from an installed, sufficiently new
  `BioOKF Studio.app` before downloading a duplicate archive. It exports the
  matching Studio executable and resource path, respects explicit overrides,
  and keeps the signed installed bundle as the source of truth.
- `bokf_studio_open` now resolves the Studio executable from the shipped `.app`
  layout as well as local sibling builds.
- The knowledge-base context menu's **Open folder** command is implemented and
  constrained to a registered base identifier.
- The frontend wiring gate now checks every local script loaded by `index.html`,
  every interactive control, and every registered Tauri invocation.

### Security and reliability hardening

- Restricted the Paperclip UI to loopback interfaces, validate its `Host`
  header, require a per-process request token for API reads and writes, enforce
  JSON request bodies and size limits, and send anti-framing, no-referrer,
  no-sniff, no-store, and content-security headers.
- Removed a reverse-DNS lookup that could stall local UI startup.
- Prevented subprocess pipe deadlocks by draining workflow results and progress
  concurrently, and made large UTF-8 chat-context truncation character-safe.
- Expanded subscription-only environment sanitization across Studio, status
  checks, Paperclip, and embedded workflows so API keys, cloud credentials,
  alternate endpoints, and hosted-provider switches cannot silently override the
  intended local subscription route.
- Updated the MCP protocol stack, spreadsheet/PDF dependencies, Tauri MCP plugin,
  and vulnerable transitive crates. Current macOS release targets contain none of
  the two remaining all-target `quick-xml` advisories reported through `xcb`'s
  build-only dependency.

### For Studio users

- Download `BioOKF.Studio_0.4.0_aarch64.dmg` for Apple Silicon Macs.
- Download `BioOKF.Studio_0.4.0_x64.dmg` for Intel Macs.
- Both DMGs and both plugin tar archives are Developer ID signed, notarized by
  Apple, stapled where the format supports it, and verified before publication.

### For agent and plugin users

- The plugin and marketplace metadata now point at `v0.4.0`; the launcher uses
  that version as its download floor while still allowing `BIOOKF_VERSION` and
  explicit binary overrides.
- Update with `/plugin marketplace update biookf`, then
  `/plugin update biookf@biookf`, and restart the client.
- Existing 0.3.x bundle content remains readable; this is a feature release, not
  a BioOKF file-format migration.

### Verification targets

- Full Rust workspace tests and strict Clippy across every target.
- Python integration tests, frontend wiring checks, launcher-resolution tests,
  version consistency, JavaScript syntax checks, and browser-driven Studio
  graph/search/sidebar/detail-panel smoke tests.
- Release-mode Studio builds for Apple Silicon and Intel with signed bundled CLI
  and MCP executables.
- Strict nested code-signature verification, Apple notarization, stapler and
  Gatekeeper assessment, read-only DMG mounting, signed tar round-trips, and
  SHA-256 checksums for every published artifact.

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
