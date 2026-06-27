# BioOKF — Coverage-Maximalist Type-System Proposal

> **Perspective:** *Guarantee nothing in biomedicine is unrepresentable.*
> Stress-tested against every subfield (epidemiology, genetics, molecular
> biology, pharmacology, orthopaedics/clinical, chemistry, chemical biology),
> and against every *source type* (papers, preprints, datasets, lab protocols,
> slides, figures, social/blog text). The design adds finite, controlled
> **node** and **edge** type universes on top of OKF, replacing OKF's open
> `type` vocabulary and untyped prose links — while staying *general, not
> granular*: a small set of well-chosen umbrella types, each backed by an
> optional `subtype` attribute that absorbs fine-grained distinctions without
> exploding the type count.
>
> Authored 2026-06-25. Grounds in UMLS Semantic Network (127 types / 54
> relations / 15 groups), SPOKE+Hetionet, the major open biomedical KGs
> (PrimeKG, Open Targets, DRKG, Monarch, ROBOKOP, CKG, Bioteque), SemMedDB/
> PubTator3 literature-mining predicates, the top-level classes of 13
> biomedical ontologies, and the Biolink Model `Association` attribute schema.

---

## 0. Design philosophy and the coverage argument in brief

OKF requires every document to carry a `type` from an **open** vocabulary, and
all cross-links are **untyped** prose links (§4.1, §5.3 of the spec). BioOKF
makes three changes while preserving 100% backward compatibility with OKF's
conformance rules (parseable YAML frontmatter, non-empty `type`, valid
`index.md`/`log.md`):

1. **`type` becomes a closed enum** of **18 node types** (an *Entity* family of
   13 biomedical-object types + a *Provenance/Method* family of 5 study/data/
   evidence types). This is deliberately near the **15 UMLS Semantic Groups**
   and the **~11 SPOKE/Hetionet metanodes** — proven-exhaustive coarse
   partitions — not the 127 UMLS Semantic Types or 57 Open-Targets node types
   (too granular).

2. **A new `edges:` frontmatter block** carries a closed enum of **20 edge
   types** (a typed-predicate layer). Untyped markdown prose links remain legal
   (OKF compatibility) but are now *optional sugar*; the machine-readable edge
   is the typed frontmatter entry. The 20 predicates are a superset-by-merge of
   the UMLS 54 relations (collapsed to their 5 super-relations + the salient
   functional leaves), the 24 Hetionet metaedges, the ~30 SemMedDB predicates,
   and the 13 PubTator3 relations.

3. **Required vs optional attributes** on every node and every edge, mirroring
   Biolink's discipline: nodes require `id` (CURIE) + `name`; edges require the
   **provenance-of-reasoning triplet** `knowledge_level` + `agent_type` +
   `primary_source`. All quantitative/statistical/qualifier detail is optional,
   so a one-line curated edge is not over-burdened while a GWAS edge can carry
   `p_value`/`odds_ratio`/`ci`/`sample_size`.

**The "general not granular" lever — `subtype` + `id`.** Every node type has an
optional free-text-but-CURIE-anchored `subtype` attribute. "Gene", "protein",
"compound", "drug", "metabolite", "ion", "antibody", "PROTAC" all live under one
**`Molecule`** node type, distinguished by `subtype` + the namespace of `id`
(`HGNC:` vs `UniProtKB:` vs `CHEBI:` vs `DRUGBANK:`). This is exactly how SPOKE
keeps Compound general while Open Targets splits 57 node types — we choose the
SPOKE/UMLS-group altitude. Coverage is guaranteed because the *granularity that
matters for retrieval* is preserved in `subtype`/`id`/`category_closure`, not
thrown away.

---

## 1. NODE UNIVERSE (18 types)

Common required node attributes (every node, all 18 types): `id` (CURIE,
`prefix:local`), `name`, `type` (the BioOKF node type, = OKF `type`).
Common optional node attributes (every node): `synonyms[]`, `xrefs[]`
(equivalent CURIEs), `subtype` (umbrella refinement), `description`,
`provided_by`, `iri`, `deprecated`, `tags[]`, `timestamp`. Type-specific
attributes are listed per type below.

