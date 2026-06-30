# BioOKF Studio — release & notarization runbook

How a new **BioOKF Studio** release is built, signed, Apple-notarized, and published
as a `.dmg` on GitHub Releases. This file contains **no credentials**: every secret
lives in the gitignored `notarization/` folder on the build machine (see below).

## Distribution model

BioOKF ships as two separately installed pieces:

1. **BioOKF Studio** — the standalone desktop app, distributed as a notarized
   `BioOKF Studio_<version>_aarch64.dmg`. On first launch it offers to install the
   `bokf` CLI to `/usr/local/bin`. All Studio config (`registry.yaml`, `.active-kb`)
   lives under `~/.config/biookf-studio`.
2. **The `biookf` agent plugin** — the primary way users drive the Studio from
   Claude Code or Codex over MCP. The published plugin root contains `.claude-plugin`
   and `.codex-plugin` manifests side by side and both use `plugins/biookf/scripts/bokf-mcp`.

The DMG is built, signed, and notarized **locally** on a Mac that holds the UCSF
Developer ID certificate. CI (`.github/workflows/release.yml`) separately builds the
unsigned multi-platform `tar.gz`/`zip` bundles on tag push; the notarized macOS DMG is
the locally produced asset uploaded on top.

## The `notarization/` folder (gitignored — never committed)

On the build machine, `notarization/` must contain:

- `notarization_credentials.env` — `APPLE_ID`, `APPLE_APP_SPECIFIC_PASSWORD`,
  `APPLE_TEAM_ID` (`F3YYBXAFJ8`), and `SIGNING_IDENTITY` (the full Developer ID string).
- `UCSF-AppleDeveloper-Main_Application.p12` — the Developer ID Application cert.
- `entitlements.plist` — reference copy of the hardened-runtime entitlements.
- `notarization.md` — the same runbook, kept next to the secrets.

`.gitignore` ignores the whole `notarization/` folder, plus the staged
`app/studio/src-tauri/bin/bokf*` binaries. Confirm with:

```bash
git check-ignore -v notarization/notarization_credentials.env
```

The signing certificate is already imported into the login keychain; confirm:

```bash
security find-identity -v -p codesigning
# -> Developer ID Application: University of California at San Francisco (F3YYBXAFJ8)
```

The signing config lives in `app/studio/src-tauri/tauri.conf.json` under `bundle.macOS`
(identity, `hardenedRuntime`, `entitlements`) and `app/studio/src-tauri/entitlements.plist`.

## Release steps

### 1. Bump the version

Set the new version in all of:

- `app/Cargo.toml` `[workspace.package] version`
- `app/studio/src-tauri/Cargo.toml` `version`
- `app/studio/src-tauri/tauri.conf.json` `version` (drives the DMG filename)
- `plugins/biookf/.claude-plugin/plugin.json`, `plugins/biookf/.codex-plugin/plugin.json`,
  and `app/.claude-plugin/plugin.json`

Then `cd app && cargo build` to refresh `Cargo.lock`.

### 2. Load credentials and map them to Tauri's env var names

```bash
cd /Users/wanjun/Desktop/BioOKF
export APPLE_ID=$(grep '^APPLE_ID=' notarization/notarization_credentials.env | cut -d= -f2)
export APPLE_TEAM_ID=$(grep '^APPLE_TEAM_ID=' notarization/notarization_credentials.env | cut -d= -f2)
export APPLE_PASSWORD=$(grep -E '^(APPLE_PASSWORD|APPLE_APP_SPECIFIC_PASSWORD)=' notarization/notarization_credentials.env | head -1 | cut -d= -f2)
export APPLE_SIGNING_IDENTITY="Developer ID Application: University of California at San Francisco (F3YYBXAFJ8)"
```

(BioRouter stored the app-specific password as `APPLE_APP_SPECIFIC_PASSWORD`; Tauri
reads `APPLE_PASSWORD`. The line above accepts either name.)

### 3. Build + stage + pre-sign the bundled CLI binaries

The `.app` bundles `bokf` and `bokf-mcp` as resources. Because they are built
separately, they must be signed with the hardened runtime **before** the Tauri build,
or notarization rejects the nested executables.

```bash
cd /Users/wanjun/Desktop/BioOKF/app
cargo build --release -p bokf-cli -p bokf-mcp
cp target/release/bokf target/release/bokf-mcp studio/src-tauri/bin/
codesign --force --options runtime --timestamp --sign "$APPLE_SIGNING_IDENTITY" \
  studio/src-tauri/bin/bokf studio/src-tauri/bin/bokf-mcp
```

### 4. Build + sign + notarize the app and DMG

```bash
cd /Users/wanjun/Desktop/BioOKF/app/studio/src-tauri
cargo tauri build --bundles dmg
```

With the `APPLE_*` env set, Tauri signs the `.app`, submits it to Apple's notary
service, waits for `Accepted`, staples the `.app`, then builds and signs the DMG.
Output: `app/target/release/bundle/dmg/BioOKF Studio_<version>_aarch64.dmg`.

### 5. Notarize + staple the DMG itself

Tauri notarizes the `.app` but not the DMG container. Submit the DMG so the downloaded
file passes Gatekeeper directly, then staple its ticket:

```bash
cd /Users/wanjun/Desktop/BioOKF
DMG="app/target/release/bundle/dmg/BioOKF Studio_<version>_aarch64.dmg"
xcrun notarytool submit "$DMG" --apple-id "$APPLE_ID" --team-id "$APPLE_TEAM_ID" --password "$APPLE_PASSWORD" --wait
xcrun stapler staple "$DMG"
```

### 6. Verify

```bash
xcrun stapler validate "$DMG"                                   # The validate action worked!
spctl --assess --type open --context context:primary-signature -vv "$DMG"   # accepted, Notarized Developer ID
# Optionally mount and assess the inner app:
MNT=$(hdiutil attach "$DMG" -nobrowse -readonly | grep -o '/Volumes/.*$')
spctl --assess --type execute -vv "$MNT/BioOKF Studio.app"
hdiutil detach "$MNT"
```

### 7. Publish to the GitHub Release

```bash
cd /Users/wanjun/Desktop/BioOKF
git tag v<version> && git push origin v<version>    # CI builds the other-platform bundles + creates the release
gh release view v<version> >/dev/null 2>&1 || gh release create v<version> --title "BioOKF v<version>" --generate-notes
gh release upload v<version> "app/target/release/bundle/dmg/BioOKF Studio_<version>_aarch64.dmg" --clobber
```

## Gotchas

- **Pre-sign the bundled `bokf`/`bokf-mcp`** (step 3) or notarization fails on the
  nested executables. They must use `--options runtime` and a timestamp.
- **Notarize the DMG too** (step 5), not just the `.app`, so the downloaded DMG itself
  is trusted by Gatekeeper.
- **WKWebView cache on upgrade.** The app identifier `com.biookf.studio` is stable, so a
  machine that ran an older build may serve a cached frontend. Fresh installs are
  unaffected. If testing an upgrade locally and the UI looks stale, clear
  `~/Library/Caches/com.biookf.studio` and `~/Library/WebKit/com.biookf.studio`.
- **Config location.** The Studio, CLI, and MCP all resolve `~/.config/biookf-studio`
  via `bokf_core::config::config_dir()`. Override with `BIOOKF_CONFIG_DIR` (used by the
  tests so they never touch the real config).
- **Apple Silicon only** for now. Add an Intel/universal target by cross-compiling the
  binaries and the Tauri build, then signing + notarizing each DMG the same way.
