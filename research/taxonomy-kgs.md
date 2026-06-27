# Taxonomy of Major Open Biomedical Knowledge Graphs

A reference catalog of the **node (entity) types** and **edge (relation/predicate) types** of the major open biomedical knowledge graphs. Compiled from primary sources (papers, GitHub repos, live schema APIs) on 2026-06-25.

For each KG: a short scope description, the full enumerated list of node types, and the full enumerated list of edge/relation types, with exact type names and counts where available.

> **Note on schemas vs. instances.** Some of these resources define a *schema* (a metagraph of allowed node/edge types, e.g. Hetionet, ROBOKOP, Open Targets), while others are reported by their *observed* node/edge inventory in a particular release dump (e.g. PrimeKG, Monarch KG, CKG, DRKG). Counts are version-specific snapshots and will drift across releases.

---

## Quick comparison

| KG | Node types | Edge/relation types | Core domain | Schema standard |
|----|-----------:|--------------------:|-------------|-----------------|
| **PrimeKG** | 10 | 30 | Precision medicine (disease-centric) | Custom |
| **Open Targets** (platform model) | 5 core entities | 7 evidence data types | Target–disease–drug | Custom / EFO |
| **Open Targets** (BioCypher KG, v25.x) | 57 | 128 | Target–disease–drug | BioCypher / custom |
| **Monarch KG** | ~80 (Biolink) | ~28 (Biolink) | Genes–phenotypes–diseases, cross-species | Biolink Model |
| **ROBOKOP** | 30 (Biolink) | 75 (Biolink) | Broad biomedical (Translator) | Biolink Model |
| **DRKG** | 13 | 107 (over 17 entity-pairs) | Drug repurposing | Custom (Source::Rel::Head:Tail) |
| **PharmKG** | 3 | 29 | Gene–drug–disease benchmark | GNBR theme vocabulary |
| **CKG** (Clinical Knowledge Graph) | 32 | 39 | Proteomics / clinical | Custom (Neo4j) |
| **Bioteque** | 12 (metanodes) | 67 (metaedges) | Embeddings over harmonized data | Custom (MetaNode-rel-MetaNode) |
| *Hetionet* (foundational, feeds DRKG) | 11 | 24 | Drug repurposing | Custom (metagraph) |
| *GNBR* (foundational, feeds DRKG/PharmKG) | 4 entity classes | ~36 themes | Text-mined relations | Theme codes |

---

## 1. PrimeKG (Precision Medicine Knowledge Graph)

**Source:** Chandak, Huang & Zitnik, *Building a knowledge graph to enable precision medicine*, Scientific Data 2023. Harvard MIMS Lab (`mims-harvard/PrimeKG`).
**Scope:** A disease-centric, multimodal KG integrating 20 resources, describing 17,080 diseases across "ten major biological scales." ~129,000 nodes, ~4.05M relationships. Genes and proteins are merged into one `gene/protein` node type. Disease phenotypes and drug side effects are merged into one `effect/phenotype` node type.

### Node types (10)

| Node type | Count | Vocabulary / source |
|-----------|------:|---------------------|
| biological_process | 28,642 | Gene Ontology (GO) |
| gene/protein | 27,671 | Entrez Gene / NCBI |
| disease | 17,080 | MONDO |
| effect/phenotype | 15,311 | HPO (phenotypes) + SIDER (drug side effects, merged) |
| anatomy | 14,035 | UBERON |
| molecular_function | 11,169 | Gene Ontology (GO) |
| drug | 7,957 | DrugBank |
| cellular_component | 4,176 | Gene Ontology (GO) |
| pathway | 2,516 | Reactome |
| exposure | 818 | CTD (Comparative Toxicogenomics DB) |

### Edge/relation types (30)

| Relation type | Count |
|---------------|------:|
| anatomy_protein_present | 3,036,406 |
| drug_drug | 2,672,628 |
| protein_protein | 642,150 |
| disease_phenotype_positive | 300,634 |
| bioprocess_protein | 289,610 |
| cellcomp_protein | 166,804 |
| disease_protein | 160,822 |
| molfunc_protein | 139,060 |
| drug_effect | 129,568 |
| bioprocess_bioprocess | 105,772 |
| pathway_protein | 85,292 |
| disease_disease | 64,388 |
| contraindication | 61,350 |
| drug_protein | 51,306 |
| anatomy_protein_absent | 39,774 |
| phenotype_phenotype | 37,472 |
| anatomy_anatomy | 28,064 |
| molfunc_molfunc | 27,148 |
| indication | 18,776 |
| cellcomp_cellcomp | 9,690 |
| phenotype_protein | 6,660 |
| off-label use | 5,136 |
| pathway_pathway | 5,070 |
| exposure_disease | 4,608 |
| exposure_exposure | 4,140 |
| exposure_bioprocess | 3,250 |
| exposure_protein | 2,424 |
| disease_phenotype_negative | 2,386 |
| exposure_molfunc | 90 |
| exposure_cellcomp | 20 |

> The Dec-2023 OMIM update adds further relation labels in the processing scripts (`mim_disease`, `mim_gene`, `mim_phenotype`, `mim_phenotypic_series`, `mim_phenotypic_series_disease`, `phenotype_map`), but the canonical published graph uses the 30 above.

---

## 2. Open Targets Platform

**Source:** Open Targets Platform (platform-docs.opentargets.org); BioCypher Open Targets adapter (`biocypher/open-targets`, release 25.12).
**Scope:** Therapeutic target identification & validation; integrates >20 sources into scored target–disease associations. Two representations are documented below: (a) the canonical **platform data model** (entity- and evidence-centric), and (b) the **BioCypher graph** materialization (a fully enumerated node/edge schema).

