---
type: Gene
identifier: HMGCR (gene)
subtype: protein_coding
xref: [HGNC:5006, NCBIGene:3156, ENSEMBL:ENSG00000113161, OMIM:142910]
synonyms: [HMG-CoA reductase gene, LDLCQ3]
in_taxon: NCBITaxon:9606
description: Protein-coding gene encoding HMG-CoA reductase, the rate-limiting enzyme of cholesterol synthesis and the statin target.
edges:
  - predicate: not_associated_with        # NEGATIVE (symmetric): no genetic association with the cancer
    object: Colorectal cancer
    knowledge_level: statistical_association
    agent_type: data_analysis_pipeline
    primary_source: Statins and colorectal-cancer risk (cohort)
    p_value: 0.41
  - predicate: reported_in
    object: Statins and colorectal-cancer risk (cohort)
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: Statins and colorectal-cancer risk (cohort)
---

# HMGCR (gene)

Encodes HMG-CoA reductase, the enzyme [atorvastatin](../molecule/atorvastatin.md) inhibits. A
Mendelian-randomization analysis in the cohort found **no association** between HMGCR variation and
[colorectal cancer](../disease/colorectal-cancer.md) — the symmetric `not_associated_with` edge.
