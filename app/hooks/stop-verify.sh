#!/usr/bin/env bash
# Stop hook: if a KB is active and `bokf verify` fails, BLOCK the stop (exit 2) so the agent
# re-checks / re-ingests to fix it, turning a failed gate into another correction pass rather
# than a dead end. Safety: capped at MAX_ATTEMPTS so it can NEVER wedge a session; gated on an
# active KB (normal sessions are untouched); fail-open on every error path (exit 0 = allow stop).
source "$(dirname "$0")/lib.sh" 2>/dev/null || true
MAX_ATTEMPTS=3

bokf=$(resolve_bokf) || exit 0                       # no binary -> allow stop
active=$("$bokf" get-active --json 2>/dev/null) || exit 0
id=$(printf '%s' "$active" | jq -r '.id // empty' 2>/dev/null)
path=$(printf '%s' "$active" | jq -r '.path // empty' 2>/dev/null)
[ -n "$id" ] || exit 0                                # no active KB -> normal session, allow stop
[ -n "$path" ] && [ -d "$path" ] || exit 0

counter="${TMPDIR:-/tmp}/bokf-stop-$(printf '%s' "$path" | cksum | cut -d' ' -f1)"
res=$("$bokf" verify "$path" --json 2>/dev/null) || { rm -f "$counter"; exit 0; }
ok=$(printf '%s' "$res" | jq -r '.ok // false' 2>/dev/null)
errs=$(printf '%s' "$res" | jq -r '.errors // 0' 2>/dev/null)

if [ "$ok" = "true" ]; then
  rm -f "$counter"
  exit 0                                              # clean -> allow stop
fi

# failing: count consecutive blocks; after MAX_ATTEMPTS give up blocking (never wedge)
n=0; [ -f "$counter" ] && n=$(cat "$counter" 2>/dev/null || echo 0)
case "$n" in ''|*[!0-9]*) n=0 ;; esac
n=$((n + 1))
if [ "$n" -gt "$MAX_ATTEMPTS" ]; then
  rm -f "$counter"
  echo "BioOKF gate: verify still failing ($errs error(s)) after $MAX_ATTEMPTS attempts; stopping for manual review. Run 'bokf verify $path --json' to see them." >&2
  exit 0
fi
echo "$n" > "$counter"
echo "BioOKF gate: 'bokf verify $path' found $errs error(s) in active KB '$id'. Read the findings ('bokf verify $path --json'), fix or re-ingest the offending node/edge, then finish again; it will re-verify. (attempt $n/$MAX_ATTEMPTS)" >&2
exit 2
