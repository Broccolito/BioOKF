# Top-Level Categories of Major Biomedical Ontologies

A reference enumeration of the **top-level entity classes** of the major biomedical
ontologies, vocabularies, and knowledge organization systems. The goal is to see
which **general entity classes** exist across the biomedical landscape so we can
design a unifying schema / knowledge-graph node typing. For each source the actual
type names are quoted as they appear in the source ontology (case and wording
preserved). A synthesized **union of top-level entity classes** is given at the end.

Compiled 2026-06-25. Primary sources: NLM MeSH docs, SNOMED International docs,
Gene Ontology, EBI ChEBI, Monarch (MONDO/HPO/Uberon/CL), Disease Ontology, PRO,
OBI, EDAM, NCBI Taxonomy. Live class lists pulled from the EBI **Ontology Lookup
Service (OLS4)** API (`https://www.ebi.ac.uk/ols4/api`).

---

## 1. MeSH — Medical Subject Headings (NLM)

Controlled vocabulary / thesaurus for indexing biomedical literature
(MEDLINE/PubMed). Descriptors are arranged in 16 top-level **tree categories**
(letters), each with subcategories, up to ~13 hierarchy levels. The 16 categories
are the broadest "general entity classes" MeSH recognizes.

| Code | Top-level category | Scope / example subcategories |
|------|--------------------|-------------------------------|
| **A** | Anatomy | Body regions, organs, tissues, cells, fluids, anatomical systems |
| **B** | Organisms | Eukaryota, Bacteria, Archaea, Viruses, organism groups |
| **C** | Diseases | Disease/disorder terms (by system, by cause, neoplasms, etc.) |
| **D** | Chemicals and Drugs | Chemicals, drugs, biologics, enzymes, hormones, polymers |
| **E** | Analytical, Diagnostic and Therapeutic Techniques, and Equipment | Procedures, equipment, diagnostics, therapeutics, surgical methods |
| **F** | Psychiatry and Psychology | Behavior, mental disorders, psychological phenomena/processes |
| **G** | Phenomena and Processes | Physiological/biological/chemical/physical phenomena & processes |
| **H** | Disciplines and Occupations | Natural/health sciences, occupations |
| **I** | Anthropology, Education, Sociology, and Social Phenomena | Social/behavioral, education, human activities |
| **J** | Technology, Industry, and Agriculture | Technology, food/industry, agriculture |
| **K** | Humanities | History, philosophy, ethics, religion, arts |
| **L** | Information Science | Communication, computing, data, information services |
| **M** | Named Groups | Persons / population groups (age, occupational, etc.) |
| **N** | Health Care | Health care quality, economics, facilities, services, public health |
| **V** | Publication Characteristics | Publication types / formats (e.g., Review, Clinical Trial) |
| **Z** | Geographicals | Geographic locations |

Notes: V and Z fall outside A–N. There is no "O–U, W, X, Y" — only these 16
letters are used. Each top category branches into numbered subcategories
(e.g., `A01` Body Regions, `C04` Neoplasms, `D27` Chemical Actions and Uses).

---

## 2. SNOMED CT — Top-Level Hierarchies (SNOMED International)

Clinical reference terminology. All concepts sit under the root `SNOMED CT Concept`
and fall into **19 top-level hierarchies** (the immediate subtypes of the root).
These are the canonical "axes" of clinical meaning.