### Entity family (13)

| # | Node type | Umbrella covers | Maps to (UMLS group / Biolink / SPOKE) |
|---|-----------|-----------------|----------------------------------------|
| 1 | **Molecule** | gene, protein, transcript/RNA, small molecule, drug, metabolite, cofactor, ion, lipid, peptide, antibody, antigen, nutrient, food compound, PROTAC/glue, natural product, payload | CHEM+GENE (UMLS) / `biolink:ChemicalEntity`+`Gene`+`Protein`+`Transcript` / SPOKE Compound+Gene+Protein+MiRNA+Nutrient |
| 2 | **MolecularComplexOrFamily** | protein complex, multiprotein machine, gene/protein family, superfamily, protein domain/motif, gene set, EC enzyme class | `biolink:MacromolecularComplex`+`GeneFamily`+`ProteinDomain` / SPOKE Complex+ProteinFamily+ProteinDomain+EC |
| 3 | **GenomicFeature** | variant (SNV/indel/CNV/SV), allele, genotype, haplotype, locus, GWAS locus, regulatory element (enhancer/promoter/CpG/cCRE), TAD, splice site, PTM site, sequence motif | GENE (UMLS) / `biolink:SequenceVariant`+`Genotype`+`Haplotype`+`RegulatoryRegion` / SPOKE (variant edges) |
| 4 | **Anatomy** | body region, organ, organ system, tissue, gross anatomical structure, cellular component/organelle, subcellular compartment, body space/fluid, anatomical abnormality | ANAT (UMLS) / `biolink:AnatomicalEntity`+`CellularComponent`+`GrossAnatomicalStructure` / SPOKE Anatomy+CellularComponent |
| 5 | **CellTypeOrLine** | cell type, cell state, cell population, cell line, xenograft model, organoid, stem/progenitor cell | `biolink:Cell`+`CellLine` / SPOKE CellType+CellLine+AnatomyCellType |
| 6 | **Organism** | species, strain, taxon, pathogen (bacterium/virus/parasite/fungus), microbial taxon/OTU/ASV, model organism, host | LIVB (UMLS) / `biolink:OrganismTaxon`+`Human`+`Virus`+`Bacterium` / SPOKE Organism+SARSCov2 |
| 7 | **BiologicalProcessOrPathway** | GO biological process, molecular function, pathway, reaction/molecular event, signaling cascade, physiologic function, metabolic process, cell function | PHYS+PHEN (UMLS) / `biolink:BiologicalProcess`+`Pathway`+`MolecularActivity`+`PhysiologicalProcess` / SPOKE BiologicalProcess+MolecularFunction+Pathway+Reaction+PwGroup |
| 8 | **Disease** | disease, syndrome, disorder, neoplastic process, infection, injury/fracture, congenital/acquired abnormality, pathologic function, cancer subtype/stage | DISO (UMLS) / `biolink:Disease`+`PathologicalProcess` / SPOKE Disease |
| 9 | **Phenotype** | sign, symptom, clinical finding, phenotypic feature, quantitative trait, endophenotype, side effect/adverse event, behavioral feature, imaging finding | DISO/PHEN (UMLS) / `biolink:PhenotypicFeature`+`ClinicalFinding`+`BehavioralFeature` / SPOKE Symptom+SideEffect |
| 10 | **ClinicalAttributeOrBiomarker** | lab test, lab/test result, vital sign, clinical measurement, score/index (EF, Harris Hip Score, mRS, PD-L1 TPS), biomarker level, polygenic score, observable | CONC/PROC (UMLS Finding/Lab Result) / `biolink:ClinicalAttribute`+`ClinicalFinding` / SPOKE ClinicalLab |
| 11 | **ProcedureOrIntervention** | therapeutic/preventive procedure, surgery, diagnostic procedure, assay/technique (PCR, scRNA-seq, IHC, MS, patch-clamp), treatment, clinical intervention, vaccination, screening | PROC (UMLS) / `biolink:Procedure`+`Treatment`+`ClinicalIntervention` / SPOKE (clinical edges) |
| 12 | **DeviceOrMaterial** | medical device, implant/prosthesis/graft/mesh, drug-delivery device, research device/instrument (MinION, sequencer, flow cytometer), reagent/kit/buffer, material sample/specimen, 3D structure/PDB model | DEVI/OBJC (UMLS) / `biolink:Device`+`MaterialSample` / SPOKE (—) |
| 13 | **EnvironmentOrSocialContext** | exposure (chemical/behavioral/environmental), social determinant of health, geographic location/region, population/ancestry group/cohort, demographic stratum, lifestyle factor, occupation | LIVB(Group)/GEOG/OCCU (UMLS) / `biolink:ExposureEvent`+`GeographicLocation`+`Cohort`+`PopulationOfIndividualOrganisms`+`SocioeconomicAttribute` / SPOKE SDoH+Location |

