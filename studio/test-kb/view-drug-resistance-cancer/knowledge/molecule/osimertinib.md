---
type: Molecule
identifier: Osimertinib
subtype: drug
synonyms: [AZD9291, Tagrisso]
xref: [DRUGBANK:DB09330, CHEBI:90943]
edges:
  - predicate: treats
    object: Non-small-cell lung cancer
    knowledge_level: statistical_association
    agent_type: data_analysis_pipeline
    primary_source: "Vasan 2019 — A view on drug resistance in cancer"
    indication: "EGFR T790M-positive / EGFR-mutant NSCLC"
    effect_metric: progression_free_survival
    effect_direction: prolonged
    note: "prolongs progression-free survival vs first-generation EGFR inhibitors as first-line treatment"
  - predicate: binds
    object: EGFR
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: "Vasan 2019 — A view on drug resistance in cancer"
    mechanism: "irreversible, mutant-selective binding to the EGFR kinase domain"
  - predicate: reported_in
    object: "Vasan 2019 — A view on drug resistance in cancer"
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: "Vasan 2019 — A view on drug resistance in cancer"
---

# Osimertinib

Third-generation, mutant-selective, irreversible EGFR tyrosine kinase inhibitor. It overcomes
on-target acquired resistance mediated by the EGFR T790M gatekeeper mutation and, used first-line,
prolongs progression-free survival in EGFR-mutated advanced NSCLC compared with first-generation
EGFR inhibitors.
