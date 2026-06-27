---
type: Molecule
identifier: Alpelisib
subtype: drug
synonyms: [BYL719, Piqray]
xref: [DRUGBANK:DB12015, CHEBI:90677]
edges:
  - predicate: treats
    object: Cancer drug resistance
    knowledge_level: statistical_association
    agent_type: data_analysis_pipeline
    primary_source: "Vasan 2019 — A view on drug resistance in cancer"
    indication: "ER+ PIK3CA-mutant breast cancer, combined with anti-endocrine therapy"
    effect_metric: progression_free_survival
    effect_direction: improved
    adverse_effect: hyperglycaemia
    note: "PI3K inhibitor with low therapeutic index; improves PFS in combination but causes on-target hyperglycaemia from insulin inhibition"
  - predicate: participates_in
    object: PI3K-AKT-mTOR signaling pathway
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: "Vasan 2019 — A view on drug resistance in cancer"
    role: inhibitor
  - predicate: reported_in
    object: "Vasan 2019 — A view on drug resistance in cancer"
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: "Vasan 2019 — A view on drug resistance in cancer"
---

# Alpelisib

PI3Kalpha-selective inhibitor with a low therapeutic index. As monotherapy it yields low response
rates, but combined with anti-endocrine therapy in ER+ PIK3CA-mutant breast cancer it improves
progression-free survival, at the cost of on-target hyperglycaemia from insulin inhibition.
Combining PI3K inhibition with a ketogenic diet (blunting adaptive insulin signalling) is proposed
to resensitize cells.