### Provenance / Method family (5)

| # | Node type | Umbrella covers | Maps to |
|---|-----------|-----------------|---------|
| 14 | **StudyOrDataset** | clinical trial, cohort/case-control/RCT study, GWAS study, registry, dataset, screening library, knowledge graph, data file (FASTQ/count matrix) | `biolink:Study`+`Dataset` / SPOKE (study/trial edges) / `clinicaltrials:NCT…` |
| 15 | **MethodOrModel** | assay protocol, lab protocol/step, computational method/pipeline/tool/software, statistical model, aging clock, risk model, ML model, knowledge-graph algorithm | OBI/EDAM classes / `biolink:Procedure`(method) — provenance node |
| 16 | **Publication** | journal article, preprint, book, patent, drug label, web page, guideline, conference abstract, poster, slide deck, blog post, lab-notebook entry | `biolink:Publication`+`Article`+`PreprintPublication`+`Patent`+`DrugLabel`+`WebPage` |
| 17 | **EvidenceOrAssertion** | a reified evidence record / curated assertion (ECO-typed), credibility classification, evidence artifact — lets evidence itself be a first-class node | `biolink:Evidence`+`InformationContentEntity` / ECO terms / ClinVar review status |
| 18 | **Agent** | person, lab, consortium, organization, author, curator, software agent, institution, funder, online community/handle | ORGA (UMLS) / `biolink:Agent`+`Organization` / `ORCID:`, `infores:` |

> **Why 18 and not fewer / more.** UMLS proves a 15-group partition covers
> 99.5% of multi-million biomedical concepts; SPOKE proves an ~11-node metagraph
> covers a 27M-node graph. 18 sits between them: the 13 Entity types = the UMLS
> groups with CHEM+GENE merged into Molecule and ANAT split lightly for cells,
> plus the 5 Provenance/Method types that *every* source-type analysis (papers,
> datasets, protocols, slides) demanded as explicit provenance nodes. Adding a
> 19th (e.g. splitting Gene from Compound) would be granular without adding
> *coverage* — the `subtype`+`id`-namespace mechanism already represents it.

### 1.1 Required/optional attributes per node type (beyond the common set)

- **Molecule** — *required:* none beyond common (the `id` namespace + `subtype`
  carry the molecular class). *optional:* `subtype` ∈ {gene, protein,
  transcript, small_molecule, drug, metabolite, ion, lipid, peptide, antibody,
  antigen, nutrient, prodrug, natural_product, …}, `in_taxon`, `sequence`,
  `chemical_formula`, `smiles`, `inchikey`, `molecular_weight`,
  `gene_symbol`, `chromosome`, `mechanism_of_action`, `drug_class`,
  `structure_ref` (PDB CURIE).
- **MolecularComplexOrFamily** — *optional:* `subtype` ∈ {complex, family,
  superfamily, domain, motif, gene_set, ec_class}, `members[]` (Molecule
  CURIEs), `ec_number`.
- **GenomicFeature** — *optional:* `subtype` ∈ {snv, indel, cnv, sv, allele,
  genotype, haplotype, locus, enhancer, promoter, cpg_site, ptm_site,
  splice_site}, `hgvs`, `rsid`, `chromosome`, `position`, `ref_allele`,
  `alt_allele`, `gene` (Molecule CURIE), `consequence`, `zygosity`,
  `allele_frequency`, `in_taxon`.
