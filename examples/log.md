# Change log

## 2026-06-27
- **Migrated the bundle to BioOKF v0.5 (node-based provenance).** Replaced every `primary_source:
  infores:X` with a reference to a **source node** by `identifier`, and created those source nodes:
  ingested-document sources `RECOVERY trial` (Study) and `IL-6 in severe COVID-19 (review)`
  (Publication), each with a `raw_source` into `raw/`; and external-reference sources `HGNC`,
  `UniProt` (Agents) and `Gene Ontology`, `SemMedDB`, `DrugBank`, `SIDER`, `ATC`, `MONDO`, `HPO`,
  `Ensembl` (Datasets, `infores:` CURIE in `xref`, no `raw_source`).
- **Dropped `provided_by`** (node-level source is now a `reported_in` edge).
- **Made predicates forward-only:** removed `encoded_by` (the `encodes` edge lives on the gene),
  `caused_by` (now `SARS-CoV-2 causes COVID-19`), and `treated_by` (the `treats` edge lives on the
  drug). Added the `SARS-CoV-2` Organism node to carry the forward `causes` edge.
- Also folded in the earlier v0.4 renames carried by this bundle: `title`+`id` → `identifier`
  (CURIEs to `xref`); `*_kind` → `subtype`; `GenomicFeature` → `SequenceFeature` (the EPAS1 3' UTR
  page moved to `knowledge/sequencefeature/`); added the missing `IL-6 inhibitors` MolecularClass
  so tocilizumab's `member_of` resolves.

## 2026-06-25
- **Ingested** RECOVERY trial primary report (PMID:33933206) → created the `RECOVERY trial` Study;
  added `treats` edge Tocilizumab→COVID-19 (RR 0.85, 95% CI 0.76–0.94, n=4116).
- **Ingested** review of IL-6 in severe COVID-19 (PMID:32504360, PMID:32979574) → created
  `IL6 (gene)`, `Interleukin-6 (protein)`, `COVID-19`; added `associated_with` IL6↔COVID-19 (HR 2.9).
- **Ingested** DrugBank entry for tocilizumab (DB06273) → created `Tocilizumab`; added `binds`
  (IL6R, Kd 2.5 nM), `regulates` (IL-6 signaling, decreased), `has_phenotype` (neutropenia),
  `member_of` (IL-6 inhibitors).
