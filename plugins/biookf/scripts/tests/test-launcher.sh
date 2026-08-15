#!/usr/bin/env bash
# Deterministic tests for the BioOKF MCP launcher's binary resolution order.
#
# Everything here is hermetic: fake `.app` bundles whose "binaries" are shell
# scripts that print what they were handed, a fake $HOME, and a `curl` shim so a
# fall-through to the download path can never touch the network.
set -u
LAUNCHER="$(cd "$(dirname "$0")/.." && pwd)/bokf-mcp"
fail() { echo "FAIL: $1"; exit 1; }

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

# A `curl` that always fails, so the download branch is deterministic + offline.
mkdir -p "$TMP/shim"
printf '#!/bin/sh\nexit 22\n' > "$TMP/shim/curl"
chmod +x "$TMP/shim/curl"

# Build a fake Studio bundle: `bokf` reports $2 as its version, and `bokf-mcp`
# echoes the environment the launcher handed it.
make_bundle() {
  local app="$1" version="$2"
  mkdir -p "$app/Contents/Resources/bin" "$app/Contents/MacOS"
  printf '#!/bin/sh\necho "bokf %s"\n' "$version" > "$app/Contents/Resources/bin/bokf"
  cat > "$app/Contents/Resources/bin/bokf-mcp" <<'EOF'
#!/bin/sh
echo "RAN=$0"
echo "STUDIO_BIN=${BIOOKF_STUDIO_BIN:-unset}"
echo "PATH1=${PATH%%:*}"
EOF
  printf '#!/bin/sh\nexit 0\n' > "$app/Contents/MacOS/biookf-studio"
  chmod +x "$app/Contents/Resources/bin/bokf" \
           "$app/Contents/Resources/bin/bokf-mcp" \
           "$app/Contents/MacOS/biookf-studio"
}

# Every case pins BIOOKF_VERSION explicitly, so bumping the launcher's shipped
# default can never silently change what these tests mean. PIN is the version
# the fixture bundles claim; cases that care about ordering override it.
PIN=v0.3.1
BUNDLE_VER=0.3.1

# Run the launcher in a sandbox: fake HOME, empty cache, offline curl.
# Usage: run_launcher <case-name> [VAR=VAL ...]   (a later VAR=VAL wins)
run_launcher() {
  local name="$1"; shift
  local box="$TMP/$name"
  mkdir -p "$box/home"
  env -i \
    PATH="$TMP/shim:/usr/bin:/bin:/usr/sbin:/sbin" \
    HOME="$box/home" \
    BIOOKF_HOME="$box/cache" \
    BIOOKF_VERSION="$PIN" \
    "$@" \
    bash "$LAUNCHER" </dev/null >"$box/out" 2>"$box/err"
  echo $? > "$box/rc"
}
out() { cat "$TMP/$1/out"; }
err() { cat "$TMP/$1/err"; }

# --- the installed Studio bundle is used instead of downloading -------------
make_bundle "$TMP/apps/BioOKF Studio.app" "$BUNDLE_VER"
run_launcher installed BIOOKF_STUDIO_APP="$TMP/apps/BioOKF Studio.app"

out installed | grep -q "^RAN=$TMP/apps/BioOKF Studio.app/Contents/Resources/bin/bokf-mcp$" \
  || fail "should exec the installed bundle's bokf-mcp (got: $(out installed | head -1))"

err installed | grep -q "first run: fetching" \
  && fail "should not download when an installed bundle supplies bokf-mcp"

# --- and it points the Studio launcher at that same bundle ------------------
out installed | grep -q "^STUDIO_BIN=$TMP/apps/BioOKF Studio.app/Contents/MacOS/biookf-studio$" \
  || fail "BIOOKF_STUDIO_BIN should point into the installed bundle (got: $(out installed | grep STUDIO_BIN))"

