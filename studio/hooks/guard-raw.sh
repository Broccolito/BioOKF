#!/usr/bin/env bash
# PreToolUse(Edit|Write|MultiEdit): block edits to immutable raw/ originals.
# raw/<id>/original.* are the verbatim ingested bytes; only raw/<id>/source.md (the
# rendering) and knowledge/ are writable. Exit 2 blocks the tool (stderr -> the model).
input=$(cat)
path=$(printf '%s' "$input" | jq -r '.tool_input.file_path // .tool_input.path // empty' 2>/dev/null)
case "$path" in
  */raw/*/original.*|*/raw/original.*)
    echo "BioOKF guardrail: raw/ originals are immutable — never edit ingested source bytes. Render to source.md via bokf_convert instead." >&2
    exit 2
    ;;
esac
exit 0
