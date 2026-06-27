---
type: Molecule
identifier: Tocilizumab
subtype: antibody
xref: [DRUGBANK:DB06273, CHEMBL:CHEMBL1237022, RXNORM:612865, UNII:I031V2H011]
synonyms: [atlizumab, Actemra, RoActemra]
in_taxon: NCBITaxon:9606
description: Recombinant humanized monoclonal antibody against the IL-6 receptor (IL6R).
edges:
  - predicate: binds
    object: IL6 receptor                       # UniProtKB:P08887
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: DrugBank
    effect_metric: Kd
    effect_size: 2.5                           # nM
  - predicate: regulates
    object: interleukin-6-mediated signaling   # GO:0070102
    direction: decreased
    aspect: activity
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: DrugBank
  - predicate: treats
    object: COVID-19
    clinical_phase: approved
    knowledge_level: statistical_association
    agent_type: data_analysis_pipeline
    primary_source: RECOVERY trial
    effect_metric: relative_risk
    effect_size: 0.85
    ci_lower: 0.76
    ci_upper: 0.94
    sample_size: 4116
    publications: [PMID:33933206]
  - predicate: has_phenotype                   # adverse effect
    object: neutropenia                        # HP:0001875
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: SIDER
    frequency: common
  - predicate: member_of
    object: IL-6 inhibitors
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: ATC
  - predicate: reported_in
    object: RECOVERY trial
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: RECOVERY trial
---

# Tocilizumab

A humanized monoclonal antibody that **binds** the IL-6 receptor and thereby **regulates** IL-6
signaling downward. Shown in [RECOVERY trial](../study/recovery-trial.md) to **treat**
[COVID-19](../disease/covid-19.md) (RR 0.85). Every `primary_source` above names a **source node**:
`DrugBank`/`SIDER`/`ATC` are external-reference `Dataset` nodes; `RECOVERY trial` is an
ingested-document `Study` with a `raw_source`.

## Citations
- [Tocilizumab in hospital COVID-19 (RECOVERY)](https://pubmed.ncbi.nlm.nih.gov/33933206/) (PMID:33933206)