### (a) Platform data model — core entities (5) + evidence data types (7)

**Core entities:**

| Entity | Definition |
|--------|-----------|
| Target | Any candidate drug-binding molecule (gene/protein; Ensembl gene IDs). |
| Disease / Phenotype | Disease indications, phenotypes, measurements, biological processes, traits (EFO ontology). |
| Drug | Molecules functioning as medicinal products (ChEMBL). |
| Variant | DNA variations linked to diseases/traits/phenotypes. |
| Study | Sources of genetic evidence (GWAS / molecular QTL) linking variants to traits. |

Plus supporting/credible-set entities: **CredibleSet**, **Colocalisation**.

**Target–disease association evidence — 7 data types:**

| Data type | Meaning |
|-----------|---------|
| genetic_association | Germline mutation in the gene associated with the disease |
| somatic_mutation | Somatic mutation in the gene associated with the disease (typically cancer) |
| known_drug | Existing drug that engages the target and is used to treat the disease |
| affected_pathway | Gene is part of a pathway that is affected in the disease |
| rna_expression | Significant gene expression change in disease |
| literature | Gene–disease association identified by text mining of the literature |
| animal_model | Animal model with a gene knockout that manifests a phenotype concordant with the human disease |

Evidence **data sources** (feeding the above types) include: ChEMBL, EVA (ClinVar), EVA-somatic, ClinGen, Gene2Phenotype, Genomics England PanelApp, Orphanet, Gene Burden, GWAS credible sets, UniProt (literature + variants), Cancer Gene Census, IntOGen, Cancer Biomarkers, CRISPR / CRISPR screen, Reactome, Expression Atlas, Europe PMC (text mining), IMPC (mouse).

### (b) BioCypher Open Targets KG — node types (57)

```
node_adverse_reaction, node_biosample, node_cell_line, node_colocalisation,
node_credible_set, node_database_cross_reference_disease,
node_database_cross_reference_hpo, node_database_cross_reference_target,
node_disease, node_disease_phenotype_association, node_disease_synonym_broad,
node_disease_synonym_exact, node_disease_synonym_narrow,
node_disease_synonym_related, node_drug_warning, node_genetic_association_study,
node_go_term, node_literature_entry, node_mechanism_of_action, node_molecule,
node_mouse_gene, node_mouse_model, node_mouse_phenotype,
node_mouse_phenotype_class, node_pathway, node_pharmacogenomics_annotation,
node_phenotype, node_reaction, node_regulatory_element, node_species,
node_subcellular_location, node_target, node_target_classification,
node_target_disease_association_cancer_biomarkers,
node_target_disease_association_cancer_gene_census,
node_target_disease_association_chembl, node_target_disease_association_clingen,
node_target_disease_association_crispr,
node_target_disease_association_crispr_screen,
node_target_disease_association_europepmc, node_target_disease_association_eva,
node_target_disease_association_eva_somatic,
node_target_disease_association_expression_atlas,
node_target_disease_association_gene2phenotype,
node_target_disease_association_gene_burden,
node_target_disease_association_genomics_england,
node_target_disease_association_gwas_credible_sets,
node_target_disease_association_impc, node_target_disease_association_intogen,
node_target_disease_association_orphanet,
node_target_disease_association_reactome,
node_target_disease_association_uniprot_literature,
node_target_disease_association_uniprot_variants, node_target_prioritisation,
node_target_target_interaction, node_tissue, node_variant
```

### (b) BioCypher Open Targets KG — edge types (128, abridged families)

The 128 edge types are mostly machine-generated reifications of the association datasets. Representative / structural ones:

| Edge type | Connects |
|-----------|----------|
| edge_disease_is_a_disease | disease → disease (ontology) |
| edge_disease_has_synonym_synonym_{broad,exact,narrow,related} | disease → synonym |
| edge_disease_phenotype_association_has_object_phenotype | disease–phenotype assoc → phenotype |
| edge_molecule_has_mechanism_of_action | molecule → MoA |
| edge_mechanism_of_action_has_target_target | MoA → target |
| edge_molecule_indicates_disease | molecule → disease |
| edge_molecule_has_adverse_reaction_adverse_reaction | molecule → adverse reaction |
| edge_molecule_has_drug_warning | molecule → drug warning |
| edge_molecule_derived_from_molecule | molecule → molecule |
| edge_target_belongs_to_target_classification | target → classification |
| edge_target_expressed_in_biosample | target → biosample |
| edge_target_located_in_subcellular_location | target → location |
| edge_target_related_to_go_term | target → GO term |
| edge_target_involves_in_pathway | target → pathway |
| edge_target_has_homologue_in_species_species | target → species |
| edge_target_modelled_by_mouse_gene | target → mouse gene |
| edge_target_has_target_target_interaction_target_{a,b} | target ↔ target |
| edge_target_associated_with_adverse_reaction | target → adverse reaction |
| edge_regulatory_element_regulates_target | regulatory element → target |
| edge_credible_set_contains_variant_variant | credible set → variant |
| edge_credible_set_predicts_target_target | credible set → target |
| edge_colocalisation_compares_signal_credible_set_{left,right} | colocalisation ↔ credible set |
| edge_genetic_association_study_reports_trait_{disease,target} | study → trait |
| edge_genetic_association_study_has_credible_set_credible_set | study → credible set |
| edge_pharmacogenomics_annotation_has_{molecule,target,variant} | PGx annotation → entity |
| edge_pathway_is_part_of_pathway / edge_reaction_is_part_of_pathway | pathway hierarchy |
| edge_target_has_summary_association_by_{datasource,datatype,overall}_{direct,indirect}_disease | target → disease (scored summaries) |
| `edge_target_disease_association_<source>_has_object_disease` ×18 | per-datasource target→disease |
| `edge_target_subject_of_target_disease_association_<source>` ×18 | per-datasource reverse |