- **Anatomy** — *optional:* `subtype` ∈ {organ, organ_system, tissue, cell_component,
  body_fluid, body_region, abnormality}, `in_taxon`, `part_of` (Anatomy CURIE).
- **CellTypeOrLine** — *optional:* `subtype` ∈ {cell_type, cell_state, cell_line,
  organoid, xenograft, stem_cell}, `in_taxon`, `tissue` (Anatomy CURIE),
  `markers[]`, `disease_origin` (Disease CURIE).
- **Organism** — *optional:* `subtype` ∈ {species, strain, pathogen, microbe,
  model_organism, host}, `taxonomic_rank`, `ncbi_taxon`, `is_pathogen` (bool).
- **BiologicalProcessOrPathway** — *optional:* `subtype` ∈ {biological_process,
  molecular_function, pathway, reaction, signaling, metabolic}, `go_aspect`
  ∈ {BP, MF, CC}, `in_taxon`, `participants[]`.
- **Disease** — *optional:* `subtype` ∈ {disease, syndrome, neoplasm, infection,
  injury, congenital_abnormality, acquired_abnormality}, `stage`, `icd_code`,
  `mondo_id`, `affected_anatomy` (Anatomy CURIE), `inheritance`.
- **Phenotype** — *optional:* `subtype` ∈ {sign, symptom, finding, trait,
  side_effect, adverse_event, imaging_finding, behavioral}, `hpo_id`,
  `severity`, `onset`, `quantitative` (bool), `unit`.
- **ClinicalAttributeOrBiomarker** — *optional:* `subtype` ∈ {lab_test,
  lab_result, vital_sign, score, biomarker, polygenic_score, observable},
  `loinc`, `unit`, `reference_range`, `value`, `measures` (Molecule/Disease CURIE).
- **ProcedureOrIntervention** — *optional:* `subtype` ∈ {therapeutic, preventive,
  surgical, diagnostic, assay, vaccination, screening}, `cpt`, `target_anatomy`
  (Anatomy CURIE), `modality`, `instrument` (DeviceOrMaterial CURIE).
- **DeviceOrMaterial** — *optional:* `subtype` ∈ {device, implant, instrument,
  reagent, specimen, structure_model}, `manufacturer`, `model_method`,
  `resolution`, `pdb_id`.
- **EnvironmentOrSocialContext** — *required:* `subtype` ∈ {exposure, sdoh,
  geographic, population, cohort_stratum, occupation, lifestyle} (required here
  because the subtypes are semantically distinct enough that an unlabeled
  instance is ambiguous). *optional:* `region_code`, `ancestry`, `sample_size`,
  `age_range`, `sex`, `exposure_route`, `dose`.
- **StudyOrDataset** — *required:* `subtype` ∈ {clinical_trial, observational_study,
  gwas, rct, cohort, registry, dataset, library, knowledge_graph}. *optional:*
  `nct_id`, `accession`, `n_participants`, `design`, `phase`, `start_date`,
  `population` (EnvironmentOrSocialContext CURIE), `endpoints[]`.
- **MethodOrModel** — *required:* `subtype` ∈ {assay_protocol, lab_protocol,
  pipeline, software, statistical_model, ml_model}. *optional:* `version`,
  `doi`, `repository_url`, `parameters`, `language`, `inputs[]`, `outputs[]`.
- **Publication** — *required:* none beyond common (`id` should be `PMID:`/`DOI:`
  /`NCT:` when available). *optional:* `subtype` ∈ {journal_article, preprint,
  book, patent, drug_label, web_page, guideline, abstract, poster, slide_deck,
  blog_post, lab_notebook}, `authors[]`, `journal`, `year`, `pmid`, `doi`,
  `publication_type`, `url`.
- **EvidenceOrAssertion** — *required:* `evidence_type` (ECO CURIE or GO 3-letter
  code). *optional:* `subtype`, `assertion_method` ∈ {manual, automatic},
  `review_status`, `confidence`, `supporting_text[]`, `curator`.
- **Agent** — *required:* none beyond common. *optional:* `subtype` ∈ {person,
  lab, consortium, organization, curator, software_agent, funder, community},
  `orcid`, `infores`, `affiliation`, `role`.

