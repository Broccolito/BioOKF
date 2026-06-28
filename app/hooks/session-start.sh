#!/usr/bin/env bash
# SessionStart: brief the agent on the BioOKF workflow + surface the active KB if set.
source "$(dirname "$0")/lib.sh" 2>/dev/null || true
echo "BioOKF Studio plugin active. Curate via the biookf-* skills: convert (Step 1) -> ingest -> verify; merge with biookf-merge. Record every step with bokf_log_sync; raw/ originals are immutable; end each run with bokf_verify."
bokf=$(resolve_bokf) || exit 0
active=$("$bokf" get-active --json 2>/dev/null | jq -r '.id // empty' 2>/dev/null)
[ -n "$active" ] && echo "Active BioOKF KB: $active"
# One-time setup: enable PDF page rendering. Nudge until installed (then this goes silent).
if ! "$bokf" install-pdfium --check >/dev/null 2>&1; then
  echo "Setup: PDF page rendering is not enabled yet. Run \`bokf install-pdfium\` once now (it auto-downloads the PDFium library to ~/.biookf and is auto-discovered afterward) so PDF pages render to images for vision. PDFs still convert without it."
fi
exit 0
