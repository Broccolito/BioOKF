# Biomedical Knowledge Graph Taxonomy: SPOKE & Hetionet

A reference for the node (metanode) and edge (metaedge) type schemas of two
biomedical heterogeneous knowledge graphs ("hetnets"):

1. **SPOKE** — the Scalable Precision Medicine Open Knowledge Engine (UCSF, Baranzini Lab)
2. **Hetionet** — the integrative network of disease (Himmelstein et al., 2017)

SPOKE is, by design, a superset descendant of the Hetionet metagraph: it reuses
Hetionet's metanode/metaedge abbreviation convention (a CamelCase node-letter +
lowercase verb-letter + CamelCase node-letter, e.g. `CtD` = Compound–treats–Disease)
and extends it with many additional node and edge types and live, weekly-rebuilt data.

---

## Abbreviation convention (shared by both graphs)

Both graphs name each metaedge with a compact abbreviation built from the
**source metanode initial(s) + a lowercase letter for the verb + target metanode initial(s)**.

- Symmetric (undirected) edges are written with a hyphen: `Compound - resembles - Compound` (`CrC`).
- Directed edges are written with a chevron: `Gene > regulates > Gene` (`Gr>G`).

Metanode abbreviations are 1–2 letters of the type name (e.g. Gene = `G`,
Compound = `C`, Disease = `D`, Anatomy = `A`, Biological Process = `BP`,
Pharmacologic Class = `PC`).

---

# Part 1 — Hetionet v1.0

**Source of truth:** `hetio/hetionet` GitHub repo (`describe/nodes/metanodes.tsv`,
`describe/edges/metaedges.tsv`) and Himmelstein et al. 2017 (eLife).

- **Scale:** 47,031 nodes of **11 types** and 2,250,197 relationships of **24 types**.
- Integrates 29 public resources.
- The canonical academic baseline that SPOKE generalizes and extends.

## 1.1 Hetionet node types (metanodes) — 11

| # | Node type (metanode) | Abbrev. | # Nodes | # Metaedges touching | Example / description |
|---|----------------------|---------|---------|----------------------|-----------------------|
| 1 | Anatomy | `A` | 402 | 4 | Anatomical structures / tissues (Uberon) |
| 2 | Biological Process | `BP` | 11,381 | 1 | Gene Ontology biological processes |
| 3 | Cellular Component | `CC` | 1,391 | 1 | Gene Ontology cellular components |
| 4 | Compound | `C` | 1,552 | 8 | Drugs / chemical compounds (DrugBank) |
| 5 | Disease | `D` | 137 | 8 | Diseases (Disease Ontology) |
| 6 | Gene | `G` | 20,945 | 16 | Human genes (Entrez) |
| 7 | Molecular Function | `MF` | 2,884 | 1 | Gene Ontology molecular functions |
| 8 | Pathway | `PW` | 1,822 | 1 | Biological pathways |
| 9 | Pharmacologic Class | `PC` | 345 | 1 | Drug classes (DrugCentral / FDA) |
| 10 | Side Effect | `SE` | 5,734 | 1 | Adverse drug effects (SIDER) |
| 11 | Symptom | `S` | 438 | 1 | Disease symptoms (MeSH) |

## 1.2 Hetionet edge types (metaedges) — 24

Directionality: `-` = symmetric/undirected, `>` = directed. The four
symmetric metaedges are `CrC`, `DrD`, `GcG`, `GiG`; the one explicitly directed
metaedge is `Gr>G` (Gene regulates Gene).

