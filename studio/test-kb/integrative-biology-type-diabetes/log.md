# Change log — Integrative Biology of Type 2 Diabetes

## 2026-06-27

Ingested the review article **"The integrative biology of type 2 diabetes"** (Roden & Shulman,
*Nature* 576:51-60, 2019; DOI:10.1038/s41586-019-1797-8) into a fresh BioOKF v0.5 bundle.

- Saved the source unchanged to `raw/source.md`.
- Created 2 source/provenance nodes: 1 ingested `Publication` (the review, anchored to
  `raw/source.md`) and 1 external-reference `Study` (the Mahajan 2018 T2D coding-variant GWAS,
  cited not ingested).
- Authored 25 typed concept docs spanning 11 node types: Disease (3), Phenotype (3), Anatomy (4),
  CellType (1), Molecule (5), BiologicalPathway (3), Gene (4), Exposure (2), MethodOrProcedure (1),
  Organism (1).
- Added 92 provenance-stamped edges. Key claims captured: insulin resistance as the common
  abnormality of obesity and T2D; the plasma-membrane sn-1,2-DAG -> PKCepsilon -> insulin-receptor
  (Thr1160) mechanism of hepatic insulin resistance; acetyl-CoA allosteric activation of
  gluconeogenesis driving fasting hyperglycaemia; NEFA flux from insulin-resistant WAT driving
  NAFLD; the role of pancreatic beta-cell compensation/failure; gene variants (TCF7L2, TM6SF2,
  NAT2, TBC1D4); a single saturated-fat load associated with ~70% higher hepatic gluconeogenesis;
  exercise reversing insulin resistance via GLUT4; and the Astyanax mexicanus cavefish IR-mutation
  model of adaptive insulin resistance.
- Lint: 0 errors, 0 warnings (every edge object resolves to a created node).
