# BioOKF operating schema (v0.5)

> This is the **agent-facing** schema document for a BioOKF knowledge base — the analogue of
> a `CLAUDE.md` for an LLM Wiki. Drop it at the root of any BioOKF bundle. It tells an LLM
> agent (or a human) the conventions and the workflows to follow. The normative format is
> [SPEC.md](SPEC.md); this is the operational distillation. **v0.5** keeps the **28** typed node
> categories, takes the edge predicates to **24** positive (adds `used_to_study`) plus **11** `not_<X>` negatives (**35** total), and makes **provenance node-based**: an edge's
> `primary_source` now names a source **node** by its `identifier` — one of the bundle's own
> `Publication`/`Study`/`Dataset`/`Agent` nodes — instead of an `infores:` CURIE; source nodes anchor
> to the immutable bytes via an optional **`raw_source`** (`raw/…` paths); the old node-level
> `provided_by` is dropped (use a `reported_in` edge). It
> builds on v0.4 (`SDOH`→`SocialFactor`; per-type `*_kind` → one agent-coined `subtype` with no
> controlled universe; `title` \+ `id` merged into one human-readable, bundle-unique `identifier`,
> CURIEs moved to optional `xref`; the §5.D boundary pass).

## What this knowledge base is

A BioOKF bundle: a Git-shippable tree of Markdown files. `raw/` holds immutable sources;
`knowledge/` holds the typed concept documents you author (the graph); `index.md` is the
catalog; `log.md` is the history. You **own** `knowledge/`, `index.md`, and `log.md`. Never
edit anything in `raw/`.

## The two rules that make this BioOKF (not just OKF)

1. **Every concept document's `type` is one of these 28 values — nothing else.**
*Biomedical entities (20):* `Gene`, `Molecule`, `MolecularClass`, `Variant`,
`SequenceFeature`, `Structure`, `Anatomy`, `CellType`, `Organism`, `BiologicalPathway`,
`BiologicalFunction`, `Disease`, `Phenotype`, `BiomedicalMeasure`, `MethodOrProcedure`,
`Exposure`, `SocialFactor`, `Food`, `Device`, `MaterialSample`.
*Provenance & context (8):* `Publication`, `Study`, `Dataset`, `Agent`, `Population`,
`GeographicLocation`, `Concept`, `Other`.
If something fits none, use `Other` with a `note:` — never invent a type.

2. **Every relationship is a typed `edges:` entry whose `predicate` is one of these 24 positive predicates (a negative finding uses a `not_<X>` negative — see Negation):**
`is_a`, `part_of`, `member_of`, `derives_from`, `located_in`, `expressed_in`, `encodes`,
`interacts_with`, `binds`, `regulates`, `catalyzes`, `converts_to`, `participates_in`,
`causes`, `predisposes_to`, `treats`, `prevents`, `contraindicated_in`,
`affects_response_to`, `has_phenotype`, `measures`, `associated_with`, `used_to_study`,
`reported_in`.
Direction is always **this document → object**. The 24 are **forward-only** — there are no
inverse predicates; to express a reverse relation, author the forward edge on the other node
(a gene's `encodes`, never a protein's `encoded_by`).

**Negation (polarity).** A genuine *negative* finding stated in the source — "X does **not** treat
Y", "**no** association between X and Y", "drug A does **not** bind target B" — is authored with the
canonical negative predicate **`not_<X>`**. Only the **11 effect predicates** that are actually
tested-and-refuted in source text are negatable: `binds`, `interacts_with`, `causes`,
`predisposes_to`, `prevents`, `treats`, `affects_response_to`, `associated_with`, `expressed_in`,
`regulates`, `has_phenotype` — giving 11 `not_*` predicates (**35 total**). Negating a
structural/definitional/provenance predicate (`is_a`, `part_of`, `encodes`, `measures`,
`reported_in`, `used_to_study`, …) is meaningless under open-world semantics — absence already
covers it — and is rejected. A `not_<X>` **inherits `<X>`'s domain/range and symmetry**; asserting
both `<X>` and `not_<X>` for the same subject→object is a contradiction. (A legacy `negated: true`
qualifier on a negatable predicate is accepted on read and normalized to `not_<X>`.)