| # | Metaedge (full name) | Abbrev. | Source | Target | Directionality |
|---|----------------------|---------|--------|--------|----------------|
| 1 | Anatomy – downregulates – Gene | `AdG` | Anatomy | Gene | directed |
| 2 | Anatomy – expresses – Gene | `AeG` | Anatomy | Gene | directed |
| 3 | Anatomy – upregulates – Gene | `AuG` | Anatomy | Gene | directed |
| 4 | Compound – binds – Gene | `CbG` | Compound | Gene | directed |
| 5 | Compound – causes – Side Effect | `CcSE` | Compound | Side Effect | directed |
| 6 | Compound – downregulates – Gene | `CdG` | Compound | Gene | directed |
| 7 | Compound – palliates – Disease | `CpD` | Compound | Disease | directed |
| 8 | Compound – resembles – Compound | `CrC` | Compound | Compound | symmetric |
| 9 | Compound – treats – Disease | `CtD` | Compound | Disease | directed |
| 10 | Compound – upregulates – Gene | `CuG` | Compound | Gene | directed |
| 11 | Disease – associates – Gene | `DaG` | Disease | Gene | directed |
| 12 | Disease – downregulates – Gene | `DdG` | Disease | Gene | directed |
| 13 | Disease – localizes – Anatomy | `DlA` | Disease | Anatomy | directed |
| 14 | Disease – presents – Symptom | `DpS` | Disease | Symptom | directed |
| 15 | Disease – resembles – Disease | `DrD` | Disease | Disease | symmetric |
| 16 | Disease – upregulates – Gene | `DuG` | Disease | Gene | directed |
| 17 | Gene – covaries – Gene | `GcG` | Gene | Gene | symmetric |
| 18 | Gene – interacts – Gene | `GiG` | Gene | Gene | symmetric |
| 19 | Gene – participates – Biological Process | `GpBP` | Gene | Biological Process | directed |
| 20 | Gene – participates – Cellular Component | `GpCC` | Gene | Cellular Component | directed |
| 21 | Gene – participates – Molecular Function | `GpMF` | Gene | Molecular Function | directed |
| 22 | Gene – participates – Pathway | `GpPW` | Gene | Pathway | directed |
| 23 | Gene › regulates › Gene | `Gr>G` | Gene | Gene | directed |
| 24 | Pharmacologic Class – includes – Compound | `PCiC` | Pharmacologic Class | Compound | directed |

**The 11 verbs in Hetionet:** associates, binds, causes, covaries, downregulates,
expresses, includes, interacts, localizes, palliates, participates, presents,
regulates, resembles, treats, upregulates. (Note: several verbs reused across
metanode pairs, e.g. `participates` spans BP/CC/MF/PW; `up/downregulates` spans
Anatomy, Compound, Disease.)

---

# Part 2 — SPOKE (UCSF Baranzini Lab)

**Sources of truth:**
- Morris, Soman, et al., "The scalable precision medicine open knowledge engine
  (SPOKE): a massive knowledge graph of biomedical information," *Bioinformatics*
  39(2): btad080 (2023). PMC9940622.
- The **live SPOKE metagraph API**: `https://spoke.rbvi.ucsf.edu/api/v1/metagraph`
  (returns a cytoscape.js JSON of the current node/edge type schema).
- SPOKE neighborhood explorer: `https://spoke.rbvi.ucsf.edu`; REST API
  docs: `https://spoke.rbvi.ucsf.edu/swagger/`.

SPOKE is rebuilt **weekly** from ~41 databases and is structured by a framework
of **11 ontologies** that maintain consistency and enable mapping/navigation
(e.g. Human Disease Ontology for Disease, Gene Ontology for BP/MF/CC, Uberon for
Anatomy). It uses the same metanode/metaedge abbreviation scheme as Hetionet.

### Two reference points: the published paper vs. the live graph

SPOKE is a moving target. The 2023 paper reports a **canonical snapshot** of
**21 node types, 55 edge types, 27M nodes, 53M edges, 41 databases**. The
**live metagraph** (fetched for this document) has grown to roughly **35 node
types and ~87 edge types**. Both are enumerated below — use the paper's 21/55 as
the citable canonical schema and the live list as the current operational schema.

## 2.1a SPOKE node types — paper canonical (21)

These are the 21 node types reported in the 2023 *Bioinformatics* paper (Table 1).

