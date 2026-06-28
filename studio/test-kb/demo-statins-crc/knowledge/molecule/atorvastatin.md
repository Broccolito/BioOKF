---
type: Molecule
identifier: Atorvastatin
subtype: statin
xref: [DRUGBANK:DB01076, CHEMBL:CHEMBL1487, RXNORM:83367, UNII:A0JWA85V8F]
synonyms: [Lipitor, atorvastatin calcium]
in_taxon: NCBITaxon:9606
description: HMG-CoA reductase inhibitor (statin) used to lower LDL cholesterol.
edges:
  - predicate: binds
    object: HMG-CoA reductase            # UniProtKB:P04035 — no page yet (tolerated broken link)
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: DrugBank
    effect_metric: IC50
    effect_size: 8.0
    unit: nM
  - predicate: not_prevents              # NEGATIVE (directed): the cohort found no protective effect
    object: Colorectal cancer
    knowledge_level: statistical_association
    agent_type: data_analysis_pipeline
    primary_source: Statins and colorectal-cancer risk (cohort)
    effect_metric: relative_risk
    effect_size: 0.98
    ci_lower: 0.90
    ci_upper: 1.07
    p_value: 0.62
    sample_size: 132000
    publications: [PMID:30000001]
  - predicate: treats                    # POSITIVE claim whose only source is RETRACTED -> flagged
    object: Colorectal cancer
    knowledge_level: prediction
    agent_type: computational_model
    primary_source: Statin antitumor activity (preprint)
  - predicate: reported_in
    object: Statins and colorectal-cancer risk (cohort)
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: Statins and colorectal-cancer risk (cohort)
  - predicate: reported_in
    object: Statin antitumor activity (preprint)
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: Statin antitumor activity (preprint)
---

# Atorvastatin

A **statin** that **binds** and inhibits HMG-CoA reductase. A large prospective cohort found it does
**not prevent** [colorectal cancer](../disease/colorectal-cancer.md) (RR 0.98, 95% CI 0.90–1.07) —
encoded as the negative `not_prevents` edge. A separate `treats` claim is backed only by a
**retracted** preprint, so the UI should flag that provenance.