---

## 2. EDGE UNIVERSE (20 typed predicates)

### 2.1 Edge representation in BioOKF

Edges live in a frontmatter `edges:` list on the **subject** concept document
(an OKF-compatible extension key). Each entry:

```yaml
edges:
  - predicate: treats            # REQUIRED — one of the 20 controlled predicates
    object: MONDO:0005835        # REQUIRED — target node CURIE (or bundle path)
    knowledge_level: knowledge_assertion   # REQUIRED (Biolink KnowledgeLevelEnum, 7 vals)
    agent_type: manual_agent               # REQUIRED (Biolink AgentTypeEnum, 8 vals)
    primary_source: infores:drugcentral    # REQUIRED (exactly one, infores: CURIE)
    # ---- optional below ----
    negated: false
    publications: [PMID:34986598]
    aggregator_source: [infores:spoke]
    evidence_type: [ECO:0000269]
    p_value: 4.2e-9
    adjusted_p_value: 1.1e-6
    effect_size: 1.73           # generic; effect_metric names which kind
    effect_metric: odds_ratio   # ∈ {beta, odds_ratio, hazard_ratio, risk_ratio, log2_fold_change, smd, correlation_r}
    ci_lower: 1.41
    ci_upper: 2.12
    standard_error: 0.11
    sample_size: 12000
    confidence_score: 0.86
    direction: increased        # DirectionQualifierEnum: increased|decreased|upregulated|downregulated
    aspect: activity            # GeneOrGeneProductOrChemicalEntityAspectEnum (57 vals)
    species_context: NCBITaxon:9606
    anatomical_context: UBERON:0002107
    sex_qualifier: female
    frequency_qualifier: HP:0040283
    onset_qualifier: HP:0003577
    timepoint: 2026-06-01
```

Untyped markdown prose links (OKF §5) remain legal but are advisory only; the
typed `edges:` block is the authoritative machine-readable relationship layer.

### 2.2 Required vs optional edge attributes (all 20 predicates)

- **Required (every edge):** `predicate`, `object`, `knowledge_level`,
  `agent_type`, `primary_source`. (The Biolink-mandated provenance-of-reasoning
  triplet + the triple ends. Subject is implicit = the host concept.)
- **Strongly recommended:** `publications[]`, `aggregator_source[]`, `negated`.
- **Recommended when applicable:** `evidence_type[]` (ECO/GO code),
  `direction`, `aspect`.
- **Optional (statistical — for analysis-derived edges):** `p_value`,
  `adjusted_p_value`, `effect_size` + `effect_metric`, `ci_lower`/`ci_upper`,
  `standard_error`, `sample_size`, `confidence_score`, `correlation_coefficient`.
- **Optional (qualifiers/context):** `species_context`, `anatomical_context`,
  `sex_qualifier`, `frequency_qualifier`, `onset_qualifier`,
  `severity_qualifier`, `stage_qualifier`, `timepoint`,
  `form_or_variant_qualifier`, `causal_mechanism_qualifier`.

### 2.3 The 20 predicates