| # | Node type | Notes |
|---|-----------|-------|
| 1 | Anatomy | Anatomical structures (Uberon) |
| 2 | AnatomyCellType | Cell type within an anatomical context |
| 3 | BiologicalProcess | GO biological processes |
| 4 | CellType | Cell types (Cell Ontology) |
| 5 | CellularComponent | GO cellular components |
| 6 | Compound | Chemical compounds / drugs |
| 7 | Disease | Diseases (Human Disease Ontology) |
| 8 | EC | Enzyme Commission classes |
| 9 | Food | Food items |
| 10 | Gene | Genes |
| 11 | MolecularFunction | GO molecular functions |
| 12 | Nutrient | Nutrients |
| 13 | Organism | Organisms |
| 14 | Pathway | Biological pathways |
| 15 | PharmacologicClass | Drug / pharmacologic classes |
| 16 | Protein | Proteins |
| 17 | ProteinDomain | Protein domains |
| 18 | ProteinFamily | Protein families |
| 19 | Reaction | Biochemical reactions |
| 20 | SideEffect | Adverse drug effects |
| 21 | Symptom | Disease symptoms |

## 2.1b SPOKE node types — live metagraph (~35)

The current live metagraph adds many node types beyond the 21 in the paper.
Full enumerated list from `api/v1/metagraph`:

| # | Node type (live) | In paper? | Description |
|---|------------------|-----------|-------------|
| 1 | Compound | yes | Chemical compounds |
| 2 | Anatomy | yes | Anatomical structures |
| 3 | AnatomyCellType | yes | Cell types within anatomy |
| 4 | BiologicalProcess | yes | GO biological processes |
| 5 | CellType | yes | Cell types |
| 6 | CellularComponent | yes | GO cellular components |
| 7 | Complex | new | Protein complexes |
| 8 | Disease | yes | Diseases |
| 9 | EC | yes | Enzyme Commission classes |
| 10 | Food | yes | Food items |
| 11 | Gene | yes | Genes |
| 12 | Location | new | Geographic / anatomic locations |
| 13 | MolecularFunction | yes | GO molecular functions |
| 14 | Nutrient | yes | Nutrients |
| 15 | Organism | yes | Organisms |
| 16 | Pathway | yes | Biological pathways |
| 17 | PharmacologicClass | yes | Drug classes |
| 18 | Protein | yes | Proteins |
| 19 | ProteinDomain | yes | Protein domains |
| 20 | ProteinFamily | yes | Protein families |
| 21 | PwGroup | new | Pathway groups |
| 22 | Reaction | yes | Biochemical reactions |
| 23 | SideEffect | yes | Adverse drug effects |
| 24 | Symptom | yes | Disease symptoms |
| 25 | SARSCov2 | new | SARS-CoV-2 virus |
| 26 | DatabaseTimestamp | new (meta) | Database version metadata |
| 27 | ClinicalLab | new | Clinical lab tests |
| 28 | Version | new (meta) | Version data |
| 29 | CellLine | new | Cell lines |
| 30 | MiRNA | new | MicroRNAs |
| 31 | DietarySupplement | new | Dietary supplements |
| 32 | Blend | new | Supplement blends |
| 33 | ExtracellularParticle | new | Extracellular particles |
| 34 | SDoH | new | Social determinants of health |

(IDs 26 `DatabaseTimestamp` and 28 `Version` are bookkeeping/metadata
nodes, not biomedical concept nodes.)

## 2.2a SPOKE edge types — paper canonical (55)

The 2023 paper reports **55 edge types**; its main-text enumeration highlights
the following representative metaedges (the full 55-row list is in the paper's
Supplementary Table S1, and is a strict subset of the live list in §2.2b).
Representative SPOKE verbs from the paper: ISA, PARTOF, TREATS, BINDS, ENCODES,
CAUSES, PARTICIPATES, ASSOCIATES, EXPRESSES, UPREGULATES, DOWNREGULATES,
CONTRAINDICATES, RESEMBLES, LOCALIZES, PRESENTS, INTERACTS, CONTAINS, HAS,
CATALYZES, PRODUCES, CONSUMES, TRANSPORTS, INCLUDES, MEMBEROF, CLEAVESTO,
AFFECTS, REGULATES.

