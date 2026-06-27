# Biolink Model — Taxonomy Reference (NCATS Biomedical Data Translator standard)

> **Source of truth:** `biolink-model.yaml` v**4.4.2** (parsed directly), plus the
> official documentation at <https://biolink.github.io/biolink-model/>.
> Model id / namespace: `https://w3id.org/biolink/vocab/` (CURIE prefix `biolink:`).
> Authored/maintained by the NCATS **Biomedical Data Translator** Consortium;
> expressed in **LinkML**. All names below are taken verbatim from the schema.

## 1. What Biolink is

Biolink Model is a **universal, upper-level schema for biomedical knowledge graphs**.
It standardizes (a) the **categories** of nodes/entities (the `NamedThing` class
hierarchy), (b) the **predicates** that relate them (the `related_to` slot hierarchy),
(c) reified **Association** classes (typed subject–predicate–object statements with
evidence/provenance), and (d) **qualifiers** that add nuance to a core triple without
changing it. It also ranks preferred identifier (CURIE) prefixes per category and maps
its terms to external ontologies (UBERON, GO, MONDO, CHEBI, SO, UMLS, BFO, etc.).

Naming convention: in the YAML, classes are written in `lower case with spaces` and
slots in `snake_case`/`spaced`; when serialized they become `PascalCase` classes
(`biolink:Gene`) and `snake_case` slots/predicates (`biolink:gene_associated_with_condition`).

**Counts in v4.4.2:** 363 classes total (≈247 entity/`named thing`-side + 116
`association`-side), 535 slots (247 predicates under `related_to`, 149 association/edge
slots, 104 node-property slots), 51 mixin classes, 30 enums.

---

## 2. The two top-level branches

The root class is **`entity`** (abstract). It has exactly two direct children:

| Branch | Root class | Meaning |
|--------|-----------|---------|
| **Nodes** | `named thing` (`biolink:NamedThing`) | "a databased entity or concept/class" — every node category descends from here |
| **Edges (reified)** | `association` (`biolink:Association`) | "a typed association between two entities, supported by evidence" — every statement type descends from here |

`entity` carries the universal slots: `id`, `iri`, `category`, `type`, `name`,
`description`, `has attribute`, `deprecated`.

---

## 3. Entity CATEGORIES — the `NamedThing` hierarchy

These are the **node categories**. Indentation = `is_a` subclassing. `[abstract]` =
abstract grouping class. Mixin grouping classes (e.g. `gene or gene product`,
`thing with taxon`, `chemical or drug or treatment`) are listed separately in §3.2.

### 3.1 Full `named thing` tree

```
named thing  (biolink:NamedThing)
├─ activity
│  └─ study
│     └─ clinical trial
├─ administrative entity            [abstract]
│  └─ agent
├─ affinity measurement
├─ attribute
│  ├─ biological sex
│  │  ├─ genotypic sex
│  │  └─ phenotypic sex
│  ├─ chemical role
│  ├─ clinical attribute
│  │  ├─ clinical course
│  │  │  └─ onset
│  │  ├─ clinical measurement
│  │  └─ clinical modifier
│  ├─ organism attribute
│  │  └─ phenotypic quality
│  ├─ severity value
│  ├─ socioeconomic attribute
│  └─ zygosity
├─ biological entity                [abstract]
│  ├─ biological process or activity
│  │  ├─ biological process
│  │  │  ├─ behavior
│  │  │  ├─ pathological process
│  │  │  ├─ pathway
│  │  │  └─ physiological process
│  │  └─ molecular activity
│  ├─ coding sequence
│  ├─ disease or phenotypic feature
│  │  ├─ disease
│  │  └─ phenotypic feature
│  │     ├─ behavioral feature
│  │     └─ clinical finding
│  ├─ exon
│  ├─ gene
│  ├─ gene family
│  ├─ genetic inheritance
│  ├─ genome
│  ├─ genotype
│  ├─ haplotype
│  ├─ macromolecular complex
│  ├─ nucleic acid sequence motif
│  ├─ nucleosome modification
│  ├─ organismal entity            [abstract]
│  │  ├─ anatomical entity
│  │  │  ├─ cell
│  │  │  ├─ cellular component
│  │  │  ├─ gross anatomical structure
│  │  │  └─ pathological anatomical structure
│  │  ├─ bacterium
│  │  ├─ cell line
│  │  ├─ cellular organism
│  │  │  ├─ fungus
│  │  │  ├─ invertebrate
│  │  │  ├─ mammal
│  │  │  │  └─ human
│  │  │  ├─ plant
│  │  │  └─ vertebrate
│  │  ├─ individual organism
│  │  │  └─ case
│  │  ├─ life stage
│  │  ├─ population of individual organisms
│  │  │  └─ study population
│  │  │     └─ cohort
│  │  └─ virus
│  ├─ polypeptide
│  │  └─ protein
│  │     └─ protein isoform
│  ├─ posttranslational modification
│  ├─ protein domain
│  ├─ protein family
│  ├─ reagent targeted gene
│  ├─ regulatory region
│  │  ├─ accessible dna region
│  │  └─ transcription factor binding site
│  ├─ sequence variant
│  │  └─ snv
│  └─ transcript
│     └─ RNA product
│        ├─ RNA product isoform
│        └─ noncoding RNA product
│           ├─ microRNA
│           └─ siRNA
├─ chemical entity
│  ├─ chemical mixture
│  │  ├─ complex molecular mixture
│  │  ├─ food
│  │  ├─ molecular mixture
│  │  │  └─ drug
│  │  └─ processed material
│  ├─ environmental food contaminant
│  ├─ food additive
│  └─ molecular entity
│     ├─ nucleic acid entity
│     └─ small molecule
├─ clinical entity
│  └─ clinical intervention
│     └─ hospitalization
├─ device
├─ diagnostic aid
├─ event
├─ evidence type
├─ exposure event
│  ├─ behavioral exposure
│  ├─ biotic exposure
│  ├─ chemical exposure
│  │  └─ drug exposure
│  │     └─ drug to gene interaction exposure
│  ├─ complex chemical exposure
│  ├─ disease or phenotypic feature exposure
│  ├─ environmental exposure
│  │  └─ geographic exposure
│  ├─ genomic background exposure
│  ├─ pathological anatomical exposure
│  ├─ pathological process exposure
│  ├─ socioeconomic exposure
│  └─ treatment
├─ information content entity        [abstract]
│  ├─ common data element
│  ├─ confidence level
│  ├─ dataset
│  ├─ dataset distribution
│  ├─ dataset summary
│  ├─ dataset version
│  ├─ evidence
│  ├─ publication
│  │  ├─ article
│  │  │  └─ journal article
│  │  ├─ book
│  │  ├─ book chapter
│  │  ├─ drug label
│  │  ├─ patent
│  │  ├─ preprint publication
│  │  ├─ serial
│  │  └─ web page
│  ├─ retrieval source
│  └─ study variable
├─ organism taxon
├─ phenomenon
├─ physical entity
│  └─ material sample
├─ planetary entity
│  ├─ environmental feature
│  ├─ environmental process
│  └─ geographic location
│     └─ geographic location at time
├─ procedure
└─ study result                      [abstract]
   ├─ chi squared analysis result
   ├─ concept count analysis result
   ├─ icees study result
   ├─ log odds analysis result
   ├─ observed expected frequency analysis result
   ├─ relative frequency analysis result
   └─ text mining study result
```

