---
type: Molecule
identifier: Alectinib
subtype: drug
synonyms: [Alecensa, CH5424802]
xref: [DRUGBANK:DB11363, CHEBI:90936]
edges:
  - predicate: treats
    object: Non-small-cell lung cancer
    knowledge_level: statistical_association
    agent_type: data_analysis_pipeline
    primary_source: "Vasan 2019 — A view on drug resistance in cancer"
    indication: "ALK-positive NSCLC"
    effect_metric: progression_free_survival
    effect_direction: prolonged
    note: "prolongs progression-free survival vs crizotinib (first-generation ALK inhibitor); active in crizotinib-resistant disease"
  - predicate: reported_in
    object: "Vasan 2019 — A view on drug resistance in cancer"
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: "Vasan 2019 — A view on drug resistance in cancer"
---

# Alectinib

Potent ALK inhibitor that binds more potently to its target than first-generation ALK inhibitors.
It is active in crizotinib-resistant ALK-positive NSCLC and, as a first-line treatment, prolongs
progression-free survival compared with crizotinib — an example of using the more-potent agent
upfront to forestall resistance.
