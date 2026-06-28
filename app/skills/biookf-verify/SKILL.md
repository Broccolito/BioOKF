---
name: biookf-verify
description: Use at the END of an ingest/merge run to double-check the work: runs the deterministic gate and adjudicates the judgment-only checklist (concept/relationship definitions, classification tiebreakers, true-match, lose-no-info).
---

# Skill: biookf-verify

Two halves. The **deterministic gate** (`bokf verify --workflow ingest|merge --json`) must show
`ok: true` (zero lint errors). The **judgment review** below only you can do. Fix what fails, then
re-check both.

## The definitions that set extraction granularity (no more, no fewer)
- A **CONCEPT (node)** is a *durable, typed, reusable biomedical knowledge unit that denotes a
  stable referent and can stand alone as a wiki node*: something you can point at independent of
  what it relates to. Not a value, a one-off phrasing, or a relationship.
- A **RELATIONSHIP (edge)** is a *typed, atomic, provenance-aware assertion connecting two canonical
  concepts through a controlled predicate.*

## Judgment checklist (the gate cannot decide these)
- [ ] Every node is a durable/reusable concept (a standalone referent), **not** a relationship, a
      measurement value, or an ephemeral phrasing. No over- or under-extraction.
- [ ] Every edge is **atomic** (one claim), **provenance-aware** (triplet present), and connects two
      **canonical** concept nodes.
- [ ] Classification is by **identity, not role** (TB-2); ambiguous types resolved via `docs/04`
      (TB-1/3, the Disease/Phenotype/BiomedicalMeasure trio, the §5.D boundary tests).
- [ ] (Ingest) every claim in the source is captured as an edge; nothing invented beyond the source
      + your knowledge of the named entity.
- [ ] (Merge) collapses were genuine same-concept matches; no information lost; contradictions kept
      (both, each tagged with its source); the MKB stayed canonical.

## Deterministic backstops (`bokf verify` gates on Errors)
28 types / 35 predicates; provenance triplet + enums; object & primary_source resolve;
`value.as_identifier`; `other.missing_note`; `node.no_reported_in`; `source.raw_missing_file`;
`source.needs_conversion` (every raw source rendered to faithful `.md`); domain/range; §7.3 stat
sanity; duplicate edges; `type.path_mismatch` (misclassification); `edge.not_negatable`;
`subtype.similar`; no duplicate identifiers.

Record the verdict: `bokf log-sync <bundle> --kind lint --summary "verify: PASS/FAIL: <notes>"`.
