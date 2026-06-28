---
name: biookf-lint
description: Use when validating or repairing a BioOKF knowledge base: run the deterministic scan, then fix Errors then Warnings by rewriting the offending pages.
---

# Skill: biookf-lint

`bokf_lint` returns a JSON report of `findings` with `severity` (error/warn/info), `rule`, `subject`, `message`, `path`.

## Fix order
1. **Errors first** (the bundle is non-conformant until these are 0):
   - `type.invalid` / `predicate.invalid` → change to one of the 28 types / 35 predicates (24 positive + 11 `not_<X>` negatives).
   - `edge.not_negatable` → you negated a predicate that isn't negatable; only the 11 effect predicates (treats, causes, binds, associated_with, expressed_in, regulates, has_phenotype, prevents, predisposes_to, interacts_with, affects_response_to) take `not_<X>`, so drop the negation or re-model.
   - `identifier.duplicate` → rename one (add a parenthetical facet, e.g. `IL6 (gene)` vs `IL6 (protein)`).
   - `edge.missing_*` / `edge.invalid_*` (knowledge_level/agent_type/primary_source) → add/correct the provenance triplet.
   - `parse` → fix the YAML (usually an unquoted `": "` in a value).
2. **Warnings** (quality):
   - `edge.object_unresolved` → create the missing concept doc, or fix the `object` to match an existing `identifier`.
   - `identifier.opaque` → rename to human-readable; move the CURIE to `xref`.
   - `edge.primary_source_unresolved` / `_not_source` → point `primary_source` at a real Publication/Study/Dataset/Agent node (create it once).
   - `source.unanchored` → add `raw_source` (ingested) or an `xref` CURIE (external) to the source node.
   - `edge.range` → fix the domain/range (e.g. `treats` must target a Disease/Phenotype).
   - `node.orphan` → connect it (often a missing `reported_in` edge).
   - `edge.contradiction` → reconcile or annotate which claim is authoritative.
   - `edge.duplicate` → identical `predicate`+`object` from the **same** `primary_source`; merge the sources onto one edge or drop the redundant one (a *different* source is a legitimate parallel edge, not a dup).
   - `type.path_mismatch` → the node's `type` disagrees with its `knowledge/<type>/` directory; re-file it (or fix the type), a misclassification signal.
3. **Infos** are advisory (`subtype.missing`, `predicate.inverse`, `edge.missing_direction`, `subtype.similar` → unify near-duplicate subtype tokens like `protein_coding`/`protein-coding`); address opportunistically.

## How to fix
For each offending page: `bokf_read_page` → edit → `bokf_validate_page` → `bokf_write_page`. Re-run `bokf_lint` until Errors = 0. Record what you changed with `bokf_append_log`.

A missing `xref` is an enrichment opportunity, never an error. `subtype` is never linted against a list.