### 3.2 Key categories at a glance (the "core" node types)

| Category | CURIE | Typical id prefixes | Notes |
|----------|-------|---------------------|-------|
| `gene` | `biolink:Gene` | NCBIGene, HGNC, ENSEMBL, MGI, ZFIN | descends biological entity; mixes in `gene or gene product`, `genomic entity`, `physical essence`, `ontology class` |
| `protein` | `biolink:Protein` | UniProtKB, PR, ENSEMBL | `is_a` polypeptide; mixes in `gene or gene product` |
| `protein isoform` | `biolink:ProteinIsoform` | UniProtKB | |
| `transcript` / `RNA product` | `biolink:Transcript` / `biolink:RNAProduct` | ENSEMBL, RNAcentral | mRNA, ncRNA, microRNA, siRNA |
| `sequence variant` | `biolink:SequenceVariant` | CAID, DBSNP, CLINVAR, HGVS, SPDI | `snv` is a subtype |
| `genotype` / `haplotype` | `biolink:Genotype` / `biolink:Haplotype` | ZFIN, FB | |
| `chemical entity` | `biolink:ChemicalEntity` | CHEBI, PUBCHEM.COMPOUND, CHEMBL.COMPOUND, DRUGBANK, UNII | |
| `small molecule` | `biolink:SmallMolecule` | CHEBI, PUBCHEM.COMPOUND, DRUGBANK | `is_a` molecular entity |
| `drug` | `biolink:Drug` | RXCUI, NDC, DRUGBANK, CHEMBL.COMPOUND | `is_a` molecular mixture (mixes in `chemical or drug or treatment`) |
| `molecular mixture` / `chemical mixture` | `biolink:MolecularMixture` / `biolink:ChemicalMixture` | | |
| `disease` | `biolink:Disease` | MONDO, DOID, OMIM, ORPHANET, MESH, HP, ICD10/11, NCIT | `is_a` disease or phenotypic feature |
| `phenotypic feature` | `biolink:PhenotypicFeature` | HP, MP, EFO, NCIT, SNOMED | clinical finding, behavioral feature are subtypes |
| `clinical finding` | `biolink:ClinicalFinding` | LOINC, NCIT, SNOMED | |
| `anatomical entity` | `biolink:AnatomicalEntity` | UBERON, CL, GO, MESH, NCIT | |
| `cell` / `cellular component` | `biolink:Cell` / `biolink:CellularComponent` | CL / GO | |
| `gross anatomical structure` | `biolink:GrossAnatomicalStructure` | UBERON | |
| `biological process` | `biolink:BiologicalProcess` | GO, REACT, KEGG | `is_a` biological process or activity |
| `pathway` | `biolink:Pathway` | GO, REACT, KEGG, PHARMGKB, SMPDB, WIKIPATHWAYS | `is_a` biological process |
| `molecular activity` | `biolink:MolecularActivity` | GO, REACT, RHEA, EC | `is_a` biological process or activity |
| `physiological process` / `pathological process` | `biolink:PhysiologicalProcess` / `biolink:PathologicalProcess` | GO, REACT | |
| `organism taxon` | `biolink:OrganismTaxon` | NCBITaxon | |
| `cellular organism`, `human`, `mammal`, `vertebrate`, `virus`, `bacterium`, `fungus`, `plant`, `invertebrate` | `biolink:Human` etc. | NCBITaxon | individual-organism taxa subtypes |
| `cell line` | `biolink:CellLine` | CLO, Cellosaurus | `is_a` organismal entity |
| `case` / `individual organism` | `biolink:Case` / `biolink:IndividualOrganism` | | a patient/sample organism |
| `cohort` / `study population` | `biolink:Cohort` | | |
| `gene family` / `protein family` / `protein domain` | `biolink:GeneFamily` etc. | PANTHER, PFAM, INTERPRO | |
| `macromolecular complex` | `biolink:MacromolecularComplex` | ComplexPortal, GO | |
| `exposure event` (+ subtypes) | `biolink:ExposureEvent` | ECTO | environmental/chemical/drug/behavioral exposures |
| `treatment` | `biolink:Treatment` | | `is_a` exposure event; mixes in `chemical or drug or treatment` |
| `procedure` | `biolink:Procedure` | CPT, NCIT, SNOMED | |
| `device` / `diagnostic aid` | `biolink:Device` / `biolink:DiagnosticAid` | | |
| `clinical intervention` / `clinical entity` | `biolink:ClinicalIntervention` | | |
| `publication` / `article` / `journal article` | `biolink:Publication` / `biolink:Article` | PMID, DOI, PMC, ISBN | `is_a` information content entity |
| `dataset` / `evidence` / `confidence level` | `biolink:Dataset` etc. | | |
| `agent` | `biolink:Agent` | ORCID, GitHub, isbn-publisher | a person/organization/group |
| `geographic location` / `environmental feature` | `biolink:GeographicLocation` | GEO, NCIT | `is_a` planetary entity |
| `material sample` | `biolink:MaterialSample` | BIOSAMPLE | `is_a` physical entity |
| `attribute` (+ `clinical attribute`, `biological sex`, `zygosity`, `severity value`, `socioeconomic attribute`, `phenotypic quality`) | `biolink:Attribute` etc. | | quality/qualifier value nodes |