| # | Top-level hierarchy | Definition / scope |
|---|---------------------|--------------------|
| 1 | **Clinical finding** | Result of a clinical observation, assessment or judgment; normal & abnormal states; **includes diagnoses** (subsumes the *Disease* sub-hierarchy) |
| 2 | **Procedure** | Activities performed in providing health care (interventions, administration, imaging, education) |
| 3 | **Situation with explicit context** | Findings/procedures with explicit context (absent, family history, planned, past) |
| 4 | **Observable entity** | A question or assessment that can produce an answer/result (e.g., systolic blood pressure, gender) |
| 5 | **Body structure** | Normal & abnormal anatomical structures; includes **Morphologically abnormal structure** |
| 6 | **Organism** | Organisms of relevance to human/animal medicine (bacteria, viruses, fungi, parasites, animals, plants) |
| 7 | **Substance** | Chemical constituents of products, foods, body substances, allergens, etc. |
| 8 | **Pharmaceutical / biologic product** | Drug products (distinct from their constituent Substances) |
| 9 | **Specimen** | Entities obtained (usually from a patient) for examination/analysis |
| 10 | **Physical object** | Natural and manufactured physical objects (devices, implants) |
| 11 | **Physical force** | Physical forces acting as mechanisms of injury (gravity, friction, electricity) |
| 12 | **Event** | Occurrences excluding procedures/interventions (e.g., flood, earthquake) |
| 13 | **Environment or geographical location** | Types of environment + named locations |
| 14 | **Social context** | Social conditions & circumstances (occupations, ethnic group, lifestyle, religion) |
| 15 | **Situation** *(see Situation with explicit context)* | — |
| 16 | **Staging and scales** | Assessment scales, tumor staging systems, score values |
| 17 | **Qualifier value** | Values used to qualify/refine other concepts via attributes |
| 18 | **Record artifact** | Content created to provide information about other records (e.g., record entry) |
| 19 | **SNOMED CT Model Component** | Metadata: technical concepts supporting the release (attributes, ref sets) |
| + | **Special concept** | Navigational concepts & non-real concepts outside the logical content |

(The widely cited "19" list: Body structure, Clinical finding, Environment or
geographical location, Event, Observable entity, Organism, Pharmaceutical/biologic
product, Physical force, Physical object, Procedure, Qualifier value, Record artifact,
Situation with explicit context, SNOMED CT Model Component, Social context, Special
concept, Specimen, Staging and scales, Substance.)

---

## 3. Gene Ontology (GO) — Three Aspects / Namespaces

GO describes gene-product function with **3 orthogonal namespaces** (its "aspects").
Every GO term belongs to exactly one.

| Aspect (abbr.) | Root term ID | Definition |
|----------------|--------------|------------|
| **Biological Process** (BP) | `GO:0008150` | A series/collection of molecular events with a defined beginning and end, pertinent to the functioning of integrated living units (cells → tissues → organs → organisms); spans metabolic pathways up to behavior |
| **Molecular Function** (MF) | `GO:0003674` | Activities of an individual gene product at the molecular level — e.g., catalysis (catalytic activity), binding, transporter activity |
| **Cellular Component** (CC) | `GO:0005575` | The location relative to cellular compartments and structures where a gene product acts — subcellular structures, macromolecular complexes, extracellular environment |

---

## 4. ChEBI — Chemical Entities of Biological Interest (EBI)

Ontology of (mostly small) molecular entities. Root = **`chemical entity`
(CHEBI:24431)**. ChEBI is split conceptually into a **structure-based** branch and
a **role-based** branch (the "two pillars"). The role branch (`role` CHEBI:50906) is
*not* under `chemical entity`; it is a separate top class.

### 4a. Structural branch — children of `chemical entity` (CHEBI:24431)

| Class | ID | Notes |
|-------|----|----|
| **molecular entity** | CHEBI:23367 | Any constitutionally/isotopically distinct atom, molecule, ion, radical, complex, conformer (the largest branch) |
| **group** | CHEBI:24433 | Part-molecular entity / substituent groups |
| **atom** | CHEBI:33250 | Single atoms / elements |
| **chemical substance** | CHEBI:59999 | Pure & mixed substances (preparations, mixtures) |

High-level children of **molecular entity (CHEBI:23367)** include: *elemental
molecular entity, inorganic molecular entity, main group molecular entity,
transition element molecular entity, ion, polyatomic entity, radical,
isotopically modified compound, exotic molecular entity, hydrogen donor/acceptor.*

### 4b. Role branch — children of `role` (CHEBI:50906)

| Role class | ID | Examples of children |
|------------|----|----------------------|
| **biological role** | CHEBI:24432 | inhibitor, antigen, antimicrobial agent, hapten, mitogen, xenobiotic, immunomodulator, poison, prohormone, provitamin, pharmacological role, physiological role, biochemical role |
| **application** | CHEBI:33232 | pharmaceutical, agrochemical, pesticide, food additive, solvent, reagent, dye, fuel, detergent, excipient, label/tracer, indicator, probe |
| **chemical role** | CHEBI:51086 | acid, base, buffer, catalyst, ligand, oxidising agent, reducing agent, donor/acceptor, antioxidant, emulsifier, solvent |