| # | Predicate | Definition | Dir. | Domain → Range | Required attrs | Notable optional attrs | Maps to |
|---|-----------|------------|------|----------------|----------------|------------------------|---------|
| 1 | **is_a** | subject is a more-specific kind of object (taxonomic/ontology hierarchy). | directed | any → same-type | base 5 | — | UMLS `isa`; Biolink `subclass_of`; SPOKE ISA |
| 2 | **part_of** | subject composes/is contained-in/is-subunit-of object (physical or conceptual mereology). | directed | Anatomy/Molecule/GenomicFeature/BioProcess → larger whole | base 5 | `anatomical_context` | UMLS `part_of`/`conceptual_part_of`; SPOKE PARTOF; SemMedDB PART_OF |
| 3 | **located_in** | subject's position/site/region is object; site of a process. | directed | Disease/Phenotype/Molecule/CellType → Anatomy/Organism | base 5 | `anatomical_context`, `species_context` | UMLS `location_of`(inv); Hetionet DlA; SemMedDB LOCATION_OF |
| 4 | **expressed_in** | gene/protein is expressed / present / abundant in an anatomy/cell/condition. | directed | Molecule/GenomicFeature → Anatomy/CellTypeOrLine/Disease | base 5 | `direction`, `effect_size`, `anatomical_context`, `p_value` | Hetionet AeG/AuG/AdG; SPOKE EXPRESSEDIN; Biolink `expressed_in` |
| 5 | **interacts_with** | subject physically/functionally interacts with object (PPI, protein-ligand, protein-DNA, host-pathogen, drug-drug, drug-food). | symmetric | Molecule/Organism/MolComplex → Molecule/Organism | base 5 | `effect_size`, `confidence_score` (STRING), `aspect` | UMLS `interacts_with`; Hetionet GiG/PiP; PubTator3 interact/drug_interact |
| 6 | **binds** | subject binds object with measurable affinity (compound→target, antibody→antigen, TF→element). | directed | Molecule/MolComplex → Molecule/MolComplex/GenomicFeature | base 5 | `effect_size`+`effect_metric` (Ki/Kd/IC50), `aspect`, `confidence_score` | Hetionet CbG; SPOKE BINDS_CbP; chem-bio BINDS |
| 7 | **regulates** | subject up/down-regulates or modulates object's activity/abundance/expression (signed). | directed | Molecule/GenomicFeature/BioProcess → Molecule/GenomicFeature/BioProcess | base 5 | **`direction`**, **`aspect`**, `causal_mechanism_qualifier`, `effect_size`, `p_value`, `anatomical_context` | Hetionet Gr>G; UMLS `affects`; SPOKE UPREGULATES/DOWNREGULATES; PubTator3 inhibit/stimulate |
| 8 | **catalyzes** | enzyme/complex catalyzes a reaction; substrate→product transformation. | directed | Molecule/MolComplex → BioProcess(reaction) | base 5 | `species_context`, `aspect` | UMLS `produces`; SPOKE ECcR/RpC; chem CATALYZES |
| 9 | **participates_in** | subject takes part in / is member of a pathway, process, or complex. | directed | Molecule/GenomicFeature → BioProcess/MolComplex | base 5 | `evidence_type` (GO code) | Hetionet GpBP/GpPW; Biolink `participates_in`; UMLS `process_of` |
| 10 | **produces** | subject brings forth / secretes / biosynthesizes / generates object. | directed | Organism/CellType/Molecule/BioProcess → Molecule | base 5 | `effect_size`, `anatomical_context` | UMLS `produces`; SemMedDB PRODUCES; SPOKE FcC |
| 11 | **converts_to** | subject is chemically transformed / metabolized into object. | directed | Molecule → Molecule | base 5 | `catalyzed_by` (Molecule CURIE), `species_context` | SemMedDB CONVERTS_TO; PubTator3 convert; chem TRANSFORMS_INTO |
| 12 | **treats** | subject is applied as remedy to cure/manage/palliate object condition. | directed | Molecule/ProcedureOrIntervention/DeviceOrMaterial → Disease/Phenotype | base 5 | `effect_size`+`effect_metric`, `sample_size`, `evidence_type`, `confidence_score` | UMLS `treats`; Hetionet CtD/CpD; SemMedDB TREATS; PubTator3 treat |
| 13 | **prevents** | subject stops/hinders/reduces risk of object condition. | directed | Molecule/ProcedureOrIntervention/EnvContext → Disease/Phenotype | base 5 | `effect_size`+`effect_metric`, `sample_size`, `p_value` | UMLS `prevents`; SemMedDB PREVENTS; PubTator3 prevent |
| 14 | **causes** | subject brings about / induces / drives / predisposes-to / is-risk-factor-for object (incl. somatic driver, exposure→outcome). | directed | any agent → Disease/Phenotype/BioProcess | base 5 | `negated`, `effect_size`+`effect_metric` (OR/HR/RR), `p_value`, `confidence_score`, `inheritance` | UMLS `causes`/`predisposes`; SemMedDB CAUSES/PREDISPOSES; PubTator3 cause; Hetionet (somatic) |
| 15 | **contraindicated_for** | subject must not be used in object condition / adverse-event context. | directed | Molecule/ProcedureOrIntervention → Disease/Phenotype | base 5 | `evidence_type`, `frequency_qualifier` | SPOKE CONTRAINDICATES_CcD; Biolink `contraindicated_in` |
| 16 | **has_phenotype** | subject (disease/organism/case/genotype) presents/manifests object phenotype/sign/side-effect. | directed | Disease/Organism/CellType/GenomicFeature → Phenotype | base 5 | **`frequency_qualifier`**, `onset_qualifier`, `severity_qualifier`, `sex_qualifier` | Hetionet DpS; Biolink `has_phenotype`; UMLS `manifestation_of`(inv); Hetionet CcSE |
| 17 | **associated_with** | a statistical / co-occurrence / general association between subject and object (GWAS, eQTL, biomarker, comorbidity, text co-occurrence) — the quantitative umbrella edge. | symmetric | any → any | base 5 (`knowledge_level` usually `statistical_association`/`text_co_occurrence`) | **`p_value`**, **`adjusted_p_value`**, **`effect_size`**+`effect_metric`, `ci_lower`/`ci_upper`, `standard_error`, `sample_size`, `correlation_coefficient`, `direction`, `species_context`, `anatomical_context` | UMLS `associated_with`; Hetionet DaG/GcG; SemMedDB ASSOCIATED_WITH; PubTator3 associate/correlate; GWAS Catalog |
| 18 | **measured_by** | object procedure/assay/biomarker measures/diagnoses/quantifies subject; reverse = a finding indicates a condition. | directed | Disease/Phenotype/Molecule/ClinicalAttribute → ProcedureOrIntervention/ClinicalAttributeOrBiomarker | base 5 | `effect_size`, `unit`, `anatomical_context` | UMLS `measures`/`diagnoses`/`indicates`; SemMedDB MEASURES/DIAGNOSES |
| 19 | **derives_from** | subject is derived/isolated/sampled/produced-from object (sample←donor, analog←parent, cell←tissue, data←experiment, library←sample); provenance/material lineage. | directed | CellType/DeviceOrMaterial/Molecule/StudyOrDataset → Organism/Anatomy/Molecule/Study | base 5 | `timepoint` | UMLS `derivative_of`/`developmental_form_of`; Biolink `derives_from` |
| 20 | **reported_in** | subject (any node or any edge, via reification) is reported/curated/studied/evidenced-in object publication/study/dataset/evidence-record/agent; the universal provenance edge. | directed | any → Publication/StudyOrDataset/EvidenceOrAssertion/Agent | base 5 | `evidence_type`, `confidence_score`, `review_status` | Biolink `publications`/`provided_by`; UMLS `issue_in`/`analyzes`; CKG MENTIONED_IN_PUBLICATION; SemMedDB/PubTator3 PMID provenance |

