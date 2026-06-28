#!/usr/bin/env bash
# PostToolUse(Edit|Write|MultiEdit): after a concept-doc write, surface lint errors as
# advisory feedback. PostToolUse cannot block (the write already happened); it only
# nudges; the blocking gate is bokf verify at the Stop/end of a run. Fail-open throughout.
source "$(dirname "$0")/lib.sh" 2>/dev/null || true
input=$(cat)
path=$(printf '%s' "$input" | jq -r '.tool_input.file_path // empty' 2>/dev/null)
case "$path" in
  */knowledge/*.md) ;;
  *) exit 0 ;;
esac
root=$(bundle_root_of "$path") || exit 0
bokf=$(resolve_bokf) || exit 0
errs=$("$bokf" lint "$root" --json 2>/dev/null | jq '[.findings[]|select(.severity=="error")]|length' 2>/dev/null)
if [ "${errs:-0}" -gt 0 ] 2>/dev/null; then
  echo "BioOKF lint: $errs error(s) in this bundle; run bokf_verify and fix before finishing the run." >&2
fi
exit 0