## Type vs subtype

**`type` is the mandatory, controlled concept; `subtype` is the one you coin yourself.** Exactly
one `type` per document, drawn from the 28 above. Always also give a `subtype` — but there is
**no fixed universe** for subtypes: you invent an appropriate lowercase token per node (e.g.
`subtype: protein`, `subtype: enhancer`, `subtype: sign`). The values in the cheatsheet are just
examples you may reuse or replace; a consumer never validates or rejects a node over its
`subtype`. Only `type` and `predicate` are checked against a closed universe.

**Only `type` and `identifier` are mandatory.** `identifier` merges the old `title` and `id` into
one field that must be **human-readable** *and* **unique across the bundle**; edges reference a
target by its `identifier` (the `object` field). Everything else is optional — record equivalent
external ontology CURIEs in `xref` when you know them, otherwise the `identifier` alone is enough.

## Typing cheatsheet (what goes where)

| If the source is talking about…                                                                                           | `type`               | `subtype` examples (agent-coined)                                                                                                                                   | namespaces for the optional `xref`                              |
| ------------------------------------------------------------------------------------------------------------------------- | -------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------- |
| a gene / locus / ncRNA-miRNA gene (heritable unit)                                                                        | `Gene`               | `protein_coding`/`ncRNA`/`miRNA`                                                                                                                                    | HGNC, NCBIGene, ENSEMBL                                         |
| a protein, drug, compound, metabolite, antibody, ion, complex, **RNA species (mRNA/miRNA/lncRNA)**                        | `Molecule`           | `protein`/`drug`/`small_molecule`/`metabolite`/`antibody`/`complex`/`ion`/`rna`                                                                                     | UniProtKB, CHEBI, PUBCHEM, DRUGBANK, RXNORM, CHEMBL, RNACENTRAL |
| a **grouping**: drug class, protein family/domain, gene set/signature, chemical class                                     | `MolecularClass`     | `pharmacologic`/`protein_family`/`protein_domain`/`gene_set`/`chemical_class`                                                                                       | ATC, NCIT, INTERPRO, PFAM, MSIGDB                               |
| a **deviation from reference**: SNV, indel, CNV/SV, allele, genotype, haplotype, fusion, STR                              | `Variant`            | `snv`/`indel`/`cnv`/`allele`/`haplotype`/`fusion` (consequence = attribute, SO/VEP)                                                                                 | DBSNP, CLINVAR, dbVar, HGVS                                     |
| a **reference functional/regulatory element**: enhancer, promoter, silencer, TFBS, CpG island, open chromatin, transposon | `SequenceFeature`    | `enhancer`/`promoter`/`silencer`/`tfbs`/`cpg_island`/`transposon`                                                                                                   | SO, ENSEMBL, Ensembl Regulatory Build / ENCODE cCRE             |
| a resolved/predicted **3D atomic structure**                                                                              | `Structure`          | `xray`/`cryo_em`/`nmr`/`predicted` (+ `resolution`)                                                                                                                 | PDB, AlphaFoldDB                                                |
| an organ, tissue, body region, organelle, body fluid                                                                      | `Anatomy`            | `organ`/`tissue`/`subcellular`/`body_fluid`                                                                                                                         | UBERON                                                          |
| a cell type / state / line / organoid                                                                                     | `CellType`           | `cell_type`/`cell_state`/`cell_line`/`organoid`                                                                                                                     | CL, CLO                                                         |
| a species, strain, pathogen, microbe                                                                                      | `Organism`           | `species`/`strain`/`pathogen`                                                                                                                                       | NCBITaxon                                                       |
| a pathway, reaction, signaling cascade, GO biological process, physiologic/pathologic process, behavior                   | `BiologicalPathway`  | `pathway`/`reaction`/`signaling`/`go_bp`/`physiologic`/`pathologic`/`behavior`                                                                                      | GO, REACT, KEGG, WIKIPATHWAYS, RHEA                             |
| a **molecular function** (catalytic / binding / transporter activity)                                                     | `BiologicalFunction` | `catalytic`/`binding`/`transporter`                                                                                                                                 | GO (MF)                                                         |
| a disease, syndrome, infection, injury, cancer (a **diagnosis**)                                                          | `Disease`            | `infection`/`neoplasm`/`syndrome`/`injury`                                                                                                                          | MONDO, DOID, OMIM, ICD10, MESH                                  |
| a symptom, **sign**, side effect, qualitative trait, morphologic/behavioral feature                                       | `Phenotype`          | `symptom`/`sign`/`side_effect`/`trait`/`morphologic`/`behavioral`                                                                                                   | HP, MP, MEDDRA                                                  |
| a **measurable variable**: lab test, vital, score, scale, biomarker, omics readout, imaging finding                       | `BiomedicalMeasure`  | `lab_test`/`vital`/`score`/`scale`/`biomarker`/`omics_readout`/`imaging_finding`                                                                                    | LOINC, NCIT, OBA, EFO                                           |
| a clinical procedure, lab assay/technique, pipeline, software, **statistical method (e.g. PCA)**, model, protocol         | `MethodOrProcedure`  | `surgical`/`imaging`/`vaccination`/`screening`/`diagnostic`/`lab_assay`/`lab_protocol`/`computational_pipeline`/`software`/`algorithm`/`statistical_method`/`model` | SNOMEDCT, CPT, NCIT, OBI, EDAM                                  |
| a behavioral / environmental / occupational / dietary-pattern **exposure**                                                | `Exposure`           | `behavioral`/`environmental`/`occupational`/`dietary`                                                                                                               | ECTO, ExO                                                       |
| a **social factor** affecting health (income, education, housing, access to care)                                         | `SocialFactor`       | `economic`/`education`/`housing`/`healthcare_access`/`social_support`                                                                                               | Biolink SDoH, local                                             |
| a food item, food group, dietary product                                                                                  | `Food`               | `food_item`/`food_group`/`dietary_product`                                                                                                                          | FooDB, FOODON                                                   |
| a device, implant/prosthesis/graft/mesh, instrument, reagent/kit                                                          | `Device`             | `implant`/`prosthesis`/`instrument`/`reagent`                                                                                                                       | local                                                           |
| a **biospecimen / material sample** (serum, biopsy, aliquot, cell-line stock)                                             | `MaterialSample`     | `serum`/`biopsy`/`aliquot`/`cell_line_stock`                                                                                                                        | BioSample                                                       |
| a paper, preprint, slide deck, blog, **tweet**, **bench/meeting note**                                                    | `Publication`        | `article`/`preprint`/`slide_deck`/`tweet`/`lab_notebook`                                                                                                            | PMID, DOI, PMC                                                  |
| a clinical trial, cohort study, GWAS, registry                                                                            | `Study`              | `rct`/`cohort`/`gwas`/`registry`                                                                                                                                    | clinicaltrials (NCT)                                            |
| a dataset, CSV/XLSX, omics matrix, image set, knowledge base                                                              | `Dataset`            | `table`/`omics_matrix`/`image_collection`/`knowledge_base`                                                                                                          | —                                                               |
| a person/author/curator, lab, org, company, regulator                                                                     | `Agent`              | `person`/`lab`/`organization`/`company`/`regulator`                                                                                                                 | ORCID, ROR, infores                                             |
| a **group of people**: cohort, study population, ancestry, demographic group                                              | `Population`         | `cohort`/`ancestry`/`demographic`                                                                                                                                   | HANCESTRO, local                                                |
| a country, region, place                                                                                                  | `GeographicLocation` | `country`/`region`/`place`                                                                                                                                          | GeoNames, GADM, ISO-3166                                        |
| an abstract score/classification/unit/ontology term                                                                       | `Concept`            | `unit`/`classification`/`ontology_term`                                                                                                                             | —                                                               |
| genuinely none of the above                                                                                               | `Other`              | — (add a `note:` saying why)                                                                                                                                        | —                                                               |

