---
type: Molecule
identifier: Olaparib
subtype: drug
synonyms: [Lynparza, AZD2281]
xref: [DRUGBANK:DB09074, CHEBI:83766]
edges:
  - predicate: treats
    object: Cancer drug resistance
    knowledge_level: statistical_association
    agent_type: data_analysis_pipeline
    primary_source: "Vasan 2019 — A view on drug resistance in cancer"
    indication: "germline BRCA-mutant ovarian cancer, maintenance after platinum-based chemotherapy"
    effect_metric: remission_rate
    effect_size: 0.60
    timepoint: "3 years after start of PARP inhibitor therapy"
    note: "two years of maintenance olaparib drives deeper, more durable responses; ~60% remission at 3 years"
  - predicate: member_of
    object: PARP inhibitor
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: "Vasan 2019 — A view on drug resistance in cancer"
  - predicate: reported_in
    object: "Vasan 2019 — A view on drug resistance in cancer"
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: "Vasan 2019 — A view on drug resistance in cancer"
---

# Olaparib

PARP inhibitor that exploits synthetic lethality in homologous-recombination-deficient (BRCA-mutant)
tumours. Two years of maintenance olaparib in newly diagnosed germline BRCA-mutant ovarian cancer
deepens responses after platinum-based chemotherapy, with a remission rate of about 60% at three
years — suggesting that early, aggressive use increases the fraction of patients cured. Acquired
resistance can arise via BRCA reversion mutations.
