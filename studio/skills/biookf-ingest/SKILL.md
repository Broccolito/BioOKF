---
name: biookf-ingest
description: Use when ingesting a source (paper, preprint, review, dataset, note) into a BioOKF knowledge base: the 7-step loop that turns one source into 10 to 15 typed concept docs with provenance-stamped edges.
---

# Skill: biookf-ingest

You are curating a **BioOKF v0.5** bundle. Drive the `biookf` MCP tools (or the `bokf` CLI). Never edit `raw/`. (Step 0, **biookf-convert**, has already put each source under `raw/` as faithful Markdown.)

**Extract exactly the right nodes and edges, no more and no fewer:**
- A **CONCEPT (node)** is a durable, typed, reusable knowledge unit denoting a stable referent that can stand alone as a wiki node. Mint a node ONLY for such a referent, never for a value, a one-off phrasing, or a relationship.
- A **RELATIONSHIP (edge)** is a typed, atomic, provenance-aware assertion connecting two canonical concepts via a controlled predicate. "A relates to B" is an edge, not a node; a measurement value or a variant consequence is edge data, never a node.

## The loop (per source)

1. **Anchor the source.** The bytes already live under `raw/<id>` (the host placed them). Create a **source node** for the source itself (a `Publication` (paper/preprint/review), `Study` (trial/cohort/GWAS), or `Dataset`) with `raw_source: [raw/<id>...]` and a human-readable `identifier`.
2. **Read it fully** (use `bokf_read_page` on the raw path). Note its modality and credibility. **View every figure with your vision**, not just its caption: for an office/html/md source the figures are files under `raw/<id>/figures/*` (use `bokf_read_page` on each); for a **PDF** the figures live in `raw/<id>/original.pdf`, so read its pages directly. Figure content carries claims: read the axes, labels, and legend, and extract typed nodes and provenance-stamped edges from it. A survival curve yields outcome edges; a pathway diagram yields `regulates`/`activates` edges; a bar chart of expression yields `expressed_in` edges with a `direction`. Stamp each figure-derived edge with `primary_source` set to the source node, the same as text-derived edges.
3. **Extract typed entities.** For each biomedical thing discussed in the text or shown in a figure, create or UPDATE a concept doc. `type` MUST be one of the 28; coin a lowercase `subtype`. **Reuse an existing `identifier`, never fork** (search first with `bokf_search`).
4. **Validate before writing.** Call `bokf_validate_page` on the draft, then `bokf_write_page` to `knowledge/<type>/<slug>.md`.
5. **Enrich `xref`** with ontology CURIEs where known (HGNC, MONDO, UniProt…). Optional; backfill later.
6. **Add edges with provenance.** Each claim becomes an `edges:` entry: `predicate` (one of 24 positive, forward-only, or `not_<X>` for an explicit negative finding, negatable only for the 11 effect predicates), `object` (target identifier), and the triplet `knowledge_level` / `agent_type` / `primary_source` (the **source node's identifier**). Put every number on the edge (`p_value`, `effect_size`+`effect_metric`, `ci_lower/upper`, `sample_size`, `direction` for regulates/expressed_in). Add a `reported_in` edge to the source node so provenance is traversable.
7. **Bookkeep + commit + verify.** Update `index.md` (`bokf_write_page`); record the step with `bokf_log_sync --kind ingest --summary "…" --counts` (appends to `log.md` AND git-commits, atomically). Then run **biookf-verify** / `bokf_verify`: fix every Error and walk the judgment checklist (each node a real concept, each edge atomic + provenance-aware) before declaring the source done.

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
- `identifier` is human-readable and bundle-unique, **not** a CURIE (CURIEs go in `xref`).
- `primary_source` names a **source node**, never `infores:…`. Create each external authority (HGNC, Gene Ontology) **once** as an `Agent`/`Dataset` node with its CURIE in `xref`.
- No inverse predicates: author `encodes` on the gene, not `encoded_by` on the protein.
- Quote any YAML scalar value that contains `": "` (a colon+space) or the frontmatter won't parse.
