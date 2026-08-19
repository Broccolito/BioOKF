---
name: generate-paperclip-kb
description: Discover biomedical evidence with local Paperclip and generate a verified BioOKF KB using a local Codex or Claude subscription. Use for Paperclip searches, dated evidence generation, and refresh candidates.
---

# Generate from Paperclip

Run `bokf connections` first. Never request API keys. Gather query, repeatable sources, name, workspace, provider, and optional date scope. Run `bokf generate-from-paperclip --query "QUERY" --source pmc --max-per-source 5 --name "NAME" --provider claude --workspace "WORKSPACE" --register`.

`--max-per-source` is per selected database; multiply it by the selected database count for the pre-deduplication maximum. Standard BioOKF curation is automatic. Use `--prompt` only for extra domain priorities and `--model` only when the user explicitly selects an exact provider model.

For refresh, generate a new dated candidate and merge it into the previous KB without overwriting the historical state.
