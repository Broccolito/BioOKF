# BioOKF source catalog

This folder is the **empirical grounding** for the BioOKF type design. Twelve sub-agents each
surveyed a different biomedical source *modality* and *subfield*, cataloging real,
openly-accessible items (with working URLs) and analyzing which biomedical **entities** (node
types) and **relationships** (edge types) recur in that source class. Together they exceed the
brief's "50–100 of each kind" target.

| Catalog | Items* | Covers | Dominant entity / relation patterns |
|---|---:|---|---|
| [epi-clinical.md](epi-clinical.md) | ~104 | epidemiology, clinical trials, public health (PMC OA, PLOS) | population/cohort, exposure, outcome; `predisposes_to`/`associated_with` with OR/HR/RR + CI |
| [genetics-genomics.md](genetics-genomics.md) | ~90 | GWAS, sequencing, population & functional genomics | gene, variant, trait; `associated_with` (β, p-value), `regulates` (eQTL) |
| [molbio-biochem.md](molbio-biochem.md) | ~114 | molecular & cell biology, biochemistry | gene, protein, pathway; `encodes`, `binds`, `regulates`, `participates_in`, `catalyzes` |
| [pharma-medicine.md](pharma-medicine.md) | ~116 | pharmacology, pharmaceutical medicine | drug, target, disease; `binds` (IC50/Ki), `treats`, `interacts_with`, `affects_response_to` |
| [specialties.md](specialties.md) | ~104 | orthopaedics + cardiology, oncology, neurology, ID, surgery | disease, procedure, device, measure; `treats`, `located_in`, `measures` |
| [chem-chembio.md](chem-chembio.md) | ~80 | chemistry, chemical & medicinal chemistry | compound, target, reaction; `binds`, `regulates`, `converts_to`, SAR via `associated_with` |
| [preprints.md](preprints.md) | ~84 | bioRxiv, medRxiv, arXiv q-bio | full entity/edge mix; lower `knowledge_level` (not yet peer-reviewed) |
| [protocols-labnotes.md](protocols-labnotes.md) | ~80 | protocols.io, Nature/Bio/STAR Protocols, open lab notebooks | method, device, sample; `derives_from` lineage, `Procedure`/`Method` nodes |
| [slides-presentations.md](slides-presentations.md) | ~112 | SlideShare, Figshare/Zenodo decks, posters | same entities, encoded as bullets+figures; `reported_in` provenance |
| [datasets-tables.md](datasets-tables.md) | ~72 | GEO, figshare/Zenodo, Kaggle, supp tables (CSV/XLSX) | rows=entities, cols=measurements; `Dataset` nodes, `measures`/`associated_with` |
| [social-blogs.md](social-blogs.md) | ~92 | science Twitter/X threads, lab blogs, tweetorials | informal but dense; `Publication(tweet/blog)`, low `knowledge_level` |
| [images-figures.md](images-figures.md) | ~84 | KEGG/Reactome/WikiPathways diagrams, figure & bioimage repos | knowledge encoded visually (nodes+arrows, plots); pathway/regulation edges |
| **Total** | **~1,132** | all of biomedicine × all modalities | — |

\* Counts are approximate (counted from the manifest tables). Each manifest also contains an
"entity & relation patterns observed" section feeding [../docs/03-rationale.md](../docs/03-rationale.md).

## How this catalog shaped the spec

- The **node universe** ([SPEC.md §5](../SPEC.md#5-the-node-universe-20-types)) had to type
  *every* entity these sources mention — confirming the 13 entity types + the need for a
  provenance/context family (a tweet, a slide deck, a CSV, a protocol, and PCA all needed a
  home → `Publication`/`Dataset`/`Method`).
- The **edge universe** ([SPEC.md §6](../SPEC.md#6-the-edge-universe-23-types)) had to express
  *every* claim — confirming that the recurring shape is a **quantified, attribute-rich
  association**, which is why statistics are first-class edge attributes, not prose.
- The **provenance model** ([SPEC.md §8](../SPEC.md#8-provenance-evidence-and-quantitative-claims))
  is a direct response to the modality spread: gold-standard databases through tweets demand
  a mandatory `knowledge_level` + `agent_type` + `primary_source` on every edge.

> These manifests are catalogs (titles, URLs, formats, and the entity/relation patterns each
> exhibits), not bulk downloads of binary files — the intellectual payload (what entities and
> edges biomedical sources contain) is fully captured, which is what the type design needs.