So ChEBI's top "entity classes" = **{atom, group, molecular entity, chemical
substance}** (structure) **+ {biological role, application, chemical role}** (role).

---

## 5. MONDO — Monarch Disease Ontology

Unified disease ontology merging DO, OMIM, Orphanet, NCIt, etc. Root =
**`disease or disorder` (MONDO:0000001)**. Recent releases reorganized the upper
levels into a small set of **classification axes**, under which the familiar
system/etiology categories sit.

### Top of MONDO

```
disease or disorder (MONDO:0000001)
├─ non-human animal disease (MONDO:0005583)
└─ human disease (MONDO:0700096)
   ├─ acute disease
   ├─ disease by body system or component   (MONDO:7770006)
   ├─ disease by developmental or physiological process (MONDO:7770007)
   └─ disease by etiologic mechanism         (MONDO:7770008)
```

### 5a. `disease by body system or component` (19 classes)

| MONDO ID | Class |
|----------|-------|
| MONDO:0004995 | cardiovascular disorder |
| MONDO:0003900 | connective tissue disorder |
| MONDO:0004335 | digestive system disorder |
| MONDO:0005151 | endocrine system disorder |
| MONDO:0005570 | hematologic disorder |
| MONDO:0005046 | immune system disorder |
| MONDO:0002051 | integumentary system disorder |
| MONDO:0002081 | musculoskeletal system disorder |
| MONDO:0005071 | nervous system disorder |
| MONDO:0005039 | reproductive system disorder |
| MONDO:0005087 | respiratory system disorder |
| MONDO:0002118 | urinary system disorder |
| MONDO:0002409 | auditory system disorder |
| MONDO:0024458 | disorder of visual system |
| MONDO:0002022 | disorder of orbital region |
| MONDO:0002657 | breast disorder |
| MONDO:0006858 | mouth disorder |
| MONDO:0024623 | otorhinolaryngologic disease |
| MONDO:0002254 | syndromic disease |

### 5b. `disease by developmental or physiological process` (12 classes)

psychiatric disorder · metabolic disease · premature aging syndrome ·
disorder of development or morphogenesis · inflammatory disease · disorder of
glycosylation · ulcer disease · mitochondrial disease · sleep disorder ·
perinatal disease · obstetric disorder · disease by molecular mechanism.

### 5c. `disease by etiologic mechanism` (5 classes)

nutritional disorder · cancer or benign tumor · idiopathic disease · disease of
genetic or genomic mechanism · disease of primarily extrinsic mechanism
(e.g., infectious, toxic, injury).

---

## 6. Disease Ontology (DO / DOID)

Etiology-based human disease classification. Root = **`disease` (DOID:4)** with
**8 top-level branches** (multi-axis; a disease can appear under more than one).

| DOID | Top-level branch | Scope |
|------|------------------|-------|
| DOID:7 | **disease of anatomical entity** | Diseases manifesting in a defined anatomical structure/system (cardiovascular, nervous, respiratory, etc.) |
| DOID:14566 | **disease of cellular proliferation** | Neoplasms — `cancer` (malignant) and `benign neoplasm` |
| DOID:0014667 | **disease of metabolism** | Inborn/acquired metabolic & nutritional disorders |
| DOID:150 | **disease of mental health** | Psychiatric / behavioral / developmental disorders |
| DOID:0050117 | **disease by infectious agent** | Infectious diseases (bacterial, viral, fungal, parasitic, prion) |
| DOID:630 | **genetic disease** | Diseases with a primary genetic cause |
| DOID:225 | **syndrome** | Recognizable patterns / symptom complexes |
| DOID:0080015 | **physical disorder** | Diseases caused by physical injury / environmental agents |

---

## 7. HPO — Human Phenotype Ontology

Standardized vocabulary of phenotypic abnormalities seen in human disease. Root =
**`All` (HP:0000001)** with these immediate subontologies:

