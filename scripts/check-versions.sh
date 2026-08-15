#!/usr/bin/env bash
# Assert that every version string in the repo agrees with the release tag.
#
# BioOKF's version appears in a dozen places — the Rust workspace, the Tauri
# manifest, the lockfile, both plugin manifests, the marketplace entry, the MCP
# launcher's pinned release, and the DMG filenames quoted in the docs. A tag that
# ships with any one of them stale produces a release whose plugin installs the
# wrong binaries, so CI runs this on every `v*` tag push before anything builds.
#
# Usage:
#   scripts/check-versions.sh            # compare against $GITHUB_REF_NAME, else app/Cargo.toml
#   scripts/check-versions.sh v0.3.2     # compare against an explicit tag
#
# Exits 0 when everything matches, 1 (listing every mismatch) when it doesn't.
set -uo pipefail
cd "$(dirname "$0")/.."

EXPECTED="${1:-${GITHUB_REF_NAME:-}}"
if [ -n "$EXPECTED" ]; then
  ORIGIN="tag $EXPECTED"
else
  EXPECTED="$(sed -n 's/^version *= *"\([^"]*\)".*/\1/p' app/Cargo.toml | head -1)"
  ORIGIN="app/Cargo.toml (no tag supplied)"
fi
EXPECTED="${EXPECTED#v}"

if ! printf '%s' "$EXPECTED" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+$'; then
  echo "not a release version: '$EXPECTED' (from $ORIGIN)" >&2
  exit 1
fi

echo "Checking every version against $EXPECTED (from $ORIGIN)"
fails=0
check() { # label actual
  if [ "${2:-}" = "$EXPECTED" ]; then
    printf '  ok    %-44s %s\n' "$1" "$2"
  else
    printf '  FAIL  %-44s %s\n' "$1" "${2:-<not found>}"
    fails=$((fails + 1))
  fi
}

first_toml_version() { sed -n 's/^version *= *"\([^"]*\)".*/\1/p' "$1" | head -1; }
first_json_version() { sed -n 's/.*"version": *"\([^"]*\)".*/\1/p' "$1" | head -1; }
nth_json_version()   { sed -n 's/.*"version": *"\([^"]*\)".*/\1/p' "$1" | sed -n "$2p"; }
lock_version() {
  awk -v n="$1" '$0 == "name = \"" n "\"" { getline; gsub(/^version = "|"$/, ""); print; exit }' \
    app/Cargo.lock
}

# --- the Rust workspace + Tauri app ----------------------------------------
check "app/Cargo.toml"                    "$(first_toml_version app/Cargo.toml)"
check "studio/src-tauri/Cargo.toml"       "$(first_toml_version app/studio/src-tauri/Cargo.toml)"
check "studio/src-tauri/tauri.conf.json"  "$(first_json_version app/studio/src-tauri/tauri.conf.json)"
for crate in bokf-core bokf-cli bokf-mcp biookf-studio; do
  check "Cargo.lock: $crate" "$(lock_version "$crate")"
done

# --- the agent plugin, both manifests + the marketplace entry ---------------
check "plugin.json (Claude Code)" "$(first_json_version plugins/biookf/.claude-plugin/plugin.json)"
check "plugin.json (Codex)"       "$(first_json_version plugins/biookf/.codex-plugin/plugin.json)"
check "marketplace.json: metadata" "$(nth_json_version .claude-plugin/marketplace.json 1)"
check "marketplace.json: plugin"   "$(nth_json_version .claude-plugin/marketplace.json 2)"

# --- the launcher's pinned release (it names the tag it downloads from) -----
check "launcher BIOOKF_VERSION pin" \
  "$(sed -n 's/^VERSION="\${BIOOKF_VERSION:-v\(.*\)}".*/\1/p' plugins/biookf/scripts/bokf-mcp)"

# --- DMG filenames and release links quoted in the docs ---------------------
# Only version-bearing patterns, so the BioOKF *format* version (v0.5) and other
# numbers in prose are never mistaken for the release version.
doc_versions() {
  grep -oE 'BioOKF\.Studio_[0-9]+\.[0-9]+\.[0-9]+_|releases/tag/v[0-9]+\.[0-9]+\.[0-9]+|styles\.css\?v=[0-9]+\.[0-9]+\.[0-9]+|BioOKF Studio [0-9]+\.[0-9]+\.[0-9]+|class="tag">v[0-9]+\.[0-9]+\.[0-9]+' "$1" 2>/dev/null \
    | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | sort -u
}
for doc in README.md landing/index.html landing/docs.html; do
  found="$(doc_versions "$doc")"
  if [ -z "$found" ]; then
    printf '  FAIL  %-44s %s\n' "$doc" "<no version reference found>"
    fails=$((fails + 1))
  else
    while IFS= read -r v; do check "$doc" "$v"; done <<< "$found"
  fi
done

# --- and the tag must actually be written up -------------------------------
if grep -q "^## v$EXPECTED - " RELEASE_NOTES.md; then
  printf '  ok    %-44s %s\n' "RELEASE_NOTES.md section" "## v$EXPECTED"
else
  printf '  FAIL  %-44s %s\n' "RELEASE_NOTES.md section" "no '## v$EXPECTED - ...' heading"
  fails=$((fails + 1))
fi

echo
if [ "$fails" -ne 0 ]; then
  echo "$fails version(s) disagree with $EXPECTED — fix them before tagging." >&2
  exit 1
fi
echo "all versions agree on $EXPECTED"