### 3.3 Important MIXIN grouping classes (not in the `is_a` tree, but used as category groupings / domains-ranges)

Biolink uses 51 **mixin** classes to group otherwise-distinct categories. The
"category" ones most relevant to typing nodes and to predicate domains/ranges:

| Mixin | Groups |
|-------|--------|
| `gene or gene product` | gene + gene product (protein/RNA) |
| `gene or gene product or gene family` | + gene family |
| `chemical entity or gene or gene product` | chemicals + genes/products (e.g. biomarkers) |
| `chemical entity or protein or polypeptide` | |
| `chemical or drug or treatment` | chemical entity + drug + treatment (subject of `treats`) |
| `thing with taxon` | anything that has an `in taxon` slot |
| `genomic entity` | gene, transcript, exon, variant, genome, etc. |
| `physical essence` / `occurrent` / `physical essence or occurrent` | BFO continuant vs occurrent groupings |
| `ontology class` | things that are also ontology terms |
| `macromolecular machine mixin` | gene, gene product, or complex (functional annotation subject) |
| `gene product mixin` / `gene product isoform mixin` | protein/RNA products of genes |
| `gene grouping mixin` | gene family/grouping |
| `pathological entity mixin` | pathological process/anatomical structure |
| `epigenomic entity` | |
| `subject of investigation` | entities that can be studied (case, material sample) |
| `outcome` | study/clinical outcomes |
| Quantifier mixins | `relationship quantifier`, `sensitivity quantifier`, `specificity quantifier`, `frequency quantifier`, `pathognomonicity quantifier` |

---

## 4. PREDICATES — the `related_to` slot hierarchy (247 predicates)

Every predicate descends from **`related to`** (`biolink:related_to`, domain/range
`named thing`, symmetric). The first split is conceptual vs instance level:

- **`related to at concept level`** (symmetric) — ontology/lexical relations:
  `subclass of`, `superclass of`, `close match`, `exact match`, `broad match`,
  `narrow match`, `same as`, `member of`, `has member`, `has chemical role`, …
- **`related to at instance level`** (symmetric) — everything biological/causal.

### 4.1 Full predicate tree (under `related to`)

`(sym)` marks predicates declared `symmetric: true`.