For the precise, fully-enumerated current edge list, see §2.2b (the live
metagraph), which supersets the paper's 55.

## 2.2b SPOKE edge types — live metagraph (full enumeration)

Full list from `api/v1/metagraph`. Each row gives the abbreviation, the SPOKE
relationship name (the Neo4j edge label, prefixed `VERB_`), and source → target.

| # | Abbrev. | Relationship (edge label) | Source | Target |
|---|---------|---------------------------|--------|--------|
| 1 | `AiA` | ISA_AiA | Anatomy | Anatomy |
| 2 | `ApA` | PARTOF_ApA | Anatomy | Anatomy |
| 3 | `ECiEC` | ISA_ECiEC | EC | EC |
| 4 | `FiF` | ISA_FiF | Food | Food |
| 5 | `LpL` | PARTOF_LpL | Location | Location |
| 6 | `DiD` | ISA_DiD | Disease | Disease |
| 7 | `ChC` | HASROLE_ChC | Compound | Compound |
| 8 | `CiC` | ISA_CiC | Compound | Compound |
| 9 | `CpC` | PARTOF_CpC | Compound | Compound |
| 10 | `OiO` | ISA_OiO | Organism | Organism |
| 11 | `PpCP` | PARTOF_PpCP | Protein | Complex |
| 12 | `PWiPW` | ISA_PWiPW | Pathway | Pathway |
| 13 | `CtD` | TREATS_CtD | Compound | Disease |
| 14 | `CbP` | BINDS_CbP | Compound | Protein |
| 15 | `GeP` | ENCODES_GeP | Gene | Protein |
| 16 | `CcSE` | CAUSES_CcSE | Compound | SideEffect |
| 17 | `GpBP` | PARTICIPATES_GpBP | Gene | BiologicalProcess |
| 18 | `GpMF` | PARTICIPATES_GpMF | Gene | MolecularFunction |
| 19 | `GpCC` | PARTICIPATES_GpCC | Gene | CellularComponent |
| 20 | `DaG` | ASSOCIATES_DaG | Disease | Gene |
| 21 | `AeG` | EXPRESSES_AeG | Anatomy | Gene |
| 22 | `AuG` | UPREGULATES_AuG | Anatomy | Gene |
| 23 | `AdG` | DOWNREGULATES_AdG | Anatomy | Gene |
| 24 | `CdG` | DOWNREGULATES_CdG | Compound | Gene |
| 25 | `CuG` | UPREGULATES_CuG | Compound | Gene |
| 26 | `GPdG` | DOWNREGULATES_GPdG | Gene | Gene |
| 27 | `GPuG` | UPREGULATES_GPuG | Gene | Gene |
| 28 | `OGuG` | UPREGULATES_OGuG | Gene | Gene |
| 29 | `OGdG` | DOWNREGULATES_OGdG | Gene | Gene |
| 30 | `KGdG` | DOWNREGULATES_KGdG | Gene | Gene |
| 31 | `KGuG` | UPREGULATES_KGuG | Gene | Gene |
| 32 | `OcD` | CAUSES_OcD | Organism | Disease |
| 33 | `PiD` | INCREASEDIN_PiD | Protein | Disease |
| 34 | `PdD` | DECREASEDIN_PdD | Protein | Disease |
| 35 | `FcC` | CONTAINS_FcC | Food | Compound |
| 36 | `CfL` | FOUNDIN_CfL | Compound | Location |
| 37 | `DpL` | PREVALENCE_DpL | Disease | Location |
| 38 | `OeP` | ENCODES_OeP | Organism | Protein |
| 39 | `PhEC` | HAS_PhEC | Protein | EC |
| 40 | `ECcR` | CATALYZES_ECcR | EC | Reaction |
| 41 | `PtC` | TRANSPORTS_PtC | Protein | Compound |
| 42 | `GaS` | ASSOCIATES_GaS | Gene | Symptom |
| 43 | `RpC` | PRODUCES_RpC | Reaction | Compound |
| 44 | `RcC` | CONSUMES_RcC | Reaction | Compound |
| 45 | `CpR` | PARTICIPATES_CpR | Compound | Reaction |
| 46 | `CpPG` | PARTOF_CpPG | Compound | PwGroup |
| 47 | `GpR` | PARTICIPATES_GpR | Gene | Reaction |
| 48 | `GpPG` | PARTOF_GpPG | Gene | PwGroup |
| 49 | `PpR` | PARTICIPATES_PpR | Protein | Reaction |
| 50 | `PpPG` | PARTOF_PpPG | Protein | PwGroup |
| 51 | `PGpR` | PARTICIPATES_PGpR | PwGroup | Reaction |
| 52 | `PGpPG` | PARTOF_PGpPG | PwGroup | PwGroup |
| 53 | `RpPW` | PARTOF_RpPW | Reaction | Pathway |
| 54 | `PctP` | CLEAVESTO_PctP | Protein | Protein |
| 55 | `CamG` | AFFECTS_CamG | Compound | Gene |
| 56 | `GpPW` | PARTICIPATES_GpPW | Gene | Pathway |
| 57 | `PDmPF` | MEMBEROF_PDmPF | ProteinDomain | ProteinFamily |
| 58 | `PDiPD` | INTERACTS_PDiPD | ProteinDomain | ProteinDomain |
| 59 | `PiP` | INTERACTS_PiP | Protein | Protein |
| 60 | `CbPD` | BINDS_CbPD | Compound | ProteinDomain |
| 61 | `DrD` | RESEMBLES_DrD | Disease | Disease |
| 62 | `DlA` | LOCALIZES_DlA | Disease | Anatomy |
| 63 | `DpS` | PRESENTS_DpS | Disease | Symptom |
| 64 | `CcD` | CONTRAINDICATES_CcD | Compound | Disease |
| 65 | `PiC` | INTERACTS_PiC | Protein | Compound |
| 66 | `PDpP` | PARTOF_PDpP | ProteinDomain | Protein |
| 67 | `PCiC` | INCLUDES_PCiC | PharmacologicClass | Compound |
| 68 | `CTiCT` | ISA_CTiCT | CellType | CellType |
| 69 | `CTpA` | PARTOF_CTpA | CellType | Anatomy |
| 70 | `PrG` | REGULATES_PrG | Protein | Gene |
| 71 | `mGrC` | RESISTANT_TO_mGrC | Gene | Compound |
| 72 | `mGrC` | RESPONSE_TO_mGrC | Gene | Compound |
| 73 | `mGrsC` | REDUCES_SEN_mGrsC | Gene | Compound |
| 74 | `CiF` | INTERACTS_CiF | Compound | Food |
| 75 | `GeiCT` | EXPRESSEDIN_GeiCT | Gene | CellType |
| 76 | `PeCT` | EXPRESSEDIN_PeCT | Protein | CellType |
| 77 | `GmpD` | MARKER_POS_GmpD | Gene | Disease |
| 78 | `GmnD` | MARKER_NEG_GmnD | Gene | Disease |
| 79 | `MtG` | TARGETS_MtG | MiRNA | Gene |
| 80 | `GeiD` | EXPRESSEDIN_GeiD | Gene | Disease |
| 81 | `DScC` | CONTAINS_DScC | DietarySupplement | Compound |
| 82 | `BcC` | CONTAINS_BcC | Blend | Compound |
| 83 | `BcF` | CONTAINS_BcF | Blend | Food |
| 84 | `DScB` | CONTAINS_DScB | DietarySupplement | Blend |
| 85 | `SiS` | ISA_SiS | SDoH | SDoH |
| 86 | `CmctD` | MENTIONED_CLINICAL_TRIALS_FOR_CmctD | Compound | Disease |
| 87 | `CictD` | IN_CLINICAL_TRIALS_FOR_CictD | Compound | Disease |

