---
type: Gene
identifier: IL6 (gene)
subtype: protein_coding
xref: [HGNC:6018, NCBIGene:3569, ENSEMBL:ENSG00000136244, OMIM:147620]
synonyms: [IL-6, interleukin 6, BSF2, HGF, IFNB2]
in_taxon: NCBITaxon:9606
description: Protein-coding gene encoding the pro-inflammatory cytokine interleukin-6.
edges:
  - predicate: encodes                         # forward-only: the gene→protein link lives here
    object: Interleukin-6 (protein)
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: HGNC
  - predicate: located_in
    object: blood                              # UBERON:0000178 — no page yet (tolerated broken link)
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: UniProt
  - predicate: associated_with
    object: COVID-19
    knowledge_level: statistical_association
    agent_type: text_mining_agent
    primary_source: SemMedDB
    publications: [PMID:32504360, PMID:32979574]
    effect_metric: hazard_ratio
    effect_size: 2.9
    ci_lower: 1.7
    ci_upper: 4.9
    p_value: 3.0e-6
    sample_size: 1484
  - predicate: reported_in
    object: IL-6 in severe COVID-19 (review)
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: IL-6 in severe COVID-19 (review)   # reported_in: the source attests itself
---

# IL6 (gene)

The protein-coding gene on chromosome 7p15.3 encoding **interleukin-6**. Its product
([Interleukin-6 (protein)](../molecule/il6-protein.md)) drives the cytokine-release phenotype of
severe [COVID-19](../disease/covid-19.md). The gene→protein link is the forward `encodes` edge
**here** — there is no `encoded_by` on the protein page.

## Citations
- [Elevated IL-6 and severe COVID-19](https://pubmed.ncbi.nlm.nih.gov/32504360/) (PMID:32504360)