| HP ID | Subontology | Role |
|-------|-------------|------|
| HP:0000118 | **Phenotypic abnormality** | Main subontology — clinical abnormalities by organ system (see below) |
| HP:0000005 | **Mode of inheritance** | Inheritance pattern (AD, AR, X-linked, mitochondrial…) |
| HP:0012823 | **Clinical modifier** | Modifiers of clinical presentation (severity, laterality, onset trigger…) |
| HP:0040279 | **Frequency** | Frequency of a feature in a population |
| HP:0032223 | **Blood group** | Blood group phenotypes |
| HP:0032443 | **Past medical history** | Prior medical events |
| HP:0020228 | **Biospecimen phenotypic feature** | Features observed in a biospecimen |

*(Historically also "Mortality/Aging".)*

### 7a. Organ-system children of `Phenotypic abnormality` (HP:0000118) — 23 classes

| HP ID | Top-level phenotype category |
|-------|------------------------------|
| HP:0000707 | Abnormality of the nervous system |
| HP:0000478 | Abnormality of the eye |
| HP:0000598 | Abnormality of the ear |
| HP:0000152 | Abnormality of head or neck |
| HP:0001626 | Abnormality of the cardiovascular system |
| HP:0002086 | Abnormality of the respiratory system |
| HP:0025031 | Abnormality of the digestive system |
| HP:0000119 | Abnormality of the genitourinary system |
| HP:0000818 | Abnormality of the endocrine system |
| HP:0001939 | Abnormality of metabolism/homeostasis |
| HP:0002715 | Abnormality of the immune system |
| HP:0001871 | Abnormality of blood and blood-forming tissues |
| HP:0033127 | Abnormality of the musculoskeletal system |
| HP:0001574 | Abnormality of the integument |
| HP:0000769 | Abnormality of the breast |
| HP:0000818 | Abnormality of the endocrine system |
| HP:0001507 | Growth abnormality |
| HP:0001197 | Abnormality of prenatal development or birth |
| HP:0001608 | Abnormality of the voice |
| HP:0002664 | Neoplasm |
| HP:0040064 | Abnormality of limbs |
| HP:0045027 | Abnormality of the thoracic cavity |
| HP:0025142 | Constitutional symptom |
| HP:0025354 | Abnormal cellular phenotype |

---

## 8. Uberon — Multi-species Anatomy Ontology

Cross-species (metazoan) gross-anatomy ontology. Bridges the CARO upper-level
ontology to species-specific anatomy ontologies. Root =
**`anatomical entity` (UBERON:0001062)**.

| UBERON ID | Top class | Subsumes |
|-----------|-----------|----------|
| UBERON:0000465 | **material anatomical entity** | anatomical structure, multicellular anatomical structure, organ, organ system, tissue, cell (via CL), body region, anatomical cluster, portion of tissue/substance |
| UBERON:0000466 | **immaterial anatomical entity** | anatomical space, anatomical surface, anatomical line/point, anatomical plane |

The practically used **mid-level** anatomical classes (from CARO/Uberon) are:
*anatomical structure → multicellular anatomical structure → (organ system,
organ, tissue, body region/segment, anatomical cluster), portion of tissue,
acellular anatomical structure, anatomical space/surface.* Uberon also imports
**cell (CL)** as the cellular-level anatomy.

---

## 9. Cell Ontology (CL)

Ontology of cell types (in vivo / native + in vitro). Root = **`cell` (CL:0000000)**.
CL has **no small fixed set of top categories** — it is a polyhierarchy classifying
cells by lineage, function, ploidy, location, etc. The direct children of `cell`
are functional/structural axes rather than tidy "kingdoms":

| Axis of the direct children of `cell` (CL:0000000) | Examples (CL terms) |
|-----------------------------------------------------|---------------------|
| By cellular organization | eukaryotic cell, prokaryotic cell, nucleate cell, anucleate cell |
| By ploidy | haploid cell, diploid cell, polyploid cell |
| By function | secretory cell, contractile cell, electrically active cell, motile cell, supporting cell, excretory cell, photosynthetic cell, nitrogen fixing cell |
| By lineage / developmental state | germ line cell, embryonic cell (metazoa), precursor cell, transit amplifying cell, zygote, hematopoietic cell, skeletogenic cell |
| By accumulation / metabolism | stuff accumulating cell, oxygen accumulating cell, foam cell |
| By fate / state | apoptosis fated cell, abnormal cell, inflammatory cell |
| Context | cell in vitro, native cell |