(The per-datasource families — chembl, eva, eva_somatic, clingen, crispr, crispr_screen, europepmc, expression_atlas, gene2phenotype, gene_burden, genomics_england, gwas_credible_sets, impc, intogen, orphanet, reactome, uniprot_literature, uniprot_variants, cancer_biomarkers, cancer_gene_census — each contribute matching `has_object_disease`, `subject_of`, and `supported_by_literature_entry` edges, which is what inflates the count to 128.)

---

## 3. Monarch Initiative / Monarch KG

**Source:** Monarch Initiative 2024 (NAR D938); `monarchinitiative.org/kg/about`; robert-haas awesome-biomedical-knowledge-graphs notebook (Monarch dump).
**Scope:** Integrates 33+ biomedical resources + ontologies (PHENIO semantic layer) for cross-species gene–phenotype–disease associations. ~1.12M nodes, ~8.9M edges. Conforms to the **Biolink Model** (all type names are `biolink:` CURIEs). The Monarch project markets "17 node types / 32 edge types" for its core associations; the full materialized dump exposes a long tail of Biolink categories (≈80) and predicates (≈28), enumerated below.

### Node types (Biolink categories — top of ~80)

| Biolink category | Count |
|------------------|------:|
| biolink:Gene | 571,074 |
| biolink:Genotype | 133,380 |
| biolink:PhenotypicFeature | 124,247 |
| biolink:BiologicalProcessOrActivity | 38,308 |
| biolink:Disease | 28,109 |
| biolink:GrossAnatomicalStructure | 24,210 |
| biolink:Cell | 22,454 |
| biolink:Pathway | 22,343 |
| biolink:NamedThing | 19,576 |
| biolink:SequenceVariant | 13,022 |
| biolink:AnatomicalEntity | 9,978 |
| biolink:CellularComponent | 5,308 |
| biolink:MolecularEntity | 4,618 |
| biolink:BiologicalProcess | 3,656 |
| biolink:MacromolecularComplex | 2,120 |
| biolink:MolecularActivity | 1,446 |
| biolink:Protein | 1,112 |
| biolink:CellularOrganism | 958 |
| biolink:Vertebrate | 547 |
| biolink:Virus | 321 |
| biolink:BehavioralFeature | 297 |
| biolink:ChemicalEntity | 267 |
| biolink:LifeStage | 238 |
| biolink:PathologicalProcess | 231 |
| biolink:Drug | 100 |
| biolink:SmallMolecule | 70 |
| biolink:OrganismTaxon | 26 |
| biolink:InformationContentEntity | 23 |
| biolink:NucleicAcidEntity | 18 |
| biolink:EvidenceType | 16 |
| biolink:RNAProduct | 8 |
| biolink:Transcript | 6 |

Plus a long tail (count 1–4 each): Plant, Fungus, ProcessedMaterial, PopulationOfIndividualOrganisms, Activity, ConfidenceLevel, Publication, Mammal, Agent, ProteinFamily, Dataset, GeneticInheritance, EnvironmentalFeature, Invertebrate, Haplotype, Bacterium, ChemicalMixture, ChemicalExposure, CellLine, OrganismalEntity, Event, EnvironmentalProcess, DrugExposure, Human, ProteinDomain, Patent, Study, AccessibleDnaRegion, BiologicalSex, StudyVariable, Zygosity, ReagentTargetedGene, Exon, DiagnosticAid, DatasetDistribution, Genome, MaterialSample, MicroRNA, IndividualOrganism, GenotypicSex, Polypeptide, PhenotypicSex, RegulatoryRegion, SiRNA, Snv, TranscriptionFactorBindingSite, Treatment, WebPage.

### Edge/relation types (Biolink predicates, 28)

| Biolink predicate | Count |
|-------------------|------:|
| biolink:interacts_with | 2,799,181 |
| biolink:expressed_in | 2,320,065 |
| biolink:has_phenotype | 1,703,070 |
| biolink:enables | 839,097 |
| biolink:actively_involved_in | 787,306 |
| biolink:orthologous_to | 551,418 |
| biolink:located_in | 500,184 |
| biolink:subclass_of | 491,204 |
| biolink:related_to | 282,852 |
| biolink:participates_in | 272,586 |
| biolink:acts_upstream_of_or_within | 181,576 |
| biolink:active_in | 160,549 |
| biolink:part_of | 96,113 |
| biolink:causes | 16,839 |
| biolink:is_sequence_variant_of | 15,605 |
| biolink:model_of | 9,902 |
| biolink:acts_upstream_of | 9,366 |
| biolink:has_mode_of_inheritance | 8,577 |
| biolink:gene_associated_with_condition | 8,026 |
| biolink:contributes_to | 7,746 |
| biolink:treats_or_applied_or_studied_to_treat | 5,653 |
| biolink:associated_with_increased_likelihood_of | 3,244 |
| biolink:colocalizes_with | 2,937 |
| biolink:genetically_associated_with | 2,156 |
| biolink:acts_upstream_of_positive_effect | 549 |
| biolink:acts_upstream_of_or_within_positive_effect | 512 |
| biolink:acts_upstream_of_negative_effect | 196 |
| biolink:acts_upstream_of_or_within_negative_effect | 180 |

