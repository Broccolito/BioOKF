#!/usr/bin/env bash
# SessionStart: brief the agent on the BioOKF workflow + surface the active KB if set.
source "$(dirname "$0")/lib.sh" 2>/dev/null || true
echo "BioOKF Studio plugin active. Curate via the biookf-* skills: convert (Step 1) -> ingest -> verify; merge with biookf-merge. Record every step with bokf_log_sync; raw/ originals are immutable; end each run with bokf_verify."
bokf=$(resolve_bokf) || exit 0
active=$("$bokf" get-active . --json 2>/dev/null | jq -r '.id // empty' 2>/dev/null)
[ -n "$active" ] && echo "Active BioOKF KB: $active"
exit 0