> **Coverage of directionality, temporal, and comparative relations.**
> Temporal ordering (`precedes`/`follows`, `co-occurs_with`) is expressed via
> `associated_with` + the `timepoint`/`temporal_context_qualifier` optional
> attribute (symmetric co-occurrence) or a `causes` chain (ordered cascade) —
> avoiding two more predicates. Comparative relations (COMPARED_WITH /
> superior_to / non_inferior_to from SemRep + clinical trials) are expressed as
> `associated_with` with `effect_metric` + `subtype`-style context, keeping the
> predicate count at 20. Negation (every SemMedDB predicate's `NEG_` dual,
> Biolink `negated`) is the `negated: true` attribute, not 20 more predicates.

### 2.4 Predicate ↔ domain/range matrix (sanity check that the 18×18 space is covered)

Every node-type pair that appears in the source taxonomies has a predicate:
Molecule↔Molecule (5,6,7,11,interacts), Molecule→Disease (12,13,14,15),
Molecule→Anatomy/Cell (4,3), Molecule→BioProcess (8,9,10), GenomicFeature→Disease
(14,17), GenomicFeature→Molecule (7,4,part_of), Disease↔Phenotype (16),
Disease→Anatomy (3), Procedure→Disease (12,13), Procedure→Anatomy (3,located),
ClinicalAttribute↔Molecule/Disease (18,17), Organism→Disease (14),
Organism→Molecule (10), EnvContext→Disease (14,13,17), CellType→Anatomy/Disease
(3,19,16), DeviceOrMaterial→Anatomy (3), and **any node → Publication/Study/
Evidence/Agent (20)** plus the universal hierarchy (1) and provenance (19,20).
No source-taxonomy edge fell outside these 20.