---

## 4. ROBOKOP (Reasoning Over Biomedical Objects linked in Knowledge Oriented Pathways)

**Source:** ROBOKOP (RENCI / NCATS Translator); live Automat schema API `automat.renci.org/robokopkg/meta_knowledge_graph` (v1.4). Aggregates 30+ sources; ~10M nodes / ~250M edges. Conforms to the **Biolink Model**. The node/predicate lists below are pulled directly from the live `meta_knowledge_graph` endpoint.

### Node categories (30, Biolink)

```
biolink:Activity, biolink:AnatomicalEntity, biolink:Behavior,
biolink:BiologicalEntity, biolink:BiologicalProcess, biolink:Cell,
biolink:CellLine, biolink:CellularComponent, biolink:ChemicalEntity,
biolink:ChemicalMixture, biolink:ClinicalAttribute,
biolink:ComplexMolecularMixture, biolink:Device, biolink:Disease,
biolink:Drug, biolink:Gene, biolink:GeneFamily,
biolink:GrossAnatomicalStructure, biolink:InformationContentEntity,
biolink:MolecularActivity, biolink:MolecularMixture,
biolink:NucleicAcidEntity, biolink:OrganismTaxon, biolink:Pathway,
biolink:Phenomenon, biolink:PhenotypicFeature, biolink:Procedure,
biolink:Protein, biolink:SequenceVariant, biolink:SmallMolecule
```

### Edge/predicate types (75, Biolink)

```
biolink:active_in, biolink:actively_involved_in, biolink:acts_upstream_of,
biolink:acts_upstream_of_negative_effect,
biolink:acts_upstream_of_or_within_negative_effect,
biolink:acts_upstream_of_or_within_positive_effect,
biolink:acts_upstream_of_positive_effect, biolink:affects,
biolink:affects_response_to, biolink:ameliorates_condition,
biolink:applied_to_treat, biolink:associated_with, biolink:capable_of,
biolink:catalyzes, biolink:causes, biolink:chemically_similar_to,
biolink:coexists_with, biolink:coexpressed_with, biolink:colocalizes_with,
biolink:composed_primarily_of, biolink:contraindicated_in,
biolink:contributes_to, biolink:correlated_with, biolink:decreases_response_to,
biolink:derives_from, biolink:develops_from, biolink:diagnoses,
biolink:directly_physically_interacts_with, biolink:disease_has_basis_in,
biolink:disrupts, biolink:enables, biolink:expressed_in,
biolink:gene_associated_with_condition, biolink:gene_product_of,
biolink:genetically_associated_with, biolink:genetically_interacts_with,
biolink:has_active_ingredient, biolink:has_adverse_event, biolink:has_input,
biolink:has_metabolite, biolink:has_output, biolink:has_part,
biolink:has_participant, biolink:has_phenotype, biolink:homologous_to,
biolink:in_complex_with, biolink:in_taxon, biolink:increases_response_to,
biolink:interacts_with, biolink:is_frameshift_variant_of,
biolink:is_missense_variant_of, biolink:is_nearby_variant_of,
biolink:is_non_coding_variant_of, biolink:is_nonsense_variant_of,
biolink:is_splice_site_variant_of, biolink:is_synonymous_variant_of,
biolink:located_in, biolink:manifestation_of, biolink:negatively_correlated_with,
biolink:occurs_in, biolink:overlaps, biolink:physically_interacts_with,
biolink:positively_correlated_with, biolink:precedes,
biolink:predisposes_to_condition, biolink:preventative_for_condition,
biolink:produces, biolink:regulates, biolink:related_to, biolink:similar_to,
biolink:subclass_of, biolink:target_for, biolink:temporally_related_to,
biolink:treats, biolink:treats_or_applied_or_studied_to_treat
```

---

## 5. DRKG (Drug Repurposing Knowledge Graph)

**Source:** `gnn4dr/DRKG` (Amazon/AWS, Ioannidis et al.). 97,238 entities of **13 types**, 5,874,261 triplets of **107 edge types** distributed over **17 entity-type pairs**. Integrates **6 source databases + bibliography**: DrugBank, GNBR, Hetionet, STRING, IntAct, DGIdb. Relation names follow the convention **`SourceDB::RelationType::HeadEntityType:TailEntityType`** (e.g. `Hetionet::CtD::Compound:Disease`, `GNBR::T::Compound:Disease`, `DGIDB::INHIBITOR::Gene:Compound`, `DRUGBANK::ddi-interactor-in::Compound:Compound`, `STRING::BINDING::Gene:Gene`, `INTACT::DIRECT INTERACTION::Gene:Gene`).

### Entity (node) types (13)

| Entity type | Count | Source(s) |
|-------------|------:|-----------|
| Gene | 39,220 | Drugbank, GNBR, Hetionet, STRING, IntAct, DGIdb, Bibliography |
| Compound | 24,313 | Drugbank, GNBR, Hetionet, IntAct, DGIdb, Bibliography |
| Biological Process | 11,381 | Hetionet |
| Side Effect | 5,701 | Hetionet |
| Disease | 5,103 | Drugbank, GNBR, Hetionet, Bibliography |
| Atc | 4,048 | Drugbank |
| Molecular Function | 2,884 | Hetionet |
| Pathway | 1,822 | Hetionet |
| Cellular Component | 1,391 | Hetionet |
| Symptom | 415 | Hetionet |
| Anatomy | 400 | Hetionet |
| Pharmacologic Class | 345 | Hetionet |
| Tax | 215 | GNBR |

