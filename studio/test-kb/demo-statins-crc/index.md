---
biookf_version: "0.5"
---

# Demo bundle — statins & colorectal cancer

A tiny **BioOKF v0.5** bundle for exercising the Studio UI on features that are new since the
redesign: **negative (`not_*`) predicates**, **node-based provenance**, and **ingested-source
credibility + figures**. See [../../../SPEC.md](../../../SPEC.md) and [../../../SCHEMA.md](../../../SCHEMA.md).

## Concept pages — biomedical entities
- [knowledge/molecule/atorvastatin](knowledge/molecule/atorvastatin.md) — **Molecule** (`statin`)
- [knowledge/gene/hmgcr](knowledge/gene/hmgcr.md) — **Gene** (`protein_coding`) · the statin target
- [knowledge/disease/colorectal-cancer](knowledge/disease/colorectal-cancer.md) — **Disease** (`neoplasm`)

## Source nodes — provenance & context
**Ingested-document sources** (carry a `raw_source` into `raw/<id>/`, with `meta.yaml` credibility + figures):
- [knowledge/publication/statin-crc-cohort](knowledge/publication/statin-crc-cohort.md) — **Publication** · peer-reviewed cohort (figure)
- [knowledge/publication/statin-antitumor-preprint](knowledge/publication/statin-antitumor-preprint.md) — **Publication** · **retracted** preprint

**External-reference source** (cited, not ingested — `infores:` CURIE in `xref`, no `raw_source`):
- [knowledge/dataset/drugbank](knowledge/dataset/drugbank.md) — **Dataset**

## Relationships demonstrated
`binds` · `not_prevents` (directed negative) · `not_associated_with` (symmetric negative) ·
`treats` (citing a **retracted** source) · `used_to_study` · `reported_in` — all node-based provenance.

## Log
See [log.md](log.md).

<!-- bokf:index:start -->
## Identifier registry

| identifier | type | subtype | description |
|---|---|---|---|
| Atorvastatin | Molecule | statin | HMG-CoA reductase inhibitor (statin) used to lower LDL cholesterol. |
| Colorectal cancer | Disease | neoplasm | Malignant neoplasm arising from the colon or rectum. |
| DrugBank | Dataset | drug-database | Curated database of drugs and drug targets. |
| HMGCR (gene) | Gene | protein_coding | Protein-coding gene encoding HMG-CoA reductase, the rate-limiting enzyme of chol |
| Statin antitumor activity (preprint) | Publication | preprint | Retracted preprint claiming high-dose atorvastatin shrinks colorectal tumors. |
| Statins and colorectal-cancer risk (cohort) | Publication | article | Prospective cohort reporting no association between statin use and colorectal-ca |

## By type

- **Dataset** (1): DrugBank
- **Disease** (1): Colorectal cancer
- **Gene** (1): HMGCR (gene)
- **Molecule** (1): Atorvastatin
- **Publication** (2): Statin antitumor activity (preprint), Statins and colorectal-cancer risk (cohort)

## Subtypes in use

- **Dataset**: drug-database
- **Disease**: neoplasm
- **Gene**: protein_coding
- **Molecule**: statin
- **Publication**: article, preprint
<!-- bokf:index:end -->
