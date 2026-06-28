# BioOKF: Biomedical Open Knowledge Format

**Version 0.5 (Draft) · 2026-06-27**

> A biomedical profile of the [Open Knowledge Format (OKF)](https://github.com/GoogleCloudPlatform/knowledge-catalog).
> BioOKF keeps OKF's portable substrate (a Git-shippable tree of Markdown files with
> YAML frontmatter) and adds the one thing OKF deliberately leaves open: **meaning**.
> It closes the document `type` to a finite, controlled universe of **28 biomedical node
> types**, and it promotes OKF's untyped prose links to a finite, controlled universe of
> **35 typed, attributed edges (24 positive + 11 `not_<X>`)**. The result is a format that an LLM agent or a human curator
> can follow to distill *any* biomedical source (a paper, a preprint, a bench note, a slide
> deck, a CSV, a figure, a tweet) into an interlinked knowledge base that compounds over
> time and can be queried as a graph.
>
> **What changed in v0.5** (full detail in [§14](#14-changelog-and-deprecated-aliases)). v0.5 is a
> **provenance** release over v0.4; the node/edge counts are unchanged (28 / 23) and no entity or
> predicate was added or removed. It completes the v0.4 move off CURIE primary keys by making
> provenance **node-based** instead of CURIE-based:
>
> 1. **An edge's `primary_source` now references a source *node* by its `identifier`**, not an
> `infores:` CURIE. The originating source is one of the bundle's own **source nodes**, a
> `Publication`/`Study`/`Dataset`/`Agent`, traversable like any other concept. Old
> `infores:`-valued `primary_source` is accepted on read and normalized to a source node carrying
> that CURIE in `xref`. The reserved value `not_provided` is a rare escape for genuinely
> unknown-origin claims, never a default.
>
> 2. **`provided_by` is dropped.** A node-level "this page came from X" is now simply a
> `reported_in` edge to the source node, so provenance is carried by exactly two mechanisms:
> per-edge `primary_source` and the `reported_in` edge. (Legacy `provided_by` normalizes to a
> `reported_in` edge.)
>
> 3. **Source nodes gain an optional `raw_source`**: one or more `raw/…` paths anchoring the node
> to the immutable bytes it was distilled from, so every claim's provenance chain terminates in
> `raw/`.
>
> 4. **Two source-node species are made explicit:** *ingested-document* sources (a paper, tweet,
> or dataset you placed in `raw/`) carry `raw_source`; *external-reference* sources (HGNC, GO,
> SemMedDB, DrugBank…) carry no `raw_source` and instead record their `infores:`/ontology CURIE
> in `xref`.
>
>
> **v0.5 builds on v0.4** (a naming + clarity release over v0.3, 28 / 23 unchanged), whose four
> changes were: (1) node type `SDOH` → **`SocialFactor`**; (2) the per-type `*_kind` family
> replaced by a single agent-coined **`subtype`** with no controlled universe; (3) the former
> `title` and `id` merged into one **human-readable, bundle-unique `identifier`** (CURIEs moved to
> optional `xref`); (4) the **§5.D node-boundary review pass**. All four carry forward unchanged;
> see [§14](#14-changelog-and-deprecated-aliases).

---

## Table of contents

1. [Why BioOKF exists](#1-why-biookf-exists)

2. [Relationship to OKF and to the LLM Wiki](#2-relationship-to-okf-and-to-the-llm-wiki)

3. [Bundle structure](#3-bundle-structure)

4. [The concept document](#4-the-concept-document)

5. [The NODE universe (28 types)](#5-the-node-universe-28-types)

6. [The EDGE universe (35 predicates)](#6-the-edge-universe-35-predicates)

7. [Attributes: required vs optional](#7-attributes-required-vs-optional)

8. [Provenance, evidence, and quantitative claims](#8-provenance-evidence-and-quantitative-claims)

9. [Identifiers and namespaces](#9-identifiers-and-namespaces)

10. [The ingest / query / lint workflow](#10-the-ingest--query--lint-workflow)

11. [Conformance](#11-conformance)

12. [Worked example](#12-worked-example)

13. [Design decisions and alternatives](#13-design-decisions-and-alternatives)

14. [Changelog and deprecated aliases](#14-changelog-and-deprecated-aliases)

---

## 1\. Why BioOKF exists

Biomedical knowledge is **fragmented across formats and people**. The same fact,
*"IL6 elevation is associated with worse COVID-19 outcomes"*, lives, partially and
inconsistently, in a peer-reviewed paper, three preprints, a lab's bench notebook, a
conference slide, a supplementary CSV, and a researcher's tweet. Today an LLM rediscovers
those connections on every query (the RAG pattern): it re-reads, re-extracts, and re-reasons
each time, and the work evaporates when the session ends.

BioOKF is a **protocol for distilling that knowledge once and keeping it**: a structured,
interlinked, version-controlled knowledge base that:

- ingests sources of **any modality** (text, PDF, image, dataset, slide, social post) and
any biomedical subfield (epidemiology, genetics, molecular & cell biology, biochemistry,
pharmacology, medicinal & chemical biology, clinical specialties, public health,
microbiology…);

- represents what it learns as **typed nodes** (entities) and **typed edges** (relationships)
drawn from a **finite, controlled vocabulary**, so two BioOKF bundles built by different
people are *interoperable* and *graph-queryable*;

- records **where every claim came from and how strongly it is supported** (provenance +
evidence), so a fact mined from a tweet is never confused with one curated from DrugBank;

- serves as **persistent, compounding memory** for an AI agent: the cross-references are
already there, the contradictions already flagged.

The problem BioOKF solves that plain OKF does not: OKF standardizes *structure* but not
*meaning* (its sharpest published critique, Marc Bara, *"A Standard, or Just a Folder?"*, is
exactly this). For a general knowledge catalog that openness is a feature. For **biomedicine
specifically**, the controlled vocabularies already exist and are mature (UMLS, Biolink,
SPOKE, Hetionet, SemMedDB, GO, MONDO, ChEBI, SO, OBA, EFO, ECTO…), so we can close the
vocabulary *without* losing generality and gain a graph that actually composes.

---

## 2\. Relationship to OKF and to the LLM Wiki

BioOKF is a **strict profile of OKF**: every conformant BioOKF bundle is also a conformant
OKF bundle (Markdown tree, YAML frontmatter, non-empty `type`, optional `index.md` /
`log.md`). BioOKF only **adds constraints** and **adds an edge layer**; it removes nothing.

| Dimension            | OKF v0.1                                            | BioOKF v0.5                                                                                                                                        |
| -------------------- | --------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------- |
| Substrate            | Markdown tree + YAML frontmatter, Git-shippable     | **same**                                                                                                                                           |
| `type` field         | required, **open vocabulary** (any string)          | required, **closed**: one of 28 biomedical node types (§5)                                                                                         |
| `subtype` field      | none                                                | **expected, agent-coined**; open vocabulary with no controlled set (§5)                                                                            |
| Cross-document links | untyped Markdown links; relationship lives in prose | **typed edges** (`edges:` frontmatter) from a closed set of 35 predicates (24 positive + 11 `not_<X>`) (§6); prose links remain legal as advisory "see also"                    |
| Attributes           | only `type` required; others recommended            | per-type **required vs optional** attributes; edges require a provenance triplet (§7-8)                                                            |
| Identifiers          | optional `resource` URI                             | a single human-readable, bundle-unique **`identifier`** (merges name + id; required); equivalent **external** CURIEs optional in `xref` (§7.1, §9) |
| Reserved files       | `index.md`, `log.md`                                | **same**, plus an optional `SCHEMA.md` operating doc                                                                                               |
| Consumer behavior    | MUST tolerate unknown types / broken links          | MUST tolerate broken links; **SHOULD validate** `type`/`predicate` against the universes and flag (not silently accept) unknowns                   |

BioOKF also operationalizes Andrej Karpathy's **LLM Wiki** pattern (the gist OKF itself
cites). Its three layers and three operations map directly onto BioOKF:

- **Raw layer** → `raw/` (immutable sources, never edited).

- **Wiki layer** → `knowledge/` (LLM-authored typed concept documents = the graph).

- **Schema layer** → `SCHEMA.md` (the operating doc telling the agent the BioOKF
conventions and the ingest/query/lint workflows).

- **Operations** → ingest, query, lint (§10), specialized for biomedicine.

What is **universal** (inherited from the LLM Wiki / OKF) vs **special to BioOKF**:

| Universal (any LLM Wiki)                                                          | Special to BioOKF                                                                                                                                                                                                                                                                    |
| --------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| read source → discuss takeaways → write/update pages → update `index.md` → log it | every page is one of **28 typed biomedical nodes**                                                                                                                                                                                                                                   |
| cross-link related pages                                                          | links are **35 typed, attributed edges (24 positive + 11 `not_<X>`)** with domain/range                                                                                                                                                                                                                           |
| `index.md` catalog + `log.md` history                                             | every claim carries **provenance + evidence level**; quantitative claims (p-value, OR/HR, IC50…) are **first-class edge attributes**                                                                                                                                                 |
| periodic lint for contradictions/staleness/orphans                                | lint additionally checks **type/predicate validity, `identifier` uniqueness/human-readability, node-based provenance resolution (`primary_source`→ a source node), and domain/range** (a missing external `xref` CURIE is only an enrichment opportunity, never a conformance error) |

---

## 3\. Bundle structure

A BioOKF bundle is a directory (recommended: a Git repository). Reserved layout:

```
my-kb/
├── SCHEMA.md          # (recommended) the operating doc: conventions + workflows for the agent
├── index.md           # (reserved) progressive-disclosure catalog of all concept pages
├── log.md             # (reserved) newest-first dated change history
├── raw/               # immutable source material (the LLM reads, never edits)
│   ├── pmid-34986598.pdf
│   ├── lab-notebook-2026-03.md
│   ├── figure-kegg-mapk.png
│   └── gwas-supp-table.csv
└── knowledge/         # LLM-authored typed concept documents (the graph)
    ├── molecule/
    │   ├── il6.md
    │   └── tocilizumab.md
    ├── disease/
    │   └── covid-19.md
    ├── study/
    │   └── recovery-trial.md
    └── ...
```

- **Reserved filenames** `index.md` and `log.md` (and the recommended `SCHEMA.md`) are
**not** concept documents and carry no `type`. Every other `*.md` file is a concept.

- **Concept identifier** = each concept's `identifier` frontmatter field (§7.1): a
human-readable name that is **unique across the bundle**. It is the stable handle that
cross-links and `edges:` target. The file path is a physical convenience and SHOULD echo the
identifier (e.g. `knowledge/molecule/interleukin-6.md`). (Type tokens are slash-free, hence
`MethodOrProcedure`, not `Method/Procedure`.)

- **Directory layout is producer-defined.** Grouping by node type (as above) is a
convention, not a requirement. A flat tree is conformant.

- `index.md` carries no frontmatter except an optional root-level `okf_version` /
`biookf_version`. `log.md` uses ISO `## YYYY-MM-DD` headings, newest first.

---

## 4\. The concept document

Each concept document = **YAML frontmatter** (the typed, queryable layer) + a **Markdown
body** (the human-readable layer). Minimal shape:

```yaml
---
type: Molecule                      # REQUIRED: exactly one of the 28 node types (§5)
identifier: Interleukin-6           # REQUIRED: human-readable AND unique across the bundle (§7.1)
subtype: protein                    # agent-coined refinement, no controlled set (§5, §7.1)
xref: [UniProtKB:P05231, HGNC:6018, NCBIGene:3569, MESH:D015850]   # optional external CURIEs
synonyms: [IL-6, IL6, interferon beta-2, BSF-2]
in_taxon: NCBITaxon:9606
edges:                              # typed relationships (§6): the graph layer
  # predicates are forward-only (§6): the gene→protein link is the `encodes` edge on the
  # IL6 *gene* page, not an `encoded_by` edge here; there is no inverse predicate.
  - predicate: binds
    object: IL6 receptor            # the target node's identifier
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: UniProt         # a source node's identifier (its infores: CURIE lives in that node's xref)
  - predicate: participates_in
    object: acute inflammatory response
    knowledge_level: knowledge_assertion
    agent_type: automated_agent
    primary_source: Gene Ontology
    evidence_type: [ECO:0000501]
  - predicate: associated_with
    object: COVID-19
    knowledge_level: statistical_association
    agent_type: text_mining_agent
    primary_source: SemMedDB
    publications: [PMID:32504360, PMID:32979574]
    effect_metric: hazard_ratio
    effect_size: 2.9
    ci_lower: 1.7
    ci_upper: 4.9
    p_value: 3.0e-6
    sample_size: 1484
---

# Interleukin-6 (IL6)

A pleiotropic pro-inflammatory cytokine secreted by macrophages and T cells…

## Schema
…structured description of the asset, if any…

## Examples
…

## Citations
- [Elevated IL-6 and severe COVID-19](https://pubmed.ncbi.nlm.nih.gov/32504360/) (PMID:32504360)
```

Body conventions (inherited from OKF, all optional): `# Schema`, `# Examples`,
`# Citations`. The body is free Markdown; it MAY restate edges in prose and MAY use
advisory `[[wiki-links]]` for "see also" navigation. **Only `edges:` entries are part of
the graph.**

### 4.1 Inline edge sugar (optional)

For lightweight authoring, an edge MAY be written inline in the body and is considered
equivalent to a frontmatter `edges:` entry:

```
[[treats:: COVID-19 | knowledge_level=knowledge_assertion;
  agent_type=manual_agent; primary_source=RECOVERY trial]]
```

The subject is always **the host document**; `object` is the target node's `identifier`;
direction is **subject → object**.

---

## 5\. The NODE universe (28 types)

`type` MUST be exactly one of the following 28 values, organized in two families:
**Biomedical Entities** (20), the science itself; and **Provenance & Context** (8), where
knowledge comes from and the abstractions used to describe it. Each type is an **umbrella**:
fine-grained distinctions are carried by a `subtype` attribute and by the `xref` CURIE
namespace, *not* by minting new types. This is the "exhaustive but not granular" lever.

> **The type is the controlled, mandatory concept; the subtype is the agent's own.**
> Exactly one `type` per document is required and MUST come from these 28. Every node should
> also carry a `subtype`, but `subtype` has **no controlled universe**: the agent coins an
> appropriate descriptive lowercase token itself. The values shown in the tables below are
> *examples the agent may reuse or replace*, never a closed enum, and consumers MUST NOT reject
> a node for an unrecognized `subtype`. Only `type` (and `predicate`, §6) are validated against
> a fixed universe. Beyond `type` and the `identifier` (§7.1), **every other field is optional**.

Every node also carries the universal attributes in §7.1.

### 5.A Biomedical Entities (20)

| \#  | `type`                 | Umbrella covers                                                                                                                                                                                                                 | `subtype` examples (agent-coined)                                                                                                                                   | Recommended | Maps to                                                                            |
| --- | ---------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------- | ---------------------------------------------------------------------------------- |
| 1   | **Gene**               | a gene, genomic locus, ncRNA/miRNA gene (the *heritable unit*)                                                                                                                                                                  | `protein_coding`·`ncRNA`·`miRNA`·`pseudogene`                                                                                                                       | `in_taxon`  | UMLS GENE; Biolink `Gene`; SPOKE Gene                                              |
| 2   | **Molecule**           | a single chemically-defined entity: protein, peptide, antibody, enzyme, complex, small molecule, drug, metabolite, lipid, ion, cofactor, nutrient, **RNA species (mRNA/pre-mRNA/miRNA/lncRNA)**                                 | `protein`·`drug`·`small_molecule`·`metabolite`·`antibody`·`complex`·`ion`·`rna`                                                                                     | none        | UMLS CHEM; Biolink `ChemicalEntity`·`Protein`·`Drug`; SPOKE Compound·Protein       |
| 3   | **MolecularClass**     | a molecular **grouping** (not a single molecule): pharmacologic/drug class, protein family, protein domain, gene set/signature, chemical class                                                                                  | `pharmacologic`·`protein_family`·`protein_domain`·`gene_set`·`chemical_class`                                                                                       | none        | SPOKE PharmacologicClass; InterPro/Pfam; MSigDB                                    |
| 4   | **Variant**            | a **deviation from the reference**: SNV/SNP, indel, CNV/SV, allele, genotype, haplotype, fusion, STR                                                                                                                            | `snv`·`indel`·`cnv`·`sv`·`allele`·`genotype`·`haplotype`·`fusion`·`str`                                                                                             | none        | Biolink `SequenceVariant`                                                          |
| 5   | **SequenceFeature**    | a **region of the reference**: enhancer, promoter, silencer, TFBS, CpG island, open chromatin, transposon, UTR/splice site (when itself the object of study)                                                                    | `enhancer`·`promoter`·`silencer`·`tfbs`·`cpg_island`·`open_chromatin`·`transposon`·`utr`                                                                            | none        | Biolink `RegulatoryRegion`·`TranscriptionFactorBindingSite`; SO `sequence_feature` |
| 6   | **Structure**          | a resolved or predicted **3D atomic-coordinate structure** of a macromolecule                                                                                                                                                   | `xray`·`cryo_em`·`nmr`·`predicted` (`resolution` optional)                                                                                                          | none        | PDB; AlphaFoldDB                                                                   |
| 7   | **Anatomy**            | body region, organ, organ system, tissue, gross structure, subcellular component/organelle, body fluid                                                                                                                          | `organ`·`tissue`·`subcellular`·`body_fluid`·`body_region`                                                                                                           | none        | UMLS ANAT; Biolink `AnatomicalEntity`·`CellularComponent`                          |
| 8   | **CellType**           | cell type, cell state, cell line, organoid                                                                                                                                                                                      | `cell_type`·`cell_state`·`cell_line`·`organoid`                                                                                                                     | none        | Biolink `Cell`·`CellLine`                                                          |
| 9   | **Organism**           | species, strain, taxon, pathogen (bacterium/virus/fungus/parasite), microbe, model organism, host                                                                                                                               | `species`·`strain`·`pathogen`·`microbe`                                                                                                                             | `in_taxon`  | UMLS LIVB; Biolink `OrganismTaxon`                                                 |
| 10  | **BiologicalPathway**  | a *process*: pathway, reaction, signaling cascade, GO biological process, physiologic/pathologic process, behavior                                                                                                              | `pathway`·`reaction`·`signaling`·`go_bp`·`physiologic`·`pathologic`·`behavior`                                                                                      | none        | Biolink `BiologicalProcess`·`Pathway`; SPOKE Pathway·BiologicalProcess             |
| 11  | **BiologicalFunction** | an *elemental molecular activity* (GO molecular function): catalytic / binding / transporter activity of a gene product                                                                                                         | `catalytic`·`binding`·`transporter`                                                                                                                                 | none        | Biolink `MolecularActivity`; SPOKE MolecularFunction                               |
| 12  | **Disease**            | disease, syndrome, disorder, neoplasm, infection, injury, congenital/acquired abnormality (a **diagnosis**)                                                                                                                     | `infection`·`neoplasm`·`syndrome`·`injury`·`congenital`                                                                                                             | none        | UMLS DISO; Biolink `Disease`                                                       |
| 13  | **Phenotype**          | an observable manifestation: symptom, **sign**, side effect, qualitative trait, morphologic/behavioral feature                                                                                                                  | `symptom`·`sign`·`side_effect`·`trait`·`morphologic`·`behavioral`                                                                                                   | none        | Biolink `PhenotypicFeature`; Hetionet Symptom·SideEffect                           |
| 14  | **BiomedicalMeasure**  | a **named measurable variable**: lab test/result, vital, score/index (BMI, PRS, TNM), scale, biomarker readout, omics readout, imaging finding                                                                                  | `lab_test`·`vital`·`score`·`scale`·`biomarker`·`omics_readout`·`imaging_finding`                                                                                    | none        | Biolink `ClinicalAttribute`; LOINC/OBA                                             |
| 15  | **MethodOrProcedure**  | a clinical procedure (surgical/imaging/preventive/diagnostic/intervention/vaccination/screening) **and** a lab assay/technique, computational pipeline/tool/software, statistical method (e.g. PCA), model, algorithm, protocol | `surgical`·`imaging`·`vaccination`·`screening`·`diagnostic`·`lab_assay`·`lab_protocol`·`computational_pipeline`·`software`·`algorithm`·`statistical_method`·`model` | none        | Biolink `Procedure`·`Treatment`; OBI/EDAM                                          |
| 16  | **Exposure**           | a behavioral / environmental / occupational / dietary-pattern exposure                                                                                                                                                          | `behavioral`·`environmental`·`occupational`·`dietary`                                                                                                               | none        | Biolink `ExposureEvent`; ECTO/ExO                                                  |
| 17  | **SocialFactor**       | a **social factor affecting health** (social determinant of health): income, education, housing, employment, food security, access to care, social support                                                                      | `economic`·`education`·`housing`·`healthcare_access`·`social_support`·`food_security`                                                                               | none        | Biolink `SocialDeterminantOfHealth`                                                |
| 18  | **Food**               | a food item, food group, dietary product                                                                                                                                                                                        | `food_item`·`food_group`·`dietary_product`                                                                                                                          | none        | SPOKE Food; FOODON                                                                 |
| 19  | **Device**             | an engineered artifact: medical device, implant/prosthesis/graft/mesh, drug-delivery device, research instrument, reagent/kit                                                                                                   | `implant`·`prosthesis`·`graft`·`instrument`·`reagent`                                                                                                               | none        | Biolink `Device`                                                                   |
| 20  | **MaterialSample**     | a biospecimen / physical sample: serum, biopsy, aliquot, tissue block, cell-line stock                                                                                                                                          | `serum`·`biopsy`·`aliquot`·`tissue_block`·`cell_line_stock`                                                                                                         | none        | Biolink `MaterialSample`; BioSample                                                |

### 5.B Provenance & Context (8)

| \#  | `type`                 | Umbrella covers                                                                                                                                                                | `subtype` examples (agent-coined)                                                          | Recommended  | Maps to                                                                      |
| --- | ---------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------ | ------------ | ---------------------------------------------------------------------------- |
| 21  | **Publication**        | a *document/artifact*: journal article, preprint, book, patent, drug label, guideline, abstract, **poster, slide deck, blog, tweet/thread, lab-notebook entry, meeting notes** | `article`·`preprint`·`slide_deck`·`tweet`·`lab_notebook`·`guideline`·`patent`·`drug_label` | none         | Biolink `Publication`: *most raw-source modalities live here as provenance* |
| 22  | **Study**              | a *designed investigation*: clinical trial, cohort/case-control/RCT, GWAS, observational study, registry, screen                                                               | `rct`·`cohort`·`case_control`·`gwas`·`registry`·`observational`                            | none         | Biolink `Study`; xref ≈ `clinicaltrials:NCT…`                               |
| 23  | **Dataset**            | a *data artifact*: dataset, data file, table/CSV/XLSX, omics matrix, image collection, knowledge base                                                                          | `table`·`omics_matrix`·`image_collection`·`knowledge_base`                                 | none         | Biolink `Dataset`                                                            |
| 24  | **Agent**              | a person/author/curator/patient-case, lab, consortium, organization, company, regulator, funder, software agent, online handle                                                 | `person`·`lab`·`organization`·`company`·`regulator`·`consortium`·`software_agent`          | none         | Biolink `Agent`·`Organization`; xref ≈ `ORCID:`/`infores:`                  |
| 25  | **Population**         | a **group of people**: cohort, study population, ancestry group, demographic group                                                                                             | `cohort`·`ancestry`·`demographic`                                                          | none         | HANCESTRO; distinct from `Study` (the investigation)                         |
| 26  | **GeographicLocation** | a country, region, or place context                                                                                                                                            | `country`·`region`·`place`                                                                 | none         | GeoNames/GADM/ISO-3166                                                       |
| 27  | **Concept**            | an abstract concept, attribute, classification system, unit, ontology term, or score definition not better typed above                                                         | `unit`·`classification`·`score_definition`·`ontology_term`                                 | none         | UMLS CONC; Biolink `Attribute`·`NamedThing`                                  |
| 28  | **Other**              | the explicit escape hatch: a biomedical concept that genuinely fits none of the 27 above (closure; preserves OKF's "minimally opinionated" spirit)                            | none                                                                                       | `note` (why) | Biolink `NamedThing` (root)                                                  |

> **Closure & granularity.** The 19 substantive v0.1 types mapped onto the 15 **UMLS Semantic
> Groups** (which partition 99.5% of the multi-million-concept UMLS Metathesaurus) + 5
> provenance/method types + `Other`. v0.3 kept that closure and added finer partitions: three
> identity-driven splits (`Variant`/`SequenceFeature`, `BiologicalPathway`/`BiologicalFunction`,
> `Device`/`MaterialSample`), the dissolved risk-factor family
> (`Exposure`·`SocialFactor`·`Food`·`Population`·`GeographicLocation`), and
> `MolecularClass`·`Structure`. Independently, every node type of SPOKE, Hetionet, PrimeKG,
> Biolink core, and PubTator3 maps onto exactly one BioOKF type. The universe is **jointly
> exhaustive** and, with `subtype` carrying sub-granularity, **mutually exclusive** in practice.
> The push past the original "\~20" target is a **deliberate granularity choice** favoring
> exposome and genetics expressiveness; consumers that prefer coarser typing can collapse via
> the alias mapping in §14.

**Typing rule: identity, not role.** Type a concept by *what it is*, and express its
*roles* as edges. Aspirin **is** a `Molecule`; that it *treats* headache and *inhibits*
COX-1 are edges, not types. A **risk factor** is a role, not a type; its subject may be a
`Disease`, `BiomedicalMeasure`, `Exposure`, `SocialFactor`, `Organism`, `Variant`, … linked by
`predisposes_to`. A **biomarker** that is a protein **is** a `Molecule`; its measured level
**is** a `BiomedicalMeasure`; the value is an edge attribute. **State** (Disease) vs
**manifested quality** (Phenotype) vs **measurable variable** (BiomedicalMeasure) vs **value**
(edge attribute) are kept distinct. This keeps types mutually exclusive and prevents the type
set from exploding into roles.

**Class vs instance.** Type a *coordinate-bearing / identified instance* as its entity type;
type a *bare abstract category* as `Concept`. "The 3′ UTR of EPAS1" (a specific region) is a
`SequenceFeature`; "a 3′ UTR" as an abstract class (the SO term used as a label) is a
`Concept`. "Principal component analysis" the technique is a `MethodOrProcedure`; "a principal
component" as a derived variable is a `Concept`. The external CURIE (`xref`) pins the referent.

### 5.C Disease vs Phenotype vs BiomedicalMeasure: the boundary

These three types intersect in real biomedicine, and the **same word often spans them**
(height is both a measurement and a trait; hyperlipidemia is both a disease and a measured
state). BioOKF resolves this exactly as the clinical-data standards do: **OMOP CDM** separates
a *Measurement* from a *Condition*, and **OBO/Monarch** separate a *disease* (a MONDO
disposition the organism *bears*) from a *phenotype* (an HPO bearer+quality it *manifests*).

| `type`                  | It *is*…                                                          | Anchor vocab                             | The test                                                |
| ----------------------- | ----------------------------------------------------------------- | ---------------------------------------- | ------------------------------------------------------- |
| **`BiomedicalMeasure`** | a measurable quantity / analyte / score (carries a unit or value) | LOINC · OBA · EFO · PATO                 | "Does it take a value, or come from a test/score?"      |
| **`Phenotype`**         | an observable sign / symptom / abnormal trait (a manifestation)   | HPO · MP · MedDRA                        | "Is it *observed/exhibited* as a characteristic?"       |
| **`Disease`**           | a diagnosed condition the organism *has* (a disposition)          | MONDO · DOID · ICD · SNOMED "(disorder)" | "Does a clinician diagnose it and a patient *have* it?" |

**Decision procedure** (apply in order; if more than one applies, mint **one node per facet**
and link them):

1. unit / value, or produced by a test / score → **`BiomedicalMeasure`**;

2. else an observed sign / symptom / abnormality → **`Phenotype`** (`subtype`:
`symptom` = subjective / patient-reported; `sign` = objective / observer-detected);

3. else a diagnosed / treated condition the organism *has* → **`Disease`**.

**Connecting edges:** a `Disease` **`has_phenotype`** its signs/symptoms; a `BiomedicalMeasure`
**`measures`** the `Phenotype`/`Disease` it quantifies. **A numeric value (e.g. "183 cm") is
edge data, never a node.**

This is consistent with *type by identity*: the three facets are **distinct identities** (each
with its own external CURIE, recorded in `xref`), so they are distinct nodes, with distinct
`identifier`s, not a fork of one entity. The **external CURIE pins the facet** (`MONDO:`→Disease,
`HP:`→Phenotype, `LOINC:`→BiomedicalMeasure), the same "namespace disambiguates referent" rule
that separates a gene from its protein.

**Worked disambiguations (canonical):**

- **Height** → `BiomedicalMeasure` "body height" (`LOINC:8302-2` / `EFO:0004339`); the abnormal
observable "tall stature" (`HP:0000098`) is a separate `Phenotype`; "183 cm" rides on a
`measures` edge.

- **BMI** → `BiomedicalMeasure` (`EFO:0004340`); "obesity" the observable → `Phenotype`
(`HP:0001513`); the diagnosed disorder → `Disease` (`MONDO:0011122`).

- **Hyperlipidemia** → three linked nodes: the disorder `Disease` (`MONDO:0021187`), the
abnormal feature `Phenotype` "elevated lipids" (`HP:0003077`), and the analyte
`BiomedicalMeasure` "LDL cholesterol" (`LOINC:13457-7`).

- **Fever** → `Phenotype` (a *sign*, `HP:0001945`); "body temperature" → `BiomedicalMeasure`.
**Pain** → `Phenotype` (a *symptom*, `HP:0012531`). **Polygenic risk score** →
`BiomedicalMeasure` (derived), `associated_with`-linked to the `Disease` it predicts.

> OMOP lumps Disease **and** Phenotype into its single "Condition" domain, so it validates the
> Measure-vs-Condition split but not the Disease-vs-Phenotype split; BioOKF makes the latter via
> the SNOMED/MONDO "(disorder)" vs HPO "(finding)" distinction above.

### 5.D Boundaries between the entity types

§5.C handles the tricky clinical trio. The other historically fuzzy pairs each resolve to a
single discriminating test; apply *identity, not role* throughout:

- **Gene vs Molecule (RNA).** The heritable locus is a `Gene`; a specific transcript or RNA
molecule is a `Molecule` (`subtype: rna`). The gene→product link is the `encodes` edge.

- **Molecule vs MolecularClass.** A single chemical/protein entity is a `Molecule`; a *grouping*
(drug class, protein family/domain, gene set, chemical class) is a `MolecularClass`; members
attach via `member_of`.

- **Variant vs SequenceFeature.** A *deviation from the reference* (SNV, indel, CNV, allele,
haplotype) is a `Variant`; a *region of the reference* (enhancer, promoter, UTR, CpG island)
is a `SequenceFeature`. Variant *consequence* (missense, 5′UTR, splice-donor) is an
**attribute**, not a node; structural partonomy (exon/intron/CDS/codon → gene) is a `part_of`
**edge**, not a node.

- **BiologicalPathway vs BiologicalFunction.** A *process* (pathway, reaction, signaling
cascade, GO-BP, physiologic process, behavior) is a `BiologicalPathway`; an *elemental
molecular activity* (GO-MF: catalytic / binding / transporter) is a `BiologicalFunction`.
Reactions live under `BiologicalPathway` (`catalyzes` points there).

- **Structure vs Molecule.** The chemical entity is a `Molecule`; a resolved/predicted 3D
atomic-coordinate model of it is a `Structure` (which `derives_from` the `Molecule`).

- **Anatomy(subcellular) vs CellType.** A cell *type/state/line/organoid* is a `CellType`; an
organelle or compartment is `Anatomy` (`subtype: subcellular`); cellular component aligns to
Anatomy, not to a process.

- **Exposure vs SocialFactor vs Food.** A behavioral/environmental/occupational/dietary-pattern
exposure is an `Exposure`; a social/economic/structural determinant (income, education,
housing, access to care) is a `SocialFactor`; a specific food item/group/product is a `Food`.
A *defined* chemical exposure routes to `Molecule`; an infectious one to `Organism`. "Risk
factor" is never a type; it is the `predisposes_to`/`prevents` role.

- **Device vs MaterialSample.** An engineered artifact (implant, instrument, reagent/kit) is a
`Device`; a biospecimen (serum, biopsy, aliquot, cell-line stock) is a `MaterialSample`.

- **Population vs Study vs Organism.** A *group of people* (cohort, ancestry, demographic) is a
`Population`; the *investigation* that enrolled them is a `Study`; a *taxon/species/strain* is
an `Organism`.

- **MethodOrProcedure vs its inputs/outputs.** The technique/protocol/algorithm is a
`MethodOrProcedure`; its inputs and outputs (a `MaterialSample`, a `Dataset`) are their own
types; a specific *execution* of the method is an edge, not a node.

- **Publication vs Study vs Dataset.** The *document/artifact* (paper, preprint, slide, tweet,
bench note) is a `Publication`; the *designed investigation* is a `Study`; the *data
file/matrix/collection* is a `Dataset`.

- **Concept vs Other.** An abstract attribute/unit/classification/ontology-term-as-label is a
`Concept`; a biomedical thing that genuinely fits none of the 27 substantive types is `Other`
(always with a `note` saying why).

---

## 6\. The EDGE universe (35 predicates)

Cross-document relationships are **typed edges**. The predicate set is **35**: **24 positive**
predicates (tabulated below) plus **11 `not_<X>` negatives** for the negatable effect predicates
(see Negation, end of this section). `SCHEMA.md` (authoritative, implemented in `bokf-core`) is the
canonical source for this set. The 24 positive predicates are organized under the five **UMLS
super-relation families** (the canonical finite-but-exhaustive relation backbone). Direction is
**subject (host document) → object** unless marked *symmetric*. The 24 are **forward-only**: there
are **no inverse predicates**; express a reverse relationship by authoring the forward edge on the
*other* node (a gene's `encodes`, never a protein's `encoded_by`; `causes`, never `caused_by`).
Inverse names are accepted on read only as deprecated aliases (§14). Each edge carries the universal
edge attributes in §7.2 (notably the required provenance triplet). The 28-type universe is expressed
by **extending domain/range** to admit the new node types; it adds **no predicates**.

### 6.A Structural & hierarchical: *physically\_related\_to*

| \#  | `predicate`       | Meaning                                                                                                               | Dir. | Typical domain → range                                                                                      | Maps to                                               |
| --- | ----------------- | --------------------------------------------------------------------------------------------------------------------- | ---- | ----------------------------------------------------------------------------------------------------------- | ----------------------------------------------------- |
| 1   | **is\_a**         | subject is a more-specific kind/instance of object (ontology backbone)                                                | →    | any → same type                                                                                             | UMLS `isa`; Biolink `subclass_of`; SPOKE ISA          |
| 2   | **part\_of**      | structural/compositional part (mereology); **structural genomic partonomy lives here** (exon/intron/CDS/codon → Gene) | →    | Anatomy·Molecule·Variant·SequenceFeature·BiologicalPathway → larger whole                                   | UMLS `part_of`; SemMedDB `PART_OF`; Biolink `part_of` |
| 3   | **member\_of**    | subject belongs to a class/family/group (drug→class, gene→gene set, protein→family)                                   | →    | Molecule·Gene → MolecularClass·BiologicalPathway                                                            | Biolink `member_of`; Hetionet PCiC                    |
| 4   | **derives\_from** | material/data lineage (sample←donor, cell←tissue, metabolite←parent, dataset←study, structure←molecule)               | →    | CellType·Device·Molecule·Dataset·MaterialSample·Structure·Food·Population → Organism·Anatomy·Study·Molecule | UMLS `derivative_of`; Biolink `derives_from`          |

### 6.B Spatial & expression: *spatially\_related\_to*

| \#  | `predicate`       | Meaning                                                                       | Dir. | Typical domain → range                                                                                                         | Maps to                                                                                                  |
| --- | ----------------- | ----------------------------------------------------------------------------- | ---- | ------------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------- |
| 5   | **located\_in**   | positioned/found in: **anatomical, genomic, or geographic**                  | →    | Disease·BiologicalPathway·Molecule·CellType·Variant·SequenceFeature → Anatomy·Organism·Gene·SequenceFeature·GeographicLocation | UMLS `location_of`; SemMedDB `LOCATION_OF`; Hetionet DlA                                                 |
| 6   | **expressed\_in** | gene/molecule is expressed / abundant / present in an anatomy or cell context | →    | Gene·Molecule → Anatomy·CellType                                                                                               | Biolink `expressed_in`; Hetionet AeG; SPOKE EXPRESSEDIN. *Carries `direction`, `effect_size`, `p_value`* |

### 6.C Molecular & functional: *functionally\_related\_to* (the rich core)

| \#  | `predicate`          | Meaning                                                                                                                           | Dir. | Typical domain → range                                                                     | Maps to                                                                                                                             |
| --- | -------------------- | --------------------------------------------------------------------------------------------------------------------------------- | ---- | ------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------- |
| 7   | **encodes**          | gene encodes / is transcribed-translated to its product (author on the Gene page; there is **no** inverse `encoded_by` predicate) | →    | Gene → Molecule                                                                            | Biolink `has_gene_product`; SPOKE GeP                                                                                               |
| 8   | **interacts\_with**  | physical/functional interaction (PPI, gene-gene, drug-drug, host-pathogen)                                                        | sym  | Molecule·Gene·Organism ↔ same                                                              | UMLS `interacts_with`; Hetionet GiG; PubTator3 interact                                                                             |
| 9   | **binds**            | direct binding with measurable affinity (drug-target, TF-DNA, antibody-antigen)                                                   | →    | Molecule → Molecule·Gene·Variant·SequenceFeature (TF→TFBS/enhancer)                        | Biolink `binds`; Hetionet CbG. *Carries `Kd`/`Ki`/`IC50` via `effect_metric`*                                                       |
| 10  | **regulates**        | subject increases/decreases activity, abundance, or expression of object (signed)                                                 | →    | Molecule·Gene·Variant·SequenceFeature → Molecule·Gene·BiologicalPathway·BiologicalFunction | Biolink `affects`+direction; SemMedDB `INHIBITS`/`STIMULATES`; SPOKE up/down-regulates. **`direction` required**; `aspect` optional |
| 11  | **catalyzes**        | enzyme/complex catalyzes a reaction (substrate → product)                                                                         | →    | Molecule → BiologicalPathway (reaction)                                                    | UMLS `produces`; GO/Rhea                                                                                                            |
| 12  | **converts\_to**     | subject is chemically transformed / metabolized into object                                                                       | →    | Molecule → Molecule                                                                        | SemMedDB `CONVERTS_TO`; PubTator3 convert                                                                                           |
| 13  | **participates\_in** | subject takes part in / **enables** / is input-output of a pathway, process, or function                                          | →    | Gene·Molecule·Organism → BiologicalPathway·BiologicalFunction                              | Biolink `participates_in`·`enables`; Hetionet GpBP/GpPW; SemMedDB `PROCESS_OF`                                                      |

### 6.D Clinical & causal: *functionally\_related\_to* (clinical)

| \#  | `predicate`               | Meaning                                                                                                                                                     | Dir. | Typical domain → range                                                                                                     | Maps to                                                                                                |
| --- | ------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------- | ---- | -------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------ |
| 14  | **causes**                | subject brings about / induces / drives object (etiology; incl. pathogen→disease, somatic driver, **drug→adverse-event** as object=Phenotype)               | →    | any agent → Disease·Phenotype·BiologicalPathway                                                                            | UMLS `causes`; SemMedDB `CAUSES`; Hetionet CcSE                                                        |
| 15  | **predisposes\_to**       | subject increases the *risk/likelihood* of object without being sufficient cause (the epidemiology risk-factor edge; **broad domain, risk factor = role**) | →    | Variant·Gene·Molecule·Exposure·SocialFactor·Food·Disease·BiomedicalMeasure·Phenotype → Disease·Phenotype·BiomedicalMeasure | UMLS `predisposes`; SemMedDB `PREDISPOSES`. *Carries `odds_ratio`/`hazard_ratio`/`relative_risk`*      |
| 16  | **treats**                | subject (drug/procedure/device/intervention) cures, manages, ameliorates, or palliates object condition                                                     | →    | Molecule·MethodOrProcedure·Device → Disease·Phenotype                                                                      | UMLS `treats`; SemMedDB `TREATS`; Hetionet CtD. *`clinical_phase`, `effect_metric`*                    |
| 17  | **prevents**              | subject stops / hinders / reduces onset risk of object condition                                                                                            | →    | Molecule·MethodOrProcedure·Exposure·Food·SocialFactor → Disease·Phenotype                                                  | UMLS `prevents`; SemMedDB `PREVENTS`                                                                   |
| 18  | **contraindicated\_in**   | subject (drug/procedure) must not be used in object condition/context                                                                                       | →    | Molecule·MethodOrProcedure → Disease·Phenotype·Organism                                                                    | Biolink `contraindicated_in`; SPOKE CcD                                                                |
| 19  | **affects\_response\_to** | subject (gene/variant/biomarker) modulates response, sensitivity, resistance, or metabolism of object drug (pharmacogenomics)                               | →    | Gene·Variant·BiomedicalMeasure → Molecule                                                                                  | Biolink `affects_response_to`; SPOKE pharmacogenomic edges. **`response_direction`**                   |
| 20  | **has\_phenotype**        | subject (disease/organism/genotype) presents / manifests object phenotype, sign, or symptom (**disease-vs-symptom is this edge, not a type**)               | →    | Disease·Organism·Variant → Phenotype                                                                                       | Biolink `has_phenotype`; SemMedDB `MANIFESTATION_OF`; Hetionet DpS. *`frequency`, `onset`, `severity`* |

### 6.E Measurement, association & provenance: *temporally/conceptually\_related\_to*

| \#  | `predicate`          | Meaning                                                                                                                                                                                 | Dir. | Typical domain → range                                                    | Maps to                                                                                                               |
| --- | -------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---- | ------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| 21  | **measures**         | subject ascertains the value of, or is diagnostic for, object (incl. `diagnoses`)                                                                                                       | →    | MethodOrProcedure·BiomedicalMeasure·Molecule → Disease·Phenotype·Molecule | UMLS `measures`·`diagnoses`; SemMedDB `MEASURES`/`DIAGNOSES`. *`sensitivity`, `specificity`, `auc`, `unit`*           |
| 22  | **associated\_with** | a statistical / observed / co-occurrence association, non-causal, non-mechanistic (GWAS, eQTL, correlation, comorbidity, literature co-occurrence). The quantitative **umbrella** edge | sym  | any ↔ any                                                                 | UMLS `associated_with`·`co-occurs_with`; SemMedDB `ASSOCIATED_WITH`; Hetionet DaG·GcG. *Full statistical bundle (§8)* |
| 23  | **reported\_in**     | the universal **provenance edge**: the subject node (or a reified claim) is reported / curated / studied / evidenced in the object Publication, Study, Dataset, or by an Agent          | →    | any → Publication·Study·Dataset·Agent                                     | Biolink `publications`·`provided_by`; CKG MENTIONED\_IN\_PUBLICATION                                                  |
| 24  | **used\_to\_study**  | an investigative resource (method / study / dataset / device / research-model) is used to study, model, probe, or make tractable the object entity under investigation (resource → entity studied)          | →    | MethodOrProcedure·Study·Dataset·Device·Organism·CellType·MaterialSample → Disease·Phenotype·BiologicalPathway·BiologicalFunction·Gene·Variant·Molecule                                     | Biolink `studied_to_understand` / `models`; the "method/model → what it studies" axis                                                  |

> **Why 24 positive, and why these.** All 24 nest under one of UMLS's five super-relation families; all
> map to a canonical Biolink predicate; \~18 also map to a named SemMedDB/SemRep predicate. The
> vocabulary stays small because **distinctions that other schemas mint as separate edges are
> folded into attributes here**: up- vs down-regulation → `regulates` \+ `direction`; binding
> affinity vs catalysis → `effect_metric`; temporal order / co-occurrence
> → `associated_with` \+ `timepoint`; per-datasource edge identity → `primary_source`. (Negation is
> the one exception: a tested-and-refuted finding is its own canonical `not_<X>` predicate, not an
> attribute; see Negation below.) Variant
> **consequence** (missense, 5′UTR, splice-donor; SO/VEP) is an **attribute** on the variant,
> never a node; a measure's **value** in context is an **edge attribute**, never a node. This
> mirrors Biolink's own "qualify, don't multiply" decision and keeps BioOKF reliably
> *assignable* from messy sources while covering the Hetionet metaedges, the SemMedDB
> predicates, and the PubTator3 relations with no remainder.

### 6.F Negation (polarity)

A genuine *negative* finding stated in the source ("X does **not** treat Y", "**no** association
between X and Y", "drug A does **not** bind target B") is authored with the canonical negative
predicate **`not_<X>`**. Only the **11 negatable effect predicates** are negatable, giving 11
`not_<X>` predicates and a total of **24 positive + 11 negative = 35**:

`binds`, `interacts_with`, `causes`, `predisposes_to`, `prevents`, `treats`,
`affects_response_to`, `associated_with`, `expressed_in`, `regulates`, `has_phenotype`
(hence `not_binds`, `not_interacts_with`, `not_causes`, `not_predisposes_to`, `not_prevents`,
`not_treats`, `not_affects_response_to`, `not_associated_with`, `not_expressed_in`,
`not_regulates`, `not_has_phenotype`).

Each `not_<X>` **inherits its base predicate's domain/range and symmetry** (so `not_binds` has the
domain/range of `binds`, `not_associated_with` is symmetric like `associated_with`); asserting both
`<X>` and `not_<X>` for the same subject → object is a contradiction (flagged by lint). Negating a
structural / definitional / provenance predicate (`is_a`, `part_of`, `encodes`, `measures`,
`reported_in`, `used_to_study`, …) is meaningless under open-world semantics (absence already covers
it) and is **rejected** (lint `edge.not_negatable`). The legacy `negated: true` qualifier (§7.2) is
**accepted on read** on a negatable predicate and **normalized to** the canonical `not_<X>`; on a
non-negatable predicate it is rejected (`edge.not_negatable`).

---

## 7\. Attributes: required vs optional

### 7.1 Universal node attributes

| Attribute     | Req?                                                            | Meaning                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| ------------- | --------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `type`        | **required**                                                    | one of the 28 node types (§5): the controlled, mandatory concept                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| `identifier`  | **required**                                                    | the node's primary key: a name that is **human-readable** *and* **unique across the bundle**. Merges the former `title` and `id`; edges target it via `object`. SHOULD avoid `:` (reserved for CURIEs); disambiguate collisions with a parenthetical facet, e.g. `IL6 (gene)` vs `IL6 (protein)`                                                                                                                                                                                                               |
| `subtype`     | **expected (agent-coined)**                                     | a refinement of `type` with **no controlled universe**: the agent coins an appropriate lowercase token (e.g. `protein`, `enhancer`, `sign`). The §5 values are examples; never validated, never rejected                                                                                                                                                                                                                                                                                                       |
| `description` | optional                                                        | one-sentence summary                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| `synonyms`    | optional                                                        | list of alternative names                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| `xref`        | optional                                                        | equivalent **external** ontology CURIEs (`prefix:local`) for interoperability/enrichment, not the primary key                                                                                                                                                                                                                                                                                                                                                                                                  |
| `in_taxon`    | optional (recommended for Gene & Organism)                      | NCBITaxon CURIE/name                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| `note`        | optional (recommended for Other: why no substantive type fits) | free text                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| `tags`        | optional                                                        | free categorization                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| `raw_source`  | optional (source nodes only)                                    | for a **source node**, the immutable bytes it was distilled from, as a **list of bundle-root-relative `raw/…` paths** (e.g. `[raw/pmid-33933206.pdf]`; multiple entries = one logical source spread over several files). Anchors the provenance chain in `raw/`. Present on *ingested-document* sources; absent on *external-reference* sources (HGNC, GO…), which instead record their `infores:`/ontology CURIE in `xref`. On a non-source node it is meaningless and SHOULD be omitted (consumers ignore it) |
| `timestamp`   | optional                                                        | ISO-8601 last-modified                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |

> **Only `type` and `identifier` are mandatory.** Exactly one `type` from the controlled set of
> 28, plus an `identifier` that is **human-readable** and **unique across the bundle** (lint
> enforces both, see §10). The `identifier` is the single merged successor of the old `title`
> and `id`. **Every other field is optional.** Always also supply a `subtype`, but the agent
> **invents** it: there is no controlled list, so it is never validated and a node is never
> rejected over it. Equivalent external CURIEs are curated as best-effort enrichment in `xref`
> (§9, §10).

### 7.2 Universal edge attributes

| Attribute           | Req?                                                   | Meaning                                                                                                                                                                                                                                                                                                                                                                                         |
| ------------------- | ------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `predicate`         | **required**                                           | one of the 35 edge types (24 positive + 11 `not_<X>`) (§6)                                                                                                                                                                                                                                                                                                                                                                   |
| `object`            | **required**                                           | the **`identifier`** of the target node (human-readable, bundle-unique)                                                                                                                                                                                                                                                                                                                         |
| `knowledge_level`   | **required**                                           | Biolink `KnowledgeLevelEnum`: `knowledge_assertion` · `statistical_association` · `prediction` · `observation` · `not_provided`                                                                                                                                                                                                                                                                 |
| `agent_type`        | **required**                                           | Biolink `AgentTypeEnum`: `manual_agent` · `automated_agent` · `text_mining_agent` · `data_analysis_pipeline` · `computational_model` · `not_provided`                                                                                                                                                                                                                                           |
| `primary_source`    | **required**                                           | the originating source: the `identifier` of a **source node** (a `Publication`/`Study`/`Dataset`/`Agent`), **not** a CURIE. (Old `infores:`-CURIE values normalize to a source node carrying that CURIE in `xref`.) The reserved value **`not_provided`** is the only non-node value, and is a **rare escape hatch** for claims whose origin is genuinely unknown, never a default (see §8.1) |
| `negated`           | optional (**legacy**)                                  | **Legacy** polarity form. The canonical mechanism is the negative `not_<X>` predicate (§6.F). `negated: true` on a **negatable** predicate is accepted on read and **normalized to** the canonical `not_<X>`; `negated: true` on a **non-negatable** predicate is **rejected** (`edge.not_negatable`)                                                                                              |
| `direction`         | required for `regulates`/`expressed_in`; else optional | `increased` · `decreased` · `unspecified`                                                                                                                                                                                                                                                                                                                                                       |
| `aspect`            | optional (used with `regulates`)                       | what is regulated: `activity` · `abundance` · `expression` · `localization`                                                                                                                                                                                                                                                                                                                     |
| `publications`      | optional                                               | supporting PMIDs/DOIs/NCTs                                                                                                                                                                                                                                                                                                                                                                      |
| `aggregator_source` | optional                                               | intermediate sources                                                                                                                                                                                                                                                                                                                                                                            |
| `evidence_type`     | optional                                               | ECO CURIEs / GO evidence codes                                                                                                                                                                                                                                                                                                                                                                  |
| `confidence_score`  | optional                                               | 0 to 1                                                                                                                                                                                                                                                                                                                                                                                             |
| `qualifiers`        | optional                                               | context map: `species_context`, `anatomical_context`, `cell_context`, `sex`, `age_group`, `timepoint`, `population`                                                                                                                                                                                                                                                                             |

### 7.3 Quantitative bundle (optional, on statistical/clinical edges)

`p_value` · `adjusted_p_value` · `effect_size` · `effect_metric`
(`beta`·`odds_ratio`·`hazard_ratio`·`relative_risk`·`incidence_rate_ratio`·`log2_fold_change`·`correlation_r`·`IC50`·`Ki`·`Kd`·`EC50`·`MIC`·`enrichment_score`) ·
`ci_lower` · `ci_upper` · `standard_error` · `sample_size` · `sensitivity` · `specificity`
· `auc` · `frequency` · `clinical_phase` · `response_direction` · `unit`.

> **The required set is deliberately tiny** (3 fields per node, 5 per edge), exactly
> following Biolink's precedent that only `knowledge_level` \+ `agent_type` are mandatory
> association metadata. A one-line curated edge is never over-burdened; a GWAS edge can
> carry its full statistics. Everything quantitative is optional but **first-class**: it
> rides on named slots, not buried in prose.

---

## 8\. Provenance, evidence, and quantitative claims

Because BioOKF ingests everything from gold-standard databases to tweets, **every edge
states how it is known**. The mandatory triplet does the work:

- `knowledge_level`: *what kind of claim is this?* A curated assertion
(`knowledge_assertion`), a statistic (`statistical_association`), a model prediction
(`prediction`), or a raw observation (`observation`).

- `agent_type`: *who/what produced it?* A human curator (`manual_agent`), a rule-based
pipeline (`automated_agent`), an LLM/NLP extractor (`text_mining_agent`), a stats
pipeline (`data_analysis_pipeline`), or an ML model (`computational_model`).

- `primary_source`: *where did it originate?* the `identifier` of a source **node**, a
`Publication`/`Study`/`Dataset`/`Agent`, **not** a CURIE.

A consumer can then filter: admit only `knowledge_level = knowledge_assertion` for clinical
decisions; admit `statistical_association` \+ `text_mining_agent` for hypothesis generation.
This single discipline is what makes a heterogeneous-source graph trustworthy.

### 8.1 Provenance is node-based (v0.5)

`primary_source` is *not* a bare external identifier; it is the `identifier` of one of the
bundle's own **source nodes**. A *source node* is a node of type `Publication`, `Study`, `Dataset`,
or `Agent`: the four Provenance & Context types that can bear a source (the other four,
`Population`, `GeographicLocation`, `Concept`, `Other`, are **not** valid `primary_source` or
`reported_in` targets). This unifies provenance with the rest of the graph: the source is a real
node you can traverse to (and that itself carries `synonyms`, `xref`, and edges), exactly like any
other concept.

**Two provenance mechanisms (v0.5 dropped `provided_by`).** Provenance is carried by exactly two
things, both pointing at source nodes by `identifier`: **`primary_source`** (required, **per edge**),
the single originating source of *that one claim*; and the **`reported_in` edge** (per
node/claim), an explicit, traversable link from a concept to its source node. A node-level "this
page came from X" is simply a `reported_in` edge, so there is no separate `provided_by`. A
`reported_in` edge is itself an edge and carries its own `primary_source`; **by convention that is
the edge's own `object`** (the source attests its own contents), the intended terminating base
case, not a lint error.

**Source nodes anchor to `raw/`.** A source node MAY carry a `raw_source` field:
one or more `raw/…` paths naming the immutable bytes it was distilled from. Every claim's
provenance chain then terminates at an immutable file:

```
claim  ──primary_source──▶  source node  ──raw_source──▶  raw/pmid-33933206.pdf
```

so a consumer can walk from any edge back to the exact source material.

**Two species of source node,** distinguished by where they bottom out:

- an **ingested-document** source (a paper, preprint, tweet, slide, dataset, or bench note you
placed in `raw/`) is a `Publication`/`Study`/`Dataset` node **with** a `raw_source`;

- an **external-reference** source (an authoritative database or ontology you cite but did *not*
ingest: HGNC, GO, UniProt, SemMedDB, DrugBank, ATC) is a node **without** a `raw_source`,
recording its `infores:`/ontology CURIE in `xref`. Choose its type by identity: a naming/identity
authority or organization (HGNC, ORCID, a regulator, a consortium) is an **`Agent`**
(`subtype: organization`); a citable data/knowledge artifact (GO, SemMedDB, DrugBank, a
downloadable KB) is a **`Dataset`** (`subtype: knowledge_base`).

A minimal external-reference source node and an ingested-document source node:

```yaml
---
type: Agent                 # external-reference source: cited, not ingested
identifier: HGNC
subtype: organization
xref: [infores:hgnc]        # the authority lives in xref; no raw_source
---
```

```yaml
---
type: Study                 # ingested-document source: distilled from raw/
identifier: RECOVERY trial
subtype: rct
xref: [clinicaltrials:NCT04381936]
raw_source: [raw/pmid-33933206.pdf]
---
```

An edge then cites either by `identifier`: `primary_source: HGNC` or
`primary_source: RECOVERY trial`. **Name each external-reference source node canonically** (the
registry's short label: `HGNC`, `Gene Ontology`, `DrugBank`) and create it **once**, reusing that
`identifier` across every claim it supports; lint flags two source nodes sharing an `infores:`
`xref` as a duplicated authority to merge.

**`not_provided` is a rare escape, not a default.** If a claim's origin is *genuinely* unknown
(e.g. an unattributed legacy import), `primary_source: not_provided` is permitted and conformant,
but it is reserved for that rare case only. A claim the curating agent can trace to a nameable
source SHOULD name that source node; a bundle where `not_provided` is common is a lint smell, not a
valid pattern.

Quantitative claims (§7.3) are modeled as **attributes on a typed edge**, never lost. The
recurring biomedical pattern *population → exposure → outcome, quantified by an effect
measure with confidence bounds* becomes a single `predisposes_to`/`associated_with` edge
carrying `odds_ratio` \+ `ci_lower`/`ci_upper` \+ `p_value` \+ `sample_size` \+ `qualifiers.population`.

### 8.2 Source origin and credibility

When a source is converted (`bokf convert`, including `--url`/`--urls`), its `raw/<id>/meta.yaml`
records two orthogonal facts about where it came from, separate from the graph itself:

- **`source_type`** (origin): one of `journal_article`, `preprint`, `review`, `book`, `dataset`,
  `database`, `clinical_guideline`, `gov_report`, `web_page`, `personal`, `unknown`. It guides
  which source node the curator creates (a `Publication`, `Study`, or `Dataset`).
- **`credibility`** (trust): a `tier` (`peer_reviewed` > `preprint` > `archive` > `gray_lit` >
  `web` > `unknown`), plus `confidence`, a `retracted` flag, and a `reasoning` string. The verdict
  is computed by a deterministic-first waterfall: identifier extraction, then Crossref then OpenAlex
  DOI resolution, then URL host patterns, then a conservative text heuristic.

This keeps the §8.1 node-based provenance (which traces *which* source supports a claim) distinct
from *how credible* that source is. Lint surfaces weak evidence: `source.not_scholarly` warns when a
`primary_source` classifies as `web`/`unknown`, and `source.retracted` warns when it is retracted.

---

## 9\. Identifiers and namespaces

Every node's primary key is its human-readable, bundle-unique **`identifier`** (§7.1); there is
no separate CURIE `id` field. Choose a clear preferred name, keep it unique across the bundle and
free of `:` (reserved for CURIEs), and disambiguate collisions with a parenthetical facet
(e.g. `IL6 (gene)` vs `IL6 (protein)`). Edges reference a target by its `identifier`.

**External cross-references are optional.** Where a standard ontology CURIE is known, record it
under `xref` for interoperability and crosswalking. `xref` CURIEs use `prefix:local` from the
Biolink/Bioregistry prefix set. Recommended `xref` namespaces by type:

| Node type               | Preferred `xref` namespaces                                                                                                      |
| ----------------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| Gene                    | `HGNC`, `NCBIGene`, `ENSEMBL`                                                                                                    |
| Molecule                | `UniProtKB` (protein), `CHEBI`/`PUBCHEM.COMPOUND` (compound), `DRUGBANK`/`RXNORM` (drug), `CHEMBL`, `RNACENTRAL`/`ENSEMBL` (RNA) |
| MolecularClass          | `ATC`, `NCIT`, `INTERPRO`, `PFAM`, `MSIGDB`, ChEBI role                                                                          |
| Variant                 | `DBSNP` (rs…), `CLINVAR`, `dbVar`, HGVS                                                                                          |
| SequenceFeature         | `SO`, `ENSEMBL`, Ensembl Regulatory Build / ENCODE cCRE                                                                          |
| Structure               | `PDB`, `AlphaFoldDB`                                                                                                             |
| Anatomy / CellType      | `UBERON`, `CL`, `CLO`                                                                                                            |
| Organism                | `NCBITaxon`                                                                                                                      |
| BiologicalPathway       | `GO` (BP), `REACT`, `KEGG`, `WIKIPATHWAYS`, `RHEA`                                                                               |
| BiologicalFunction      | `GO` (MF)                                                                                                                        |
| Disease                 | `MONDO`, `DOID`, `OMIM`, `ICD10`, `MESH`                                                                                         |
| Phenotype               | `HP`, `MP`, `MEDDRA` (side effects), `SYMP`                                                                                      |
| BiomedicalMeasure       | `LOINC`, `NCIT`, `OBA`, `EFO`                                                                                                    |
| MethodOrProcedure       | `SNOMEDCT`, `CPT`, `NCIT`, `OBI`, `EDAM`                                                                                         |
| Exposure                | `ECTO`, `ExO`                                                                                                                    |
| SocialFactor            | Biolink SDoH, local                                                                                                              |
| Food                    | `FOODON`, `FooDB`                                                                                                                |
| MaterialSample          | `BioSample`, local                                                                                                               |
| Population              | `HANCESTRO`, local                                                                                                               |
| GeographicLocation      | `GeoNames`, `GADM`, `ISO-3166`                                                                                                   |
| Publication / Study     | `PMID`, `DOI`, `clinicaltrials` (NCT), `PMC`                                                                                     |
| Agent                   | `ORCID`, `ROR`, `infores`                                                                                                        |
| (cross-vocabulary glue) | `UMLS` (CUI) on `xref`                                                                                                           |

If no standard CURIE exists (a tweet, a bench-note observation, a novel compound), the
`identifier` alone suffices and provenance is recorded via a `reported_in` edge. Cross-namespace
equivalence is asserted with `xref` (treat as `same_as`/`exact_match`).

**Source databases are nodes, not provenance CURIEs (v0.5).** An `infores:` CURIE never appears in
`primary_source`; it lives in the `xref` of the **source node** (a
`Publication`/`Study`/`Dataset`/`Agent`) that represents that source (e.g. an `Agent` `HGNC` with
`xref: [infores:hgnc]`). Edges reference the
source by its `identifier`. See [§8.1](#81-provenance-is-node-based-v05).

---

## 10\. The ingest / query / lint workflow

BioOKF specializes the LLM Wiki's three operations. An agent following `SCHEMA.md` runs:

### Ingest

1. Place the source in `raw/` (immutable). It may be **any modality**: PDF, image,
slide deck, CSV, notebook, tweet, meeting note.

2. Read/parse it (OCR images, parse tables, transcribe slides).

3. **Extract typed entities** → for each, create or update a concept document with the
mandatory `type` (§5) and a human-readable, bundle-unique `identifier`, plus an
**agent-coined `subtype`**. If the entity already has a page, reuse its `identifier`;
don't fork (one identifier per entity).

4. **Curate external cross-references (`xref`), optional enrichment.** Where a standard
ontology CURIE is known, list it under `xref`; otherwise the `identifier` alone suffices.
Treat `xref` curation as best-effort: resolve what you can now, and **backfill the rest on
a later pass or during lint** (e.g. once a name resolves to an HGNC/MONDO/CHEBI CURIE).

5. **Extract typed relationships** → add `edges:` entries (§6), each `object` referencing the
target node's `identifier`, with the required provenance triplet (§8) and any quantitative
attributes (§7.3).

6. **Make the source a node and point provenance at it.** Create a `Publication`/`Study`/`Dataset`
node for the ingested source itself, giving it a `raw_source` that lists its `raw/…` path(s);
set each claim's `primary_source` to that node's `identifier` and link the entity to it with
`reported_in`. For a claim taken from an **external database you did not ingest** (HGNC, GO,
SemMedDB…), create a lightweight `Agent`/`Dataset` source node **once** (its `infores:` CURIE
in `xref`, no `raw_source`) and reuse its `identifier` as the `primary_source` everywhere.

7. Update `index.md`; append a dated entry to `log.md`. *A single source typically
touches 10 to 15 concept pages.*

### Query

Read `index.md` → open the relevant typed pages → traverse `edges:` → synthesize a cited
answer. Because edges are typed and attributed, queries can be **graph-shaped**
("drugs that `treat` a disease `associated_with` gene X, ranked by `knowledge_level`").
Good answers MAY be filed back as new concept pages.

### Lint

Periodic health check: contradictions (same subject-predicate-object with opposite
`negated` / incompatible effect sizes), stale claims, orphan pages, missing
cross-references, **plus BioOKF-specific checks**: invalid `type` (one of 28) / `predicate`
(one of 35: 24 positive + 11 `not_<X>`); **`identifier` validity: every node has one, each is unique across the bundle
(flag duplicates), and each is human-readable (flag opaque codes / bare CURIEs)**; that every
edge `object` resolves to some node's `identifier`; domain/range violations (e.g. a `treats`
edge whose object is a `Molecule` rather than a `Disease`/`Phenotype`); edges missing the
provenance triplet; **a `primary_source` that doesn't resolve to a source
node** (a `Publication`/`Study`/`Dataset`/`Agent`; flagged like any unresolved `object`, unless it
is the reserved value `not_provided`); and **source nodes that anchor to neither a `raw_source`
path nor an external `xref`** (an unanchored source, an enrichment opportunity). A
**missing external CURIE in `xref` is flagged as an enrichment opportunity, not a conformance
error** (only `type`/`identifier` are mandatory). `subtype` is *not* linted against a fixed list;
it is agent-coined.

---

## 11\. Conformance

A bundle is a conformant **BioOKF v0.5** bundle iff:

1. **(inherits OKF)** every non-reserved `.md` file has a parseable YAML frontmatter mapping
with a non-empty `type`; `index.md`/`log.md`, if present, follow their structures.

2. **Closed node vocabulary**: every `type` is one of the 28 values in §5.

3. **Closed edge vocabulary**: every `edges[].predicate` is one of the 35 values in §6 (24 positive + 11 `not_<X>`).

4. **Edge provenance**: every edge has `object`, `knowledge_level`, `agent_type`, and
`primary_source`; `primary_source` references a **source node** (a
`Publication`/`Study`/`Dataset`/`Agent`) by its `identifier`, not a CURIE.

5. **Identifiers**: every node has a `type` and an `identifier` that is **non-empty,
human-readable, and unique across the bundle**. Edges' `object` references a node's
`identifier`. External CURIEs (`xref`) are optional, not required.

Consumer rules:

- Consumers **MUST** tolerate broken cross-links (a linked concept may not yet exist).

- Consumers **SHOULD** validate `type`/`predicate`/domain-range **and `identifier`
uniqueness/human-readability**, and **flag** violations (BioOKF, unlike OKF, does *not*
silently accept unknown types; that is the whole point).

- Consumers **MUST NOT** reject a node for a missing `xref` or an unrecognized `subtype`; only
`type` and `identifier` are mandatory, and `subtype` is agent-coined open vocabulary.

- Producers **MAY** add extra frontmatter keys; consumers **MUST** preserve unknown keys.

- A document that cannot be typed within the 27 substantive types uses `type: Other` with a
`note`, never an invented type string.

A **lenient** consumer MAY accept rules 1 to 2 only (treating it as plain OKF) and ignore the
edge layer; a **strict** consumer enforces 1 to 5.

**Deprecated aliases.** For backward compatibility, consumers **SHOULD accept** the deprecated
`type` and attribute aliases in §14 on read and normalize them to the v0.5 names; producers
**SHOULD NOT** emit aliases. This lets v0.1/v0.2/v0.3/v0.4 and parallel-draft bundles validate
without rewriting.

---

## 12\. Worked example

`knowledge/molecule/tocilizumab.md`, distilled from a paper, a trial, and a drug label:

```yaml
---
type: Molecule
identifier: Tocilizumab
subtype: antibody
xref: [DRUGBANK:DB06273, CHEMBL:CHEMBL1237022, RXNORM:612865, UNII:I031V2H011]
synonyms: [atlizumab, Actemra]
in_taxon: NCBITaxon:9606
edges:
  - predicate: binds
    object: IL6 receptor (IL6R)
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: DrugBank
    effect_metric: Kd
    effect_size: 2.5            # nM
  - predicate: regulates
    object: IL6 signaling
    direction: decreased
    aspect: activity
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: DrugBank
  - predicate: treats
    object: rheumatoid arthritis
    clinical_phase: approved
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: DrugCentral
  - predicate: treats
    object: COVID-19
    clinical_phase: approved
    knowledge_level: statistical_association
    agent_type: data_analysis_pipeline
    primary_source: RECOVERY trial
    effect_metric: relative_risk
    effect_size: 0.85
    ci_lower: 0.76
    ci_upper: 0.94
    sample_size: 4116
    publications: [PMID:33933206]
  - predicate: has_phenotype             # adverse effect
    object: neutropenia
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: SIDER
    frequency: common
  - predicate: member_of
    object: IL6 inhibitors                # MolecularClass (subtype: pharmacologic)
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: ATC
  - predicate: reported_in
    object: RECOVERY trial
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: RECOVERY trial
---

# Tocilizumab

A recombinant humanized monoclonal antibody against the interleukin-6 receptor (IL-6R),
blocking IL-6 signaling. First approved for rheumatoid arthritis; repurposed during
COVID-19 (RECOVERY, REMAP-CAP).

## Citations
- [Tocilizumab in patients admitted to hospital with COVID-19 (RECOVERY)](https://pubmed.ncbi.nlm.nih.gov/33933206/) (PMID:33933206)
```

One document, one node type, seven typed edges spanning five edge families, every edge
provenance-stamped, the COVID-19 efficacy edge carrying full trial statistics, all
type-checkable against the controlled universes. Each `object` is the target node's
human-readable `identifier`; external CURIEs (e.g. `DRUGBANK:DB06273`) ride in `xref`, not as the
key. Note `member_of` targets a **MolecularClass** node, not a `Molecule`. Every `primary_source`
likewise names a **source node** by its `identifier`: `DrugBank`, `DrugCentral`, `SIDER`, and `ATC`
are external-reference `Agent`/`Dataset` nodes (their `infores:` CURIE in `xref`, no `raw_source`),
while `RECOVERY trial` is an ingested-document `Study` node carrying a `raw_source` into `raw/`
([§8.1](#81-provenance-is-node-based-v05)).

---

## 13\. Design decisions and alternatives

The full rationale, prior-art mapping, and source evidence live in
[docs/03-rationale.md](docs/03-rationale.md). The load-bearing decisions:

- **Gene + Molecule + Variant + SequenceFeature kept distinct (not all "Molecule").** The brief
noted IL6 *"is a molecule but also a gene."* We keep these as separate types because (a) the
gene→product edge `encodes` and feature/variant→trait edges are the most frequent,
highest-value claims in the corpus, and (b) gene/protein/feature/variant are genuinely
distinct identities that every downstream KG (Biolink, SPOKE, Hetionet, PrimeKG, PubTator3)
treats as first-class. `Molecule` remains the broad umbrella for *everything else chemically
defined* (protein, drug, compound, metabolite, complex, RNA species).

- **`Variant` split from `SequenceFeature`.** A variant is a *deviation from reference*; a
feature is a *region of the reference*; SO, Biolink, GA4GH VRS, and Ensembl all keep them
apart, and their edges differ (variant `predisposes_to`; enhancer `regulates`).
`SequenceFeature` is scoped to functional/regulatory elements; structural partonomy
(exon/intron/CDS/codon) and variant consequence are edges/attributes, not nodes.

- **`BiologicalProcess` split into `BiologicalPathway` \+ `BiologicalFunction`.** GO's process and
molecular-function branches are distinct (Biolink `BiologicalProcess` vs `MolecularActivity`;
SPOKE keeps both). The v0.1 name also collided with the GO BP branch. Reactions live under
`BiologicalPathway` (`catalyzes` points there); cellular component remains `Anatomy(subcellular)`.

- **`ClinicalMeasure` → `BiomedicalMeasure`.** "Clinical" was too narrow; the type now covers
experimental biomarkers, scales, and omics readouts. The dual-nature cases (BMI, LDL,
hyperlipidemia) resolve by **identity** via the §5.C facet rule: the variable is a
`BiomedicalMeasure`, the qualitative state a `Phenotype`, the diagnosis a `Disease`, the value
an edge attribute: distinct facets, distinct CURIEs, one node each, linked by edges. No
separate `Trait` type; "trait" is a role grounded via EFO/OBA `xref`.

- **`ExposureOrFactor` dissolved.** "Risk factor" is a role, not an identity, so it lives on the
edge (`predisposes_to`/`prevents`), whose subject may be any type. The residue splits by
identity into `Exposure`, `SocialFactor`, `Food`, `Population`, `GeographicLocation`;
defined-compound exposures route to `Molecule`, infectious ones to `Organism`.

- **`SDOH` → `SocialFactor` (v0.4).** The type covers the social determinants of health, but the
acronym read as jargon and as one fixed external vocabulary; `SocialFactor` names the identity
plainly (income, education, housing, employment, food security, access to care, social
support) while still mapping to Biolink `SocialDeterminantOfHealth` for interoperability.

- **`Device` split from `MaterialSample`; `Structure` added.** Biolink/OBI keep engineered
artifacts, biospecimens, and 3D structures distinct.

- **`Procedure` \+ `Method` merged into `MethodOrProcedure`.** The same assay/technique was both;
the clinical-vs-computational and act-vs-recipe distinction is carried by `subtype`, and a
specific *execution* is an edge, not a node.

- **`MolecularClass` added.** Drug classes, protein families/domains, and gene sets are
*groupings*, not single molecules; one type with `subtype` (members via `member_of`).

- **`Disease` / `Phenotype` / `BiomedicalMeasure` kept separate, with an explicit boundary
(§5.C)** and the other entity boundaries made explicit (§5.D). Their overlaps are inherent to
biomedicine, not flaws in the type set; the fix is facet-based decision rules grounded in OMOP
CDM and OBO/Monarch, plus the "one node per facet, linked by edges" convention.

- **Type controlled; one merged `identifier`; subtype agent-coined (v0.4).** Only `type` is a
closed, mandatory vocabulary. The former `title` and CURIE `id` collapse into a single
human-readable, bundle-unique `identifier` (the primary key edges target); external CURIEs
become optional `xref` enrichment, so a fact from a tweet or a novel compound is never blocked
for lack of an ontology id. The former per-type `*_kind` (and `class_basis`, and Structure's
`method`) collapse into one `subtype` field that has **no controlled universe**: the agent
coins it per node. This keeps the controlled surface tiny and lets sources name
sub-granularity freely without schema churn.

- **Node-based provenance (v0.5).** `primary_source` references a source *node* by its `identifier`
rather than an `infores:` CURIE (and the redundant node-level `provided_by` is dropped in favour
of the `reported_in` edge), completing v0.4's move off CURIE primary keys so the bundle has
**one** identity mechanism (human-readable identifiers) instead of two (identifiers for entities,
CURIEs for sources). Sources become first-class nodes that anchor to `raw/` via
`raw_source` (ingested documents) or to an external authority via `xref` (reference databases), so
every claim is traceable to immutable bytes or a named authority and provenance is queryable like
any other edge. The cost (a lightweight node per external database, reused across its claims)
buys a provenance chain that terminates inside the bundle. Alternatives weighed: keeping an
`infores:` CURIE on `primary_source` (rejected as a second, parallel identity space) and dropping
`primary_source` to rely on the node-level `reported_in` edge alone (rejected because it loses
*per-claim* attribution when one page carries claims from many sources).

- **35 edges (24 positive + 11 `not_<X>`) via qualify-don't-multiply.** Direction, affinity,
temporal order, and per-source identity are *attributes*, not separate predicates: the single most
important lever for keeping the edge set small yet exhaustive (negation is the deliberate exception,
carried as its own canonical `not_<X>` predicate rather than an attribute, §6.F). The 28-type
universe added **no** predicates; every refinement is a domain/range extension or an attribute.

---

## 14\. Changelog and deprecated aliases

> **Lineage note.** Two parallel drafts existed at the 0.2/0.3 stage: a **rename-only** line
> (20 types: `Variant`→`GenomicFeature`, `BiologicalProcess`→`Process`, + the §5.C boundary
> rule) and a **granular-expansion** line (20 → 28 types). v0.4 adopts the **28-type** universe
> as the node design and folds in the boundary / class-vs-instance / alias refinements from the
> rename line, then applies the v0.4 naming + clarity changes below. The deprecated-alias table
> lets bundles from *either* lineage validate without rewriting.

### v0.5.1 (2026-06-27): predicate reconciliation (24 positive + 11 negative → 35)

- **`used_to_study` added** (24th positive predicate) and **11 `not_<X>` negatives** for the
  negatable effect predicates (`binds`, `interacts_with`, `causes`, `predisposes_to`, `prevents`,
  `treats`, `affects_response_to`, `associated_with`, `expressed_in`, `regulates`, `has_phenotype`)
  → **35 predicates** (24 positive + 11 negative). `SCHEMA.md` and `bokf-core` are authoritative;
  §6 now enumerates the 24 positive predicates (the `used_to_study` row in §6.E) and the 11
  negatives (§6.F Negation). A legacy `negated: true` qualifier normalizes to `not_<X>` on read.
  Negating a non-negatable (structural/provenance) predicate is rejected (`edge.not_negatable`).

### v0.5 (2026-06-27): node-based provenance (no entities/predicates added; 28 / 23 unchanged)

- **An edge's `primary_source` now references a source *node* by its `identifier`**, not an
`infores:` CURIE. A *source node* is a `Publication`/`Study`/`Dataset`/`Agent` (the four
Provenance & Context types that bear a source), traversable like any other concept and the same
node the `reported_in` edge targets. The reserved value `not_provided` is a **rare escape** for
genuinely unknown-origin claims, never a default.

- **Dropped `provided_by`.** A node-level "this page came from X" is now simply a `reported_in`
edge, leaving exactly two provenance mechanisms (per-edge `primary_source` \+ the `reported_in`
edge). A `reported_in` edge's own `primary_source` is, by convention, its own `object` (the
terminating base case).

- **Added an optional `raw_source` field on source nodes**: a list of bundle-root-relative `raw/…`
paths anchoring a source node to the immutable bytes it was distilled from, so every claim's
provenance chain terminates in `raw/` ([§8.1](#81-provenance-is-node-based-v05)).

- **Made the two source-node species explicit**: *ingested-document* sources (a `Publication`/
`Study`/`Dataset` with `raw_source`) vs *external-reference* databases (an `Agent: organization`
or `Dataset: knowledge_base` with no `raw_source`; the `infores:`/ontology CURIE in `xref`).

- **Confirmed predicates are forward-only.** The 23 are a closed, forward-only set; inverse names
(`encoded_by`, `caused_by`, `treated_by`, `produces`…) are **not** legal predicates; author the
forward edge on the other node. Inverse names are accepted on read only as deprecated aliases.

- **Lint additions:** `primary_source` must resolve to a source node (flagged like any unresolved
`object`), and every source node should anchor to either a `raw_source` path or an external
`xref`.

- Old `infores:`-valued `primary_source` is **accepted on read** and normalized to a source node
carrying that CURIE in `xref`; legacy `provided_by` normalizes to a `reported_in` edge (see the
deprecated-alias table). The node/edge counts (28 / 23) and every v0.4 change are otherwise
unchanged.

### v0.4 (2026-06-27): naming + clarity (no entities added/removed; 28 / 23 unchanged)

- **Renamed node type `SDOH` → `SocialFactor`.** Same umbrella (social determinants of health:
income, education, housing, employment, food security, access to care, social support); still
maps to Biolink `SocialDeterminantOfHealth`. `SDOH`/`SDoH` accepted on read as an alias.

- **Replaced the per-type `*_kind` attribute family with a single `subtype` attribute that has
no controlled universe**: the agent coins it per node. The §5 values are examples, not an
enum; consumers MUST NOT reject an unrecognized `subtype`. `class_basis` (MolecularClass) and
Structure's `method` fold into `subtype`; old `*_kind`/`class_basis`/`method` keys are accepted
on read and normalized to `subtype`. Every node should carry a `subtype`, but it is never
validated.

- **Merged `title` and `id` into a single `identifier`.** A node's primary key is now one
human-readable, **bundle-unique** `identifier` (no separate CURIE `id`); edges reference a
target by its `identifier` (`object`). **Only `type` and `identifier` are mandatory**: every
other field, including the equivalent external CURIEs now kept in optional `xref`, is optional.
**Lint checks the `identifier` is unique and human-readable.** Previously some types also
required their `*_kind` / `in_taxon` / `note`; those are recommended, not required. External
CURIE (`xref`) curation is reframed as best-effort enrichment in the ingest workflow (§10):
curate what resolves now, backfill the rest later, and a missing CURIE is a lint *enrichment
opportunity*, not a conformance error.

- **Node-boundary review pass**: added **§5.D** with a single discriminating identity test for
every historically fuzzy pair (Gene vs Molecule(RNA); Molecule vs MolecularClass; Variant vs
SequenceFeature; BiologicalPathway vs BiologicalFunction; Structure vs Molecule;
Anatomy(subcellular) vs CellType; Exposure vs SocialFactor vs Food; Device vs MaterialSample;
Population vs Study vs Organism; MethodOrProcedure vs its I/O; Publication vs Study vs Dataset;
Concept vs Other), so each of the 28 types has a crisp boundary.

### v0.3 (2026-06-26): the 28-type expansion (edges unchanged, 23)

- **20 → 28 node categories.** `Molecule` spun off `MolecularClass` (groupings);
`GenomicFeature` split into `Variant` \+ `SequenceFeature`; `Process` split into
`BiologicalPathway` \+ `BiologicalFunction`; `ClinicalMeasure` renamed/extended to
`BiomedicalMeasure`; `ExposureOrFactor` dissolved into `Exposure` \+ `SDOH` \+ `Food` \+
`Population` \+ `GeographicLocation` (risk factor reframed as an edge role); `Device` split
into `Device` \+ `MaterialSample`; `Structure` added; `molecule_kind: rna` (now `subtype: rna`)
added for RNA species.

- Merged from the rename-line draft: the **§5.C** `Disease`/`Phenotype`/`BiomedicalMeasure`
boundary rule (facet-based; OMOP CDM + OBO/Monarch grounding; "one node per facet, linked by
edges"; CURIE namespace pins the facet); the **class-vs-instance** typing rule (§5); and the
**deprecated-alias** mechanism (§11 + table below).

- The 23 edge predicates are **unchanged**: every refinement was absorbed by `subtype`
attributes and by domain/range extensions, with no new predicates.

### v0.2 (2026-06-26): the rename-line draft (20 / 23)

- **Renamed `Variant` → `GenomicFeature`** and broadened it to cover constitutive sequence
features (UTR, codon, exon, intron, CDS, splice site, promoter, enhancer, silencer, CpG
island, transposon, regulatory region, locus, TAD, motif) alongside variants, via
`feature_kind`. Nucleic-acid only; protein domains/families stayed in `Molecule`.

- **Renamed `BiologicalProcess` → `Process`** to remove the collision with GO's
`biological_process`; umbrella over pathway / reaction / molecular function / physiologic &
pathologic process / behavior.

- **Added the §5.C** `Disease` vs `Phenotype` vs `ClinicalMeasure` boundary rule, and made the
**class-vs-instance** rule explicit. Node/edge counts unchanged (20 / 23). *(In v0.4 these
ideas live on in the 28-type universe as the `Variant`/`SequenceFeature` split and the
`BiomedicalMeasure` boundary.)*

### v0.1 (2026-06-25)

- Initial draft: 20-node / 23-edge universes, provenance model, conformance, the
ingest/query/lint workflow, grounded in UMLS / Biolink / SPOKE / Hetionet / SemMedDB and a
1,100+ item biomedical source survey.

### Deprecated `type` aliases (accepted on read, normalized to v0.5)

| Deprecated name                  | Source           | Normalizes to                                                                         |
| -------------------------------- | ---------------- | ------------------------------------------------------------------------------------- |
| `SDOH`, `SDoH`                   | v0.3             | `SocialFactor`                                                                        |
| `GenomicFeature`                 | v0.2 rename line | `Variant` / `SequenceFeature` by `subtype`                                            |
| `Variant` (as the v0.1 umbrella) | v0.1             | `Variant` (kept) **or** `SequenceFeature` by `subtype`                                |
| `Process`                        | v0.2 rename line | `BiologicalPathway` (or `BiologicalFunction` if MF)                                   |
| `BiologicalProcess`              | v0.1             | `BiologicalPathway` (or `BiologicalFunction` if MF)                                   |
| `ClinicalMeasure`                | v0.1 / v0.2      | `BiomedicalMeasure`                                                                   |
| `ExposureOrFactor`               | v0.1 / v0.2      | `Exposure` / `SocialFactor` / `Food` / `Population` / `GeographicLocation` by content |
| `Procedure`, `Method`            | v0.1 / v0.2      | `MethodOrProcedure`                                                                   |

### Deprecated attribute aliases (accepted on read, normalized to v0.5)

| Deprecated key                                                                                                          | Normalizes to                                                                                                                                                                                                                                  |
| ----------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `title`                                                                                                                 | `identifier`                                                                                                                                                                                                                                   |
| `id` (CURIE primary key)                                                                                                | `identifier` (the human-readable key) + `xref` (the CURIE)                                                                                                                                                                                     |
| `primary_source: infores:X` (edge, CURIE value)                                                                         | `primary_source: <source node identifier>`: synthesize one source node per distinct CURIE (`type: Agent`, a human-readable `identifier` derived from the CURIE label, `xref: [infores:X]`); all edges bearing that CURIE resolve to it (v0.5) |
| `provided_by` (node field, any value, removed in v0.5)                                                                 | a `reported_in` edge from the node to that source node (v0.5)                                                                                                                                                                                  |
| `encoded_by` / `caused_by` / `treated_by` / `produces` and other inverse predicate names                                | the **forward** predicate (`encodes` / `causes` / `treats` / `catalyzes`) authored on the *other* node (v0.5; predicates are forward-only)                                                                                                     |
| `<type>_kind` (e.g. `molecule_kind`, `feature_kind`, `phenotype_kind`, `measure_kind`, `method_kind`, `factor_kind`, …) | `subtype`                                                                                                                                                                                                                                      |
| `class_basis` (MolecularClass)                                                                                          | `subtype`                                                                                                                                                                                                                                      |
| `method` (Structure)                                                                                                    | `subtype`                                                                                                                                                                                                                                      |

---

*BioOKF v0.5 is a draft. It is a community profile of OKF for biomedicine and is not an
official Google or UCSF product. Contributions welcome.*