```
related to  (domain=named thing, range=named thing, sym)
├─ composed primarily of · primarily composed of
├─ disease has location · location of disease
├─ related to at concept level (sym)
│  ├─ broad match · narrow match · close match (sym) ─ exact match (sym) ─ same as (sym)
│  ├─ subclass of · superclass of
│  ├─ member of · has member
│  └─ has chemical role · is chemical role of
└─ related to at instance level (sym)
   ├─ active in
   ├─ acts upstream of
   │  ├─ acts upstream of negative effect / positive effect
   │  └─ acts upstream of or within ( + negative/positive effect )
   ├─ affected by
   │  ├─ adverse event of · is side effect of
   │  ├─ condition ameliorated/exacerbated/prevented by
   │  ├─ disrupted by · regulated by
   ├─ affects                                   ← canonical broad causal predicate
   │  ├─ ameliorates condition · exacerbates condition
   │  ├─ disrupts · regulates
   │  └─ has adverse event · has side effect
   ├─ affects likelihood of
   │  ├─ predisposes to condition · promotes condition · preventative for condition
   ├─ affects sensitivity to ( increases/decreases sensitivity to )
   ├─ amount or activity increased/decreased by
   ├─ increases/decreases amount or activity of
   ├─ applied to treat
   ├─ associated with (sym)
   │  ├─ associated with likelihood of ( increased / decreased )
   │  ├─ associated with response to ( resistance to · sensitivity to )
   │  ├─ correlated with (sym)
   │  │  ├─ biomarker for · has biomarker
   │  │  ├─ coexpressed with (sym)
   │  │  ├─ positively / negatively correlated with (sym)
   │  │  └─ occurs together in literature with (sym)
   │  ├─ genetic association (sym)
   │  ├─ genetically associated with (sym)
   │  │  ├─ gene associated with condition
   │  │  └─ condition associated with gene
   │  ├─ likelihood associated with ( increased / decreased )
   │  ├─ resistance associated with · sensitivity associated with · response associated with
   ├─ coexists with (sym)
   │  ├─ colocalizes with (sym) · in cell population with (sym)
   │  └─ in complex with (sym) · in pathway with (sym)
   ├─ contraindicated in · has contraindication
   ├─ contributes to ─ causes                   ← causes is_a contributes to
   ├─ contribution from ─ caused by
   ├─ contributor ( author · editor · provider · publisher )
   ├─ has contributor ( has author · has editor · has provider · has publisher )
   ├─ derives from ─ is metabolite of · derives into ─ has metabolite
   ├─ develops from · develops into
   ├─ diagnoses · is diagnosed by
   ├─ disease has basis in
   ├─ gene product of · has gene product
   ├─ has active component · has active ingredient (see overlaps→has part)
   ├─ has molecular consequence · is molecular consequence of
   ├─ has manifestation · manifestation of ( has/of mode of inheritance )
   ├─ has participant
   │  ├─ actively involves ─ can be carried out by
   │  ├─ enabled by · has catalyst
   │  ├─ has input ─ consumes · has output · has substrate
   ├─ participates in
   │  ├─ actively involved in ─ capable of
   │  ├─ catalyzes · enables · is input of ─ consumed by · is output of · is substrate of
   ├─ has phenotype · phenotype of
   ├─ has sequence location · sequence location of
   ├─ has sequence variant · is sequence variant of
   │  └─ has/is (frameshift / missense / nearby / non coding / nonsense / splice site / synonymous) variant
   ├─ has target · target for
   ├─ has upstream actor ( + negative/positive/upstream-or-within actor )
   ├─ in linkage disequilibrium with (sym)
   ├─ in taxon · taxon of
   ├─ interacts with (sym)                       ← canonical broad interaction predicate
   │  ├─ genetically interacts with (sym) ( gene_fusion_with · genetic_neighborhood_of )
   │  ├─ pharmacologically interacts with (sym)
   │  └─ physically interacts with (sym)
   │     ├─ directly physically interacts with (sym) ─ binds (sym)
   │     └─ indirectly physically interacts with (sym)
   ├─ likelihood affected by ( condition predisposed/promoted by )
   ├─ located in ─ expressed in · location of ─ expresses
   ├─ mentions · mentioned by
   ├─ missing from · lacks part
   ├─ model of · models
   ├─ occurs in · occurs in disease
   ├─ opposite of (sym)
   ├─ overlaps (sym)
   │  ├─ has part ( has active ingredient · has excipient · has food component ─ has nutrient ·
   │  │            has plasma membrane part · has variant part )
   │  └─ part of ( food component of ─ nutrient of · is active ingredient of · is excipient of ·
   │              plasma membrane part of · variant part of )
   ├─ preceded by · precedes · temporally related to (sym)
   ├─ produces · produced by
   ├─ related condition (sym)
   ├─ sensitivity affected by ( sensitivity decreased by ) · sensitivity increased by
   ├─ similar to (sym)
   │  ├─ chemically similar to (sym)
   │  └─ homologous to (sym) ─ orthologous to (sym) · paralogous to (sym) · xenologous to (sym)
   ├─ studied to treat ( in clinical trials for · in preclinical trials for ─ beneficial in models for )
   ├─ subject of treatment application or study for treatment by
   │  └─ treated by ─ treated in studies by ( tested by clinical/preclinical trials of … )
   ├─ transcribed from · transcribed to · translates to · translation of
   ├─ treatment applications from
   ├─ treats or applied or studied to treat ─ treats   ← treats is_a treats-or-studied
   └─ was tested for effect of · was tested for effect on
```

### 4.2 The most-used predicates (canonical "core" set), with domain → range