Notes on the live SPOKE edge set:
- SPOKE distinguishes the *provenance* of gene regulation in its abbreviations:
  `GP…G` (gene-perturbation), `OG…G` (over-expression), `KG…G` (knockdown) all
  produce UPREGULATES/DOWNREGULATES Gene→Gene edges (rows 26–31).
- Drug-response pharmacogenomics edges (`mGrC`/`mGrsC`, rows 71–73) carry verbs
  RESISTANT_TO, RESPONSE_TO, REDUCES_SEN(sitivity), Gene → Compound.
- Clinical-trial provenance is explicit: MENTIONED_CLINICAL_TRIALS_FOR (`CmctD`)
  and IN_CLINICAL_TRIALS_FOR (`CictD`), both Compound → Disease (rows 86–87).

---

# Part 3 — SPOKE ↔ Hetionet correspondence

SPOKE inherits Hetionet's core schema and convention. Shared metaedge
abbreviations (identical source/target and abbreviation) include: `CtD`
(Compound treats Disease), `CcSE` (Compound causes SideEffect), `DaG` (Disease
associates Gene), `AeG`/`AuG`/`AdG` (Anatomy expresses/up/down-regulates Gene),
`CuG`/`CdG` (Compound up/down-regulates Gene), `GpBP`/`GpMF`/`GpCC`/`GpPW`
(Gene participates in BP/MF/CC/Pathway), `DrD` (Disease resembles Disease),
`DlA` (Disease localizes Anatomy), `DpS` (Disease presents Symptom), and `PCiC`
(PharmacologicClass includes Compound).

