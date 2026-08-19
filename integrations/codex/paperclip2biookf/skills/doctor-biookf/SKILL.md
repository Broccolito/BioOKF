---
name: doctor-biookf
description: Inspect and transactionally revise a local BioOKF knowledge base against its cited raw evidence using the user's Codex or Claude subscription. Use when asked to recheck a specific edge against papers, repair provenance, correct a claim, split a conflated concept, merge duplicate nodes, resolve identifiers, or perform evidence-backed manual KB revision.
---

# Use BioOKF Doctor

Use the installed `bokf doctor` workflow. Never edit the live KB directly for this task.

1. Run `bokf connections` and require the selected subscription agent.
2. Resolve the exact bundle and revision instruction. For node merging, place both exact identifiers in the instruction. For edge review, include subject, predicate, and object.
3. Run `bokf doctor "BUNDLE" "INSTRUCTION" --workspace "WORKSPACE" --provider codex --json`. Use `claude` when selected. Omit `--model` for the subscription default.
4. Doctor clones the KB, reads cited raw sources, limits edits to knowledge Markdown and the index, runs deterministic verification, applies the candidate atomically, and creates a Git/log checkpoint. A failed or unsupported revision leaves the KB unchanged.
5. Report the summary, rationale, evidence checked, changed files, unresolved questions, verification, and commit.

Never invent evidence, collapse homonyms, remove contradictions, or merge nodes solely because their labels look similar. If available sources do not support the requested change, preserve the current claim and report the missing evidence.