| Predicate (CURIE) | Domain → Range | Symmetric | Meaning |
|-------------------|----------------|-----------|---------|
| `biolink:related_to` | named thing → named thing | yes | top-level catch-all |
| `biolink:affects` | (broad) | no | A affects the abundance/activity/expression… of B (use qualifiers) |
| `biolink:regulates` | physical essence or occurrent → same | no | `is_a` affects; A regulates B |
| `biolink:disrupts` | (broad) | no | `is_a` affects |
| `biolink:causes` | (broad) | no | `is_a` contributes to — A causally produces B |
| `biolink:contributes_to` | (broad) | no | partial causal contribution |
| `biolink:treats` | chemical or drug or treatment → disease or phenotypic feature | no | intervention ameliorates/stabilizes/cures a condition |
| `biolink:treats_or_applied_or_studied_to_treat` | chemical or drug or treatment → disease or phenotypic feature | no | superproperty of `treats` (asserted/used/studied) |
| `biolink:ameliorates_condition` / `biolink:exacerbates_condition` | chemical or drug or treatment → disease or phenotypic feature | no | sub-effects of treatment |
| `biolink:contraindicated_in` | chemical or drug or treatment → biological entity | no | |
| `biolink:has_adverse_event` / `biolink:has_side_effect` | (broad) | no | |
| `biolink:interacts_with` | named thing → named thing | yes | any direct/indirect interaction |
| `biolink:physically_interacts_with` | | yes | physical interaction |
| `biolink:directly_physically_interacts_with` / `biolink:binds` | | yes | binding |
| `biolink:gene_associated_with_condition` | gene → disease or phenotypic feature | no | gene–disease genetic association |
| `biolink:condition_associated_with_gene` | (inverse) | no | |
| `biolink:genetically_associated_with` | | yes | |
| `biolink:associated_with` | named thing → named thing | yes | statistical/observed association |
| `biolink:correlated_with` | named thing → named thing | yes | |
| `biolink:positively_correlated_with` / `biolink:negatively_correlated_with` | | yes | |
| `biolink:biomarker_for` | chemical entity or gene or gene product → disease or phenotypic feature | no | measurable indicator of condition |
| `biolink:has_biomarker` | (inverse) | no | |
| `biolink:coexpressed_with` | gene or gene product → gene or gene product | yes | |
| `biolink:expressed_in` | gene or gene product → anatomical entity | no | `is_a` located in |
| `biolink:expresses` | (inverse: anatomical entity → gene or gene product) | no | |
| `biolink:located_in` | named thing → named thing | no | |
| `biolink:has_phenotype` | biological entity → phenotypic feature | no | |
| `biolink:phenotype_of` | (inverse) | no | |
| `biolink:has_participant` / `biolink:participates_in` | bio process/activity ↔ occurrent | no | |
| `biolink:enables` / `biolink:enabled_by` | gene product ↔ molecular activity | no | GO functional |
| `biolink:catalyzes` / `biolink:has_catalyst` | | no | reaction catalyst |
| `biolink:capable_of` / `biolink:actively_involved_in` | macromolecular machine → process | no | GO functional |
| `biolink:active_in` | gene product → cellular component | no | GO cellular component |
| `biolink:acts_upstream_of` ( + `_or_within`, `_positive_effect`, `_negative_effect`) | gene → process | no | GO causal |
| `biolink:gene_product_of` / `biolink:has_gene_product` | gene ↔ gene product | no | |
| `biolink:transcribed_to` / `biolink:transcribed_from` / `biolink:translates_to` / `biolink:translation_of` | gene → transcript → protein | no | central dogma |
| `biolink:has_sequence_variant` / `biolink:is_sequence_variant_of` | gene ↔ sequence variant | no | |
| `biolink:in_taxon` | thing with taxon → organism taxon | no | species attribution |
| `biolink:subclass_of` / `biolink:superclass_of` | ontology class ↔ ontology class | no | ontology hierarchy |
| `biolink:close_match` / `biolink:exact_match` / `biolink:same_as` | concept ↔ concept | yes | identifier equivalence (SKOS-like) |
| `biolink:has_part` / `biolink:part_of` | (overlaps) | no | mereology (+ `has_active_ingredient`, `has_excipient`, `has_nutrient`, …) |
| `biolink:derives_from` / `biolink:derives_into` / `biolink:has_metabolite` / `biolink:is_metabolite_of` | molecular entity ↔ molecular entity | no | derivation/metabolism |
| `biolink:homologous_to` / `biolink:orthologous_to` / `biolink:paralogous_to` / `biolink:xenologous_to` | gene/product ↔ gene/product | yes | homology |
| `biolink:similar_to` / `biolink:chemically_similar_to` | | yes | |
| `biolink:model_of` / `biolink:models` | | no | disease/organism models |
| `biolink:occurs_in_disease` | | no | |
| `biolink:diagnoses` / `biolink:is_diagnosed_by` | | no | |
| `biolink:coexists_with` / `biolink:colocalizes_with` / `biolink:in_complex_with` / `biolink:in_pathway_with` / `biolink:in_cell_population_with` | | yes | co-occurrence |

> **Note on inverses & symmetry.** Many predicates come as inverse pairs
> (`has_X` / `X_of`, `treats` / `treated_by`, `located_in` / `location_of`). Translator
> often normalizes edges to a single **canonical direction** per pair, recording the
> author's original direction in `original_predicate`. Symmetric predicates
> (`interacts_with`, `associated_with`, `correlated_with`, the `*_match` family,
> `homologous_to`, etc.) have no distinct inverse.

---

## 5. ASSOCIATION classes (reified edges, 116 classes)

`association` (`biolink:Association`) is the reified-edge base: a typed
subject–predicate–object statement plus evidence/provenance. Specialized subclasses
constrain the allowed subject/object categories and predicates and add domain-specific
qualifier slots. Selected (the hierarchy is broad; see §5.1 for the standard slots that
apply to *all* of them):

