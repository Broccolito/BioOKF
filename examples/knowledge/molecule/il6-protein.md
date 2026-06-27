---
type: Molecule
identifier: Interleukin-6 (protein)
subtype: protein
xref: [UniProtKB:P05231, MESH:D015850, CHEMBL:CHEMBL4408, PR:000001137]
synonyms: [IL-6, interferon beta-2, BSF-2, hybridoma growth factor]
in_taxon: NCBITaxon:9606
description: Pro-inflammatory cytokine secreted by macrophages and T cells; signals via IL6R.
edges:
  - predicate: binds
    object: IL6 receptor                       # UniProtKB:P08887 — no page yet (tolerated broken link)
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: UniProt
  - predicate: participates_in
    object: inflammatory response              # GO:0006954 — no page yet
    knowledge_level: knowledge_assertion
    agent_type: automated_agent
    primary_source: Gene Ontology
    evidence_type: [ECO:0000501]
  - predicate: predisposes_to
    object: COVID-19
    knowledge_level: statistical_association
    agent_type: data_analysis_pipeline
    primary_source: SemMedDB
    effect_metric: odds_ratio
    effect_size: 2.2
    ci_lower: 1.5
    ci_upper: 3.3
    p_value: 1.0e-4
  - predicate: reported_in
    object: IL-6 in severe COVID-19 (review)
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: IL-6 in severe COVID-19 (review)
---

# Interleukin-6 (protein)

The secreted cytokine product of [IL6 (gene)](../gene/il6.md), and the pharmacological target of
[Tocilizumab](tocilizumab.md). This is a **separate node from the gene** (identity, not role) —
they are linked by the gene's forward `encodes` edge, **not** an `encoded_by` edge here
(predicates are forward-only).

## Citations
- [UniProt P05231](https://www.uniprot.org/uniprotkb/P05231/entry)
