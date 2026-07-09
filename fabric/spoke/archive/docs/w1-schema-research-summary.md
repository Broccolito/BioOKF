# BioOKF and SPOKE Schema Research Summary

This summary captures the first research pass used by the linker harness. `BioLKF` does not appear in the repo; the implementation truth is BioOKF/BOKF v0.5.

## BioOKF Shape

BioOKF uses a closed set of 28 node types and 35 predicates: 24 positive predicates plus 11 canonical `not_*` negative predicates. Concept documents use human-readable bundle-unique `identifier` values; external database IDs live in `xref`; `subtype` is open and agent-coined.

### BioOKF Node Mapping Principles

| BioOKF type family | SPOKE mapping principle |
|---|---|
| Gene | Map primarily to SPOKE `Gene`; avoid collapsing RNA molecules into genes without subtype/xref support. |
| Molecule | Split by subtype/xref into `Protein`, `Compound`, `MiRNA`, `Complex`, `Nutrient`, or `DietarySupplement`; this is the largest ambiguity class. |
| MolecularClass | Map to `PharmacologicClass`, `ProteinFamily`, `ProteinDomain`, `PwGroup`, or review. |
| Variant / SequenceFeature | Map variants to `Variant`/`Haplotype`; map sequence regions only when `Chromosome`/`Cytoband` or exact feature evidence exists. |
| Anatomy / CellType | Map to `Anatomy`, `CellularComponent`, `CellType`, `CellLine`, or `ProvCellType` based on subtype. |
| BiologicalPathway / BiologicalFunction | Split to `Pathway`, `BiologicalProcess`, `Reaction`, `PwGroup`, `MolecularFunction`, or `EC`. |
| Disease / Phenotype / BiomedicalMeasure | Keep disease, symptom, side-effect, and lab measure boundaries separate. |
| Provenance/context types | `Publication`, `Study`, `Dataset`, `Agent`, `Population`, and many locations are normally BioOKF provenance/context rather than biomedical SPOKE nodes. |

Major ambiguity boundaries: Gene vs RNA Molecule, Molecule vs MolecularClass, Variant vs SequenceFeature, BiologicalPathway vs BiologicalFunction, Disease vs Phenotype vs BiomedicalMeasure, and provenance/context nodes vs biomedical entities.

## SPOKE Node Labels

Live SPOKE dev returned 42 labels. Four observed labels had no sampled rows in the current dev graph: `AnatomyCellType`, `ExtracellularParticle`, `Nutrient`, and `SARSCov2`.

| SPOKE label group | Meaning and identifier hints | BioOKF candidate |
|---|---|---|
| `Gene`, `PanGene`, `MiRNA` | Entrez integer genes, pangenome gene clusters, mature miRNAs (`MIMAT`, `hsa-miR-*`). | Gene or Molecule/RNA |
| `Protein`, `ProteinDomain`, `ProteinFamily`, `Complex` | UniProt, Pfam domains/families, Complex Portal/CORUM. | Molecule or MolecularClass |
| `Compound`, `PharmacologicClass`, `Nutrient`, `DietarySupplement`, `Blend`, `Food` | InChIKey/ChEMBL/PubChem/DrugBank compounds; FDA/NDFRT drug classes; FOODON/NHANES food/supplement content. | Molecule, MolecularClass, Food |
| `Variant`, `Haplotype`, `Chromosome`, `Cytoband` | dbSNP rsIDs, PharmVar haplotypes, cytogenomic regions. | Variant or SequenceFeature |
| `Disease`, `Symptom`, `SideEffect`, `ClinicalLab` | DOID diseases, HPO/MeSH symptoms, SIDER side effects, LOINC-like labs. | Disease, Phenotype, BiomedicalMeasure |
| `Anatomy`, `CellType`, `CellLine`, `ProvCellType`, `CellularComponent` | UBERON/HOMBA anatomy, CL/CLO/Cellosaurus/PCL, GO CC. | Anatomy or CellType |
| `BiologicalProcess`, `MolecularFunction`, `Pathway`, `Reaction`, `PwGroup`, `EC` | GO, Reactome, KEGG, MetaCyc, ExplorEnz. | BiologicalPathway or BiologicalFunction |
| `Organism`, `Environment`, `SDoH`, `Location` | BV-BRC taxa/strains, ENVO, SNOMED social determinants, GeoNames/ISO/FIPS. | Organism, Exposure, SocialFactor, GeographicLocation |
| `Version`, `DatabaseTimestamp`, `TestNode` | SPOKE metadata/test nodes. | Dataset metadata or exclude |

## SPOKE Relationship Families

Live SPOKE dev returned 129 relationship types. SPOKE relationship names encode source/target classes, so the linker maps BioOKF predicates to relationship prefixes instead of one fixed relationship name.

| SPOKE family | Meaning | BioOKF predicate |
|---|---|---|
| `ISA_*`, `PARTOF_*`, `CONTAINS_*` | Ontology hierarchy and partonomy; `CONTAINS_*` often requires direction reversal. | `is_a`, `part_of` |
| `ENCODES_*` | Gene/organism/pangene encodes protein/miRNA. | `encodes` |
| `BINDS_*`, `INTERACTS_*` | Compound-protein/domain and protein/compound/complex interactions. | `binds`, `interacts_with` |
| `UPREGULATES_*`, `DOWNREGULATES_*`, `REGULATES_*`, `TARGETS_*`, `AFFECTS_*` | Signed regulation, miRNA targeting, drug sensitivity effects. | `regulates`, sometimes `associated_with` |
| `TREATS_*`, `IN_CLINICAL_TRIALS_FOR_*`, `CONTRAINDICATES_*` | Clinical treatment/trial/contraindication. | `treats`, `prevents`, `contraindicated_in` |
| `CAUSES_*`, `PRESENTS_*` | Etiology/adverse effects and disease symptoms. | `causes`, `has_phenotype` |
| `ASSOCIATES_*`, `RESEMBLES_*`, `CORRELATES_*`, `MARKER_*` | Association, similarity, correlation, marker evidence. | `associated_with`, sometimes `predisposes_to` or `measures` |
| `EXPRESSEDIN_*`, `EXPRESSES_*`, `ENRICHED_IN_*` | Expression/enrichment context. | `expressed_in` |
| `MEASURES*` | Clinical lab measures entity/context. | `measures` |
| `PARTICIPATES_*`, `CATALYZES_*`, `CONSUMES_*`, `PRODUCES_*`, `CLEAVESTO_*` | Pathway/reaction participation and biochemical transforms. | `participates_in`, `catalyzes`, `converts_to` |
| `LOCALIZES_*`, `FOUNDIN_*`, `ISOLATEDIN_*`, `PREVALENCE*`, `MORTALITY_*` | Location and epidemiologic/geographic metrics. | `located_in` or quantified `associated_with` |

High-risk zones are reverse-direction class membership and containment, clinical trials that do not prove treatment, prevalence/mortality as quantified context rather than location, and `not_*` BioOKF predicates. The linker treats `not_*` as possible contradictions only when a positive SPOKE base relationship exists; absence of a SPOKE edge is not proof of a negative claim.
