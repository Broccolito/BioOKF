---
name: biookf-ingest
description: Use when ingesting a source (paper, preprint, review, dataset, note) into a BioOKF knowledge base — the 7-step loop that turns one source into 10–15 typed concept docs with provenance-stamped edges.
---

# Skill: biookf-ingest

You are curating a **BioOKF v0.5** bundle. Drive the `biookf` MCP tools (or the `okf` CLI). Never edit `raw/`.

## The loop (per source)

1. **Anchor the source.** The bytes already live under `raw/<id>` (the host placed them). Create a **source node** for the source itself — a `Publication` (paper/preprint/review), `Study` (trial/cohort/GWAS), or `Dataset` — with `raw_source: [raw/<id>...]` and a human-readable `identifier`.
2. **Read it fully** (use `okf_read_page` on the raw path). Note its modality and credibility.
3. **Extract typed entities.** For each biomedical thing discussed, create or UPDATE a concept doc. `type` MUST be one of the 28; coin a lowercase `subtype`. **Reuse an existing `identifier` — never fork** (search first with `okf_search`).
4. **Validate before writing.** Call `okf_validate_page` on the draft, then `okf_write_page` to `knowledge/<type>/<slug>.md`.
5. **Enrich `xref`** with ontology CURIEs where known (HGNC, MONDO, UniProt…). Optional; backfill later.
6. **Add edges with provenance.** Each claim → an `edges:` entry: `predicate` (one of 23, forward-only), `object` (target identifier), and the triplet `knowledge_level` / `agent_type` / `primary_source` (the **source node's identifier**). Put every number on the edge (`p_value`, `effect_size`+`effect_metric`, `ci_lower/upper`, `sample_size`, `direction` for regulates/expressed_in). Add a `reported_in` edge to the source node so provenance is traversable.
7. **Bookkeep.** Update `index.md` (`okf_write_page`) and append a dated entry to `log.md` (`okf_append_log`). Then `okf_lint` and fix any Errors.

## Concept-doc template

```markdown
---
type: Molecule
identifier: Tocilizumab
subtype: drug
synonyms: [Actemra, RoActemra]
xref: [DRUGBANK:DB06273]
edges:
  - predicate: treats
    object: COVID-19
    knowledge_level: statistical_association
    agent_type: data_analysis_pipeline
    primary_source: RECOVERY trial
    effect_metric: relative_risk
    effect_size: 0.85
    ci_lower: 0.76
    ci_upper: 0.94
  - predicate: reported_in
    object: RECOVERY trial
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: RECOVERY trial
---

# Tocilizumab
Prose with citations.
```

## Pitfalls
- `identifier` is human-readable and bundle-unique — **not** a CURIE (CURIEs go in `xref`).
- `primary_source` names a **source node**, never `infores:…`. Create each external authority (HGNC, Gene Ontology) **once** as an `Agent`/`Dataset` node with its CURIE in `xref`.
- No inverse predicates: author `encodes` on the gene, not `encoded_by` on the protein.
- Quote any YAML scalar value that contains `": "` (a colon+space) or the frontmatter won't parse.
