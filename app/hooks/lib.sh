#!/usr/bin/env bash
# Shared helpers for the BioOKF guardrail hooks.

# Resolve the `bokf` binary: $BOKF_BIN, then PATH, then the plugin's built target.
resolve_bokf() {
  if [ -n "$BOKF_BIN" ] && [ -x "$BOKF_BIN" ]; then echo "$BOKF_BIN"; return 0; fi
  local p
  p="$(command -v bokf 2>/dev/null)" && { echo "$p"; return 0; }
  for cand in \
    "${CLAUDE_PLUGIN_ROOT}/target/release/bokf" \
    "${CLAUDE_PLUGIN_ROOT}/target/debug/bokf"; do
    [ -x "$cand" ] && { echo "$cand"; return 0; }
  done
  return 1
}

# Given a path under a bundle, echo the bundle root (the ancestor containing knowledge/).
bundle_root_of() {
  case "$1" in
    */knowledge/*) printf '%s' "${1%%/knowledge/*}" ;;
    *) return 1 ;;
  esac
}