# --- and puts that bundle's bin dir first on PATH ---------------------------
out installed | grep -q "^PATH1=$TMP/apps/BioOKF Studio.app/Contents/Resources/bin$" \
  || fail "bundle bin dir should be first on PATH (got: $(out installed | grep PATH1))"

# --- a per-user ~/Applications install is found too -------------------------
mkdir -p "$TMP/peruser/home"
make_bundle "$TMP/peruser/home/Applications/BioOKF Studio.app" "$BUNDLE_VER"
run_launcher peruser BIOOKF_STUDIO_APP="$TMP/nope.app"
out peruser | grep -q "Applications/BioOKF Studio.app/Contents/Resources/bin/bokf-mcp$" \
  || fail "should find a bundle under \$HOME/Applications (got: $(out peruser | head -1))"

# --- BIOOKF_MCP_BIN still wins over an installed bundle ---------------------
printf '#!/bin/sh\necho "RAN=override"\n' > "$TMP/override-mcp"
chmod +x "$TMP/override-mcp"
run_launcher override \
  BIOOKF_MCP_BIN="$TMP/override-mcp" \
  BIOOKF_STUDIO_APP="$TMP/apps/BioOKF Studio.app"
out override | grep -q "^RAN=override$" \
  || fail "BIOOKF_MCP_BIN must take precedence over the bundle (got: $(out override | head -1))"

# --- a bundle NEWER than the pinned version is used (the Studio self-updates)
run_launcher newer \
  BIOOKF_VERSION=v0.0.1 \
  BIOOKF_STUDIO_APP="$TMP/apps/BioOKF Studio.app"
out newer | grep -q "bokf-mcp$" \
  || fail "a bundle newer than the pin should still be used (got: $(out newer | head -1))"

# --- a bundle OLDER than the pin is refused; it falls back to the download --
run_launcher older \
  BIOOKF_VERSION=v9.9.9 \
  BIOOKF_STUDIO_APP="$TMP/apps/BioOKF Studio.app"
out older | grep -q "RAN=" \
  && fail "a bundle older than the pin must not be exec'd"
err older | grep -qi "older" \
  || fail "should say why the older bundle was skipped (got: $(err older | head -3))"
[ "$(cat "$TMP/older/rc")" = "1" ] \
  || fail "offline fallback should exit 1 (got $(cat "$TMP/older/rc"))"

# --- a bundle whose version can't be read is still used ---------------------
make_bundle "$TMP/noversion/BioOKF Studio.app" "$BUNDLE_VER"
rm -f "$TMP/noversion/BioOKF Studio.app/Contents/Resources/bin/bokf"
run_launcher noversion BIOOKF_STUDIO_APP="$TMP/noversion/BioOKF Studio.app"
out noversion | grep -q "bokf-mcp$" \
  || fail "an unversionable bundle should still be used (got: $(err noversion | head -2))"

# --- a prerelease bundle compares on its numeric part -----------------------
make_bundle "$TMP/rc/BioOKF Studio.app" 0.4.0-rc.1
run_launcher prerelease \
  BIOOKF_STUDIO_APP="$TMP/rc/BioOKF Studio.app"
out prerelease | grep -q "bokf-mcp$" \
  || fail "0.4.0-rc.1 should outrank the pinned $PIN (got: $(err prerelease | head -2))"

# --- with no bundle at all, an existing cache is still used (unchanged) -----
mkdir -p "$TMP/cached/cache/$PIN/bin"
cat > "$TMP/cached/cache/$PIN/bin/bokf-mcp" <<'EOF'
#!/bin/sh
echo "RAN=cache"
EOF
chmod +x "$TMP/cached/cache/$PIN/bin/bokf-mcp"
run_launcher cached BIOOKF_STUDIO_APP="$TMP/nope.app"
out cached | grep -q "^RAN=cache$" \
  || fail "should fall back to the cached install when no bundle exists (got: $(out cached | head -1))"

echo "launcher OK"