Key differences:
- **Protein as a first-class node.** Hetionet collapses gene products into the
  `Gene` node; SPOKE adds a distinct `Protein` metanode with Gene–ENCODES→Protein
  (`GeP`), Compound–BINDS→Protein (`CbP`), Protein–INTERACTS→Protein (`PiP`), etc.
- **Reaction / EC / Pathway-group machinery.** SPOKE models biochemistry
  explicitly (EC catalyzes Reaction, Reaction produces/consumes Compound, PwGroup).
- **Food, Nutrient, DietarySupplement, Blend.** Nutrition layer absent in Hetionet.
- **Clinical & real-world layer.** ClinicalLab, Location/SDoH, MiRNA, CellLine,
  SARSCov2, clinical-trial edges — none in Hetionet.
- **Hetionet's `Gr>G` (directed gene-regulates-gene)** maps in SPOKE onto the
  family of provenance-typed UPREGULATES/DOWNREGULATES Gene→Gene edges and the
  Protein–REGULATES→Gene (`PrG`) edge.

---

# Sources

- Hetionet metanodes: https://raw.githubusercontent.com/hetio/hetionet/master/describe/nodes/metanodes.tsv
- Hetionet metaedges: https://raw.githubusercontent.com/hetio/hetionet/master/describe/edges/metaedges.tsv
- Hetionet repo: https://github.com/hetio/hetionet
- Hetionet in Neo4j: https://neo4j.het.io/guides/hetionet.html
- Himmelstein et al. 2017 (eLife), "Systematic integration of biomedical knowledge prioritizes drugs for repurposing": https://pmc.ncbi.nlm.nih.gov/articles/PMC5640425/
- SPOKE paper (Morris/Soman et al., Bioinformatics 2023, btad080): https://pmc.ncbi.nlm.nih.gov/articles/PMC9940622/
- SPOKE live metagraph API: https://spoke.rbvi.ucsf.edu/api/v1/metagraph
- SPOKE explorer: https://spoke.rbvi.ucsf.edu
- Baranzini Lab Data Science: https://baranzinilab.ucsf.edu/data-science
