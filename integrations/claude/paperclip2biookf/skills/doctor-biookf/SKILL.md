---
name: doctor-biookf
description: Inspect and transactionally revise a local BioOKF knowledge base against cited raw evidence using a local Codex or Claude subscription. Use to recheck edges against papers, repair provenance, correct claims, split conflated concepts, merge true duplicate nodes, and resolve identifiers.
---

# Use BioOKF Doctor

Run `bokf connections`, resolve the exact bundle and instruction, then run `bokf doctor "BUNDLE" "INSTRUCTION" --workspace "WORKSPACE" --provider claude --json`.

For an edge review, name subject, predicate, and object. For a merge, name both exact identifiers. Doctor works in an isolated copy, reads raw evidence, permits only knowledge/index edits, verifies the candidate, applies it atomically, and creates a reversible Git/log checkpoint. Report evidence checked, rationale, changed files, unresolved issues, and verification. Never invent support, merge homonyms, or discard contradictory evidence.
