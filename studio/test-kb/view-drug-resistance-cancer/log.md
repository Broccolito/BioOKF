# Change log — A view on drug resistance in cancer

## 2026-06-27

Ingested the review "A view on drug resistance in cancer" (Vasan, Baselga & Hyman, Nature 575,
299-309, 2019; DOI:10.1038/s41586-019-1730-1) from `raw/source.md`.

- Created the `Publication` source node `Vasan 2019 — A view on drug resistance in cancer`
  (raw_source: raw/source.md).
- Authored 21 typed concept docs across Disease, Gene, Variant, Molecule, MolecularClass,
  BiomedicalMeasure, BiologicalPathway, MethodOrProcedure and Phenotype.
  - Diseases: Cancer drug resistance, Non-small-cell lung cancer, Melanoma, HER2-positive breast
    cancer.
  - Genes: EGFR, BRAF, KRAS, ESR1, BRCA1.
  - Variants: EGFR T790M, KRAS G12C.
  - Molecules (drugs/ADC): Osimertinib, Alectinib, Trastuzumab deruxtecan (DS-8201a), Olaparib,
    Venetoclax, Alpelisib.
  - MolecularClass: PARP inhibitor.
  - BiomedicalMeasure: Circulating tumour DNA (ctDNA).
  - BiologicalPathway: PI3K-AKT-mTOR, RAS-RAF-MEK-ERK.
  - MethodOrProcedure: CRISPR-Cas9 synthetic lethality screen.
  - Phenotype: Tumour heterogeneity.
- Stamped each claim edge with the provenance triplet (knowledge_level, agent_type,
  primary_source = the Publication node) and added reported_in back-links. Effect sizes/metrics put
  on edges where reported (e.g. ~60% olaparib remission at 3 years; PFS prolongation for osimertinib
  and alectinib).