### Relation types (107, organized by entity-pair)

The 107 edge types span 17 entity-pairs. Triplet counts per entity-pair × source:

| Entity-type pair | Drugbank | GNBR | Hetionet | STRING | IntAct | DGIdb | Biblio. | Total |
|------------------|---------:|-----:|---------:|-------:|-------:|------:|--------:|------:|
| (Gene, Gene) | — | 66,722 | 474,526 | 1,496,708 | 254,346 | — | 58,629 | 2,350,931 |
| (Compound, Gene) | 24,801 | 80,803 | 51,429 | — | 1,805 | 26,290 | 25,666 | 210,794 |
| (Disease, Gene) | — | 95,399 | 27,977 | — | — | — | 461 | 123,837 |
| (Atc, Compound) | 15,750 | — | — | — | — | — | — | 15,750 |
| (Compound, Compound) | 1,379,271 | — | 6,486 | — | — | — | — | 1,385,757 |
| (Compound, Disease) | 4,968 | 77,782 | 1,145 | — | — | — | — | 83,895 |
| (Gene, Tax) | — | 14,663 | — | — | — | — | — | 14,663 |
| (Biological Process, Gene) | — | — | 559,504 | — | — | — | — | 559,504 |
| (Disease, Symptom) | — | — | 3,357 | — | — | — | — | 3,357 |
| (Anatomy, Disease) | — | — | 3,602 | — | — | — | — | 3,602 |
| (Disease, Disease) | — | — | 543 | — | — | — | — | 543 |
| (Anatomy, Gene) | — | — | 726,495 | — | — | — | — | 726,495 |
| (Gene, Molecular Function) | — | — | 97,222 | — | — | — | — | 97,222 |
| (Compound, Pharmacologic Class) | — | — | 1,029 | — | — | — | — | 1,029 |
| (Cellular Component, Gene) | — | — | 73,566 | — | — | — | — | 73,566 |
| (Gene, Pathway) | — | — | 84,372 | — | — | — | — | 84,372 |
| (Compound, Side Effect) | — | — | 138,944 | — | — | — | — | 138,944 |

**Per-source relation vocabularies** (the 107 edge types are the union of):
- **Hetionet** (24 metaedges — see §10): `CtD` (treats), `CpD` (palliates), `CbG` (binds), `CuG`/`CdG` (up/downregulates), `CrC` (resembles), `CcSE` (causes side effect), `DaG`/`DuG`/`DdG`, `DlA`, `DpS`, `DrD`, `AeG`/`AuG`/`AdG`, `GiG`, `GcG`, `Gr>G`, `GpBP`/`GpCC`/`GpMF`/`GpPW`, `PCiC`.
- **GNBR** (text-mined themes — see §11): chemical-gene `A+`,`A-`,`B`,`E+`,`E-`,`E`,`N`,`O`,`K`,`Z`; gene-disease `U`,`Ud`,`D`,`J`,`Te`,`Y`,`G`,`Md`,`X`,`L`; chemical-disease `T`,`C`,`Sa`,`Pr`,`Pa`,`J`,`Mp`; gene-gene `B`,`W`,`V+`,`E+`,`E`,`I`,`H`,`Rg`,`Q`.
- **DrugBank**: compound-gene target/enzyme/carrier/transporter relations, drug-drug interactions (`ddi-interactor-in`), ATC codes, treats.
- **STRING**: `BINDING`, `ACTIVATION`, `INHIBITION`, `CATALYSIS`, `REACTION`, `EXPRESSION`, `PTMOD`, `OTHER` (gene-gene).
- **IntAct**: `DIRECT INTERACTION`, `PHYSICAL ASSOCIATION`, `ASSOCIATION`, etc. (gene-gene).
- **DGIdb**: drug–gene interaction classes such as `INHIBITOR`, `AGONIST`, `ANTAGONIST`, `BLOCKER`, `ACTIVATOR`, `MODULATOR`, `ANTIBODY`, `CHANNEL BLOCKER`, `OTHER`, etc.

> The complete machine-readable enumeration of all 107 edge names lives in `relation_glossary.tsv` / `embed/relations.tsv` inside the downloadable `drkg.tar.gz`, not in the repo source tree.

---

## 6. PharmKG

**Source:** Zheng et al., *PharmKG: a dedicated knowledge graph benchmark for biomedical data mining*, Briefings in Bioinformatics 2021. (`MindRank-Biotech/PharmKG`, Zenodo 4525237.)
**Scope:** A curated benchmark KG of **gene–drug–disease** triples, ~500,000 interconnections over ~7,600 disambiguated entities, with **29 relation types** (some sources cite 28 for the de-duplicated PharmKG-8K split) in 4 top-level categories. Integrated from 6 curated databases (OMIM, DrugBank, PharmGKB, TTD, SIDER, HumanNet) plus GNBR text-mined triples; entities carry multi-omics features (gene expression, chemical structure, disease word embeddings).

### Node (entity) types (3)

| Entity type | Count |
|-------------|------:|
| Gene | 4,759 |
| Disease | 1,347 |
| Chemical / Drug | 1,497 (FDA-approved) |

### Relation types (29, in 4 categories)

PharmKG adopts the **GNBR semantic-theme vocabulary** as its relation labels. Grouped by category:

