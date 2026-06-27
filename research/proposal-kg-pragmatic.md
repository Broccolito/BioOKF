# BioOKF — Pragmatic Queryable-KG Proposal

**Perspective:** Optimize for a practical, queryable biomedical knowledge graph in
the spirit of **SPOKE / Hetionet**. Node types must be ones a curator or LLM agent
can **reliably assign from messy sources** (papers, notes, tweets, tables, slide
decks, lab protocols). Edge types must capture **real biomedical claims**, including
quantitative / statistical associations (GWAS betas, eQTL effects, IC50s, hazard
ratios). Coverage of all of biomedicine is achieved with a **small set of general
umbrella types**, not a granular ontology.

This document specifies how BioOKF constrains OKF, then enumerates the full
**node universe** (15 types) and **edge universe** (18 types) with attributes,
directionality, domain/range, examples, and mappings to UMLS Semantic Groups,
Biolink categories/predicates, and SPOKE/Hetionet metanodes/metaedges.

---

## 1. How BioOKF constrains OKF

OKF (v0.1) requires every concept document to carry a `type` in YAML frontmatter
but leaves `type` an **open vocabulary** ("Type values are not registered
centrally… consumers MUST tolerate unknown types"). OKF also has **no typed edges** —
relationships are plain markdown links plus prose.

BioOKF makes three changes and keeps everything else (markdown bundle, `index.md`,
`log.md`, citations, free body) intact:

1. **`type` is closed.** `type` MUST be one of the **15 node types** in §3. An
   unknown `type` is invalid in BioOKF (vs. "tolerate gracefully" in OKF). Each
   node still carries a free markdown body for human-readable detail.

2. **Typed edges are added.** A new reserved frontmatter key **`edges`** holds a
   YAML list of typed, attributed edges. Each edge names an `edge` type from the
   **18 edge types** in §4, a `target` (a concept ID or a CURIE), and an attribute
   bundle. This reifies edges (à la Biolink `Association`) so quantitative and
   provenance metadata can hang off each claim. Plain markdown links remain legal
   for un-typed "see also" cross-references; only `edges` entries are part of the
   graph.

3. **Every node and edge carries identity + provenance attributes** drawn from a
   controlled set (CURIEs, `knowledge_level`, `agent_type`, `primary_source`),
   mirroring the two attributes Biolink makes mandatory.

### 1.1 Minimal BioOKF concept document

```yaml
---
type: Disease                          # REQUIRED — one of the 15 node types
title: Multiple sclerosis
id: MONDO:0005301                       # REQUIRED — primary CURIE
xref: [DOID:2377, MESH:D009103, UMLS:C0026769]
synonyms: [MS, disseminated sclerosis]
edges:
  - edge: ASSOCIATES                    # gene–disease genetic association
    target: HGNC:4932                   # HLA-DRB1
    direction: bidirectional
    knowledge_level: statistical_association   # REQUIRED on every edge
    agent_type: text_mining_agent              # REQUIRED on every edge
    primary_source: infores:gwas-catalog       # REQUIRED on every edge
    p_value: 5.0e-30
    odds_ratio: 3.1
    publications: [PMID:21833088]
  - edge: TREATS
    target: DRUGBANK:DB00073            # rituximab
    direction: forward
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: infores:drugbank
---
# Multiple sclerosis
Chronic inflammatory demyelinating disease of the CNS. …
```

---

## 2. Design principles (why these types)

- **Umbrella over granular.** A single **`Molecule`** type covers small molecules,
  drugs, metabolites, cofactors, and ions — the distinction is an optional
  `molecule_kind` attribute, not a separate node type. This matches the recurring
  curator pain point: "is caffeine a drug, a compound, a metabolite, or a
  nutrient?" — in BioOKF it is always a `Molecule`. Likewise one **`Gene`** type
  covers gene / transcript / locus, one **`Protein`** type covers protein /
  complex / family / domain, and one **`Phenotype`** type covers symptoms, signs,
  clinical findings, and abnormal phenotypes (PrimeKG and PrimeKG-derivatives
  already merge phenotype + side-effect, and gene + protein; we keep gene and
  protein separate because the gene/protein edge — `ENCODES` — is a real,
  frequently-asserted claim worth representing).

- **Assignable from messy text.** Every type maps to a concept an NER tagger or an
  LLM reliably recognizes. The seven core types (`Gene`, `Protein`, `Molecule`,
  `Disease`, `Phenotype`, `Anatomy`, `Pathway`) are exactly the entities
  PubTator3, SemRep, Hetionet, and SPOKE all tag. We deliberately do **not** add
  fine ontology distinctions (e.g. ChEBI's role-vs-structure split) that a curator
  cannot decide from a tweet.

- **Provenance is first-class.** Sources range from gold-standard curated
  databases to tweets. Following Biolink, **every edge** carries
  `knowledge_level` + `agent_type` + `primary_source`, so a `statistical_association`
  mined from a preprint is never confused with a `knowledge_assertion` from
  DrugBank. This is the single most important affordance for a graph fed by
  heterogeneous sources.

- **Quantitative claims are native.** Edge types that carry statistics
  (`ASSOCIATES`, `REGULATES_EXPRESSION`, `BINDS`, `CORRELATES_WITH`,
  `CAUSES`/`TREATS` from trials) have optional `p_value`, `adjusted_p_value`,
  `effect_size`, `effect_metric` (beta/OR/HR/RR/logFC/r/IC50/Kd…), `ci_lower`,
  `ci_upper`, `sample_size` slots, so a GWAS row, a DE table, and a dose-response
  curve all land as fully-attributed edges.

- **A small set of "context" node types** (`Variant`, `CellType`, `Organism`,
  `BiologicalProcess`, `ClinicalAttribute`, `Exposure`) round out coverage of
  genetics, single-cell, microbiology, GO function, lab measurements, and
  environmental/behavioral factors — each justified by a recurring source pattern
  in the research corpus.

- **Provenance / document nodes deliberately excluded as node types.** Studies,
  trials, datasets, tools, authors, and publications are **not** node types — they
  appear as *edge attributes* (`primary_source`, `publications`,
  `supporting_study`). This keeps the node universe purely biomedical and avoids
  the SPOKE "DatabaseTimestamp / Version" bookkeeping-node clutter. (A producer who
  truly needs a trial as a graph node can model it as an `Exposure` or attach it
  via `publications`.)

---

## 3. Node universe (15 types)

Counts in parentheses indicate the umbrella's breadth. Every node requires
`type`, `id` (a CURIE), and `name`. Optional attributes are listed per type plus
the universal optionals: `xref`, `synonyms`, `description`, `provided_by`,
`in_taxon`.

| # | Node type | Covers (umbrella) | UMLS group | Biolink | SPOKE/Hetionet |
|---|-----------|-------------------|-----------|---------|----------------|
| 1 | **Gene** | gene, transcript, locus, regulatory element, ncRNA/miRNA | GENE | `biolink:Gene` | Gene (G), MiRNA |
| 2 | **Protein** | protein, complex, family, domain, isoform, enzyme | CHEM/GENE | `biolink:Protein` | Protein, Complex, ProteinFamily, ProteinDomain |
| 3 | **Molecule** | small molecule, drug, metabolite, cofactor, ion, nutrient, food chemical | CHEM | `biolink:ChemicalEntity` | Compound, Nutrient, Food(chem) |
| 4 | **Variant** | SNV/SNP, indel, CNV/SV, allele, genotype, haplotype | GENE | `biolink:SequenceVariant` | (SPOKE Variant, live) |
| 5 | **Disease** | disease, syndrome, disorder, infection, cancer subtype | DISO | `biolink:Disease` | Disease (D) |
| 6 | **Phenotype** | symptom, sign, clinical finding, side effect, abnormal/quantitative phenotype | DISO/PHEN | `biolink:PhenotypicFeature` | Symptom (S), SideEffect (SE) |
| 7 | **Anatomy** | organ, tissue, body region, anatomical system, subcellular component | ANAT | `biolink:AnatomicalEntity` | Anatomy (A), CellularComponent |
| 8 | **CellType** | cell type, cell state, cell line | ANAT/LIVB | `biolink:Cell` / `biolink:CellLine` | CellType, CellLine |
| 9 | **Pathway** | pathway, gene set, reaction, biochemical network | PHYS | `biolink:Pathway` | Pathway (PW), Reaction, PwGroup |
| 10 | **BiologicalProcess** | GO biological process, molecular function, physiological/pathological process | PHYS | `biolink:BiologicalProcess` / `biolink:MolecularActivity` | BiologicalProcess, MolecularFunction |
| 11 | **Organism** | organism, species, strain, pathogen, microbial taxon | LIVB | `biolink:OrganismTaxon` | Organism, SARSCov2 |
| 12 | **ClinicalAttribute** | lab test / measurement, biomarker readout, vital sign, risk score (PRS), imaging finding | CONC/PROC | `biolink:ClinicalAttribute` / `biolink:ClinicalFinding` | ClinicalLab |
| 13 | **Exposure** | environmental/chemical/behavioral/dietary exposure, social determinant, procedure/intervention | CONC/PROC | `biolink:ExposureEvent` / `biolink:Procedure` | SDoH, Food |
| 14 | **PharmacologicClass** | drug class, mechanism class, ATC class | CHEM | `biolink:ChemicalEntity` (PC) | PharmacologicClass (PC), Atc |
| 15 | **AnatomicalSystem** *(merged into Anatomy)* | — | — | — | — |

> Note: type 15 is intentionally **folded into Anatomy** (an organ system is an
> `Anatomy` node with `anatomy_scale: system`). The universe is therefore **15
> rows but 14 distinct node types** plus a documented merge; we list it to show the
> umbrella decision explicitly. The structured `node_universe` returned below
> enumerates the 14 real types.

### 3.1 Per-type required / optional attributes

**Universal (every node):** required `type`, `id` (CURIE), `name`; optional
`xref`, `synonyms`, `description`, `provided_by`, `in_taxon` (NCBITaxon CURIE).

| Type | Type-specific required | Type-specific optional |
|------|------------------------|------------------------|
| Gene | — | `gene_kind` (protein_coding/ncRNA/miRNA/pseudogene/locus), `chromosome`, `strand`, `gene_symbol`, `constraint_loeuf` |
| Protein | — | `protein_kind` (protein/complex/family/domain/isoform/enzyme), `uniprot`, `ec_number`, `sequence_length` |
| Molecule | — | `molecule_kind` (small_molecule/drug/metabolite/cofactor/ion/nutrient), `smiles`, `inchikey`, `formula`, `is_approved_drug` |
| Variant | `variant_kind` (snv/indel/cnv/sv/allele/genotype/haplotype) | `hgvs`, `rsid`, `chromosome`, `position`, `ref_allele`, `alt_allele`, `gene` (CURIE), `clinical_significance` |
| Disease | — | `disease_category`, `icd10`, `mondo`, `is_cancer` |
| Phenotype | `phenotype_kind` (symptom/sign/finding/side_effect/abnormal/quantitative) | `hpo`, `body_system`, `severity` |
| Anatomy | — | `anatomy_scale` (system/organ/region/tissue/subcellular), `uberon` |
| CellType | — | `cell_kind` (cell_type/cell_state/cell_line), `cl_id`, `markers` [], `lineage` |
| Pathway | — | `pathway_kind` (pathway/reaction/gene_set), `source_db` (Reactome/KEGG/GO/WikiPathways), `species` |
| BiologicalProcess | `bp_kind` (biological_process/molecular_function/physiological/pathological) | `go_id`, `go_aspect` |
| Organism | — | `taxon_rank`, `ncbi_taxon`, `is_pathogen` |
| ClinicalAttribute | `attribute_kind` (lab_test/biomarker/vital_sign/risk_score/imaging_finding) | `loinc`, `unit`, `reference_range` |
| Exposure | `exposure_kind` (environmental/chemical/behavioral/dietary/social/procedure) | `ecto_id`, `duration`, `route` |
| PharmacologicClass | — | `class_system` (ATC/MeSH-PA/NDF-RT), `atc_code`, `mechanism` |

### 3.2 Why each node type earns its place (assignability evidence)

- **Gene / Protein / Molecule / Disease / Phenotype / Anatomy** — the universal
  core: tagged by PubTator3 (Gene, Disease, Chemical), SemRep, Hetionet (G/C/D/A/S/SE),
  SPOKE, every dataset and figure source. An LLM never struggles to label these.
- **Variant** — genetics/genomics and clinical-genetics sources (`rsID`, HGVS,
  ClinVar, GWAS Catalog) require a first-class variant node distinct from Gene; it
  is PubTator3's `Variant` type and the subject of the most quantitative edge of
  all (GWAS `ASSOCIATES`).
- **CellType** — single-cell / spatial sources, lab protocols, and figure captions
  constantly assert "marker-of cell type", "expressed-in cell type"; merges the
  `cell_line` distinction (SPOKE keeps them separate, we don't need to).
- **Pathway / BiologicalProcess** — pathway diagrams, GO annotations, enrichment
  tables. We split them because `PARTICIPATES_IN` (gene→process) and
  `PART_OF` (reaction→pathway) are different claims, but both are reliably taggable.
- **Organism** — microbiome, infectious disease, host-pathogen, and "source
  organism" of natural products; PubTator3 `Species`.
- **ClinicalAttribute** — clinical literature, EHR/tabular sources, and labs assert
  biomarker/lab/risk-score readouts that are neither a Disease nor a Molecule.
- **Exposure** — environmental epidemiology, SDoH, diet, and procedures/interventions;
  the Biolink `ExposureEvent`/`Procedure` umbrella, needed for Mendelian-randomization
  and "exposure→outcome" edges.
- **PharmacologicClass** — drug-class reasoning and repurposing (Hetionet `PC`,
  `PCiC` includes Compound); cheaply taggable from drug-label text.

---

## 4. Edge universe (18 types)

Edges are reified: every edge is a `(subject = the host concept, edge, target)`
triple plus an attribute bundle. **Directionality** is one of `forward`
(subject→object, default), `bidirectional` (symmetric), or `undirected`.

**Universal required edge attributes (Biolink-mandated trio + target):**
`edge` (type), `target` (concept ID or CURIE), `direction`, `knowledge_level`
(7-value enum), `agent_type` (8-value enum), `primary_source` (`infores:` CURIE).

**Universal optional edge attributes:** `negated` (bool), `publications` [] (PMID/DOI/NCT),
`aggregator_source` [], `supporting_text` [], `evidence_count`,
`has_evidence_of_type` (ECO), `qualifiers` (free map: species/anatomy/context),
`timepoint`.

**Quantitative optional attributes** (present on the statistical edges, marked ✚):
`p_value`, `adjusted_p_value`, `effect_size`, `effect_metric`
(beta/odds_ratio/hazard_ratio/relative_risk/log2_fold_change/correlation_r/IC50/Kd/Ki/EC50/MIC/enrichment_score),
`ci_lower`, `ci_upper`, `standard_error`, `sample_size`, `confidence_score` (0–1),
`direction_of_effect` (increased/decreased).

| # | Edge type | Definition | Dir. | Domain → Range | Quant ✚ | UMLS / Biolink / SPOKE |
|---|-----------|------------|------|----------------|---------|------------------------|
| 1 | **ASSOCIATES** | Statistical / genetic / observed association between two entities (GWAS, gene–disease, biomarker–disease, taxon–phenotype) | bi | Gene/Variant/Molecule/CellType/Organism/ClinicalAttribute → Disease/Phenotype/ClinicalAttribute | ✚ | `associated_with` / `biolink:associated_with`,`gene_associated_with_condition` / DaG |
| 2 | **CAUSES** | Subject causally produces / induces the object condition or effect | fwd | Molecule/Variant/Organism/Exposure/Disease → Disease/Phenotype/BiologicalProcess | ✚ | `causes` / `biolink:causes` / — |
| 3 | **TREATS** | Intervention ameliorates, manages, or cures a condition | fwd | Molecule/Exposure/PharmacologicClass → Disease/Phenotype | ✚ | `treats` / `biolink:treats` / CtD |
| 4 | **PREVENTS** | Subject stops, hinders, or reduces risk of the object condition | fwd | Molecule/Exposure → Disease/Phenotype | ✚ | `prevents` / `biolink:prevents` / — |
| 5 | **CONTRAINDICATED_IN** | Subject is contraindicated / causes adverse effect in the object condition | fwd | Molecule/PharmacologicClass → Disease/Phenotype | — | (R3.1 disrupts) / `biolink:contraindicated_in` / CcD,CcSE |
| 6 | **BINDS** | Physical binding / direct molecular interaction (drug–target, PPI, TF–DNA, antibody–antigen) | bi | Molecule/Protein/Gene → Protein/Gene | ✚ | `interacts_with` / `biolink:binds` / CbP,PiP,CbG |
| 7 | **REGULATES_ACTIVITY** | Subject increases/decreases activity or function of object (activates/inhibits/agonist/antagonist) | fwd | Molecule/Protein → Protein/BiologicalProcess | ✚ | `affects`(INHIBITS/STIMULATES) / `biolink:affects`+direction qual / — |
| 8 | **REGULATES_EXPRESSION** | Subject up-/down-regulates expression of a gene/protein (incl. eQTL, perturbation, anatomy-expresses) | fwd | Gene/Variant/Molecule/Anatomy/Disease → Gene/Protein | ✚ | (AFFECTS) / `biolink:regulates`,`affects` (expression aspect) / AuG,AdG,CuG,CdG |
| 9 | **ENCODES** | Gene encodes / is transcribed-translated to its product | fwd | Gene → Protein | — | (R) / `biolink:has_gene_product` / GeP |
| 10 | **PARTICIPATES_IN** | Subject participates in / enables / is a member of a pathway or process | fwd | Gene/Protein/Molecule → Pathway/BiologicalProcess | — | `process_of`/`affects` / `biolink:participates_in`,`enables` / GpPW,GpBP |
| 11 | **EXPRESSED_IN** | Gene/protein is expressed in (marker of) an anatomy or cell type | fwd | Gene/Protein → Anatomy/CellType | ✚ | `location_of`(inv) / `biolink:expressed_in` / AeG,GeiCT |
| 12 | **LOCATED_IN** | Spatial/anatomical localization or part-of (disease→anatomy, cell→tissue, structure→structure) | fwd | Disease/Phenotype/CellType/Anatomy → Anatomy | — | `location_of`,`part_of` / `biolink:located_in` / DlA |
| 13 | **HAS_PHENOTYPE** | A disease/organism/genotype presents a phenotype or symptom | fwd | Disease/Organism/Variant → Phenotype | ✚ | `manifestation_of`(inv) / `biolink:has_phenotype` / DpS |
| 14 | **AFFECTS_RESPONSE_TO** | Gene/variant/biomarker affects response to / metabolism of a drug (pharmacogenomic) | fwd | Gene/Variant/Protein → Molecule | ✚ | `affects` / `biolink:affects_response_to` / mGrC |
| 15 | **INTERACTS_WITH** | Drug–drug, drug–food, gene–gene, or host–pathogen interaction (non-binding, general) | bi | Molecule/Gene/Organism → Molecule/Gene/Organism | ✚ | `interacts_with` / `biolink:interacts_with` / GiG,CiF |
| 16 | **CORRELATES_WITH** | Quantitative co-variation / co-expression / similarity between two same-kind entities | bi | Gene/Molecule/Disease/ClinicalAttribute → same | ✚ | `co-occurs_with` / `biolink:correlated_with`,`coexpressed_with` / GcG |
| 17 | **MEMBER_OF** | Subject belongs to a class/family/group (drug∈class, protein∈family, gene∈geneset) | fwd | Molecule/Protein/Gene → PharmacologicClass/Protein/Pathway | — | `isa` / `biolink:member_of`,`subclass_of` / PCiC,ISA |
| 18 | **SUBCLASS_OF** | Hierarchical is-a / part-of between two nodes of the same node type (ontology backbone) | fwd | any → same node type | — | `isa` / `biolink:subclass_of` / DiD,CiC,AiA |

### 4.1 Edge-design rationale

- **Statistical edges carry the full quant bundle.** `ASSOCIATES` is the workhorse
  for GWAS / gene–disease / biomarker rows; its `effect_metric` discriminates
  beta vs odds_ratio vs hazard_ratio so a GWAS row, a case-control study, and a Cox
  model all fit one edge type with self-describing statistics. This directly
  implements the research corpus's "variant — associated-with → trait (effect
  size/p-value/ancestry)" pattern and Biolink's `Association` statistical slots.

- **Two regulation edges, split by what is regulated.** `REGULATES_ACTIVITY`
  (function/activity: activate/inhibit, the kinase/drug-target world) vs
  `REGULATES_EXPRESSION` (abundance: up/down-regulate, eQTL, perturbation, anatomy
  expression). This is the single most common source of confusion in text mining;
  splitting on *aspect* (activity vs expression) — rather than minting 8 directed
  edges as Hetionet/SPOKE do (AuG/AdG/CuG/CdG/GPuG…) — keeps the universe small
  while preserving the claim. Direction (up vs down) is the optional
  `direction_of_effect` attribute, mirroring Biolink's direction qualifier.

- **`BINDS` vs `INTERACTS_WITH`.** `BINDS` is physical/structural (drug–target,
  PPI, TF–DNA) and carries affinity (`Kd`/`Ki`/`IC50` via `effect_metric`).
  `INTERACTS_WITH` is the general non-binding catch-all (DDI, drug–food,
  host–pathogen, genetic interaction). Curators reliably tell "binds/inhibits a
  target" from "interacts with another drug".

- **Therapeutic cluster** (`TREATS`, `PREVENTS`, `CONTRAINDICATED_IN`,
  `CAUSES`) covers the clinical literature, pharmacology, and SemRep/PubTator
  predicate sets; adverse events fold into `CAUSES` (Compound→Phenotype/side_effect)
  or `CONTRAINDICATED_IN`, avoiding a separate `has_side_effect` edge.

- **Two hierarchy edges** keep the ontology backbone explicit without polluting the
  associative edges: `SUBCLASS_OF` (same-type is-a, e.g. disease→disease) and
  `MEMBER_OF` (cross-type membership, e.g. drug→PharmacologicClass, gene→gene-set).
  These let consumers do query closure (Biolink `*_category_closure`) the way SPOKE
  uses its 11 ontology ISA layers.

- **Provenance discipline on every edge** is what makes a graph fed by tweets *and*
  DrugBank usable: a reader filters on `knowledge_level ∈ {knowledge_assertion}`
  for clinical decisions and admits `statistical_association` / `text_co_occurrence`
  for hypothesis generation. This is the corpus's repeated "provenance with
  evidence level / review status" requirement made mandatory.

---

## 5. Coverage argument (exhaustive but not granular)

**Node coverage.** Map the union of every entity list in the prior-art corpus onto
the 14 node types:

- *Molecular science / biochemistry:* Gene, Protein (incl. complex/family/domain),
  Molecule (incl. metabolite/cofactor/ion), Pathway, BiologicalProcess (BP+MF),
  Variant — covers genes-as-molecules, enzymes, PTMs (as Protein attributes),
  reactions (Pathway `pathway_kind: reaction`), and signaling.
- *Chemistry / chem-biology / med-chem:* Molecule subsumes compound/lead/fragment/
  probe/PROTAC/natural-product/payload; substructure/scaffold are Molecule
  attributes (`smiles`, `inchikey`); assays/IC50 live as `BINDS`/`REGULATES_ACTIVITY`
  edge statistics. PharmacologicClass covers drug classes.
- *Genetics / genomics:* Variant (SNV/indel/CNV/allele/genotype/haplotype), Gene
  (locus/regulatory element/ncRNA/miRNA), plus eQTL/pQTL via `REGULATES_EXPRESSION`
  and PRS via ClinicalAttribute (`risk_score`).
- *Medicine / clinical:* Disease, Phenotype (symptom/sign/finding/side-effect),
  ClinicalAttribute (labs/biomarkers/vitals/imaging), Exposure (procedures/SDoH),
  Anatomy, CellType.
- *Microbiology / infectious disease / microbiome:* Organism (species/strain/
  pathogen/taxon), with host–pathogen as `INTERACTS_WITH`/`CAUSES`.
- *Anatomy / cell biology:* Anatomy (organ→subcellular, systems folded in),
  CellType (type/state/line).

Cross-checked against UMLS's 15 Semantic Groups: ANAT→Anatomy/CellType,
CHEM→Molecule/Protein/PharmacologicClass, DISO→Disease/Phenotype,
GENE→Gene/Variant, PHYS→Pathway/BiologicalProcess, LIVB→Organism,
PROC/CONC→ClinicalAttribute/Exposure. The remaining UMLS groups (DEVI, GEOG,
OCCU, ORGA, OBJC, ACTI) are **deliberately out of scope** — they are not
biomedical *claims* and are pushed to edge attributes/provenance (e.g. a device or
study is `primary_source`/`publications`, not a node). This is the intended
non-granularity, not a gap.

**Edge coverage.** The 18 edges subsume the recurring relation verbs across all
source families: SemRep's ~30 predicates collapse into this set (TREATS, PREVENTS,
CAUSES, ASSOCIATES, BINDS/INTERACTS_WITH, REGULATES_*, LOCATED_IN, PARTICIPATES_IN,
HAS_PHENOTYPE, SUBCLASS_OF); PubTator3's 13 relations map (treat→TREATS,
cause→CAUSES, inhibit/stimulate→REGULATES_ACTIVITY, interact/drug_interact→
INTERACTS_WITH/BINDS, associate/correlate→ASSOCIATES/CORRELATES_WITH,
prevent→PREVENTS); Hetionet's 24 metaedges map (CtD→TREATS, CbG/CbP→BINDS,
DaG→ASSOCIATES, AeG/AuG/AdG→EXPRESSED_IN/REGULATES_EXPRESSION, GpPW/GpBP→
PARTICIPATES_IN, DpS→HAS_PHENOTYPE, GiG→INTERACTS_WITH, GcG→CORRELATES_WITH,
PCiC→MEMBER_OF, CrC/DrD→CORRELATES_WITH, CcSE→CAUSES, CpD→TREATS); SPOKE's
pharmacogenomic, biochemistry, and nutrition edges fold into AFFECTS_RESPONSE_TO,
PARTICIPATES_IN/BINDS, and INTERACTS_WITH respectively. Quantitative claims that
prior schemas drop into opaque "score" fields are instead first-class
`effect_metric`+`p_value`+CI attributes here.

**Non-granularity is explicit.** Where Open Targets BioCypher mints 128 edge types
and SPOKE-live ~87, BioOKF uses 18 by (a) folding direction into attributes
(`direction_of_effect`), (b) folding source/datatype into provenance
(`primary_source`, `has_evidence_of_type`), and (c) folding sign/affinity into
`effect_metric`. The cost — losing per-datasource edge identity — is paid back by
the `primary_source`/`aggregator_source` attributes, so nothing is actually lost.

---

## 6. Summary

BioOKF = OKF's portable markdown bundle + a **closed 14-type node vocabulary** +
a **closed 18-type typed-edge vocabulary**, each edge reified with a mandatory
provenance trio (`knowledge_level`, `agent_type`, `primary_source`) and optional
quantitative bundle. The universes are **exhaustive** (every biomedical entity and
claim in the prior-art corpus maps in) yet **general** (umbrella `Molecule` /
`Protein` / `Phenotype` types and aspect-split regulation edges keep the count
low and the types reliably assignable from messy sources).
