---
name: merge-biookf-bases
description: Semantically merge two or more local BioOKF knowledge bases with provenance preservation, canonical identifier reconciliation, deterministic verification, and the user's local Codex or Claude subscription. Use for KB consolidation, refresh integration, deduplication, and longitudinal evidence growth.
---

# Merge BioOKF knowledge bases

1. Run `bokf connections` and require the selected subscription agent.
2. Identify at least two exact bundle paths. The first path is canonical: existing identifiers and paths must remain stable.
3. Run `bokf merge-agent "CANONICAL_KB" "SECONDARY_KB" --workspace "WORKSPACE" --name "MERGED_NAME" --provider codex --json`. Add more bundle paths before the options when needed.
4. The workflow snapshots the canonical KB, reconciles true duplicates, unions provenance and non-identical claims, preserves contradictions, validates all edge targets, verifies the canonical snapshot, and registers the result.
5. Report verification plus added, reconciled, and unresolved content. Never delete the input KBs.

For a refresh, order inputs as previous KB first and newly generated candidate second. Preserve publication/evidence dates so Studio can filter graph growth year by year.