Common high-level cell **types** used downstream (not literal direct children but
canonical mid-level CL classes): epithelial cell, connective tissue cell, neuron,
glial cell, muscle cell, blood cell / hematopoietic cell, immune cell, germ cell,
stem cell, stromal cell.

---

## 10. NCBI Taxonomy

Phylogenetic classification of organisms with names. Not a typed ontology but a
**rank-based tree**. As of 2024–2025, NCBI retired `superkingdom` in favor of
`domain` (cellular life) and `realm` (viruses), with new roots.

### Top of the tree

```
root
├─ cellular root          (rank: "cellular root")
│  ├─ Bacteria   (domain)
│  ├─ Archaea    (domain)
│  └─ Eukaryota  (domain)   → Animalia, Plantae, Fungi, Protista (kingdoms)
└─ acellular root         (rank: "acellular root")
   └─ Viruses    (realm-level groupings)
```

### Standard ranks (top → bottom)

| Rank | Notes |
|------|-------|
| **domain** (was superkingdom) | Bacteria, Archaea, Eukaryota |
| **realm** | top virus grouping |
| **kingdom** | e.g., Metazoa/Animalia, Viridiplantae, Fungi |
| **phylum** | ~245 |
| **class** | ~380 |
| **order** | ~1,500 |
| **family** | ~9,200 |
| **genus** | ~92,000 |
| **species** | ~1.8M+ |

(Plus intermediate ranks: subphylum, subclass, suborder, superfamily, subfamily,
tribe, subgenus, subspecies, strain, clade, no rank.) The **entity class** here is
simply **Organism / Taxon** at a given rank.

---

## 11. Protein Ontology (PRO)

Ontology of protein entities (single chains + complexes) at multiple granularities.
The single-chain entities are categorized into **metaclasses by level of
specificity**, plus a separate complex metaclass.

| PRO metaclass (level) | Definition |
|-----------------------|------------|
| **family** | Protein products of a distinct gene family from a common ancestor |
| **gene** | The protein products of a distinct gene |
| **sequence** | Protein products with a distinct sequence on initial translation (e.g., splice/allelic isoforms) |
| **modification** | Products from one mRNA differing by co-/post-translational change (PTM forms) |
| **protein complex** | Complexes with a specific defined subunit composition |

All single-chain metaclasses also have **organism-specific derivatives**. PRO also
organizes around three high-level branches in practice:
**ProEvo** (evolutionary / family & gene), **ProForm** (sequence & modification
forms), and **ProComp** (complexes).

---

## 12. OBI — Ontology for Biomedical Investigations

