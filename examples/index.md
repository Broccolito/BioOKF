---
biookf_version: "0.5"
---

# Example BioOKF bundle — IL-6 / COVID-19 mini knowledge base

A tiny, illustrative **BioOKF v0.5** bundle distilled from a few sources about IL-6 biology and its
role in COVID-19. It demonstrates the bundle layout, several node types, the typed-edge layer, and —
new in v0.5 — **node-based provenance**: every claim's `primary_source` names a **source node** by
its `identifier`, and ingested sources anchor to `raw/` via `raw_source`. See
[../SPEC.md](../SPEC.md) for the normative format and [../schema.md](../schema.md) for the workflow.

## Concept pages — biomedical entities
- [knowledge/gene/il6](knowledge/gene/il6.md) — **Gene** (`protein_coding`) · IL6 (gene)
- [knowledge/molecule/il6-protein](knowledge/molecule/il6-protein.md) — **Molecule** (`protein`) · Interleukin-6
- [knowledge/molecule/tocilizumab](knowledge/molecule/tocilizumab.md) — **Molecule** (`antibody`) · IL-6R antagonist
- [knowledge/molecularclass/il6-inhibitors](knowledge/molecularclass/il6-inhibitors.md) — **MolecularClass** (`pharmacologic`) · the drug class
- [knowledge/disease/covid-19](knowledge/disease/covid-19.md) — **Disease** (`infection`) · COVID-19
- [knowledge/organism/sars-cov-2](knowledge/organism/sars-cov-2.md) — **Organism** (`pathogen`) · the cause of COVID-19
- [knowledge/sequencefeature/epas1-3prime-utr](knowledge/sequencefeature/epas1-3prime-utr.md) — **SequenceFeature** (`utr`) · the EPAS1 3' UTR (Variant-vs-SequenceFeature + class-vs-instance)

## Source nodes — provenance & context (new model in v0.5)
**Ingested-document sources** (carry a `raw_source` into `raw/`):
- [knowledge/study/recovery-trial](knowledge/study/recovery-trial.md) — **Study** (`rct`) · RECOVERY trial
- [knowledge/publication/il6-covid-review](knowledge/publication/il6-covid-review.md) — **Publication** (`article`) · IL-6 / COVID-19 review

**External-reference sources** (cited, not ingested — no `raw_source`; `infores:` CURIE in `xref`):
- Agents: [HGNC](knowledge/agent/hgnc.md) · [UniProt](knowledge/agent/uniprot.md)
- Datasets: [Gene Ontology](knowledge/dataset/gene-ontology.md) · [SemMedDB](knowledge/dataset/semmeddb.md) · [DrugBank](knowledge/dataset/drugbank.md) · [SIDER](knowledge/dataset/sider.md) · [ATC](knowledge/dataset/atc.md) · [MONDO](knowledge/dataset/mondo.md) · [HPO](knowledge/dataset/hpo.md) · [Ensembl](knowledge/dataset/ensembl.md)

## Relationships demonstrated
`encodes` · `binds` · `regulates` · `participates_in` · `causes` · `predisposes_to` ·
`associated_with` · `treats` · `has_phenotype` · `located_in` · `part_of` · `member_of` ·
`reported_in` — all **forward-only** (no inverse predicates).

> A few edge `object`s (`blood`, `IL6 receptor`, `inflammatory response`, `lung`, `neutropenia`,
> `EPAS1 (gene)`…) reference entities not yet given their own page — **broken links are tolerated**;
> lint flags them as enrichment opportunities, not errors.

## Log
See [log.md](log.md).
