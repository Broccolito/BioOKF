---
name: generate-paperclip-kb
description: Discover biomedical papers, trials, or regulatory evidence with the machine-local Paperclip CLI and generate a verified BioOKF knowledge base using the user's local Codex or Claude subscription. Use for Paperclip searches, evidence-to-graph generation, date-scoped KB creation, and Paperclip2BioOKF refresh runs.
---

# Generate a Paperclip BioOKF knowledge base

Use the installed `bokf` command. Never call OpenAI or Anthropic APIs and never request API keys.

1. Run `bokf connections` and require Paperclip plus the selected subscription agent to report connected.
2. Ask only for missing choices: query, one or more Paperclip sources, output name, provider, workspace, and optional date scope.
3. Explain that `--max-per-source N` means up to N results from every selected database before cross-database deduplication. With three sources and N=5, the pre-deduplication maximum is 15.
4. Run `bokf generate-from-paperclip --query "QUERY" --source pmc --max-per-source 5 --name "NAME" --provider codex --workspace "WORKSPACE" --register`. Repeat `--source` for each database. Add `--year-min`, `--year-max`, or `--since` only when requested. Omit `--model` for the subscription default; otherwise pass the exact model selected by the user.
5. Standard BioOKF curation is always applied. Use `--prompt` only for extra domain priorities, never to restate the schema.
6. Report the final bundle path and verification result. On failure, preserve and report the run directory and diagnostic JSON.

For refreshes, generate a dated candidate KB using the same query/source policy, then compare and merge it with the previous KB using the merge skill. Do not silently overwrite the prior evidence state.
