---
name: merge-biookf-bases
description: Semantically merge two or more BioOKF bundles while preserving provenance and canonical identifiers using a local Codex or Claude subscription. Use for consolidation, refresh integration, deduplication, and longitudinal evidence growth.
---

# Merge BioOKF bases

Run `bokf connections`, then `bokf merge-agent "CANONICAL_KB" "SECONDARY_KB" --workspace "WORKSPACE" --name "MERGED_NAME" --provider claude --json`. The first KB is canonical. Preserve input KBs, contradictions, evidence dates, sources, and distinct claims. Report verification and the output bundle. For refreshes, put the previous KB first and the new dated candidate second.