```
association
├─ gene to disease association
│  ├─ causal gene to disease association  (and correlated gene to disease association)
│  ├─ druggable gene to disease association
│  ├─ gene as a model of disease association
│  └─ gene has variant that contributes to disease association
├─ variant to disease association ─ variant as a model of disease association
├─ variant to gene association ─ variant to gene expression association
├─ gene to phenotypic feature association · disease to phenotypic feature association
├─ chemical to disease or phenotypic feature association (chemical or drug or treatment …)
├─ chemical affects gene association · gene affects chemical association
├─ chemical gene interaction association · drug to gene association
├─ chemical entity or gene or gene product regulates gene association
├─ gene regulates gene association · gene to gene (homology / coexpression / interaction)
├─ functional association ( gene to go term · macromolecular machine to
│                            biological process / cellular component / molecular activity )
├─ chemical or drug or treatment to disease or phenotypic feature association
├─ chemical or drug or treatment side effect / adverse event association
├─ exposure event to outcome / phenotypic feature association
├─ sequence variant modulates treatment association
├─ genotype to (disease / gene / phenotypic feature / variant) association
├─ anatomical entity to anatomical entity (part of / has part / ontogenic) association
├─ organism taxon to organism taxon (interaction / specialization) association
├─ contributor association · information content entity to named thing association
└─ … (case-to-*, cell-line-to-*, material-sample-to-*, sequence/genomic-localization, etc.)
```

---

## 6. EDGE PROPERTIES / association slots (149 slots)

These are the slots an `Association` (edge) may carry. **Required** ones are flagged.

### 6.1 The core triple + required edge slots

| Slot | CURIE | Required? | Range | Notes |
|------|-------|-----------|-------|-------|
| `subject` | `biolink:subject` | **REQUIRED** | named thing | the source node |
| `predicate` | `biolink:predicate` | **REQUIRED** | predicate type | from `related_to` hierarchy |
| `object` | `biolink:object` | **REQUIRED** | named thing | the target node |
| `knowledge_level` | `biolink:knowledge_level` | **REQUIRED** | `KnowledgeLevelEnum` | how the statement was derived |
| `agent_type` | `biolink:agent_type` | **REQUIRED** | `AgentTypeEnum` | what kind of agent produced it |

> Per the Association class docs, these **5 slots are mandatory (cardinality 1)**.
> `knowledge_level` and `agent_type` became required to make provenance machine-checkable.

### 6.2 Provenance / evidence / confidence

| Slot | Range / values |
|------|----------------|
| `knowledge_source`, `primary_knowledge_source`, `aggregator_knowledge_source` | InfoRes identifier (information resource registry) |
| `original_subject`, `original_predicate`, `original_object` | as-asserted values before normalization |
| `publications` | publication CURIEs (PMID/DOI/…) |
| `sources` | `retrieval source` objects (provenance chain) |
| `has_evidence`, `has_evidence_of_type`, `evidence_count` | evidence type CURIEs/counts |
| `supporting_text`, `supporting_text_section_type`, `subject/object_location_in_text` | text-mining support |
| `supporting_documents`, `supporting_document_type/year` | |
| `has_supporting_studies`, `supporting_study_metadata` (cohort, context, date range, method, size) | |
| `has_confidence_level`, `has_confidence_score`, `extraction_confidence_score` | |
| `negated` | boolean — the statement is asserted to be false |
| `retrieval_source_ids`, `support_graphs`, `elevate_to_prediction` | |

### 6.3 Statistical / quantitative edge slots

`p_value`, `adjusted_p_value`, `bonferonni_adjusted_p_value`, `log_odds_ratio`,
`log_odds_ratio_95_ci`, `ln_ratio`, `ln_ratio_confidence_interval`,
`relative_frequency_subject/object` (+ confidence intervals),
`fisher_exact_p`, `fisher_exact_odds_ratio`, `chi_squared_statistic`, `chi_squared_p`,
`chi_squared_dof`, `z_score`, `concept_count_subject/object`, `concept_pair_count`,
`expected_count`, `observed` counts, `total_sample_size`, `dataset_count`,
`semmed_agreement_count`, `has_study_results`.

### 6.4 Denormalized / closure helper slots (graph-indexing convenience)

`subject_category`, `object_category`, `subject_closure`, `object_closure`,
`subject_category_closure`, `object_category_closure`, `subject_namespace`,
`object_namespace`, `subject_label_closure`, `object_label_closure`,
`subject_feature_name`, `object_feature_name`.

---

## 7. QUALIFIERS — the standard qualifier slots

Qualifiers add nuance to a **core triple while keeping it true**. They are themselves
association slots (all descend from `biolink:qualifier`). The pattern is:
**broad predicate (`affects`/`regulates`) + aspect + direction qualifiers** rather than
hundreds of fine-grained predicates.

### 7.1 Subject/object aspect & direction (the workhorse pair)

| Qualifier | Range | Purpose |
|-----------|-------|---------|
| `subject_aspect_qualifier` / `object_aspect_qualifier` | `GeneOrGeneProductOrChemicalEntityAspectEnum` | *what* is affected (abundance, activity, expression, synthesis, degradation, localization, transport, secretion, splicing, stability, mutation_rate, …; 57 values) |
| `subject_direction_qualifier` / `object_direction_qualifier` | `DirectionQualifierEnum` | *how*: `increased`, `upregulated`, `decreased`, `downregulated` |
| `subject_form_or_variant_qualifier` / `object_form_or_variant_qualifier` | `ChemicalOrGeneOrGeneProductFormOrVariantEnum` | genetic_variant_form, loss_of_function_variant_form, gain_of_function_variant_form, mutant_form, snp_form, modified_form, … |
| `subject_part_qualifier` / `object_part_qualifier` | `GeneOrGeneProductOrChemicalPartQualifierEnum` | 3_prime_utr, 5_prime_utr, promoter, enhancer, exon, intron, polya_tail |
| `subject_derivative_qualifier` / `object_derivative_qualifier` | `ChemicalEntityDerivativeEnum` | e.g. `metabolite` |
| `subject_specialization_qualifier` / `object_specialization_qualifier` | category | narrows the node's category for this edge |
| `subject_context_qualifier` / `object_context_qualifier` | named thing | contextual entity |
| `subject_process_qualifier` / `object_process_qualifier` | biological process | |
| `subject_activity_qualifier` / `object_activity_qualifier` | molecular activity | |

