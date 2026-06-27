---
type: BiomedicalMeasure
identifier: Circulating tumour DNA
subtype: biomarker
synonyms: [ctDNA, "circulating tumor DNA", "liquid biopsy"]
xref: [NCIT:C113739]
edges:
  - predicate: measures
    object: Cancer drug resistance
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: "Vasan 2019 — A view on drug resistance in cancer"
    note: "non-invasive, dynamic, global detection of tumour burden and clonal evolution under therapy"
  - predicate: measures
    object: Non-small-cell lung cancer
    knowledge_level: statistical_association
    agent_type: data_analysis_pipeline
    primary_source: "Vasan 2019 — A view on drug resistance in cancer"
    note: "tracking 5 mutations improves minimal-residual-disease detection vs 1 mutation in early-stage NSCLC"
  - predicate: reported_in
    object: "Vasan 2019 — A view on drug resistance in cancer"
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: "Vasan 2019 — A view on drug resistance in cancer"
---

# Circulating tumour DNA

ctDNA is tumour-derived DNA detectable in plasma — the basis of non-invasive "liquid biopsies."
The review presents it as a tool for early detection, depth-of-response quantification, and adaptive
monitoring of clonal evolution. After primary breast surgery, ctDNA detected minimal residual
disease eight months before imaging; in early NSCLC, tracking five mutations rather than one
improves detection; and ctDNA detection of EGFR T790M or emergent KRAS alleles guides therapy
switching. Low-burden detection is limited by template number, which is proportional to tumour
burden.
