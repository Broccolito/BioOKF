---
type: Disease
identifier: COVID-19
subtype: infection
xref: [MONDO:0100096, DOID:0080600, MESH:D000086382, ICD10:U07.1, UMLS:C5203670]
synonyms: [coronavirus disease 2019, SARS-CoV-2 infection]
description: Respiratory and systemic disease caused by SARS-CoV-2 infection.
edges:
  - predicate: located_in
    object: lung                               # UBERON:0002048
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: MONDO
  - predicate: has_phenotype
    object: respiratory distress               # HP:0002098
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: HPO
    frequency: frequent
  - predicate: associated_with
    object: IL6 (gene)
    knowledge_level: statistical_association
    agent_type: text_mining_agent
    primary_source: SemMedDB
    effect_metric: hazard_ratio
    effect_size: 2.9
    ci_lower: 1.7
    ci_upper: 4.9
  - predicate: reported_in
    object: IL-6 in severe COVID-19 (review)
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: IL-6 in severe COVID-19 (review)
---

# COVID-19

The disease caused by SARS-CoV-2. Its etiology is the forward `causes` edge on the
[SARS-CoV-2](../organism/sars-cov-2.md) node (forward-only — there is no `caused_by` here).
Severe disease features an IL-6-driven phenotype, hence the `associated_with` link to
[IL6 (gene)](../gene/il6.md) and why [Tocilizumab](../molecule/tocilizumab.md) `treats` it.

## Citations
- [MONDO:0100096](https://monarchinitiative.org/disease/MONDO:0100096)