### 7.2 Predicate-level / statement qualifiers

| Qualifier | Range / values |
|-----------|----------------|
| `qualified_predicate` | a predicate (typically `biolink:causes`) — the *true* relationship once aspect/direction are applied |
| `causal_mechanism_qualifier` | `CausalMechanismQualifierEnum` (~100 values: agonism, antagonism, inhibition, activation, phosphorylation, methylation, ubiquitination, … ) |
| `anatomical_context_qualifier` | anatomical entity (UBERON/CL) — where the statement holds |
| `species_context_qualifier` | organism taxon |
| `disease_context_qualifier` | disease |
| `population_context_qualifier` | population of individual organisms |
| `temporal_context_qualifier`, `temporal_interval_qualifier`, `stage_qualifier` | timing / life stage |
| `context_qualifier` (generic base) | named thing |

### 7.3 Other qualifier slots

`frequency_qualifier`, `onset_qualifier`, `severity_qualifier`, `sex_qualifier`,
`clinical_modifier_qualifier`, `quantifier_qualifier`, `aspect_qualifier` /
`direction_qualifier` / `derivative_qualifier` / `form_or_variant_qualifier` /
`part_qualifier` / `specialization_qualifier` / `process_qualifier` (the generic bases),
`response_context_qualifier`, `response_target_context_qualifier`,
`catalyst_qualifier`, `sequence_variant_qualifier`, `phenotypic_state`.

> Deprecated: `qualifier` and `qualifiers` (free-text). The structured
> aspect/direction/context qualifiers above are the modern replacement.

### 7.4 Worked example (canonical "fully-qualified edge")

> *"Methionine deficiency results in increased expression of ADRB2 in adipose tissue."*

| Slot | Value |
|------|-------|
| `subject` | CHEBI:methionine |
| `predicate` | `biolink:affects` |
| `object` | NCBIGene:ADRB2 |
| `subject_aspect_qualifier` | `abundance` |
| `subject_direction_qualifier` | `decreased` |
| `object_aspect_qualifier` | `expression` |
| `object_direction_qualifier` | `increased` |
| `qualified_predicate` | `biolink:causes` |
| `anatomical_context_qualifier` | UBERON:0001013 (adipose tissue) |

The core triple `Methionine — affects — ADRB2` stays true; the qualifiers carry the
full meaning. (Likewise *"Fenofibrate is an agonist of PPARA"* = core `affects` +
`causal_mechanism_qualifier: agonism`.)

---

## 8. NODE PROPERTIES (104 slots) and required vs optional node slots

### 8.1 Required vs optional NODE slots

| Slot | CURIE | Required? | Notes |
|------|-------|-----------|-------|
| `id` | `biolink:id` | **REQUIRED** (on `entity`, identifier) | a CURIE that uniquely identifies the node |
| `category` | `biolink:category` | **REQUIRED on `named thing`** | one or more Biolink category CURIEs (multivalued) |
| `name` | `biolink:name` | optional (recommended) | human-readable label |
| `iri` | `biolink:iri` | optional | full IRI |
| `description` | `biolink:description` | optional | |
| `type` (`rdf:type`) | `biolink:type` | optional | |
| `xref` | `biolink:xref` | optional | cross-references |
| `provided_by` | `biolink:provided_by` | optional | source InfoRes |
| `synonym` / `full_name` / `has_attribute` | | optional | |
| `has_attribute_type` | `biolink:has_attribute_type` | **REQUIRED on `attribute`** | the kind of attribute |
| `in_taxon` (+ `in_taxon_label`) | `biolink:in_taxon` | optional (on `thing with taxon`) | species |

> **Summary of "required" in Biolink:** globally `required: true` is declared for
> `has_attribute_type`, `id`, `subject`, `object`, `predicate`, `knowledge_level`,
> `agent_type`. `category` is required via `slot_usage` on `named thing`. Everything
> else is optional. So: **a valid node = `id` + `category`; a valid edge =
> `subject` + `predicate` + `object` + `knowledge_level` + `agent_type`.**

### 8.2 Other node-property slots (selected)

`symbol`, `synonym` (+ exact/broad/narrow/related/systematic synonym), `xref`,
`full_name`, `has_biological_sequence`, `has_chemical_formula`, `hgvs_nomenclature`,
`is_metabolite`, `is_toxic`, `is_supplement`, `trade_name`, `max_tolerated_dose`,
`has_taxonomic_rank`, `taxon`, `inheritance`, `mesh_terms`, `keywords`, `authors`,
`published_in`, `iso_abbreviation`, `volume`/`issue`/`pages`/`chapter`,
`creation_date`/`update_date`/`ingest_date`, `license`/`rights`/`format`/`version`,
`latitude`/`longitude`/`address`, `provided_by`, `available_from`, `affiliation`,
`clinical_trial_*` (phase, status, enrollment, conditions, interventions, dates…),
`exposure_*` (route, duration, magnitude, vehicle…), aggregate statistics
(`has_count`, `has_total`, `has_percentage`, `has_quotient`, `number_of_cases`).

---

## 9. Key controlled-vocabulary enums (30 total; the standard ones)