Describes the design, execution, and reporting of biomedical investigations. Built
on **BFO (Basic Formal Ontology)**, so its top-level entity classes are the BFO
upper categories (seen as OBI's OLS roots), under which OBI's domain classes hang.

### BFO-derived top-level classes (OBI roots)

| Top class (BFO/IAO) | Meaning | Key OBI domain classes underneath |
|---------------------|---------|------------------------------------|
| **material entity** (BFO:0000040) | Physical things | organism, specimen (OBI:0100051), biological/biomaterial, device (COB:0001300), instrument, reagent |
| **immaterial entity** (BFO:0000141) | Sites/boundaries | spatial regions |
| **process** (BFO:0000015) | Occurrents / processes | **planned process** (OBI:0000011) → **investigation** (OBI:0000066), **assay** (OBI:0000070), study design execution, data transformation, material processing |
| **information content entity** (IAO:0000030) | Information artifacts | **data item**, data set, document, objective specification, study design, plan, conclusion/report (investigation results report) |
| **characteristic** (COB:0000502) | Qualities/roles/functions | quality, role (e.g., investigation agent role), function, disposition |

The practically important OBI "entity classes": **investigation, study design,
assay, planned process, material entity / specimen / organism, device / instrument,
data item / information content entity, objective, role, function, quality.**

---

## 13. EDAM — Ontology of Bioinformatics Operations, Data, Formats, Topics

Domain ontology of data analysis & management. Deliberately simple: **4 main
sub-ontologies (sections)**, each a top-level entity class.

| Section | Root | What it types |
|---------|------|---------------|
| **Topic** | `topic_0003` | Research fields / subject domains (e.g., Genomics, Proteomics, Phylogenetics) |
| **Operation** | `operation_0004` | Computational operations / methods (alignment, clustering, variant calling, prediction) |
| **Data** | `data_0006` | Types of data and identifiers (incl. the **Identifier** sub-branch) |
| **Format** | `format_1915` | Data formats / file formats / standards (FASTA, VCF, BAM, GFF) |

---

## 14. Synthesis — Union of Top-Level Entity Classes

Collapsing all 13 sources into a single set of **general entity classes** that a
unifying biomedical knowledge graph would need. Each row lists which source(s)
contribute that class. (★ = a primary node type recurring across many ontologies.)

| # | Unified entity class | Contributing sources / their term |
|---|----------------------|-----------------------------------|
| 1 | **Anatomical structure / Body structure** ★ | MeSH A (Anatomy); SNOMED *Body structure*; Uberon *anatomical entity* (material/immaterial); GO *Cellular component* (subcellular) |
| 2 | **Cell / Cell type** ★ | Cell Ontology *cell*; MeSH A11 (Cells); SNOMED (cell structure under Body structure); Uberon imports CL |
| 3 | **Organism / Taxon** ★ | MeSH B (Organisms); SNOMED *Organism*; NCBI Taxonomy *Organism/Taxon*; (DO infectious agent) |
| 4 | **Gene / Genome / Genetic element** | (implicit) GO annotation targets; PRO *gene-level*; MeSH G05 (Genetic Phenomena); EDAM data |
| 5 | **Protein / Protein complex / Macromolecule** ★ | PRO (family/gene/sequence/modification/complex); GO MF/CC targets; ChEBI (macromolecules) |
| 6 | **Chemical / Substance / Drug** ★ | MeSH D (Chemicals & Drugs); SNOMED *Substance* + *Pharmaceutical/biologic product*; ChEBI *chemical entity* (atom/group/molecular entity/chemical substance) |
| 7 | **Molecular Function** | GO *Molecular Function*; ChEBI/PRO (activities) |
| 8 | **Biological Process / Pathway** ★ | GO *Biological Process*; MeSH G (Phenomena & Processes); SNOMED (some Observable/Event) |
| 9 | **Phenotype / Clinical finding / Phenotypic abnormality** ★ | HPO *Phenotypic abnormality* (+23 organ-system categories); SNOMED *Clinical finding*; MeSH F (some) |
| 10 | **Disease / Disorder** ★ | MeSH C (Diseases); SNOMED *Clinical finding/Disease*; MONDO *disease or disorder*; DO *disease*; HPO *Neoplasm* |
| 11 | **Procedure / Technique / Intervention** ★ | MeSH E (Techniques & Equipment); SNOMED *Procedure*; OBI (planned process/assay); EDAM *Operation* |
| 12 | **Assay / Investigation / Study** | OBI *investigation*, *assay*, *study design*; EDAM *Operation*; MeSH E01 (Diagnosis) |
| 13 | **Device / Instrument / Physical object** | SNOMED *Physical object*; MeSH E07 (Equipment & Supplies); OBI *device* |
| 14 | **Specimen / Sample / Biospecimen** | SNOMED *Specimen*; OBI *specimen*; HPO *Biospecimen phenotypic feature* |
| 15 | **Observable entity / Measurement / Quality** | SNOMED *Observable entity*, *Qualifier value*; OBI *characteristic/quality*; GO (some) |
| 16 | **Data / Format / Information artifact** | EDAM *Data*, *Format*; OBI *information content entity*; SNOMED *Record artifact*; MeSH L (Information Science) |
| 17 | **Role / Function / Application** | ChEBI *role* (biological/chemical/application); OBI *role*, *function*; SNOMED (Qualifier/role) |
| 18 | **Event / Situation / Context** | SNOMED *Event*, *Situation with explicit context*; OBI (planned process) |
| 19 | **Environment / Geographic location** | SNOMED *Environment or geographical location*; MeSH Z (Geographicals) |
| 20 | **Phenomenon / Physical force** | SNOMED *Physical force*; MeSH G01 (Physical Phenomena) |
| 21 | **Mode of inheritance / Genetic mechanism** | HPO *Mode of inheritance*; MONDO *disease of genetic mechanism*; DO *genetic disease* |
| 22 | **Mode / Modifier / Frequency / Severity** | HPO *Clinical modifier*, *Frequency*; SNOMED *Qualifier value*, *Staging and scales* |
| 23 | **Anatomical system / Organ system** | MeSH A (systems); Uberon *organ system*; HPO & MONDO organ-system disease/phenotype groupings |
| 24 | **Social / Demographic context** | SNOMED *Social context*; MeSH M (Named Groups), I (Sociology) |
| 25 | **Discipline / Topic / Field** | EDAM *Topic*; MeSH H (Disciplines), K (Humanities) |
| 26 | **Health care / Service / Organization** | SNOMED (some), MeSH N (Health Care) |
| 27 | **Publication / Document type** | MeSH V (Publication Characteristics); OBI/IAO documents |

### Condensed "core node types" (the most reusable ~12)

For a biomedical KG, the recurring backbone node types are:

1. **Anatomy** (Body structure / Anatomical entity)
2. **Cell / Cell type**
3. **Organism / Taxon**
4. **Gene**
5. **Protein / Macromolecule (+ complex)**
6. **Chemical / Drug / Substance**
7. **Molecular Function**
8. **Biological Process / Pathway**
9. **Phenotype**
10. **Disease / Disorder**
11. **Procedure / Technique / Assay**
12. **Anatomical / Organ System**

…supplemented by metadata/contextual types: **Device, Specimen, Observable/Measurement,
Data/Format, Role/Function, Event/Situation, Environment/Location, Inheritance/Modifier,
Discipline/Topic, Document/Publication.**

---

## Source URLs

- MeSH Tree Structures — https://www.nlm.nih.gov/mesh/intro_trees.html ; https://en.wikipedia.org/wiki/Medical_Subject_Headings
- SNOMED CT Concept Model (top-level hierarchies) — https://docs.snomed.org/snomed-ct-practical-guides/snomed-ct-starter-guide/6-snomed-ct-concept-model
- Gene Ontology overview — https://geneontology.org/docs/ontology-documentation/ ; https://www.ebi.ac.uk/training/online/courses/goa-and-quickgo-quick-tour/what-is-go/
- ChEBI — https://www.ebi.ac.uk/chebi/ ; OLS classes: https://www.ebi.ac.uk/ols4/ontologies/chebi
- MONDO — https://mondo.monarchinitiative.org/ ; OLS: https://www.ebi.ac.uk/ols4/ontologies/mondo
- Disease Ontology — https://disease-ontology.org/ ; OLS: https://www.ebi.ac.uk/ols4/ontologies/doid
- HPO — https://hpo.jax.org/ ; OLS: https://www.ebi.ac.uk/ols4/ontologies/hp
- Uberon — https://obofoundry.org/ontology/uberon.html ; OLS: https://www.ebi.ac.uk/ols4/ontologies/uberon
- Cell Ontology — https://obofoundry.org/ontology/cl.html ; OLS: https://www.ebi.ac.uk/ols4/ontologies/cl
- NCBI Taxonomy — https://www.ncbi.nlm.nih.gov/taxonomy ; ranks update: https://ncbiinsights.ncbi.nlm.nih.gov/2025/02/27/new-ranks-ncbi-taxonomy/
- Protein Ontology (PRO) — https://proconsortium.org/ ; tutorial: https://pmc.ncbi.nlm.nih.gov/articles/PMC4556231/
- OBI — https://obi-ontology.org/ ; https://en.wikipedia.org/wiki/Ontology_for_Biomedical_Investigations
- EDAM — https://edamontology.org/ ; https://github.com/edamontology/edamontology
- EBI Ontology Lookup Service (OLS4) API — https://www.ebi.ac.uk/ols4/api
