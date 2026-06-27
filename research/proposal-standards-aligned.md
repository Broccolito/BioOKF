# BioOKF — Standards-Aligned Type System Proposal

> **Perspective:** Maximize alignment with the **UMLS Semantic Groups** (the 15-group
> partition that covers 99.5% of the UMLS Metathesaurus) and the **Biolink Model**
> (`NamedThing` categories + `related_to` predicate hierarchy), with cross-checks
> against **SPOKE/Hetionet** and **SemMedDB/SemRep**. Goal: a *small* (≈18 node,
> ≈24 edge) set of **general, mutually-exclusive, jointly-exhaustive** types, each
> directly mappable to UMLS Semantic Groups and Biolink CURIEs.
>
> Compiled 2026-06-25. Companion to `taxonomy-umls.md`, `taxonomy-biolink.md`,
> `taxonomy-spoke-hetionet.md`, `taxonomy-semmeddb.md`, `taxonomy-ontologies.md`,
> and `taxonomy-attributes-provenance.md`.

---

## 1. Design rationale

### 1.1 What OKF gives us, and what BioOKF must add

OKF v0.1 represents knowledge as a directory of Markdown-with-YAML-frontmatter
files. Its only required frontmatter field is `type`, and `type` is an **open
vocabulary** — any string. Cross-document links are untyped `[[wiki-style]]` prose
links. The sharpest external critique of OKF (Marc Bara, "A Standard, or Just a
Folder?") is that it **standardizes structure but not meaning**: two conformant
bundles can share no vocabulary, so an agent can read another team's bundle without
*understanding* it.

BioOKF closes exactly that gap **for biomedicine** by making two changes:

1. **Constrain `type` to a finite controlled NODE universe** (§3) — every document
   declares one of ~18 biomedical node types.
2. **Add a finite controlled EDGE universe** (§4) — cross-document links are no
   longer untyped prose; each edge declares one of ~24 typed predicates with
   directionality, domain/range constraints, and a slot for quantitative evidence
   (p-value, effect size, etc.).

This is the Cagle/Shannon "DataBook = OKF + semantic-web superpowers" move,
specialized to the one domain where the controlled vocabularies already exist and
are mature.

### 1.2 Why this perspective picks UMLS Groups + Biolink

The brief is to favor a **small set of general umbrella types** that are
**mutually exclusive and jointly exhaustive**. Among all the prior art, exactly
one source already solved "small + exhaustive + general" for biomedicine: the
**UMLS Semantic Groups**. There are 15 of them, they were explicitly designed to
*partition* the Metathesaurus (every concept lands in exactly one group), and they
cover 99.5% of millions of concepts. They are the gold standard for "exhaustive but
not granular."

The **Biolink Model** is the gold standard for the *interoperable serialization*:
it is the schema the entire NCATS Translator ecosystem (Monarch, ROBOKOP,
RTX/ARAX), Open Targets' BioCypher build, and most modern KGs map onto. Biolink
node *categories* nest cleanly under UMLS groups, and Biolink *predicates* are a
superset of SemMedDB's controlled predicate list.

So BioOKF's node universe = **the 15 UMLS Semantic Groups, lightly refactored**,
and its edge universe = **the canonical Biolink predicates that also appear in
SemMedDB/SemRep**, organized under UMLS's five super-relation families. This makes
every BioOKF type *natively translatable* to a UMLS Semantic Group, a Biolink
category/predicate, and a SPOKE metanode/metaedge — three round-trips for free.

### 1.3 The four refactors away from a literal 15-group copy

A literal copy of the 15 UMLS groups would be *too coarse* in a few places that the
entire downstream KG world (Biolink, SPOKE, Hetionet, PrimeKG) treats as
first-class, and *too granular* in a couple of places. Four deliberate deltas:

| Delta | Move | Why |
|-------|------|-----|
| **Split CHEM** | UMLS lumps drugs, proteins, genes-as-molecules, and small molecules into one "Chemicals & Drugs" group. BioOKF keeps a single **`Molecule`** umbrella (the brief's explicit example) but **promotes `Gene`** out of it. | Biolink, SPOKE, Hetionet, PrimeKG, every NER benchmark treat Gene as the single most-connected node. Folding it into "Molecule" would make the dominant edge class (gene–disease, gene–gene, variant–gene) unexpressible at the type level. `Molecule` still covers proteins, drugs, compounds, metabolites — the brief's intended umbrella. |
| **Split GENE** | Promote **`SequenceFeature`** (variant + genomic region/locus) distinct from `Gene`. | Variants are the subject of GWAS/ClinVar/eQTL — a huge, distinct edge population (variant→trait, variant→gene). SemMedDB-era systems missed this; PubTator3 added Variant as a top entity. Biolink `SequenceVariant`. |
| **Merge PHYS+PHEN+partial ACTI** | One **`BiologicalProcess`** umbrella (normal + pathologic processes, pathways, molecular activities, GO BP/MF). | UMLS splits Physiology, Phenomena, and some Activities; for an LLM-wiki the useful distinction is "process/function" vs "disorder," and Biolink already unifies these under `biological process or activity`. |
| **Add `InformationResource`** | A provenance node type (database, study, publication, dataset). | OKF/Biolink both reify provenance; SPOKE has `DatabaseTimestamp`/`Version`; every KG needs `provided_by`/`primary_knowledge_source` to point *somewhere*. Biolink `information content entity`. |

Everything else is a near-1:1 rename of a UMLS Semantic Group.

### 1.4 Mutual exclusivity / exhaustiveness argument — see §5.

---

## 2. Conventions

- **Frontmatter `type`** carries the node type (PascalCase, e.g. `type: Disease`).
- **Edges** live in the document body as typed links. Proposed surface syntax
  (a typed extension of OKF's `[[…]]`): `[[predicate:: Target Title | attr=value …]]`,
  e.g. `[[treats:: Type 2 Diabetes | knowledge_level=knowledge_assertion;
  primary_knowledge_source=infores:drugcentral]]`. A machine-readable mirror can
  also live in an `edges:` frontmatter list. Direction is **subject → object**:
  the **document is the subject**, the link target is the **object**.
- Every node carries the **universal OKF/Biolink slots** (so they are not repeated
  in each type below):
  - **Required (universal):** `type` (the BioOKF node type) · `title` (preferred
    name) · `id` (a CURIE in a recognized namespace, §1.1 of the attributes doc).
  - **Optional (universal):** `description` · `xref` (equivalent CURIEs) ·
    `synonym[]` · `tags[]` · `timestamp` · `provided_by` (source).
- Every edge carries the **universal Biolink `Association` slots** (so they are not
  repeated in each edge below):
  - **Required (universal):** `subject` (this doc) · `predicate` · `object` ·
    `knowledge_level` (Biolink `KnowledgeLevelEnum`) · `agent_type` (Biolink
    `AgentTypeEnum`). *(Biolink makes exactly these two metadata slots required;
    BioOKF follows that precedent.)*
  - **Optional (universal):** `negated` (bool) · `primary_knowledge_source`
    (`infores:` CURIE) · `aggregator_knowledge_source[]` · `publications[]`
    (PMID/DOI/NCT) · `has_evidence_of_type[]` (ECO) · `evidence_count` ·
    `has_confidence_score` (0–1).

Type-specific required/optional attributes below are *in addition to* these
universals.

---

## 3. NODE UNIVERSE (18 types)

Ordered by UMLS Semantic Group. Each row of the "maps_to" lists the UMLS Semantic
Group, the Biolink category, and the SPOKE/Hetionet metanode.

### 3.1 Physical / structural entities

**1. `Organism`** — A living individual or taxonomic group: human, animal, plant,
microbe, virus, pathogen, cell line, or a population/cohort of organisms.
- *Required:* `taxon` (NCBITaxon CURIE or name).
- *Optional:* `rank` (species/genus/strain…) · `is_pathogen` (bool) ·
  `host` (for pathogens) · `population_descriptor` (for cohorts/ancestry groups).
- *Maps to:* UMLS **LIVB** (Living Beings) [+ `cell line` from ANAT-adjacent]; Biolink
  `OrganismTaxon` / `cellular organism` / `Human` / `Virus` / `Bacterium` /
  `CellLine` / `Cohort`; SPOKE `Organism` / `SARSCov2`.
- *Examples:* `Homo sapiens (NCBITaxon:9606)`, `SARS-CoV-2`, `Mycobacterium
  tuberculosis`, `HeLa cell line`, `UK Biobank European-ancestry cohort`.

**2. `AnatomicalStructure`** — A normal anatomical part at any scale above the
single molecule: body region, organ, tissue, cell type, cellular component/organelle,
or body system.
- *Required:* none beyond universals.
- *Optional:* `scale` (system/organ/tissue/cell/subcellular) · `part_of`
  (parent structure CURIE) · `in_taxon`.
- *Maps to:* UMLS **ANAT** (Anatomy); Biolink `AnatomicalEntity` / `GrossAnatomicalStructure`
  / `Cell` / `CellularComponent`; SPOKE `Anatomy` / `CellType` / `CellularComponent` /
  `AnatomyCellType`.
- *Examples:* `liver (UBERON:0002107)`, `CD4-positive T cell (CL:0000624)`,
  `mitochondrion (GO:0005739)`, `hippocampus`.

### 3.2 Molecular entities

**3. `Gene`** — A gene or genome: a heritable unit of the genome (protein-coding,
pseudogene, ncRNA gene), promoted out of `Molecule` because of its central
connective role.
- *Required:* `in_taxon` (organism CURIE/name).
- *Optional:* `gene_symbol` (HGNC symbol) · `gene_type` (protein_coding /
  lncRNA / pseudogene …) · `chromosome` / `genomic_location` · `loss_of_function_constraint`
  (pLI/LOEUF) · `encodes` (protein CURIE).
- *Maps to:* UMLS **GENE** (Genes & Molecular Sequences); Biolink `Gene`;
  SPOKE/Hetionet `Gene (G)`.
- *Examples:* `TP53 (HGNC:11998 / NCBIGene:7157)`, `APOE`, `BRCA1`.

**4. `Molecule`** — Any chemically-defined substance other than a gene: protein,
peptide, antibody, small-molecule compound, drug, metabolite, ion, cofactor,
hormone, enzyme, lipid, nutrient, protein complex/family/domain, or
pharmacologic-class abstraction. *(The brief's deliberate umbrella; ~one type for
"genes-as-molecules/proteins/compounds/drugs.")*
- *Required:* `molecule_kind` (controlled: `protein` | `small_molecule` | `drug` |
  `metabolite` | `peptide` | `antibody` | `complex` | `protein_family` |
  `protein_domain` | `ion` | `nutrient` | `pharmacologic_class` | `other`).
- *Optional:* `formula` · `inchikey` / `smiles` (structure) · `sequence`
  (for biopolymers) · `gene_origin` (encoding gene CURIE, for proteins) ·
  `drug_class` · `is_approved_drug` (bool) · `mechanism_of_action`.
- *Maps to:* UMLS **CHEM** (Chemicals & Drugs, minus genes); Biolink `ChemicalEntity`
  / `SmallMolecule` / `Drug` / `Protein` / `MacromolecularComplex` / `ProteinFamily`
  / `ProteinDomain`; SPOKE `Compound` / `Protein` / `ProteinFamily` / `ProteinDomain`
  / `Complex` / `PharmacologicClass` / `Nutrient`; Hetionet `Compound (C)` /
  `Pharmacologic Class (PC)`.
- *Examples:* `aspirin (CHEBI:15365)`, `metformin (DRUGBANK:DB00331)`,
  `p53 protein (UniProtKB:P04637)`, `ATP`, `EGFR kinase domain`, `NSAIDs`.

**5. `SequenceFeature`** — A genetic variant or non-gene genomic region: SNP/SNV,
indel, structural variant/CNV, haplotype/allele, or a regulatory/structural locus
(enhancer, promoter, GWAS locus, CpG site, binding site).
- *Required:* `feature_kind` (controlled: `variant` | `region` | `allele` |
  `haplotype`).
- *Optional:* `hgvs` / `rsid` (variant id) · `genome_build` · `coordinates`
  (chrom:pos) · `consequence` (missense/LoF/…) · `maps_to_gene` (gene CURIE) ·
  `allele_frequency`.
- *Maps to:* UMLS **GENE** (variant/sequence side); Biolink `SequenceVariant` /
  `Snv` / `RegulatoryRegion` / `Genotype` / `Haplotype`; SPOKE (no direct metanode —
  folded into edge attributes; PubTator3 `Variant`).
- *Examples:* `rs7412 (DBSNP)`, `BRAF p.Val600Glu`, `9p21 GWAS locus`,
  `enhancer chr8:127…`.

### 3.3 Function, process, and disorder

**6. `BiologicalProcess`** — A normal or pathological function/process/pathway:
biological process, molecular function/activity, signaling/metabolic pathway,
physiologic function, cell/organism function, behavior, or a biochemical reaction.
- *Required:* `process_kind` (controlled: `biological_process` |
  `molecular_function` | `pathway` | `physiologic_function` | `reaction` |
  `behavior`).
- *Optional:* `in_taxon` · `localized_to` (anatomy/cellular component CURIE) ·
  `is_pathological` (bool) · `participants[]` (gene/molecule CURIEs).
- *Maps to:* UMLS **PHYS** (Physiology) + **PHEN** (Phenomena) + part of **ACTI**;
  Biolink `BiologicalProcess` / `MolecularActivity` / `Pathway` /
  `PhysiologicalProcess` / `PathologicalProcess` / `Behavior`; SPOKE
  `BiologicalProcess` / `MolecularFunction` / `Pathway` / `Reaction`; Hetionet
  `Biological Process (BP)` / `Molecular Function (MF)` / `Pathway (PW)`.
- *Examples:* `apoptosis (GO:0006915)`, `DNA binding (GO:0003677)`,
  `glycolysis (REACT:R-HSA-70171)`, `MAPK signaling pathway`.

**7. `Disease`** — A disordered state/syndrome of an organism: disease, syndrome,
neoplasm, mental/behavioral dysfunction, congenital/acquired abnormality, or
pathologic condition.
- *Required:* none beyond universals.
- *Optional:* `disease_category` (e.g., neoplastic / infectious / autoimmune /
  neurodegenerative) · `affected_anatomy` (CURIE) · `inheritance_mode` ·
  `icd_code`.
- *Maps to:* UMLS **DISO** (Disorders, disease branch); Biolink `Disease` /
  `PathologicalProcess`; SPOKE/Hetionet `Disease (D)`.
- *Examples:* `type 2 diabetes mellitus (MONDO:0005148)`,
  `multiple sclerosis (DOID:2377)`, `breast carcinoma`.

**8. `Phenotype`** — An observable characteristic, sign, symptom, side effect,
clinical finding, or quantitative trait/biomarker level (the "Finding/Sign or
Symptom" slice of UMLS Disorders, plus drug side effects).
- *Required:* none beyond universals.
- *Optional:* `is_side_effect` (bool) · `quantitative` (bool, for trait/biomarker
  levels) · `measured_in` (anatomy/sample CURIE) · `hpo_term`.
- *Maps to:* UMLS **DISO** (Sign or Symptom / Finding) [+ SE]; Biolink
  `PhenotypicFeature` / `ClinicalFinding` / `BehavioralFeature`; SPOKE/Hetionet
  `Symptom (S)` / `Side Effect (SE)`.
- *Examples:* `fever (HP:0001945)`, `nausea (drug side effect)`,
  `elevated LDL cholesterol`, `Harris Hip Score`.

### 3.4 Clinical, procedural, and contextual

**9. `Procedure`** — A health-care activity: diagnostic, therapeutic, preventive,
or laboratory procedure; surgery; assay; imaging modality; clinical intervention.
- *Required:* `procedure_kind` (controlled: `diagnostic` | `therapeutic` |
  `preventive` | `laboratory` | `surgical` | `imaging` | `assay`).
- *Optional:* `acts_on` (anatomy/disease CURIE) · `uses_device` (device CURIE) ·
  `measures` (analyte/phenotype CURIE).
- *Maps to:* UMLS **PROC** (Procedures); Biolink `Procedure` / `ClinicalIntervention`
  / `Treatment`; SPOKE (clinical-trial/lab edges); SemMedDB Procedure semtypes.
- *Examples:* `MRI of the brain`, `coronary artery bypass graft`,
  `LDL cholesterol assay (LOINC:2089-1)`, `mammography screening`.

**10. `Device`** — A manufactured physical object used in care or research:
medical device, implant, prosthesis, drug-delivery device, research instrument.
- *Required:* none beyond universals.
- *Optional:* `device_class` · `used_in_procedure` (CURIE) · `regulatory_status`.
- *Maps to:* UMLS **DEVI** (Devices); Biolink `Device` / `DiagnosticAid`;
  (SPOKE: none).
- *Examples:* `hip prosthesis`, `coronary stent`, `Illumina NovaSeq sequencer`,
  `insulin pump`.

**11. `ExposureEvent`** — An exposure, behavior, occupational activity, or
environmental/social factor acting on an organism: chemical/drug exposure, diet,
smoking, physical activity, social determinant of health (SDoH), occupational
hazard.
- *Required:* `exposure_kind` (controlled: `chemical` | `behavioral` |
  `environmental` | `socioeconomic` | `dietary` | `occupational`).
- *Optional:* `agent` (the exposing molecule/factor CURIE) · `duration` ·
  `dose` · `population`.
- *Maps to:* UMLS **ACTI** (Activities & Behaviors, behavior/occupational slice) +
  part of **PHEN**; Biolink `ExposureEvent` (+ subtypes) / `Behavior`; SPOKE `SDoH`
  / `Food` (consumption context).
- *Examples:* `tobacco smoking`, `chronic arsenic exposure`,
  `Mediterranean diet`, `low socioeconomic status (SDoH)`.

### 3.5 Conceptual, geographic, organizational, provenance

**12. `Concept`** — An abstract conceptual entity, attribute, qualitative/quantitative/
temporal/spatial concept, classification, score, or intellectual product that is
not better typed elsewhere. The deliberate catch-all for the UMLS **CONC** group.
- *Required:* none beyond universals.
- *Optional:* `concept_kind` (qualitative / quantitative / temporal / spatial /
  classification / score) · `unit` (for quantitative concepts/scores).
- *Maps to:* UMLS **CONC** (Concepts & Ideas); Biolink `Attribute` / `NamedThing`
  (generic) / `clinical attribute`; (SPOKE: none).
- *Examples:* `body mass index`, `disease stage (TNM)`, `polygenic risk score`,
  `evidence grade`.

**13. `Geography`** — A geographic area, region, location, or place relevant to
epidemiology/public health.
- *Required:* none beyond universals.
- *Optional:* `geo_level` (country / region / city / facility) · `coordinates`.
- *Maps to:* UMLS **GEOG** (Geographic Areas); Biolink `GeographicLocation` /
  `planetary entity`; SPOKE `Location`.
- *Examples:* `Sub-Saharan Africa`, `California`, `UCSF Medical Center`.

**14. `Organization`** — A structured group: health-care organization, professional
society, lab, consortium, company, regulatory body.
- *Required:* none beyond universals.
- *Optional:* `org_kind` (hospital / lab / society / company / regulator) ·
  `located_in` (geography CURIE).
- *Maps to:* UMLS **ORGA** (Organizations); Biolink `Agent` (organizational) /
  `administrative entity`; (SPOKE: none).
- *Examples:* `Baranzini Lab (UCSF)`, `FDA`, `WHO`, `Genentech`.

**15. `Occupation`** — A profession, discipline, occupational/professional group,
or field of study.
- *Required:* none beyond universals.
- *Optional:* `discipline` (clinical / research / etc.).
- *Maps to:* UMLS **OCCU** (Occupations); Biolink `Agent` (occupational group);
  (SPOKE: none).
- *Examples:* `cardiologist`, `bioinformatician`, `medicinal chemistry`.

**16. `Person`** — A specific individual human agent: author, patient/case, donor,
or named scientist (distinct from `Organism` which carries the *taxon*-level human,
and from cohorts). Kept narrow to hold provenance/authorship and case-level data.
- *Required:* none beyond universals.
- *Optional:* `role` (author / patient / donor / investigator) · `orcid` ·
  `affiliation` (organization CURIE).
- *Maps to:* UMLS **LIVB** (individual person) / **CONC** (author as agent);
  Biolink `Agent` / `Case` / `IndividualOrganism`; (SPOKE: none).
- *Examples:* `Sergio Baranzini (ORCID:…)`, `Patient 0042 (case)`,
  `donor D17 (GTEx)`.

**17. `InformationResource`** — A provenance/evidence artifact: database, knowledge
source, dataset, study/clinical trial, publication, guideline, model/tool. The
target of `provided_by` / `primary_knowledge_source` / `publications`.
- *Required:* `resource_kind` (controlled: `database` | `publication` | `dataset` |
  `study` | `clinical_trial` | `guideline` | `tool` | `model`).
- *Optional:* `infores_id` (`infores:` CURIE) · `version` · `url` ·
  `pmid` / `doi` / `nct` · `evidence_level`.
- *Maps to:* UMLS **CONC** (Intellectual Product); Biolink `information content
  entity` / `Publication` / `Dataset` / `Study` / `ClinicalTrial` / `evidence`;
  SPOKE `DatabaseTimestamp` / `Version` (metadata analog).
- *Examples:* `GWAS Catalog (infores:gwas-catalog)`, `gnomAD v4`,
  `PMID:34986598`, `NCT04178122`, `ESC 2023 guideline`.

### 3.6 Catch-all closure

**18. `Other`** — An explicit escape hatch for a biomedical concept that genuinely
fits none of the 17 above (preserving OKF's "minimally opinionated" spirit and the
0.5% of UMLS concepts the Semantic Groups do not partition). Authors SHOULD prefer
a specific type; `Other` exists so BioOKF is *closed* (jointly exhaustive) without
forcing mis-typing.
- *Required:* `note` (free-text reason this is not one of the 17 types).
- *Optional:* `nearest_type` (the closest BioOKF type, if any).
- *Maps to:* UMLS (unpartitioned 0.5%); Biolink `NamedThing` (root); (SPOKE: none).
- *Examples:* edge-case concepts that resist biomedical typing.

> **Node count: 18** (17 substantive + 1 explicit closure). 15 map 1:1 onto a UMLS
> Semantic Group; `Gene` and `SequenceFeature` are the two GENE-group splits;
> `Person` and `InformationResource` carve the agent/provenance concepts out of
> LIVB/CONC; `Other` is the closure.

---

## 4. EDGE UNIVERSE (24 types)

Organized under the **five UMLS super-relation families** (the canonical
finite-but-exhaustive relation backbone). Each edge gives a Biolink predicate
mapping and, where applicable, the SemMedDB predicate and SPOKE/Hetionet metaedge.
All edges inherit the universal `Association` slots from §2; only **edge-specific**
required/optional attributes are listed. Direction is **subject (this document) →
object**.

### 4.A `physically_related_to` (UMLS R1) — composition & structure

**E1. `part_of`** — The subject is a structural/compositional component of the
object (mereology; also `has_part` via inverse).
- *Directionality:* directed (part → whole). Inverse: `has_part`.
- *Domain → Range:* `AnatomicalStructure`/`Molecule`/`Gene`/`SequenceFeature` →
  `AnatomicalStructure`/`Molecule`/`BiologicalProcess`.
- *Required attrs:* none beyond universals.
- *Optional attrs:* `stoichiometry` (for complexes).
- *Maps to:* Biolink `part_of`/`has_part`; UMLS `part_of` (T133); SemMedDB `PART_OF`;
  SPOKE `PARTOF`/`ISA` ontology edges.

**E2. `composed_of`** — The subject is structurally made up of / contains the object
substance (consists_of / contains / has_ingredient).
- *Directionality:* directed (whole → constituent). Inverse: `constituent_of`.
- *Domain → Range:* `Molecule`/`AnatomicalStructure`/`ExposureEvent` →
  `Molecule`/`Gene`.
- *Required attrs:* none beyond universals.
- *Optional attrs:* `amount` · `unit`.
- *Maps to:* Biolink `has_active_ingredient`/`has_part`/`has_nutrient`; UMLS
  `consists_of` (T172)/`contains` (T134); SPOKE `CONTAINS` (Food→Compound).

### 4.B `spatially_related_to` (UMLS R2) — location & expression

**E3. `located_in`** — The subject is positioned/found in the object (anatomical
site of a disease, process, or molecule). Also covers `expressed_in`.
- *Directionality:* directed (entity → location). Inverse: `location_of` / `expresses`.
- *Domain → Range:* `Disease`/`BiologicalProcess`/`Molecule`/`Gene` →
  `AnatomicalStructure`/`Organism`.
- *Required attrs:* none beyond universals.
- *Optional attrs:* `expression_level` · `cell_type_context` (anatomy CURIE).
- *Maps to:* Biolink `located_in`/`expressed_in`/`location_of`/`expresses`; UMLS
  `location_of` (T135); SemMedDB `LOCATION_OF`; SPOKE/Hetionet `AeG`
  (Anatomy-expresses-Gene), `DlA` (Disease-localizes-Anatomy).

### 4.C `temporally_related_to` (UMLS R4) — time & co-occurrence

**E4. `precedes`** — The subject occurs earlier in time than / leads to / develops
into the object (developmental or causal-temporal ordering).
- *Directionality:* directed (earlier → later). Inverse: `preceded_by`.
- *Domain → Range:* `BiologicalProcess`/`Disease`/`Procedure`/`ExposureEvent` →
  same set.
- *Required attrs:* none beyond universals.
- *Optional attrs:* `time_interval`.
- *Maps to:* Biolink `precedes`/`develops_into`; UMLS `precedes` (T138);
  SemMedDB `PRECEDES`.

**E5. `co_occurs_with`** — The subject occurs together with the object (symmetric
comorbidity / co-localization / co-expression / literature co-occurrence).
- *Directionality:* **symmetric** (undirected).
- *Domain → Range:* any → any (most often `Disease`↔`Disease`,
  `Gene`↔`Gene`, `Molecule`↔`Molecule`).
- *Required attrs:* none beyond universals.
- *Optional attrs:* `co_occurrence_count` · `p_value`.
- *Maps to:* Biolink `coexists_with`/`coexpressed_with`/`colocalizes_with`/
  `occurs_together_in_literature_with`; UMLS `co-occurs_with` (T137); SemMedDB
  `COEXISTS_WITH`; Hetionet `GcG` (Gene-covaries-Gene).

### 4.D `functionally_related_to` (UMLS R3) — the rich causal/clinical core

**E6. `causes`** — The subject brings about / induces / is the etiology of the
object (strong causal claim).
- *Directionality:* directed (cause → effect). Inverse: `caused_by`.
- *Domain → Range:* `Molecule`/`Gene`/`SequenceFeature`/`Organism`/`ExposureEvent`/
  `Disease` → `Disease`/`Phenotype`/`BiologicalProcess`.
- *Required attrs:* none beyond universals.
- *Optional attrs:* `effect_size` · `p_value` · `odds_ratio` · `confidence_interval`.
- *Maps to:* Biolink `causes`/`contributes_to`; UMLS `causes` (T147); SemMedDB
  `CAUSES`; SPOKE `Bacterium causes Disease`.

**E7. `predisposes_to`** — The subject increases the likelihood/risk of the object
without being sufficient cause (risk factor).
- *Directionality:* directed (risk factor → outcome). Inverse: `predisposed_by`.
- *Domain → Range:* `SequenceFeature`/`Gene`/`Molecule`/`ExposureEvent`/`Phenotype` →
  `Disease`/`Phenotype`.
- *Required attrs:* none beyond universals.
- *Optional attrs:* `odds_ratio` · `hazard_ratio` · `relative_risk` · `p_value` ·
  `confidence_interval` · `population` (ancestry/cohort).
- *Maps to:* Biolink `predisposes_to_condition`/`affects_likelihood_of`; UMLS
  (under `affects`); SemMedDB `PREDISPOSES`.

**E8. `treats`** — The subject (drug/procedure/intervention) is applied to cure,
manage, ameliorate, or palliate the object condition.
- *Directionality:* directed (intervention → condition). Inverse: `treated_by`.
- *Domain → Range:* `Molecule`/`Procedure`/`Device`/`Organism` →
  `Disease`/`Phenotype`.
- *Required attrs:* none beyond universals.
- *Optional attrs:* `clinical_phase` (preclinical/Phase I–IV/approved) ·
  `efficacy` · `effect_size` · `p_value`.
- *Maps to:* Biolink `treats`/`treats_or_applied_or_studied_to_treat`/
  `ameliorates_condition`; UMLS `treats` (T154); SemMedDB `TREATS`; SPOKE/Hetionet
  `CtD` (Compound-treats-Disease), `CpD` (palliates).

**E9. `prevents`** — The subject stops/hinders the onset of the object condition.
- *Directionality:* directed (intervention → condition). Inverse: `prevented_by`.
- *Domain → Range:* `Molecule`/`Procedure`/`ExposureEvent` → `Disease`/`Phenotype`.
- *Required attrs:* none beyond universals.
- *Optional attrs:* `effect_size` · `relative_risk` · `p_value`.
- *Maps to:* Biolink `preventative_for_condition`; UMLS `prevents` (T148); SemMedDB
  `PREVENTS`.

**E10. `contraindicated_in`** — The subject (drug/procedure) must not be used in the
object condition/context.
- *Directionality:* directed (intervention → condition). Inverse: `has_contraindication`.
- *Domain → Range:* `Molecule`/`Procedure` → `Disease`/`Phenotype`/`Organism`.
- *Required attrs:* none beyond universals.
- *Optional attrs:* `severity`.
- *Maps to:* Biolink `contraindicated_in`; SPOKE `CcD` (Compound-contraindicates-Disease).

**E11. `affects`** — The subject alters/modulates/influences the object's state,
abundance, or activity, with **direction unspecified** (umbrella for the regulatory
edges; use the `direction` qualifier to specialize).
- *Directionality:* directed (modulator → target). Inverse: `affected_by`.
- *Domain → Range:* `Molecule`/`Gene`/`SequenceFeature`/`Disease`/`ExposureEvent` →
  `Gene`/`Molecule`/`BiologicalProcess`/`Phenotype`/`Disease`.
- *Required attrs:* `direction` (controlled: `increases` | `decreases` |
  `unspecified` — Biolink `qualified_predicate`/direction qualifier).
- *Optional attrs:* `aspect` (abundance/activity/expression/…) · `effect_size` ·
  `p_value` · `fold_change`.
- *Maps to:* Biolink `affects`/`regulates`/`disrupts`/`increases_or_decreases…`;
  UMLS `affects` (T151)/`disrupts` (T146); SemMedDB `AFFECTS`/`DISRUPTS`/`AUGMENTS`;
  SPOKE/Hetionet `AuG`/`AdG`/`CuG`/`CdG`/`DuG`/`DdG` (up/down-regulates).

**E12. `inhibits`** — The subject decreases/blocks the action or function of the
object (signed negative regulation; a frequent, high-value specialization kept
distinct for literature-mining fidelity).
- *Directionality:* directed (inhibitor → target). Inverse: `inhibited_by`.
- *Domain → Range:* `Molecule`/`Gene` → `Molecule`/`Gene`/`BiologicalProcess`.
- *Required attrs:* none beyond universals.
- *Optional attrs:* `ic50` · `ki` · `kd` · `assay` (ECO/procedure CURIE).
- *Maps to:* Biolink `affects` + `decreases` direction qualifier; SemMedDB
  `INHIBITS`; PubTator3 `inhibit`.

**E13. `activates`** — The subject increases/facilitates the action or function of
the object (signed positive regulation; symmetric counterpart to `inhibits`).
- *Directionality:* directed (activator → target). Inverse: `activated_by`.
- *Domain → Range:* `Molecule`/`Gene` → `Molecule`/`Gene`/`BiologicalProcess`.
- *Required attrs:* none beyond universals.
- *Optional attrs:* `ec50` · `assay`.
- *Maps to:* Biolink `affects` + `increases` direction qualifier; SemMedDB
  `STIMULATES`; PubTator3 `stimulate`.

**E14. `interacts_with`** — The subject physically or functionally interacts/binds
with the object (symmetric; covers PPI, drug–target binding, drug–drug, gene–gene).
- *Directionality:* **symmetric** (undirected) by default; a `binding` sub-flag can
  mark direct physical binding.
- *Domain → Range:* `Molecule`/`Gene`/`SequenceFeature` ↔ same set.
- *Required attrs:* none beyond universals.
- *Optional attrs:* `interaction_type` (binding / PPI / drug-drug / genetic) ·
  `kd` · `affinity`.
- *Maps to:* Biolink `interacts_with`/`physically_interacts_with`/`binds`/
  `directly_physically_interacts_with`; UMLS `interacts_with` (T142); SemMedDB
  `INTERACTS_WITH`; Hetionet `GiG`, SPOKE `PiP`/`CbP`/`CbG`.

**E15. `produces`** — The subject brings forth / secretes / biosynthesizes /
metabolizes-into the object substance.
- *Directionality:* directed (producer → product). Inverse: `produced_by`.
- *Domain → Range:* `Organism`/`AnatomicalStructure`/`Gene`/`Molecule`/
  `BiologicalProcess` → `Molecule`.
- *Required attrs:* none beyond universals.
- *Optional attrs:* `rate` · `conditions`.
- *Maps to:* Biolink `produces`/`has_metabolite`/`derives_into`; UMLS `produces`
  (T144); SemMedDB `PRODUCES`/`CONVERTS_TO`.

**E16. `encodes`** — The subject gene encodes / is transcribed/translated to the
object gene product (central dogma).
- *Directionality:* directed (gene → product). Inverse: `encoded_by`.
- *Domain → Range:* `Gene` → `Molecule` (protein/RNA).
- *Required attrs:* none beyond universals.
- *Optional attrs:* `product_type` (mRNA / protein / ncRNA).
- *Maps to:* Biolink `has_gene_product`/`transcribed_to`/`translates_to`; SPOKE
  `GeP` (Gene-encodes-Protein).

**E17. `participates_in`** — The subject (gene/molecule/organism) takes part in /
enables / catalyzes the object process or pathway (GO-style functional annotation).
- *Directionality:* directed (participant → process). Inverse: `has_participant`.
- *Domain → Range:* `Gene`/`Molecule`/`Organism` → `BiologicalProcess`.
- *Required attrs:* none beyond universals.
- *Optional attrs:* `role` (enables / catalyzes / input / output / regulator) ·
  `evidence_code` (GO evidence / ECO).
- *Maps to:* Biolink `participates_in`/`enables`/`catalyzes`/`actively_involved_in`;
  UMLS `process_of` (T140)/`carries_out` (T141); SemMedDB `PROCESS_OF`; Hetionet
  `GpBP`/`GpMF`/`GpCC`/`GpPW`.

**E18. `manifests_as`** — The subject (disease/process) is observably expressed as /
presents the object phenotype, sign, or symptom.
- *Directionality:* directed (underlying condition → observable). Inverse:
  `manifestation_of`.
- *Domain → Range:* `Disease`/`BiologicalProcess` → `Phenotype`.
- *Required attrs:* none beyond universals.
- *Optional attrs:* `frequency` (HP frequency term) · `onset`.
- *Maps to:* Biolink `has_phenotype`/`has_manifestation`; UMLS `manifestation_of`
  (T150); SemMedDB `MANIFESTATION_OF`; Hetionet `DpS` (Disease-presents-Symptom).

**E19. `has_adverse_effect`** — The subject (drug/procedure) causes the object
adverse event / side effect (a clinically important specialization of `causes`).
- *Directionality:* directed (intervention → adverse event). Inverse:
  `adverse_effect_of`.
- *Domain → Range:* `Molecule`/`Procedure`/`Device` → `Phenotype`/`Disease`.
- *Required attrs:* none beyond universals.
- *Optional attrs:* `frequency` · `disproportionality` (ROR/PRR/EBGM) · `severity`.
- *Maps to:* Biolink `has_side_effect`/`has_adverse_event`; SemMedDB `COMPLICATES`;
  Hetionet `CcSE` (Compound-causes-Side Effect).

**E20. `affects_response_to`** — The subject (gene/variant) modulates the object
drug's response, sensitivity, resistance, or metabolism (pharmacogenomics).
- *Directionality:* directed (genomic factor → drug). Inverse: `response_affected_by`.
- *Domain → Range:* `Gene`/`SequenceFeature` → `Molecule`.
- *Required attrs:* `response_direction` (controlled: `resistance` | `sensitivity` |
  `altered_metabolism` | `response`).
- *Optional attrs:* `evidence_level` (CPIC/PharmGKB level) · `effect_size`.
- *Maps to:* Biolink `associated_with_response_to`/`affects_response_to`; SPOKE
  pharmacogenomics edges (`mGrC`).

### 4.E `conceptually_related_to` (UMLS R5) — measurement, diagnosis, provenance, ontology

**E21. `diagnoses`** — The subject (procedure/finding/biomarker) identifies or is
diagnostic for the object condition.
- *Directionality:* directed (test/finding → condition). Inverse: `diagnosed_by`.
- *Domain → Range:* `Procedure`/`Phenotype`/`Molecule` → `Disease`.
- *Required attrs:* none beyond universals.
- *Optional attrs:* `sensitivity` · `specificity` · `auc`.
- *Maps to:* Biolink `diagnoses`/`biomarker_for`; UMLS `diagnoses` (T163); SemMedDB
  `DIAGNOSES`; PharmKG `Md` (diagnostic biomarker).

**E22. `measures`** — The subject (procedure/assay/concept) ascertains the quantity
or value of the object analyte, phenotype, or molecule.
- *Directionality:* directed (method → measured thing). Inverse: `measured_by`.
- *Domain → Range:* `Procedure`/`Concept` → `Molecule`/`Phenotype`/`BiologicalProcess`.
- *Required attrs:* none beyond universals.
- *Optional attrs:* `unit` · `reference_range`.
- *Maps to:* Biolink (measurement slots); UMLS `measures` (T162); SemMedDB `MEASURES`.

**E23. `associated_with`** — A statistical or observed (non-causal, non-mechanistic)
association between subject and object — the default GWAS/eQTL/correlation edge and
the umbrella when no stronger relation applies (symmetric).
- *Directionality:* **symmetric** (undirected) by default.
- *Domain → Range:* any → any (most often `Gene`/`SequenceFeature`/`Molecule` ↔
  `Disease`/`Phenotype`).
- *Required attrs:* none beyond universals.
- *Optional attrs:* `p_value` · `adjusted_p_value` · `effect_size` (beta) ·
  `odds_ratio` · `correlation_coefficient` · `sample_size` · `population` ·
  `tissue_context` (for eQTL).
- *Maps to:* Biolink `associated_with`/`gene_associated_with_condition`/
  `genetic_association`/`correlated_with`; UMLS `associated_with` (T166); SemMedDB
  `ASSOCIATED_WITH`; Hetionet `DaG` (Disease-associates-Gene), SPOKE eQTL/GWAS edges.

**E24. `subclass_of`** — Ontological/taxonomic subsumption: the subject is a more
specific kind of / instance of the object (the `isa` backbone, plus identifier
equivalence via the `match_type` qualifier).
- *Directionality:* directed (specific → general). Inverse: `superclass_of`.
- *Domain → Range:* any → same type (and `same_as` for equivalence).
- *Required attrs:* none beyond universals.
- *Optional attrs:* `match_type` (`exact_match` | `close_match` | `same_as` for
  cross-vocabulary equivalence).
- *Maps to:* Biolink `subclass_of`/`superclass_of`/`exact_match`/`same_as`/
  `member_of`; UMLS `isa` (T186); SemMedDB `ISA`; SPOKE/Hetionet `ISA` ontology edges.

> **Edge count: 24.** All 24 nest under one of UMLS's five super-relation families
> (R1 ×2, R2 ×1, R4 ×2, R3 ×15, R5 ×4). All 24 map to a canonical Biolink predicate;
> 18 of them additionally map to a named SemMedDB/SemRep predicate; the regulatory
> trio (`affects`/`inhibits`/`activates`) carries Biolink's `direction` qualifier so
> a single edge family covers up/down-regulation without exploding the vocabulary.

---

## 5. Coverage argument — exhaustive but not granular

**Exhaustive (jointly cover all of biomedicine).** The node universe is anchored to
the **15 UMLS Semantic Groups**, which were *constructed* to partition the entire
UMLS Metathesaurus and cover 99.5% of its multi-million concepts. Every one of the
15 groups is represented:

| UMLS group | BioOKF node type(s) |
|---|---|
| ANAT (Anatomy) | `AnatomicalStructure` |
| CHEM (Chemicals & Drugs) | `Molecule` (+ `Gene` split out) |
| GENE (Genes & Molecular Sequences) | `Gene`, `SequenceFeature` |
| DISO (Disorders) | `Disease`, `Phenotype` |
| PHYS (Physiology) + PHEN (Phenomena) | `BiologicalProcess` |
| PROC (Procedures) | `Procedure` |
| DEVI (Devices) | `Device` |
| ACTI (Activities & Behaviors) | `ExposureEvent` (+ behavior); process side → `BiologicalProcess` |
| LIVB (Living Beings) | `Organism`, `Person` |
| CONC (Concepts & Ideas) | `Concept`, `InformationResource` |
| GEOG (Geographic Areas) | `Geography` |
| ORGA (Organizations) | `Organization` |
| OCCU (Occupations) | `Occupation` |
| OBJC (Objects) | absorbed by `Device`/`Molecule`/`AnatomicalStructure` |
| (unpartitioned 0.5%) | `Other` |

Because the source partition is exhaustive and BioOKF maps every group plus an
explicit `Other` closure, the node universe is **jointly exhaustive**. A second,
independent exhaustiveness check: every node type in **SPOKE (≈21–35), Hetionet
(11), PrimeKG (10), Biolink's core (≈25), and PubTator3 (6)** lands in exactly one
BioOKF type (e.g., Hetionet's 11 metanodes →
Gene→`Gene`, Compound/PharmacologicClass→`Molecule`, Disease→`Disease`,
Anatomy→`AnatomicalStructure`, BP/MF/CC/Pathway→`BiologicalProcess`,
Symptom/SideEffect→`Phenotype`). No prior-art node type is left untypeable.

For edges, the **five UMLS super-relation families** (physically/spatially/
temporally/functionally/conceptually related to) are the exhaustive backbone of the
54-relation UMLS Semantic Network; all 24 BioOKF edges nest under them, and the
SemMedDB ~30-predicate list — itself the exhaustive literature-mining edge set —
maps onto the 24 with no remainder (the SemMedDB comparatives HIGHER/LOWER/SAME_AS
fold into `associated_with`+qualifiers; negation folds into the universal `negated`
slot).

**Not granular (general umbrella types).** The design *deliberately* collapses
fine-grained distinctions the source ontologies make:

- UMLS's 127 Semantic Types → **18 node types** (≈7× compression); Biolink's ~247
  `NamedThing` categories → 18 (≈14× compression).
- `Molecule` is one umbrella over proteins, peptides, antibodies, small molecules,
  drugs, metabolites, ions, complexes, families, domains, and pharmacologic classes —
  the brief's explicit example — using a single `molecule_kind` attribute rather
  than 10 separate types.
- `BiologicalProcess` unifies GO BP/MF/CC-adjacent processes, pathways, reactions,
  physiologic functions, and behaviors that UMLS scatters across PHYS/PHEN/ACTI.
- The regulatory edge explosion (Hetionet/SPOKE have separate up- and
  down-regulation metaedges per node-pair, and provenance-typed variants
  GPuG/OGuG/KGuG) collapses into **`affects` + a `direction` qualifier**, mirroring
  Biolink's own decision to qualify rather than multiply predicates.

**Mutually exclusive.** The node types partition by a single primary question
("what *kind of thing* is the document's subject?"), and the four refactors were
chosen precisely to remove the only real overlaps (Gene vs Molecule; variant vs
gene; process vs disorder; concept vs provenance). The `*_kind` controlled
attributes (`molecule_kind`, `process_kind`, `feature_kind`, `resource_kind`,
`procedure_kind`, `exposure_kind`) carry the sub-type granularity *inside* a type,
so a concept never legitimately belongs to two top-level types. Residual ambiguity
(e.g., a metabolite that is also a biomarker) is resolved by the **"identity, not
role"** rule: type by *what the entity is* (`Molecule`), and express its *role* via
an edge (`diagnoses`/`biomarker_for`) — the standard Biolink convention.

---

## 6. Worked micro-example

A document `aspirin.md`:

```yaml
---
type: Molecule
id: CHEBI:15365
title: Aspirin
molecule_kind: drug
is_approved_drug: true
xref: [DRUGBANK:DB00945, PUBCHEM.COMPOUND:2244]
synonym: [acetylsalicylic acid, ASA]
provided_by: infores:drugcentral
---

Aspirin is a non-steroidal anti-inflammatory drug (NSAID).

[[inhibits:: Prostaglandin G/H synthase 1 (PTGS1) |
  ic50=1.7uM; knowledge_level=knowledge_assertion; agent_type=manual_agent;
  primary_knowledge_source=infores:drugbank]]

[[treats:: Headache |
  knowledge_level=knowledge_assertion; agent_type=manual_agent;
  clinical_phase=approved; primary_knowledge_source=infores:drugcentral]]

[[has_adverse_effect:: Gastrointestinal hemorrhage |
  frequency=common; knowledge_level=knowledge_assertion; agent_type=manual_agent]]

[[part_of:: NSAIDs |
  knowledge_level=knowledge_assertion; agent_type=manual_agent]]   # pharmacologic_class as Molecule
```

This single document exercises four edge families (R3 `inhibits`, R3 `treats`,
R3-adverse `has_adverse_effect`, R1 `part_of`), every edge carries the two required
Biolink metadata slots, and quantitative attributes (`ic50`, `frequency`) ride on
the optional slots — all type-checkable against the controlled universes above.

---

## 7. Summary

- **18 node types**, 15 mapping 1:1 to UMLS Semantic Groups plus `Gene`/
  `SequenceFeature` (GENE split) and `Person`/`InformationResource` (agent/provenance),
  closed by `Other`. Each maps to Biolink categories and SPOKE/Hetionet metanodes.
- **24 edge types**, all nesting under the five UMLS super-relation families, all
  mapping to canonical Biolink predicates, 18 to named SemMedDB predicates, with the
  regulatory family compressed via a Biolink `direction` qualifier.
- **Required vs optional** follows Biolink's precedent: nodes require only
  `type`/`title`/`id`; edges require only `subject`/`predicate`/`object` +
  `knowledge_level`/`agent_type`. Quantitative attributes (p-value, effect size,
  OR/HR, IC50, Kd, sensitivity/specificity) are optional and live on the relevant
  edges.
- The result is **exhaustive** (anchored to the 99.5%-coverage UMLS partition +
  `Other` closure; absorbs all SPOKE/Hetionet/PrimeKG/Biolink/PubTator3 types) yet
  **not granular** (≈7–14× compression vs the source ontologies; umbrella `Molecule`
  and `BiologicalProcess` types; qualifier-based regulation), and every type
  round-trips to UMLS + Biolink + SPOKE.