| Category | Relation theme codes (GNBR) |
|----------|------------------------------|
| **Interactions (Gene–Gene)** | B (binding), W (enhances response), V+ (activates/stimulates), E+ (increases expression), E (affects expression), I (signaling pathway), H (same protein/complex), Rg (regulation), Q (production by cell pop.) |
| **Chemical–Gene** | A+ (agonism), A- (antagonism), B (binding/ligand), E+ (increases expression), E- (decreases expression), E (affects expression), N (inhibits), O (transport/channels), K (metabolism/PK), Z (enzyme activity) |
| **Chemical–Disease** | T (treatment/therapy), C (inhibits cell growth), Sa (side effect/adverse), Pr (prevents/suppresses), Pa (alleviates/reduces), J (role in pathogenesis), Mp (biomarker of progression) |
| **Gene–Disease** | U (causal mutations), Ud (mutations affect disease course), D (drug targets), J (role in pathogenesis), Te (possible therapeutic effect), Y (polymorphisms alter risk), G (promotes progression), Md (diagnostic biomarkers), X (overexpression in disease), L (improper regulation) |

(Full theme definitions are listed in §11. The exact 29 vs. 28 count depends on the PharmKG split; the semantic vocabulary above is the canonical source.)

---

## 7. CKG (Clinical Knowledge Graph)

**Source:** Santos et al., *A knowledge graph to interpret clinical proteomics data*, Nature Biotechnology 2022. (`MannLabs/CKG`.) ~16M nodes / ~220M relationships, Neo4j-backed. Proteomics- and clinical-centric. Node/relationship type names are Neo4j labels (PascalCase) and relationship types (UPPER_SNAKE_CASE). Counts below are from a representative dump.

### Node (entity) types (32)

| Node type | Count |
|-----------|------:|
| Known_variant | 10,630,108 |
| Publication | 1,791,712 |
| Peptide | 1,001,105 |
| Transcript | 280,910 |
| Protein | 228,725 |
| Clinically_relevant_variant | 190,334 |
| Metabolite | 114,222 |
| Pathway | 51,219 |
| Protein_structure | 49,317 |
| Gene | 42,571 |
| Biological_process | 28,642 |
| Modified_protein | 21,407 |
| Amino_acid_sequence | 20,614 |
| Functional_region | 16,169 |
| Phenotype | 15,872 |
| Molecular_function | 11,169 |
| Disease | 10,791 |
| Experimental_factor | 9,883 |
| GWAS_study | 8,713 |
| Tissue | 5,897 |
| Cellular_component | 4,176 |
| Experiment | 2,829 |
| Complex | 2,700 |
| Modification | 1,978 |
| Food | 992 |
| Units | 442 |
| Analytical_sample | 172 |
| Biological_sample | 170 |
| Subject | 169 |
| Chromosome | 25 |
| Project | 7 |
| User | 2 |

### Relationship types (39)

| Relationship type | Count |
|-------------------|------:|
| MENTIONED_IN_PUBLICATION | 111,109,238 |
| VARIANT_FOUND_IN_PROTEIN | 26,807,293 |
| ASSOCIATED_WITH | 16,707,629 |
| VARIANT_FOUND_IN_GENE | 10,638,935 |
| VARIANT_FOUND_IN_CHROMOSOME | 10,630,108 |
| BELONGS_TO_PROTEIN | 3,629,058 |
| COMPILED_INTERACTS_WITH | 1,956,612 |
| DETECTED_IN_PATHOLOGY_SAMPLE | 1,697,248 |
| ANNOTATED_IN_PATHWAY | 1,203,809 |
| ACTS_ON | 988,705 |
| HAS_QUANTIFIED_PROTEIN | 797,651 |
| TRANSLATED_INTO | 374,294 |
| CURATED_INTERACTS_WITH | 299,188 |
| LOCATED_IN | 295,912 |
| TRANSCRIBED_INTO | 258,487 |
| HAS_QUANTIFIED_MODIFIED_PROTEIN | 224,478 |
| FOUND_IN_PROTEIN | 204,244 |
| HAS_STRUCTURE | 195,640 |
| HAS_PARENT | 128,349 |
| HAS_MODIFIED_SITE | 21,421 |
| HAS_MODIFICATION | 21,407 |
| HAS_SEQUENCE | 20,614 |
| VARIANT_FOUND_IN_GWAS | 16,128 |
| IS_SUBUNIT_OF | 10,968 |
| CURATED_AFFECTS_INTERACTION_WITH | 10,873 |
| STUDIES_TRAIT | 9,250 |
| PUBLISHED_IN | 3,939 |
| MAPS_TO | 2,289 |
| IS_SUBSTRATE_OF | 994 |
| IS_BIOMARKER_OF_DISEASE | 515 |
| IS_QCMARKER_IN_TISSUE | 249 |
| SPLITTED_INTO | 172 |
| BELONGS_TO_SUBJECT | 170 |
| HAS_ENROLLED | 169 |
| VARIANT_IS_CLINICALLY_RELEVANT | 169 |
| STUDIES_TISSUE | 7 |
| IS_RESPONSIBLE | 7 |
| STUDIES_DISEASE | 7 |
| PARTICIPATES_IN | 7 |

---

## 8. Bioteque

**Source:** Fernández-Torras, Duran-Frigola, Bertoni, et al., *Integrating and formatting biomedical data as pre-calculated knowledge graph embeddings in the Bioteque*, Nature Communications 2022.
**Scope:** Harmonizes >150 data sources into a KG of **12 biological entity types (metanodes)** linked by **67 association types (metaedges)**, used to compute network-embedding "descriptors." Metaedges follow the convention **`MetaNode-relcode-MetaNode`** (e.g. `GEN-ass-DIS` = gene associates-with disease, `CPD-trt-DIS` = compound treats disease, `GEN-ppi-GEN` = gene/protein–protein interaction). The KG explores length-1/2 metapaths plus 135 curated metapaths of length ≥3.