---

## 3. Coverage argument — stress test by subfield and by source type

**By subfield.**
- *Epidemiology:* exposure (EnvContext) `causes`/`associated_with` outcome
  (Disease) with `odds_ratio`/`relative_risk`/`hazard_ratio` + `ci` +
  `sample_size`; disease `associated_with` population with prevalence via
  `has_count`/`has_total`. ✓
- *Genetics/genomics:* variant (GenomicFeature) `associated_with` trait
  (`p_value`/`beta`/`effect_allele_frequency`), `part_of`/`located_in` gene,
  `regulates` expression (eQTL, with `anatomical_context` + `direction`),
  `causes` Mendelian disease (`inheritance`); PRS as ClinicalAttribute
  `associated_with` disease. ✓
- *Molecular/cell biology & biochemistry:* kinase (Molecule) `regulates`
  substrate with `aspect: phosphorylation`+`direction`; enzyme `catalyzes`
  reaction; complex `part_of`/MolComplex membership; `binds` with Kd; PTM site
  (GenomicFeature) `part_of` protein; `converts_to` for metabolism. ✓
- *Pharmacology:* drug `binds`/`regulates` target (Ki/IC50), `treats`/
  `contraindicated_for` disease, `interacts_with` drug (DDI), `converts_to`
  metabolite, `causes` side-effect (`has_phenotype`), substrate-of CYP via
  `regulates`/`interacts_with`; PK params as ClinicalAttribute. ✓
- *Orthopaedics/clinical:* fracture (Disease subtype injury) `located_in`
  bone (Anatomy); surgery (Procedure) `treats` injury, `derives_from`/uses
  implant (DeviceOrMaterial via part_of/located_in); Harris Hip Score
  (ClinicalAttribute) `measured_by`; complication via `causes`/`has_phenotype`;
  head-to-head trials via `associated_with` + `effect_metric`. ✓
- *Chemistry/medicinal chem/chem-bio:* compound `binds`/`regulates`
  (covalent → `aspect`+`causal_mechanism_qualifier`), SAR via `associated_with`
  potency change; PROTAC `regulates`(degrades) target + `binds` E3 ligase;
  reaction `catalyzes`/`converts_to`; `derives_from` natural-product source
  organism; assay readout `measured_by`. ✓

**By source type.** Datasets/tabular → nodes are Sample/Patient/Cell
(CellTypeOrLine/EnvContext) + measurements (ClinicalAttribute) with
`associated_with`/`measured_by`. Lab protocols → MethodOrModel/Procedure +
DeviceOrMaterial + `derives_from` lineage (cells→lysate→library→reads). Slides/
figures/posters → same entity types + `reported_in` to the Publication node.
Preprints/papers/blogs → full predicate set + `reported_in` provenance with
`knowledge_level`/`agent_type` distinguishing curated vs text-mined vs
predicted claims. Every entity and relation pattern in the supplied
source-type catalogs maps onto an (≤18 node, ≤20 edge) pair — **nothing is
unrepresentable**; granularity that the sources expressed is preserved in
`subtype`, `id`-namespace, and the optional qualifier/statistical attributes.

**Why finite-yet-exhaustive holds.** The node set is bounded above by the
two proven-exhaustive coarse partitions (UMLS 15 groups, SPOKE 11 metanodes)
and below by the requirement that every source-type entity list be covered; 18
is the smallest set meeting both. The edge set is the de-duplicated union of the
5 UMLS super-relations' salient leaves + the 24 Hetionet metaedges + the ~30
SemMedDB predicates + 13 PubTator3 relations, collapsed wherever two predicates
differ only by an attribute we already model (direction/negation/effect-metric/
temporal qualifier). 20 predicates absorb all of them.