## Disease vs Phenotype vs BiomedicalMeasure (the tricky trio)

The same word can span all three; pick by **facet**, and if more than one applies, make **one
node per facet and link them** (the `xref` CURIE namespace pins the facet — the same rule that
separates a gene from its protein). Grounded in OMOP CDM (Measurement vs Condition) and
OBO/Monarch (disease-disposition vs phenotype-manifestation).

- **`BiomedicalMeasure`** — has a unit/value or comes from a test/score (LOINC/OBA/EFO).
*body height, BMI, LDL cholesterol, PRS, body temperature, a flow-cytometry %.*

- **`Phenotype`** — an observed sign/symptom/abnormal trait (HPO). *tall stature, obesity (the
feature), elevated lipids, fever (sign), pain (symptom).* `subtype`: `symptom` =
subjective/patient-reported; `sign` = objective/observer-detected.

- **`Disease`** — a diagnosed condition the patient *has* (MONDO / SNOMED "(disorder)").
*hyperlipidemia (the disorder), hypertension, obesity disorder.*

Connect them: `Disease` `has_phenotype` `Phenotype`; `BiomedicalMeasure` `measures`
`Phenotype`/`Disease`. A raw number ("183 cm") is **edge data, never a node**. When a word is
dual-listed (hyperlipidemia = `MONDO:0021187` disease + `HP:0003077` phenotype + `LOINC:13457-7`
analyte), the **external CURIE (in `xref`) tells you the facet**. Full rule:
[SPEC.md §5.C](SPEC.md#5c-disease-vs-phenotype-vs-biomedicalmeasure--the-boundary).

## Boundaries between the other types (one test each)

Type by **identity, not role**. The historically fuzzy pairs each resolve to a single test
([SPEC.md §5.D](SPEC.md#5d-boundaries-between-the-entity-types)):

- **Gene vs Molecule(RNA):** heritable locus = `Gene`; a specific transcript = `Molecule` (`subtype: rna`).

- **Molecule vs MolecularClass:** one entity = `Molecule`; a *grouping* (class/family/domain/gene-set) = `MolecularClass`; members link by `member_of`.

- **Variant vs SequenceFeature:** *deviation from reference* = `Variant`; *region of reference* = `SequenceFeature`. (exon/intron/CDS/codon = a `part_of` edge, not a node.)

- **BiologicalPathway vs BiologicalFunction:** a *process* = `BiologicalPathway`; an *elemental molecular activity* (GO-MF) = `BiologicalFunction`.

- **Structure vs Molecule:** chemical entity = `Molecule`; its 3D coordinate model = `Structure` (`derives_from` the Molecule).

- **Exposure vs SocialFactor vs Food:** behavioral/environmental/occupational/dietary exposure = `Exposure`; social/economic/structural determinant = `SocialFactor`; a food item/group = `Food`. ("Risk factor" is the `predisposes_to` role, never a type.)

- **Device vs MaterialSample:** engineered artifact = `Device`; biospecimen = `MaterialSample`.

- **Population vs Study vs Organism:** group of people = `Population`; the investigation = `Study`; a taxon/species/strain = `Organism`.

- **Publication vs Study vs Dataset:** the document/artifact = `Publication`; the designed investigation = `Study`; the data file/matrix = `Dataset`.

- **Concept vs Other:** abstract attribute/unit/classification = `Concept`; a biomedical thing fitting none of the 27 substantive types = `Other` (with `note:`).

## Required fields (keep it minimal)

- **Node — mandatory:** `type` and `identifier` (human-readable **and** unique across the
bundle). Nothing else is required.

- **Node — always supply (but agent-coined):** a `subtype`. There is no controlled list; you
invent it.

- **Node — recommended where available:** equivalent external CURIEs in `xref` (curate when
known — see [SPEC.md §9](SPEC.md#9-identifiers-and-namespaces)); `in_taxon` for
`Gene`/`Organism`; `note` for `Other`; **`raw_source`** (one or more `raw/…` paths) on a source
node (`Publication`/`Study`/`Dataset`) you created for an ingested document.

- **Edge — mandatory:** `predicate`, `object` (the target node's `identifier`),
`knowledge_level`, `agent_type`, `primary_source` (the `identifier` of a source node — a
`Publication`/`Study`/`Dataset`/`Agent` — **not** a CURIE).

Everything else (synonyms, statistics, qualifiers) is optional but **encouraged** — and when you
have a number (p-value, OR/HR, IC50, fold-change, sensitivity), put it on the edge as a named
attribute, never only in prose. See [SPEC.md §7](SPEC.md#7-attributes-required-vs-optional).

## Edges: domain/range notes (24 positive predicates; `not_<X>` negatives inherit)

The positive predicate set is **24** as of v0.5 — the v0.1–v0.4 core of 23 plus `used_to_study` —
plus **11** `not_<X>` negatives for the negatable effect predicates (**35 total**); each `not_<X>`
inherits its base predicate's domain/range and symmetry. The 28
node types ride these predicates via domain/range extensions — the key ones:

- `located_in` — anatomical **or** genomic **or** geographic: domain += `Variant`,
`SequenceFeature`; range += `Gene`, `SequenceFeature`, `GeographicLocation`.

- `part_of` — domain += `SequenceFeature`, `Variant`; range += `Gene`, `SequenceFeature`
(structural partonomy: exon/intron/CDS/codon expressed here, not as nodes).

- `member_of` — range += `MolecularClass`.

- `regulates` — domain += `SequenceFeature`; range += `BiologicalFunction`.

- `binds` — range += `SequenceFeature` (TF → TFBS/enhancer).

- `participates_in` / `catalyzes` — range = `BiologicalPathway` / `BiologicalFunction`.

- `predisposes_to` / `prevents` — **broad domain**: any factor as subject (`Variant`, `Gene`,
`Molecule`, `Exposure`, `SocialFactor`, `Food`, `Disease`, `BiomedicalMeasure`, `Phenotype`)
→ `Disease`, `Phenotype`, `BiomedicalMeasure`.

- `derives_from` — domain += `MaterialSample` (← Organism/Anatomy/donor), `Structure`
(← Molecule), `Food` (← Organism), `Population` (← source).

- `measures` — domain `BiomedicalMeasure`/`MethodOrProcedure` → `Disease`/`Phenotype`/`Molecule`.

- `used_to_study` — an investigative resource → the entity it is used to study, model, probe, or
make tractable. Domain: `MethodOrProcedure`, `Study`, `Dataset`, `Device`, and the tangible
research-model nodes (`Organism`, `CellType`, `MaterialSample` — e.g. an organoid strain or model
line). Range: any biomedical entity under investigation — `Disease`, `Phenotype`,
`BiologicalPathway`, `BiologicalFunction`, `Gene`, `Variant`, `Molecule`. Use it for the
"method/model/resource → what it studies" axis that previously collapsed to `associated_with`:
e.g. lung-organoid model `used_to_study` COVID-19 pathogenesis; a GWAS `used_to_study` a disease's
genetic architecture. Per TB-3, keep the tangible model and the *act* of using it separate — the
organoid is an `Organism`/`MaterialSample`, its experimental use is a `MethodOrProcedure`, and
`used_to_study` runs from whichever node the source frames as the investigative instrument to the
entity under study. Direction is **resource → subject of study** (forward-only; no inverse).

- `treats`/`contraindicated_in` — domain `Molecule`/`MethodOrProcedure`/`Device`.

Variant **consequence** (missense, 5′UTR, splice-donor — SO/VEP) is an **attribute** on the
variant, never a node. A measure's **value** in context is an **edge attribute**
(`effect_size`+`unit`+`direction`), never a node.

## Provenance: the source is a node (v0.5)

Provenance has **two mechanisms**, both naming a **source node by its `identifier`** (one of your
`Publication`/`Study`/`Dataset`/`Agent` nodes — never a bare `infores:` CURIE): the required per-edge
**`primary_source`**, and the **`reported_in` edge** (the traversable, node-level link). There is no
`provided_by` (dropped in v0.5 — a node-level "came from X" is just a `reported_in` edge). Two kinds
of source node:

- **Ingested document** (a paper, preprint, tweet, slide, dataset, or bench note you saved in
`raw/`): a `Publication`/`Study`/`Dataset` node with a **`raw_source`** listing its `raw/…`
path(s).

- **External reference** (HGNC, GO, UniProt, SemMedDB, DrugBank, ATC — cited, not ingested): a node
with **no** `raw_source`, recording its `infores:`/ontology CURIE in `xref`. Type it by identity:
a naming authority/organization (HGNC, a regulator) → `Agent` (`subtype: organization`); a
citable data/knowledge artifact (GO, SemMedDB, DrugBank) → `Dataset` (`subtype: knowledge_base`).

So every claim is traceable — `edge → primary_source → source node → raw_source` (immutable file) or
`→ xref` (named external authority). **Name each external-reference source node canonically** (its
registry short label — `HGNC`, `Gene Ontology`, `DrugBank`) and create it **once**, reusing its
`identifier` everywhere. `primary_source: not_provided` is allowed **only** as a rare escape for a
genuinely unknown origin — never the default. Full rule:
[SPEC.md §8.1](SPEC.md#81-provenance-is-node-based-v05).

## Workflows

### Ingest a source

1. Save the source into `raw/` unchanged. Note its modality.

2. Read/parse it fully (OCR images, parse tables, transcribe slides/tweets).

3. Create a `Publication`/`Study`/`Dataset` node **for the source itself**, and give it a
`raw_source` listing the `raw/…` path(s) you just saved.

4. For each biomedical entity discussed: create or update its typed concept doc — set the
mandatory `type` and a human-readable, bundle-unique `identifier`, and coin a `subtype`.
Reuse an existing entity's `identifier` instead of forking.

5. **Curate external cross-references (`xref`) — optional enrichment.** Where you know a
standard ontology CURIE, list it under `xref`; otherwise the `identifier` alone is enough.
`xref` isn't required, so resolve what you can now and **backfill the rest on a later pass
or during lint**.

6. For each claim: add a typed `edges:` entry (each `object` = the target's `identifier`) with
the provenance triplet (`knowledge_level`, `agent_type`, and `primary_source` = the **source
node's `identifier`**) and any statistics; and a `reported_in` edge to the source node. For a
claim from an **external database you didn't ingest** (HGNC, GO…), create a lightweight
`Agent`/`Dataset` source node once (its `infores:` CURIE in `xref`, no `raw_source`) and reuse
its `identifier` as `primary_source`.

7. Update `index.md`; append a `## YYYY-MM-DD` entry to `log.md` summarizing what changed.

> A single source typically creates/updates **10–15 concept pages**. That bookkeeping is
> your job, not the human's.

### Answer a query

Read `index.md` → open the relevant typed pages → follow `edges:` → synthesize a **cited**
answer (filter by `knowledge_level` when the question is clinical). File durable answers
back as new concept pages. Prefer graph-shaped reasoning ("what `treats` a `Disease`
`associated_with` this `Gene`?").

### Lint (periodic)

Flag: invalid `type`/`predicate`; **`identifier` problems — missing, duplicated across the
bundle, or not human-readable (opaque codes / bare CURIEs)**; edge `object`s that don't resolve
to any node's `identifier`; domain/range violations (e.g. a `treats` edge pointing at a
`Molecule` instead of a `Disease`/`Phenotype`); edges missing the provenance triplet;
**a `primary_source` that doesn't resolve to a source node** (a
`Publication`/`Study`/`Dataset`/`Agent`; flag like any broken `object`, unless it is the reserved
`not_provided`); **`not_provided` used as anything but a rare exception**; **source nodes anchored to
neither a `raw_source` path nor an external `xref`**
(an unanchored source — backfill); contradictions (same triple, opposite `negated`, or incompatible
effect sizes); stale claims; orphan pages; missing back-links. A **missing external CURIE in `xref`
is an enrichment opportunity to backfill, not a conformance error** (only `type`/`identifier` are
mandatory). Do **not** lint `subtype` against a fixed list — the agent coins it.

## Golden rules

- **Type by identity, link by role.** Aspirin *is* a `Molecule`; "treats headache" and
"inhibits COX-1" are `edges`. A *risk factor* is a role, not a type — its subject can be a
`Disease`, `BiomedicalMeasure`, `Exposure`, `SocialFactor`, `Organism`, etc., linked by
`predisposes_to`.

- **Type is controlled, subtype is coined.** Pick the `type` from the closed set of 28; always
add a `subtype`, but invent the value yourself — there is no fixed list.

- **Only `type` and `identifier` are mandatory.** The `identifier` (one merged successor of name

- id) must be human-readable and unique across the bundle; external CURIEs are optional `xref`.

- **State vs variable vs value.** A diagnosis is a `Disease`; an observable quality is a
`Phenotype`; a measurable variable is a `BiomedicalMeasure`; its value is an edge attribute.

- **One source of truth per entity.** If IL6 already has a page, update it — don't fork.

- **Stamp every edge with provenance — and the source is a node.** A graph fed by papers *and*
tweets is only useful if you can tell them apart. Every `primary_source` names a source **node**
(which anchors to `raw/` via `raw_source`, or to an external authority via `xref`), never a bare
CURIE.

- **Numbers are first-class.** Effect sizes and p-values live on edges, not in prose.

- **When unsure between two types, pick the more specific entity type and add `xref`.**

## Deprecated aliases (accept on read, normalize to v0.5)

Older/parallel bundles may use these names; accept them and normalize. Do not emit them.

| Deprecated                                                            | Normalizes to                                                                                      |
| --------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------- |
| `SDOH`, `SDoH`                                                        | `SocialFactor`                                                                                     |
| `GenomicFeature`                                                      | `Variant` / `SequenceFeature` by `subtype`                                                         |
| `Process`, `BiologicalProcess`                                        | `BiologicalPathway` / `BiologicalFunction`                                                         |
| `ClinicalMeasure`                                                     | `BiomedicalMeasure`                                                                                |
| `ExposureOrFactor`                                                    | `Exposure` / `SocialFactor` / `Food` / `Population` / `GeographicLocation`                         |
| `Procedure`, `Method`                                                 | `MethodOrProcedure`                                                                                |
| `title`, `id` (attribute keys)                                        | `identifier` (+ `xref` for the CURIE)                                                              |
| `primary_source` with an `infores:` CURIE value                       | the **`identifier` of a source node** (synthesize one `Agent` node per CURIE, `xref: [infores:X]`) |
| `provided_by` (node field — removed in v0.5)                          | a `reported_in` edge to that source node                                                           |
| `encoded_by`/`caused_by`/`treated_by`/`produces` (inverse predicates) | the **forward** predicate (`encodes`/`causes`/`treats`/`catalyzes`) on the other node              |
| `<type>_kind`, `class_basis`, Structure `method` (attribute keys)     | `subtype`                                                                                          |