### Entity (metanode) types (12)

| Code | Metanode |
|------|----------|
| GEN | Genes and proteins |
| CLL | Cell lines |
| TIS | Tissues |
| CPD | Compounds (small molecules) |
| DIS | Diseases |
| PHC | Pharmacological classes |
| CHE | Chemical entities |
| PWY | Pathways |
| CMP | Cellular components |
| DOM | Protein domains |
| MFN | Molecular functions |
| PGN | Perturbagens |

### Association (metaedge) relation codes — vocabulary

The 67 metaedges are built from the metanode pairs and the relation-code vocabulary below (combined as `MetaNode-relcode-MetaNode`). Representative metaedges: `CLL-upr-GEN`, `CLL-mut-GEN`, `CHE-hsp-CHE`, `CHE-hsp-CPD`, `CPD-int-GEN`, `CPD-trt-DIS`, `CPD-cau-DIS`, `CPD-has-PHC`, `GEN-ass-DIS`, `GEN-ppi-GEN`, `GEN-pho-GEN`, `GEN-has-MFN`, `GEN-ass-PWY`, `GEN-ass-TIS`, `MFN-hsp-MFN`, `PWY-hsp-PWY`, `PHC-hsp-PHC`, `PGN-bfn-CLL`, `PGN-gfn-CLL`, `TIS-upr-GEN`.

| Relation code | Meaning |
|---------------|---------|
| ass | Association |
| int | Interaction |
| ppi | Protein–protein interaction |
| trt | Treatment |
| cau | Causation |
| upr | Upregulation |
| dwr | Downregulation |
| mut | Mutation |
| pho | Phosphorylation |
| has | Has / contains |
| hsp | Has same parent (ontological similarity) |
| bfn | (Perturbagen) biological function effect |
| gfn | (Perturbagen) genetic function effect |
| xrf | Cross-reference |
| sim | Similarity |
| reg | Regulation |
| cex | Co-expression |
| pdw | Perturbagen downregulates |
| pup | Perturbagen upregulates |
| cnu | Copy number |

> The full enumerated 67-metaedge table is published as Supplementary Data 1–3 of the Nat. Commun. paper; the vocabulary above plus the metanode pairs generate them.

---

## 9. PrimeKG-like / PreCISE and related resources

The prompt mentions "PreCISE/PrimeKG-like resources." These are disease-centric, precision-medicine KGs that follow the same design pattern as PrimeKG (merged gene/protein nodes, merged phenotype/side-effect nodes, disease-anchored relations). Representative members:

| Resource | Node types | Relations | Notes |
|----------|-----------:|----------:|-------|
| **PrimeKG** | 10 | 30 | See §1 (canonical). |
| **TxGNN / PrimeKG-derived** | 10 | 30 | Uses PrimeKG directly for zero-shot drug-indication prediction. |
| **TarKG** | 15 entity categories | 171 relation types | Target-discovery KG centered on 3 core types (Disease, Gene, Compound); 1.14M entities, 32.8M relations. |
| **CBKH / Cornell Biomedical Knowledge Hub** | 10 | ~18 | Integrates 17 sources; entity types: Anatomy, Disease, Drug, Gene, Molecule, Pathway, etc. |
| **Hetionet** (foundational) | 11 | 24 | See §10. |
| **Oregano** | ~11 | ~10 | Drug-repurposing KG (drugs, targets, diseases, side effects, phenotypes, MoA). |

> "PreCISE" appears in the literature as a precision-medicine analysis pipeline built *on top of* PrimeKG rather than as an independent schema; its node/edge taxonomy is inherited from PrimeKG (§1).

---

## 10. Hetionet (foundational metagraph — feeds DRKG)

**Source:** Himmelstein et al., *Systematic integration of biomedical knowledge prioritizes drugs for repurposing*, eLife 2017. (`hetio/hetionet`.) The metagraph schema (11 metanodes, 24 metaedges) is the structural backbone of DRKG's Hetionet portion and the template for several precision-medicine KGs.

### Metanodes (node types, 11)

| Metanode | Abbrev. | Nodes |
|----------|:-------:|------:|
| Anatomy | A | 402 |
| Biological Process | BP | 11,381 |
| Cellular Component | CC | 1,391 |
| Compound | C | 1,552 |
| Disease | D | 137 |
| Gene | G | 20,945 |
| Molecular Function | MF | 2,884 |
| Pathway | PW | 1,822 |
| Pharmacologic Class | PC | 345 |
| Side Effect | SE | 5,734 |
| Symptom | S | 438 |

### Metaedges (relation types, 24)

