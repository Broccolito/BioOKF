# Biomedical NER & Relation-Extraction Benchmark Schemas — Reference

A precise, exhaustive catalog of the **entity TYPE sets** and **relation/event TYPE sets** defined by the major biomedical and clinical named-entity-recognition (NER) and relation-extraction (RE) benchmarks, corpora, and tools. Type names are reproduced verbatim as the originating dataset/tool defines them (including underscores, hyphens, and casing where it is load-bearing).

Last verified: 2026-06-25.

---

## Contents

1. [BioRED](#1-biored)
2. [BC5CDR (BioCreative V CDR)](#2-bc5cdr-biocreative-v-cdr)
3. [ChemProt (BioCreative VI)](#3-chemprot-biocreative-vi)
4. [DrugProt (BioCreative VII)](#4-drugprot-biocreative-vii)
5. [DDIExtraction 2013 (drug–drug interaction)](#5-ddiextraction-2013-drugdrug-interaction)
6. [i2b2 2010 / n2c2 clinical relations](#6-i2b2-2010-clinical-concepts-assertions-relations)
7. [n2c2 2018 Track 2 (ADE & Medication)](#7-n2c2-2018-track-2--ade--medication-extraction)
8. [scispaCy NER models & linkers](#8-scispacy-ner-models--linkers)
9. [medspaCy clinical labels](#9-medspacy)
10. [GENIA corpus (term ontology) & JNLPBA](#10-genia-corpus--jnlpba)
11. [BioNLP Shared Task event types](#11-bionlp-shared-task-event-types)
12. [Quick cross-reference summary](#12-quick-cross-reference-summary)

---

## 1. BioRED

**Document-level** biomedical RE dataset over 600 PubMed abstracts; multi-entity, multi-relation. Each relation is **non-directional** (entity pair) and additionally carries a binary **Novelty** attribute (`Novel` vs known background knowledge).

### Entity / concept types (6)

| Type | Definition |
|------|------------|
| `GeneOrGeneProduct` (Gene) | Genes, proteins, mRNA and other gene products |
| `ChemicalEntity` (Chemical) | Chemicals and drugs |
| `DiseaseOrPhenotypicFeature` (Disease) | Diseases, symptoms, and some disease-related phenotypes |
| `SequenceVariant` (Variant) | Genomic/protein variants — substitutions, deletions, insertions, etc. |
| `OrganismTaxon` (Species) | Species in the hierarchical taxonomy of organisms |
| `CellLine` | Cell lines |

(Short forms `Gene`, `Chemical`, `Disease`, `Variant`, `Species`, `CellLine` are commonly used in the literature; the verbose forms above are the corpus tags.)

### Relation types (8)

| Type | Meaning |
|------|---------|
| `Positive_Correlation` | One entity increases/co-occurs positively with the other (activating/beneficial; e.g., chemical increases disease risk, gene up with disease) |
| `Negative_Correlation` | Inhibitory/protective inverse association (e.g., drug reduces disease) |
| `Association` | General/unspecified connection between two entities |
| `Bind` | Direct molecular binding (e.g., chemical–gene, gene–gene) |
| `Cotreatment` | Two chemicals jointly used to treat a condition |
| `Comparison` | Two entities contrasted (e.g., two drugs compared) |
| `Drug_Interaction` | Interaction between chemical substances (drug–drug) |
| `Conversion` | One entity is transformed/metabolized into another |

> Note: some BioCreative VIII BioRED-track materials and tooling collapse/rename a few labels; the canonical eight above are from the original BioRED paper (Luo et al., *Briefings in Bioinformatics* 2022). The paper enumerates the label set as Association, Bind, Cause/Conversion, Comparison, Cotreatment, Drug_Interaction, Negative_Correlation, Positive_Correlation. "Cause"/"Conversion" naming varies across releases; treat `Conversion` and `Cause` as the same metabolic-transformation slot.

### Entity-pair combinations relations are annotated over (8)

`<Disease,Gene>`, `<Disease,Chemical>`, `<Gene,Chemical>`, `<Gene,Gene>`, `<Disease,Variant>`, `<Chemical,Variant>`, `<Chemical,Chemical>`, `<Variant,Variant>`

### Attributes
- **Novelty** (per relation): `Novel` | `No` (i.e., known background).

---

## 2. BC5CDR (BioCreative V CDR)

1500 PubMed abstracts (500 train / 500 dev / 500 test). Entities normalized to **MeSH**; the CID relation annotated **at the document level** (not per-mention) by CTD curators. Used for the BioCreative V Disease NER (DNER) and Chemical-Induced Disease (CID) RE tasks.

### Entity types (2)

| Type | Definition |
|------|------------|
| `Chemical` | Chemicals and drugs (MeSH-normalized) |
| `Disease` | Diseases, symptoms, disease-related phenotypes (MeSH-normalized) |

### Relation types (1)

| Type | Direction | Meaning |
|------|-----------|---------|
| `CID` (Chemical-Induced Disease) | **Directional**: Chemical → Disease | The chemical induces/causes the disease (document-level chemical–disease pair) |

There is exactly **one** relation label (CID). ~30% of CID relations are cross-sentence (hence the document-level framing).

---

## 3. ChemProt (BioCreative VI)

Chemical–protein interaction corpus. Fine-grained relations are grouped into **10 "ChemProt Relation" (CPR) groups**, CPR:1–CPR:10. Only **five** groups (CPR:3, CPR:4, CPR:5, CPR:6, CPR:9 — flagged "Y") are used for the official benchmark evaluation.

### Entity types (2)

| Type | Definition |
|------|------------|
| `CHEMICAL` | Chemical entity mentions (CEMs) — compounds, drugs |
| `GENE` | Gene/protein-related objects (GPROs); subtyped `GENE-Y` (DB-normalizable) and `GENE-N` (non-normalizable) |

### Relation groups (CPR:1–CPR:10) and their member sub-relations

| CPR group | Evaluated? | Member fine-grained relations |
|-----------|:---------:|-------------------------------|
| `CPR:1` | no | PART_OF |
| `CPR:2` | no | REGULATOR, DIRECT_REGULATOR, INDIRECT_REGULATOR |
| `CPR:3` | **yes** | UPREGULATOR, ACTIVATOR, INDIRECT_UPREGULATOR |
| `CPR:4` | **yes** | DOWNREGULATOR, INHIBITOR, INDIRECT_DOWNREGULATOR |
| `CPR:5` | **yes** | AGONIST, AGONIST-ACTIVATOR, AGONIST-INHIBITOR |
| `CPR:6` | **yes** | ANTAGONIST |
| `CPR:7` | no | MODULATOR, MODULATOR-ACTIVATOR, MODULATOR-INHIBITOR |
| `CPR:8` | no | COFACTOR |
| `CPR:9` | **yes** | SUBSTRATE, PRODUCT_OF, SUBSTRATE_PRODUCT_OF |
| `CPR:10` | no | NOT (explicitly no relation) |

The five evaluated groups are commonly described as: **CPR:3 = upregulator/activator, CPR:4 = downregulator/inhibitor, CPR:5 = agonist, CPR:6 = antagonist, CPR:9 = substrate/product**.

---

## 4. DrugProt (BioCreative VII)

Successor to ChemProt. 5000 PubMed abstracts, manually annotated with granular drug–gene/protein interactions. Replaces the 10 grouped CPR classes with **13 flat, directional relation types**.

### Entity types (2)

| Type | Definition |
|------|------------|
| `CHEMICAL` | Chemical Entity Mentions (CEMs) — drugs/compounds |
| `GENE` | Gene & Protein Related Objects (GPROs); `GENE-Y`/`GENE-N` distinction in the gene corpus, both collapsed to `GENE` for relation extraction |

### Relation types (13)

| # | Type |
|---|------|
| 1 | `ACTIVATOR` |
| 2 | `AGONIST` |
| 3 | `AGONIST-ACTIVATOR` |
| 4 | `AGONIST-INHIBITOR` |
| 5 | `ANTAGONIST` |
| 6 | `DIRECT-REGULATOR` |
| 7 | `INDIRECT-DOWNREGULATOR` |
| 8 | `INDIRECT-UPREGULATOR` |
| 9 | `INHIBITOR` |
| 10 | `PART-OF` |
| 11 | `PRODUCT-OF` |
| 12 | `SUBSTRATE` |
| 13 | `SUBSTRATE_PRODUCT-OF` |

All relations are directional `CHEMICAL → GENE`. `INHIBITOR` is the most frequent class; `AGONIST-INHIBITOR` the rarest. The DrugProt-merged variants (e.g., "Merged ChemProt-DrugProt") map ChemProt CPR groups onto these 13 where possible.

---

## 5. DDIExtraction 2013 (drug–drug interaction)

DDI corpus from **DrugBank** + **MEDLINE** abstracts. Two subtasks: drug NER (4 entity types) and DDI relation classification (4 positive types + False).

### Entity types (4)

| Type | Definition |
|------|------------|
| `drug` | Generic/approved drug names (human-use, in DrugBank) |
| `brand` | Branded/trade-name drugs |
| `group` | Drug groups/classes (e.g., "beta-blockers") |
| `drug_n` | Active substances not approved for human use (e.g., toxins, pesticides; "drug not human") |

### Relation types (4 + negative)

| Type | Meaning |
|------|---------|
| `mechanism` | A **pharmacokinetic** mechanism of the DDI is described |
| `effect` | The **effect / pharmacodynamic** outcome of the DDI is described |
| `advice` | A **recommendation/advice** about concomitant use is given |
| `int` | A DDI simply **occurs** (stated) with no further mechanism/effect detail |
| `False` | No interaction (negative class) |

---

## 6. i2b2 2010 (clinical concepts, assertions, relations)

2010 i2b2/VA challenge over clinical records. Three subtasks: concept extraction (3 types), assertion classification (6 classes), and relation classification (8 types).

### Concept / entity types (3)

| Type | Definition |
|------|------------|
| `problem` | Medical problems — conditions, diagnoses, symptoms |
| `test` | Diagnostic tests/procedures and investigations |
| `treatment` | Treatments — medications and therapeutic procedures |

### Assertion classes for `problem` (6)

`present`, `absent`, `possible`, `conditional`, `hypothetical`, `not associated with the patient`

### Relation types (8)

**Treatment–Problem (5):**

| Type | Expansion |
|------|-----------|
| `TrIP` | Treatment **improves** medical problem |
| `TrWP` | Treatment **worsens** medical problem |
| `TrCP` | Treatment **causes** medical problem |
| `TrAP` | Treatment is **administered for** medical problem |
| `TrNAP` | Treatment is **not administered because of** medical problem |

**Test–Problem (2):**

| Type | Expansion |
|------|-----------|
| `TeRP` | Test **reveals** medical problem |
| `TeCP` | Test **conducted to investigate** medical problem |

**Problem–Problem (1):**

| Type | Expansion |
|------|-----------|
| `PIP` | Medical problem **indicates** medical problem |

(The 2010 task is the canonical "clinical relation" schema; n2c2 is the organizational successor to i2b2.)

---

## 7. n2c2 2018 Track 2 — ADE & Medication Extraction

505 MIMIC-III discharge summaries. Three sub-tasks: concept extraction (9 entity types), relation classification, end-to-end. Relations are all between a non-drug entity and a `Drug`.

### Entity types (9)

| Type | Definition |
|------|------------|
| `Drug` | Medication/drug name (the anchor entity) |
| `Strength` | Strength of the medication |
| `Dosage` | Dose amount |
| `Duration` | How long the drug is taken |
| `Frequency` | How often the drug is taken |
| `Form` | Form (tablet, injection, etc.) |
| `Route` | Route of administration |
| `Reason` | Reason/indication for the drug |
| `ADE` | Adverse Drug Event |

### Relation types (8)

All directional, linking a modifier entity to its `Drug`:

| Type |
|------|
| `Strength-Drug` |
| `Dosage-Drug` |
| `Duration-Drug` |
| `Frequency-Drug` |
| `Form-Drug` |
| `Route-Drug` |
| `Reason-Drug` |
| `ADE-Drug` |

---

## 8. scispaCy NER models & linkers

scispaCy (AllenAI) ships full pipelines plus four specialized NER models. The four **general** models (`en_core_sci_sm`, `en_core_sci_md`, `en_core_sci_lg`, `en_core_sci_scibert`) detect mentions with a single generic label `ENTITY` (any span that might be a UMLS concept). The four **specialized** NER models carry typed labels:

### Specialized NER model label sets

| Model | Corpus | Entity labels |
|-------|--------|---------------|
| `en_ner_bc5cdr_md` | BC5CDR | `DISEASE`, `CHEMICAL` |
| `en_ner_jnlpba_md` | JNLPBA | `DNA`, `RNA`, `PROTEIN`, `CELL_TYPE`, `CELL_LINE` |
| `en_ner_craft_md` | CRAFT | `GGP` (gene/gene-product), `SO` (Sequence Ontology), `TAXON` (NCBI Taxonomy), `CHEBI` (chemicals), `GO` (Gene Ontology), `CL` (Cell Ontology / cell types) |
| `en_ner_bionlp13cg_md` | BioNLP13CG (Cancer Genetics) | `AMINO_ACID`, `ANATOMICAL_SYSTEM`, `CANCER`, `CELL`, `CELLULAR_COMPONENT`, `DEVELOPING_ANATOMICAL_STRUCTURE`, `GENE_OR_GENE_PRODUCT`, `IMMATERIAL_ANATOMICAL_ENTITY`, `MULTI-TISSUE_STRUCTURE`, `ORGAN`, `ORGANISM`, `ORGANISM_SUBDIVISION`, `ORGANISM_SUBSTANCE`, `PATHOLOGICAL_FORMATION`, `SIMPLE_CHEMICAL`, `TISSUE` (16 labels) |

> scispaCy does **not** define RE relation types — it is NER + entity linking + abbreviation + dependency parsing only.

### Entity linkers (knowledge bases) — 5

| Linker | Target KB | Approx. size |
|--------|-----------|--------------|
| `umls` | Unified Medical Language System (levels 0,1,2,9) | ~3M concepts |
| `mesh` | Medical Subject Headings | ~30k entities |
| `rxnorm` | RxNorm (normalized clinical drug names) | ~100k concepts |
| `go` | Gene Ontology (gene functions) | ~67k concepts |
| `hpo` | Human Phenotype Ontology | phenotype concepts |

Plus an `AbbreviationDetector` pipe (resolves long↔short forms before linking).

---

## 9. medspaCy

medspaCy is a clinical-NLP **toolkit** (rule-based target matching + ConText + section detection); it does **not ship a fixed taxonomy**. Labels are user-defined via `TargetRule`. Convention/default examples in tutorials use clinical concept labels analogous to i2b2:

### Conventional / example entity labels
`PROBLEM`, `TREATMENT`, `TEST` (user-configurable; not a fixed schema)

### Context / assertion attributes (ConText algorithm)
medspaCy's `ConText` asserts modifier attributes on entities rather than entity types. Standard attributes:

| Attribute | Meaning |
|-----------|---------|
| `is_negated` | Negation (e.g., "no evidence of …") |
| `is_uncertain` / `is_possible` | Uncertainty / possibility |
| `is_historical` | Historical (not current) |
| `is_hypothetical` | Hypothetical / conditional |
| `is_family` (experiencer) | Concerns family member, not the patient |

(These mirror the i2b2 assertion classes: present/absent/possible/conditional/hypothetical/not-associated-with-patient.)

---

## 10. GENIA corpus & JNLPBA

The **GENIA corpus** (2000 MEDLINE abstracts) annotates biomedical terms against the **GENIA term ontology** (~47–48 classes; 36 used to annotate the corpus), plus the **GENIA event ontology** (events — see §11) and meta-knowledge (negation/speculation).

### GENIA term ontology — key annotated classes

The ontology is a forest of three trees rooted at **biological source**, **biological substance**, and **other**. Most-used substance/source classes:

| Class | Notes |
|-------|-------|
| `protein` | Subtyped: `protein_molecule`, `protein_family_or_group`, `protein_complex`, `protein_domain_or_region`, `protein_substructure`, `protein_subunit`, `protein_ETC` |
| `DNA` | Subtyped: `DNA_domain_or_region`, `DNA_family_or_group`, `DNA_molecule`, `DNA_substructure`, … |
| `RNA` | Subtyped analogously (`RNA_molecule`, `RNA_family_or_group`, `RNA_domain_or_region`, …) |
| `cell_type` | Biological source — cell types |
| `cell_line` | Biological source — cultured cell lines |
| `cell_component` | Cellular components |
| `tissue` | |
| `body_part` | |
| `organism` / `multi_cell` / `mono_cell` / `virus` | Biological sources |
| `lipid`, `carbohydrate`, `amino_acid_monomer`, `peptide`, `nucleotide`, `inorganic`, `atom`, `other_organic_compound` | Other biological substances |
| `other_name`, `other_artificial_source` | Catch-alls |

### JNLPBA (2004 shared task — simplified GENIA NER)

JNLPBA collapses the GENIA term ontology to **5 NE classes** for the bio-entity recognition task:

| Type |
|------|
| `protein` |
| `DNA` |
| `RNA` |
| `cell_type` |
| `cell_line` |

---

## 11. BioNLP Shared Task event types

The BioNLP Shared Tasks define **event** schemas (a trigger word + typed arguments such as `Theme`, `Cause`, `Site`, `AtLoc`, `ToLoc`), distinct from binary relations. The base entity is `Protein` (genes/gene products); additional `Entity` spans (e.g., cellular locations) are introduced for argument tasks.

### BioNLP'09 / GENIA event task (GE) — 9 event types

| Class | Event types |
|-------|-------------|
| Simple (protein production/breakdown) | `Gene_expression`, `Transcription`, `Protein_catabolism` |
| Localization | `Localization` (args: `Theme`, optional `AtLoc`/`ToLoc`) |
| Binding | `Binding` (protein–protein / protein–DNA; can be multi-Theme) |
| Modification | `Phosphorylation` (optional `Site`) |
| Regulation (Theme + Cause) | `Regulation`, `Positive_regulation`, `Negative_regulation` |

Subtasks: Task 1 (core events), Task 2 (argument augmentation — `Site`, `AtLoc`, `ToLoc`, `CSite`), Task 3 (event modification — Negation, Speculation). The GE task recurred at BioNLP 2011 and 2013 with the same nine core types.

### BioNLP'11 EPI task (Epigenetics & Post-translational Modifications) — 14 event types

7 PTM/DNA-modification events + their reverse reactions + Catalysis:

| Forward | Reverse |
|---------|---------|
| `Phosphorylation` | `Dephosphorylation` |
| `Ubiquitination` | `Deubiquitination` |
| `Hydroxylation` | `Dehydroxylation` |
| `Methylation` | `Demethylation` |
| `Acetylation` | `Deacetylation` |
| `Glycosylation` | `Deglycosylation` |
| `DNA_methylation` | `DNA_demethylation` |
| `Catalysis` (catalysis of a modification by a protein) | — |

### BioNLP'11 — other 2011 tracks

| Track | Scope |
|-------|-------|
| GE — GENIA Event | the 9 GENIA event types (above) |
| EPI | the 14 epigenetics/PTM events (above) |
| ID — Infectious Diseases | biomolecular mechanisms of infection/virulence/resistance (two-component signaling); GE-style event types adapted to infection biology, plus core-relations (REL) |
| BB — Bacteria Biotopes | bacteria habitat/localization entities & relations |
| BI — Bacteria Interactions | gene-regulation interactions in bacteria |
| REL | entity relations (e.g., `Subunit-Complex`, `Protein-Component`) for coreference/relation support |

### BioNLP'13 Cancer Genetics (CG) task

A large schema for cancer biology: **40 event types** over **18 entity types**, reusing and extending the 2011 GE and EPI event definitions. Representative event types include `Cell_proliferation`, `Development`, `Cell_death`/`Death`, `Cell_transformation`, `Metabolism`, `Synthesis`, `Catabolism`, `Mutation`, `DNA_methylation`, `Carcinogenesis`, `Metastasis`, `Growth`, `Remodeling`, `Glycolysis`, `Blood_vessel_development`, plus the GENIA simple/binding/regulation events. Entity types span molecular (`Gene_or_gene_product`, `Simple_chemical`, `Amino_acid`) through anatomical (`Cell`, `Tissue`, `Organ`, `Cancer`, `Pathological_formation`, `Organism`, …) — i.e., the same family used by `en_ner_bionlp13cg_md` (§8).

### BioNLP'13 Pathway Curation (PC) task

Reaction/pathway events (e.g., `Conversion`, `Transport`, `Phosphorylation`, `Activation`, `Inactivation`, `Binding`, `Dissociation`, `Gene_expression`, `Degradation`, `Localization`, `Regulation`/`Positive_regulation`/`Negative_regulation`) over `Simple_chemical`, `Gene_or_gene_product`, `Complex`, `Cellular_component` entities.

---

## 12. Quick cross-reference summary

| Benchmark | Domain | # Entity types | # Relation/Event types |
|-----------|--------|:--------------:|:----------------------:|
| **BioRED** | literature, multi-domain | 6 | 8 (+ Novelty attr) |
| **BC5CDR** | literature | 2 (Chemical, Disease) | 1 (CID) |
| **ChemProt** | chemical–protein | 2 (CHEMICAL, GENE) | 10 CPR groups (5 evaluated) |
| **DrugProt** | chemical–protein | 2 (CHEMICAL, GENE) | 13 |
| **DDIExtraction 2013** | drug–drug | 4 (drug, brand, group, drug_n) | 4 (+ False) |
| **i2b2 2010** | clinical | 3 (problem, test, treatment) | 8 (+ 6 assertions) |
| **n2c2 2018 Track 2** | clinical (ADE/meds) | 9 | 8 |
| **scispaCy bc5cdr** | literature | 2 | — (NER only) |
| **scispaCy jnlpba** | literature | 5 | — |
| **scispaCy craft** | literature | 6 | — |
| **scispaCy bionlp13cg** | cancer genetics | 16 | — |
| **medspaCy** | clinical | user-defined (PROBLEM/TREATMENT/TEST) | — (ConText assertion attrs) |
| **GENIA term** | molecular biology | ~36 (47/48 in ontology) | (see events) |
| **JNLPBA** | molecular biology | 5 | — |
| **BioNLP'09/11 GE** | molecular events | 1 (Protein) + Entity | 9 events |
| **BioNLP'11 EPI** | epigenetics/PTM | Protein + Entity | 14 events |
| **BioNLP'13 CG** | cancer genetics | 18 | 40 events |

---

## Sources

- BioRED: https://pmc.ncbi.nlm.nih.gov/articles/PMC9487702/ · https://academic.oup.com/bib/article/23/5/bbac282/6645993 · https://arxiv.org/pdf/2204.04263 · BioCreative VIII overview https://pmc.ncbi.nlm.nih.gov/articles/PMC11306928/
- BC5CDR: https://academic.oup.com/database/article/doi/10.1093/database/baw068/2630414 · https://pmc.ncbi.nlm.nih.gov/articles/PMC4860626/ · corpus https://github.com/JHnlp/BioCreative-V-CDR-Corpus
- ChemProt (CPR groups): https://server.ccl.net/cca/info/conferencelist/mess0024250.html · https://academic.oup.com/database/article/doi/10.1093/database/bay066/5053190 · merged mapping https://arxiv.org/html/2405.18605
- DrugProt: https://academic.oup.com/database/article/doi/10.1093/database/baad080/7453369 · https://www.ncbi.nlm.nih.gov/pmc/articles/PMC10683943/ · corpus https://zenodo.org/records/4955411 · guidelines https://zenodo.org/records/4957138
- DDIExtraction 2013: https://academic.oup.com/bioinformatics/article/37/12/1739/5938075 · https://pmc.ncbi.nlm.nih.gov/articles/PMC4752975/
- i2b2 2010: https://pmc.ncbi.nlm.nih.gov/articles/PMC3168320/ · https://pmc.ncbi.nlm.nih.gov/articles/PMC3168312/
- n2c2 2018 Track 2: https://pmc.ncbi.nlm.nih.gov/articles/PMC7489085/ · https://n2c2.dbmi.hms.harvard.edu/challenge/2018-track-2-ade-medication-extraction · https://huggingface.co/datasets/bigbio/n2c2_2018_track2
- scispaCy: https://github.com/allenai/scispacy · https://raw.githubusercontent.com/allenai/scispacy/main/docs/index.md · https://github.com/allenai/scispacy/issues/251 · https://aclanthology.org/W19-5034.pdf
- medspaCy: https://github.com/medspacy/medspacy · https://medium.com/geekculture/introduction-to-the-medspacy-the-medical-named-entity-recognition-ner-package-e7c6f0f06496
- GENIA / JNLPBA: https://bmcbioinformatics.biomedcentral.com/articles/10.1186/1471-2105-9-10 · https://link.springer.com/article/10.1186/2041-1480-2-S5-S1 · revised JNLPBA https://arxiv.org/pdf/1901.10219
- BioNLP Shared Tasks: GE'09 https://www.nactem.ac.uk/GENIA/SharedTask/detail.shtml · GE'11 https://pmc.ncbi.nlm.nih.gov/articles/PMC3384256/ · EPI'11 https://aclanthology.org/W11-1803.pdf · ID/EPI/REL'11 https://pmc.ncbi.nlm.nih.gov/articles/PMC3384257/ · CG'13 https://pmc.ncbi.nlm.nih.gov/articles/PMC4511510/