| Enum | Permissible values |
|------|--------------------|
| **`KnowledgeLevelEnum`** (required) | `knowledge_assertion`, `logical_entailment`, `prediction`, `statistical_association`, `text_co_occurrence`, `observation`, `not_provided` |
| **`AgentTypeEnum`** (required) | `manual_agent`, `automated_agent`, `data_analysis_pipeline`, `computational_model`, `text_mining_agent`, `image_processing_agent`, `manual_validation_of_automated_agent`, `not_provided` |
| **`DirectionQualifierEnum`** | `increased`, `upregulated`, `decreased`, `downregulated` |
| **`GeneOrGeneProductOrChemicalEntityAspectEnum`** (57) | `activity_or_abundance`, `abundance`, `activity`, `expression`, `synthesis`, `degradation`, `cleavage`, `hydrolysis`, `metabolic_processing`, `mutation_rate`, `stability`, `folding`, `localization`, `transport`, `absorption`, `aggregation`, `interaction`, `release`, `secretion`, `uptake`, `splicing`, `molecular_modification`, `acetylation`, `acylation`, `alkylation`, `amination`, … |
| **`ChemicalOrGeneOrGeneProductFormOrVariantEnum`** | `genetic_variant_form`, `modified_form`, `loss_of_function_variant_form`, `non_loss_of_function_variant_form`, `gain_of_function_variant_form`, `dominant_negative_variant_form`, `polymorphic_form`, `snp_form`, `mutant_form`, `analog_form` |
| **`GeneOrGeneProductOrChemicalPartQualifierEnum`** | `3_prime_utr`, `5_prime_utr`, `polya_tail`, `promoter`, `enhancer`, `exon`, `intron` |
| **`CausalMechanismQualifierEnum`** (~100) | `agonism`, `antagonism`, `inhibition`, `activation`, `inverse_agonism`, `partial_agonism`, `agonism`, `potentiation`, `suppression`, `phosphorylation`, `methylation`, `ubiquitination`, `acetylation`, … |
| **`ResearchPhaseEnum`** | `pre_clinical_research_phase`, `clinical_trial_phase`, `clinical_trial_phase_1`, `_1_to_2`, `_2`, `_2_to_3`, `_3`, `_4`, `not_provided` |
| **`ClinicalApprovalStatusEnum`** | `approved_for_condition`, `fda_approved_for_condition`, `not_approved_for_condition`, `post_approval_withdrawal`, `off_label_use`, `not_provided` |
| **`ReactionDirectionEnum`** | `left_to_right`, `right_to_left`, `bidirectional`, `neutral` |
| **`ReactionSideEnum`** | `left`, `right` |
| **`LogicalInterpretationEnum`** | `some_some`, `all_some`, `inverse_all_some` |
| **`DrugDeliveryEnum`** | `inhalation`, `oral`, `absorption_through_the_skin`, `injection`, `intravenous_injection`, `subcutaneous_injection`, `intramuscular_injection` |
| **`ChemicalEntityDerivativeEnum`** | `metabolite` |
| **`ResponseEnum`** | `therapeutic_response`, `negative` |
| `ClinicalTrialStatusEnum`, `ClinicalTrialAgeStageEnum`, `BinaryRelationEnum`, `ResponseTargetEnum`, `PhaseEnum`, `StrandEnum`, `SequenceEnum`, `DruggableGeneCategoryEnum`, `DrugAvailabilityEnum`, `ResourceRoleEnum`, `AffinityParameterEnum`, `FDAIDAAdverseEventEnum`, `GeneToPhenotypicFeaturePredicateEnum`, `GeneToDiseasePredicateEnum` | (additional/auxiliary) |

---

## 10. Practical notes for downstream use (Translator conventions)

- **Pin to a release.** Any tool consuming `biolink-model.yaml` must pin to a tagged
  version (this doc = v4.4.2); the model changes frequently and has no semver guarantees.
- **Canonical-form edges.** Translator KPs/ARAs normalize each edge to the canonical
  predicate direction and category, recording the asserted form in `original_*` slots.
- **Categories are multivalued and hierarchical.** A node typically lists its most
  specific category; consumers expand via the `is_a` closure (`subject_category_closure`).
- **Prefer broad predicate + qualifiers** over inventing fine-grained predicates, e.g.
  `affects` + `object_aspect_qualifier=expression` + `object_direction_qualifier=increased`
  + `qualified_predicate=biolink:causes` instead of a single "increases_expression_of".
- **`mixin` classes are not category values per se** but are used as predicate
  domains/ranges and as cross-cutting groupings (e.g. `gene or gene product`).

---

## Sources (primary)

- Biolink Model docs: <https://biolink.github.io/biolink-model/>
- Schema YAML (v4.4.2, parsed): <https://raw.githubusercontent.com/biolink/biolink-model/master/biolink-model.yaml>
- Repository: <https://github.com/biolink/biolink-model>
- Association class: <https://biolink.github.io/biolink-model/Association/>
- Predicate slot: <https://biolink.github.io/biolink-model/predicate/>
- Qualifier examples: <https://biolink.github.io/biolink-model/association-examples-with-qualifiers/>
- FAQ: <https://biolink.github.io/biolink-model/faq/>
- Paper — Unni et al., *Biolink Model: A universal schema for knowledge graphs in clinical,
  biomedical, and translational science*, CPT (2022): <https://arxiv.org/pdf/2203.13906>