| Metaedge | Abbrev. | Edges |
|----------|:-------:|------:|
| Anatomy – downregulates – Gene | AdG | 102,240 |
| Anatomy – expresses – Gene | AeG | 526,407 |
| Anatomy – upregulates – Gene | AuG | 97,848 |
| Compound – binds – Gene | CbG | 11,571 |
| Compound – causes – Side Effect | CcSE | 138,944 |
| Compound – downregulates – Gene | CdG | 21,102 |
| Compound – palliates – Disease | CpD | 390 |
| Compound – resembles – Compound | CrC | 6,486 |
| Compound – treats – Disease | CtD | 755 |
| Compound – upregulates – Gene | CuG | 18,756 |
| Disease – associates – Gene | DaG | 12,623 |
| Disease – downregulates – Gene | DdG | 7,623 |
| Disease – localizes – Anatomy | DlA | 3,602 |
| Disease – presents – Symptom | DpS | 3,357 |
| Disease – resembles – Disease | DrD | 543 |
| Disease – upregulates – Gene | DuG | 7,731 |
| Gene – covaries – Gene | GcG | 61,690 |
| Gene – interacts – Gene | GiG | 147,164 |
| Gene – participates – Biological Process | GpBP | 559,504 |
| Gene – participates – Cellular Component | GpCC | 73,566 |
| Gene – participates – Molecular Function | GpMF | 97,222 |
| Gene – participates – Pathway | GpPW | 84,372 |
| Gene > regulates > Gene | Gr>G | 265,672 |
| Pharmacologic Class – includes – Compound | PCiC | 1,029 |

---

## 11. GNBR semantic themes (foundational vocabulary — feeds DRKG & PharmKG)

**Source:** Percha & Altman, *A global network of biomedical relationships derived from text*, Bioinformatics 2018. Text-mined relationships clustered into dependency-path "themes." These theme codes are the relation vocabulary reused by DRKG's GNBR portion and by PharmKG.

### Entity classes (4): Chemical, Gene, Disease (pairs: chemical-gene, gene-gene, chemical-disease, gene-disease)

### Theme codes

**Chemical–Gene:**

| Code | Theme |
|------|-------|
| A+ | Agonism, activation |
| A- | Antagonism, blocking |
| B | Binding, ligand (esp. receptors) |
| E+ | Increases expression/production |
| E- | Decreases expression/production |
| E | Affects expression/production (neutral) |
| N | Inhibits |
| O | Transport, channels |
| K | Metabolism, pharmacokinetics |
| Z | Enzyme activity |

**Gene–Gene:**

| Code | Theme |
|------|-------|
| B | Binding, ligand (esp. receptors) |
| W | Enhances response |
| V+ | Activates, stimulates |
| E+ | Increases expression/production |
| E | Affects expression/production (neutral) |
| I | Signaling pathway |
| H | Same protein or complex |
| Rg | Regulation |
| Q | Production by cell population |

**Chemical–Disease:**

| Code | Theme |
|------|-------|
| T | Treatment / therapy (incl. investigatory) |
| C | Inhibits cell growth (esp. cancers) |
| Sa | Side effect / adverse event |
| Pr | Prevents, suppresses |
| Pa | Alleviates, reduces |
| J | Role in pathogenesis |
| Mp | Biomarkers (of progression) |

**Gene–Disease:**

| Code | Theme |
|------|-------|
| U | Causal mutations |
| Ud | Mutations affecting disease course |
| D | Drug targets |
| J | Role in pathogenesis |
| Te | Possible therapeutic effect |
| Y | Polymorphisms alter risk |
| G | Promotes progression |
| Md | Biomarkers (diagnostic) |
| X | Overexpression in disease |
| L | Improper regulation linked to disease |

---

## Sources

- **PrimeKG** — Chandak, Huang, Zitnik. *Building a knowledge graph to enable precision medicine.* Sci Data 2023. https://www.nature.com/articles/s41597-023-01960-3 · https://github.com/mims-harvard/PrimeKG · https://zitniklab.hms.harvard.edu/projects/PrimeKG/ · https://robert-haas.github.io/awesome-biomedical-knowledge-graphs/notebooks/primekg.html
- **Open Targets** — Platform docs https://platform-docs.opentargets.org/ (associations, evidence, getting-started) · BioCypher adapter https://github.com/biocypher/open-targets (`open_targets/definition/reference_kg/kg.py`)
- **Monarch KG** — Monarch Initiative 2024, NAR https://academic.oup.com/nar/article/52/D1/D938/7449493 · https://monarchinitiative.org/kg/about · https://robert-haas.github.io/awesome-biomedical-knowledge-graphs/notebooks/monarch.html · Biolink Model https://github.com/biolink/biolink-model
- **ROBOKOP** — https://robokop.renci.org/about · live schema API https://automat.renci.org/robokopkg/meta_knowledge_graph · ROBOKOP v1.0 Sci Rep https://www.nature.com/articles/s41598-026-53036-y
- **DRKG** — https://github.com/gnn4dr/DRKG (Readme.md statistics tables)
- **PharmKG** — Zheng et al., Brief Bioinform 2021 https://academic.oup.com/bib/article/22/4/bbaa344/6042240 · https://github.com/MindRank-Biotech/PharmKG · https://zenodo.org/records/4525237
- **CKG** — Santos et al., Nat Biotechnol 2022 · https://github.com/MannLabs/CKG · https://robert-haas.github.io/awesome-biomedical-knowledge-graphs/notebooks/ckg.html
- **Bioteque** — Fernández-Torras et al., Nat Commun 2022 https://www.nature.com/articles/s41467-022-33026-0 · https://pmc.ncbi.nlm.nih.gov/articles/PMC9463154/
- **Hetionet** — Himmelstein et al., eLife 2017 https://pmc.ncbi.nlm.nih.gov/articles/PMC5640425/ · https://github.com/hetio/hetionet (describe/nodes/metanodes.tsv, describe/edges/metaedges.tsv)
- **GNBR** — Percha & Altman, Bioinformatics 2018 https://academic.oup.com/bioinformatics/article/34/15/2614/4911883 · https://github.com/jakelever/GNBR
- **Survey index** — robert-haas, *awesome-biomedical-knowledge-graphs* https://github.com/robert-haas/awesome-biomedical-knowledge-graphs
