# BioOKF design rationale — the node & edge universes

> Why exactly these **20 node types** and **23 edge types**, how they map onto the established
> biomedical type systems, and the load-bearing design decisions. This is the chain-of-thought
> behind [SPEC.md](../SPEC.md §5–6). It is grounded in eight prior-art taxonomy references in
> [../research/](../research/) and in the entity/relation patterns observed across **1,100+
> cataloged biomedical sources** in [../sources/](../sources/).

> **v0.2 naming note.** This document derives the universe using the original v0.1 type names.
> Two were renamed in v0.2 (see [SPEC.md §14](../SPEC.md#14-changelog)): **`Variant` →
> `GenomicFeature`** (broadened to cover constitutive sequence features — UTRs, codons, exons,
> promoters, transposons, loci — alongside variants) and **`BiologicalProcess` → `Process`**
> (to avoid the GO `biological_process` collision). Read the `Variant`/`BiologicalProcess`
> mentions below as `GenomicFeature`/`Process`; the derivation is otherwise unchanged.

## 1. The design problem

The brief: design a **finite, controlled** universe of node types and edge types for
biomedicine that is **exhaustive** (covers epidemiology, genetics, molecular & cell biology,
biochemistry, pharmacology, medicinal/chemical biology, clinical specialties like
orthopaedics, public health, microbiology…) but **not granular** (prefer general umbrella
types; the *union* of concepts must be comprehensive, not the depth of any one).

Two failure modes to avoid:
- **Too granular** — Open Targets' BioCypher build mints ~57 node types and ~128 edge types;
  UMLS has 127 semantic types. Curators (and LLMs reading messy sources) cannot reliably
  assign at that resolution, and bundles stop interoperating.
- **Too coarse** — a single "Entity / relates_to" pair is trivially exhaustive and useless.

The right altitude is the one biomedicine *already solved* twice: the **UMLS Semantic
Groups** (15 groups partitioning 99.5% of millions of concepts) and the **SPOKE/Hetionet
metagraph** (~11 metanodes spanning a 27M-node graph). BioOKF sits between them.

## 2. The prior art we synthesized

| Resource | Nodes | Edges | What it contributes |
|---|---|---|---|
| **UMLS Semantic Network** | **127 semantic types** in **15 Semantic Groups** | **54 relations** in **5 super-relation families** | The proven exhaustive partition; the relation backbone. *[research/taxonomy-umls.md](../research/taxonomy-umls.md)* |
| **Biolink Model** (NCATS Translator) | ~25 core `NamedThing` categories (of ~247) | predicate hierarchy + association qualifiers; **only `knowledge_level`+`agent_type` required** | The interoperable serialization + the provenance/qualifier discipline. *[research/taxonomy-biolink.md](../research/taxonomy-biolink.md)* |
| **SPOKE** (UCSF Baranzini Lab) | ~21–35 metanodes | ~55 metaedges | The target graph BioOKF round-trips to. *[research/taxonomy-spoke-hetionet.md](../research/taxonomy-spoke-hetionet.md)* |
| **Hetionet** | **11 metanodes** | **24 metaedges** | The canonical compact KG; the abbreviation scheme. |
| **SemMedDB / SemRep** | UMLS semtypes | **~30 predicates** | The literature-mining edge universe (TREATS, CAUSES, INHIBITS, STIMULATES, INTERACTS_WITH, COEXISTS_WITH, LOCATION_OF, PART_OF, PROCESS_OF, AFFECTS, PREVENTS, PREDISPOSES, DISRUPTS, AUGMENTS, ASSOCIATED_WITH, MANIFESTATION_OF…). *[research/taxonomy-semmeddb.md](../research/taxonomy-semmeddb.md)* |
| **PubTator3** | 6 entities (Gene, Disease, Chemical, Species, Variant, CellLine) | 13 relations | What an NER tagger reliably finds in free text. |
| **PrimeKG / Open Targets / Monarch / ROBOKOP / DRKG / CKG** | 10–57 nodes | many | Cross-checks for coverage & for what is "too granular." *[research/taxonomy-kgs.md](../research/taxonomy-kgs.md)* |
| **Ontology top-levels** (MeSH, SNOMED, GO, ChEBI, MONDO, HPO, Uberon, CL, NCBITaxon) | — | — | The general entity classes each implies; the `id` namespaces. *[research/taxonomy-ontologies.md](../research/taxonomy-ontologies.md)* |
| **NER/RE benchmarks** (BioRED, BC5CDR, ChemProt, DDI) | entity sets | relation sets | What is reliably *assignable*. *[research/taxonomy-ner-benchmarks.md](../research/taxonomy-ner-benchmarks.md)* |
| **Attribute/provenance models** (ECO, Biolink Association, CURIEs) | — | — | Required-vs-optional attribute design. *[research/taxonomy-attributes-provenance.md](../research/taxonomy-attributes-provenance.md)* |

Three independent design passes (standards-aligned, KG-pragmatic, coverage-maximalist) were
run over this prior art and **converged**: 14–18 nodes, 18–24 edges, umbrella `Molecule`,
provenance triplet on edges, statistics as edge attributes. BioOKF v0.1 is the reconciliation.

## 3. The node universe — derivation

### 3.1 Anchor to the 15 UMLS Semantic Groups, then apply four deltas

| UMLS Semantic Group | BioOKF node type(s) | Delta |
|---|---|---|
| ANAT (Anatomy) | `Anatomy`, `CellType` | split cells out (single-cell era) |
| CHEM (Chemicals & Drugs) | `Molecule` (+ `Gene` promoted out) | **umbrella**; promote Gene |
| GENE (Genes & Sequences) | `Gene`, `Variant` | **split** variant from gene |
| DISO (Disorders) | `Disease`, `Phenotype` | split disorder vs finding/symptom |
| PHYS + PHEN (Physiology, Phenomena) | `BiologicalProcess` | **merge** into one process umbrella |
| PROC (Procedures) | `Procedure`, `ClinicalMeasure` | split out lab/measurement readouts |
| DEVI + OBJC (Devices, Objects) | `Device` | merge |
| ACTI (Activities & Behaviors) | `ExposureOrFactor`; (process side → `BiologicalProcess`) | |
| LIVB (Living Beings) | `Organism`; (individual person → `Agent`) | |
| GEOG (Geographic Areas) | `ExposureOrFactor` (`factor_kind: geographic`) | fold |
| ORGA (Organizations) | `Agent` | fold |
| OCCU (Occupations) | `Agent`/`ExposureOrFactor` | fold |
| CONC (Concepts & Ideas) | `Concept`, plus provenance types | split |
| (unpartitioned 0.5%) | `Other` | closure |

**The four deltas, justified:**
1. **Promote `Gene` out of CHEM; split `Variant` out of GENE.** Gene is the single
   most-connected node in every downstream KG; `encodes` (Gene→product) and variant→trait
   are the highest-frequency edges in the corpus. Folding them into `Molecule` would make
   the dominant edge classes inexpressible at the type level. (Biolink, SPOKE, Hetionet,
   PrimeKG, PubTator3 all keep Gene and Variant first-class.)
2. **`Molecule` as the broad umbrella.** Everything else chemically defined — protein,
   peptide, antibody, enzyme, complex, family/domain, small molecule, drug, metabolite, ion,
   cofactor, nutrient, pharmacologic class — is one type with a `molecule_kind` attribute.
   This is the brief's explicit example ("all pharmaceutical compounds… can all be called
   molecules"). ~10 UMLS/Biolink types collapse to one.
3. **`BiologicalProcess` merges PHYS+PHEN+process-side-of-ACTI** (GO BP/MF, pathways,
   reactions, physiologic/pathologic processes, behaviors). For an LLM wiki the useful split
   is "process/function" vs "disorder," not UMLS's three separate groups.
4. **Add a provenance/context family.** UMLS has no clean home for "the paper / trial /
   dataset / method / author this came from," yet BioOKF *must* (it ingests tweets and bench
   notes). So `Publication`, `Study`, `Dataset`, `Method`, `Agent` are first-class — every
   KG needs `provided_by`/`publications` to point somewhere.

### 3.2 The result: 20 types, ≈7× compression

13 biomedical-entity types + 7 provenance/context types. UMLS's 127 semantic types compress
~7×; Biolink's ~247 categories ~12×. Every node type of SPOKE, Hetionet, PrimeKG, Biolink
core, and PubTator3 maps onto exactly one BioOKF type — so the set is **jointly exhaustive**
— and `*_kind` carries sub-granularity inside a type so types stay **mutually exclusive**.

### 3.3 The "type by identity, not role" rule
This is what keeps the set from exploding. A biomarker that is a protein is a `Molecule`
(identity), and its diagnostic use is a `measures` edge (role). Aspirin is a `Molecule`; its
"drug" status is `molecule_kind`, and "treats headache" is an edge. Without this rule, every
role (biomarker, drug target, risk factor, treatment) would tempt a new type.

## 4. The edge universe — derivation

### 4.1 Backbone: the 5 UMLS super-relation families
Every UMLS relation (54 of them) and every SemMedDB predicate (~30) nests under one of:
`physically_related_to`, `spatially_related_to`, `temporally_related_to`,
`functionally_related_to`, `conceptually_related_to`. BioOKF's 23 predicates are the
**salient leaves** of these families, chosen by frequency-in-corpus and assignability.

### 4.2 Qualify, don't multiply — the key lever
Other schemas explode the edge count by encoding *attributes* as separate edge types.
BioOKF folds them back into attributes:

| Distinction other schemas mint as edges | BioOKF handles via attribute |
|---|---|
| up-regulates vs down-regulates (Hetionet AuG/AdG/CuG/CdG…, SPOKE UP/DOWNREGULATES) | `regulates` + `direction: increased\|decreased` |
| binding affinity vs catalytic activity | `effect_metric: Kd\|Ki\|IC50\|EC50` on `binds`/`regulates` |
| negative claims (every SemMedDB `NEG_` dual) | `negated: true` |
| temporal order / co-occurrence (`precedes`, `co-occurs_with`) | `associated_with` + `timepoint`/qualifier |
| per-datasource edge identity (Open Targets' 128 edges) | `primary_source` / `aggregator_source` |
| species / tissue / sex / age context | `qualifiers` map |

This single lever is why 23 predicates absorb Hetionet's 24 metaedges, SemMedDB's ~30
predicates, and PubTator3's 13 relations with **no remainder**, while staying small enough
that an LLM assigns them reliably from a tweet.

### 4.3 The 23, and why each earns its place
- **Structural (4):** `is_a`, `part_of`, `member_of`, `derives_from` — the ontology backbone
  + material/data lineage (sample←donor, dataset←study). Needed for query closure and for
  protocols/datasets.
- **Spatial (2):** `located_in`, `expressed_in` — anatomy of disease/process + gene
  expression (the single most common omics edge).
- **Molecular core (7):** `encodes`, `interacts_with`, `binds`, `regulates`, `catalyzes`,
  `converts_to`, `participates_in` — central dogma, PPI/DDI, drug-target binding, signed
  regulation, enzymology, metabolism, pathway membership. Covers molecular & cell biology,
  biochemistry, chemical biology.
- **Clinical/causal (7):** `causes`, `predisposes_to`, `treats`, `prevents`,
  `contraindicated_in`, `affects_response_to`, `has_phenotype` — the clinical and
  pharmacology core. `predisposes_to` is kept distinct from `causes` because epidemiology
  fundamentally distinguishes *risk factor* from *cause* (and the OR/HR/RR lives here).
  Adverse events fold into `causes`/`has_phenotype`. Pharmacogenomics gets its own
  `affects_response_to`.
- **Measurement/association/provenance (3):** `measures` (incl. diagnoses), `associated_with`
  (the quantitative statistical umbrella — GWAS, eQTL, correlation, comorbidity), and
  `reported_in` (the universal provenance edge that makes heterogeneous sources tractable).

### 4.4 The corpus check
Across the 12 source classes in [../sources/](../sources/), the recurring relational pattern
is *subject → quantified, attribute-rich claim → object*. Epidemiology's *population →
exposure → outcome, quantified by OR/HR/RR + CI* is `predisposes_to`/`associated_with` with
the statistical bundle. GWAS rows are `associated_with` (+ beta, p-value, ancestry). Drug
mechanism is `binds`/`regulates` (+ IC50). Trials are `treats` (+ RR, sample size). All 23
predicates are exercised by the corpus; none is vestigial; nothing in the corpus fell
outside them.

## 5. Required vs optional attributes — the discipline

Following Biolink (which makes *only* `knowledge_level` + `agent_type` mandatory on an
association), BioOKF keeps the required set tiny:
- **Node:** `type`, `title`, `id` (+ a few required `*_kind`).
- **Edge:** `predicate`, `object`, `knowledge_level`, `agent_type`, `primary_source`.

Everything quantitative and contextual is **optional but first-class** (named slots, §7.3 of
the spec). Rationale: a one-line curated edge must not be burdensome, yet a GWAS edge must be
able to carry its full statistics — so the *capacity* is rich, the *requirement* is minimal.
Provenance is the exception: it is mandatory on every edge, because a graph fed by papers
*and* tweets is only usable if you can tell them apart (§8 of the spec).

## 6. The load-bearing decisions (and their alternatives)

1. **Gene/Molecule/Variant distinct** (vs all-under-`Molecule`). Chosen for the gene-centric
   edge classes; alternative noted in [SPEC.md §13](../SPEC.md#13-design-decisions-and-alternatives).
2. **20 nodes** (vs 11 like Hetionet, or 127 like UMLS). The smallest set that covers every
   source-type entity list while staying assignable.
3. **Provenance as nodes** (`Publication`/`Study`/`Dataset`/`Method`/`Agent`) vs as
   edge-attributes-only. Chosen because BioOKF's *raison d'être* is ingesting heterogeneous
   sources — the source must be addressable. (A lean "pure-science" profile MAY drop the
   provenance family and keep only the 13 entity types + `reported_in` to bare CURIEs.)
4. **23 edges via qualify-don't-multiply.** The discipline that keeps the format both
   exhaustive and small.
5. **CURIEs from the Biolink/Bioregistry prefix set** for identity + cross-vocabulary glue
   via `xref`/`UMLS` CUIs — so BioOKF round-trips to SPOKE and Biolink for free.

## 7. What a future v0.2 might revisit
- Whether `predisposes_to` should merge into `causes` with a `causal_strength` qualifier.
- Whether `Study` + `Dataset` should merge (`StudyOrDataset`) as one provenance type.
- A controlled `aspect` enum for `regulates` (expression / activity / abundance / stability).
- First-class **edge reification** (giving an edge its own page) for claims that themselves
  need provenance chains and contradiction tracking.
- A registry of `infores:` source identifiers for `primary_source`.
