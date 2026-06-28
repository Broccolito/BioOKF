# Biomedical Datasets & Tabular Data Source Catalog (BioOKF)

Openly-accessible biomedical **datasets in CSV / Excel / tabular form**: NCBI GEO
series, figshare / Zenodo / Dryad / Mendeley deposits, Kaggle biomedical datasets,
UCI ML repository medical sets, supplementary data tables, and large clinical /
omics matrices and registries. The defining schema for this source class is a
**matrix / table**: rows are biomedical entities (samples, patients, genes,
variants, cells, compounds, taxa) and columns are measurements, features, or
other entities. Each item notes its dominant entity row/column axis and the
implicit relationship the table encodes.

All URLs verified June 2026 (data-portal landing pages, GEO accessions, repository
DOIs, and registry download pages all resolve and are openly downloadable, modulo
credentialed-access sets like MIMIC/UK Biobank which are flagged).

Total items: 72

## Catalog

| # | Title | URL | Format | Subfield | Key entities & relationships |
|---|-------|-----|--------|----------|------------------------------|
| 1 | NCBI GEO — Gene Expression Omnibus (repository root + download docs) | https://www.ncbi.nlm.nih.gov/geo/info/download.html | HTML/TSV | Functional genomics / repository | Series (GSE) → samples (GSM) × genes; series-matrix + supplementary count tables → sample–gene expression |
| 2 | GEO GSE132465 — colorectal cancer 10x scRNA-seq raw UMI count matrix + cell annotation | https://www.ncbi.nlm.nih.gov/geo/query/acc.cgi?acc=GSE132465 | TXT/TSV (gz) | Single-cell genomics | Cells × genes UMI matrix + cell-type annotation → cell–gene, cell–celltype, gene–tumor |
| 3 | GEO GSE137710 — human melanoma scRNA-seq cell metadata (9315×14) + counts | https://www.ncbi.nlm.nih.gov/geo/query/acc.cgi?acc=GSE137710 | TSV (gz) | Single-cell genomics | Cells × metadata/genes → cell–gene, cell–sample, cell–phenotype |
| 4 | GEO GSE50161 — brain tumor microarray (basis of CuMiDa) | https://www.ncbi.nlm.nih.gov/geo/query/acc.cgi?acc=GSE50161 | CSV/SOFT | Cancer transcriptomics | Samples (tumor types) × probes/genes → sample–tumor-class, gene–expression |
| 5 | NCBI-generated RNA-seq raw counts (GEO RNA-seq counts portal) | https://www.ncbi.nlm.nih.gov/geo/info/rnaseqcounts.html | TSV | Functional genomics / repository | Samples × genes uniformly recomputed counts → sample–gene expression |
| 6 | ARCHS4 — all human/mouse RNA-seq counts (uniformly processed) | https://archs4.org/ | H5 / TSV | Functional genomics / compendium | ~1M+ samples × genes/transcripts → sample–gene expression, sample–tissue/celltype |
| 7 | recount3 — 750k+ uniformly processed RNA-seq samples | https://rna.recount.bio/ | RSE / TSV | Functional genomics / compendium | Samples × genes/exons/junctions, 8.6k studies → sample–gene, sample–study expression |
| 8 | Expression Atlas / Single Cell Expression Atlas (EMBL-EBI) | https://www.ebi.ac.uk/gxa/sc/home | TSV/MTX | Functional genomics / atlas | Genes × tissues/cell-types baseline + differential → gene–tissue, gene–condition expression |
| 9 | CZ CELLxGENE Discover — curated single-cell atlases (incl. Human Brain Cell Atlas v1.0) | https://cellxgene.cziscience.com/collections/283d65eb-dd53-496d-adb7-7570c7caa443 | h5ad / CSV | Single-cell atlas | Cells × genes + cell metadata → cell–gene, cell–celltype, cell–donor |
| 10 | Human Cell Atlas — Data Coordination Platform (matrix service) | https://data.humancellatlas.org/ | MTX/CSV | Single-cell atlas | Cells × genes across organs/donors → cell–gene, cell–tissue, cell–celltype |
| 11 | hECA v2.0 — AI-ready ensemble single-cell RNA+ATAC atlas | https://pmc.ncbi.nlm.nih.gov/articles/PMC12852668/ | h5 / matrix | Single-cell multiomics | Cells × genes/peaks → cell–gene, cell–region accessibility, cell–celltype |
| 12 | TCGA PanCanAtlas — clinical_PANCAN_patient_with_followup.tsv + omics tables | https://gdc.cancer.gov/about-data/publications/pancanatlas | TSV | Cancer genomics / clinical | Patients × clinical/survival; samples × genes (expr/mutation/CNV) → patient–outcome, sample–gene–alteration |
| 13 | cBioPortal — Breast Invasive Carcinoma (TCGA, PanCancer Atlas) study | https://www.cbioportal.org/study/summary?id=brca_tcga_pan_can_atlas_2018 | TSV | Cancer genomics | Samples × mutations/CNA/expression + patient clinical → sample–gene alteration, patient–survival |
| 14 | cBioPortal — Colorectal Adenocarcinoma (TCGA, PanCancer Atlas) study | https://www.cbioportal.org/study/summary?id=coadread_tcga_pan_can_atlas_2018 | TSV | Cancer genomics | Samples × gene alterations + clinical → sample–gene, patient–outcome |
| 15 | DepMap (Cancer Dependency Map) — CRISPR gene effect + drug sensitivity | https://depmap.org/portal/download/ | CSV | Cancer functional genomics | Cell lines × genes (dependency); cell lines × compounds (logfold) → cellline–gene essentiality, cellline–drug sensitivity |
| 16 | CCLE — Cancer Cell Line Encyclopedia omics (via DepMap) | https://depmap.org/portal/download/all/ | CSV | Cancer pharmacogenomics | Cell lines × genes (expr/mutation/CNV) → cellline–gene, cellline–lineage |
| 17 | GDSC — Genomics of Drug Sensitivity in Cancer (IC50 matrices) | https://www.cancerrxgene.org/downloads/bulk_download | CSV/XLSX | Cancer pharmacogenomics | Cell lines × drugs (IC50/AUC) → cellline–drug response, drug–gene-marker |
| 18 | depmap Bioconductor data package (DepMap/CCLE as tidy tables) | https://www.bioconductor.org/packages/release/data/experiment/html/depmap.html | tibble/CSV | Cancer pharmacogenomics | depmap_id × gene/compound long tables → cellline–gene–dependency, cellline–drug |
| 19 | GWAS Catalog — full association table (all SNP-trait associations) | https://www.ebi.ac.uk/gwas/docs/file-downloads | TSV | GWAS / catalog | Rows = SNP–trait associations × study/p-value/OR → variant–trait, variant–gene mapping |
| 20 | GWAS Catalog — harmonized full summary statistics (85k datasets) | https://www.ebi.ac.uk/gwas/summary-statistics | TSV (gz) | GWAS / summary stats | Variants × beta/SE/p per study → variant–trait effect size (genome-wide) |
| 21 | ClinVar — variant_summary.txt (variant–condition interpretations) | https://ftp.ncbi.nlm.nih.gov/pub/clinvar/tab_delimited/ | TSV | Clinical variant database | Variants × gene/condition/clinical-significance → variant–phenotype pathogenicity |
| 22 | gnomAD — per-variant allele frequencies (sites tables / Hail/TSV exports) | https://gnomad.broadinstitute.org/downloads | TSV/VCF | Population genomics | Variants × population allele frequency / constraint → variant–population frequency, gene–LOEUF constraint |
| 23 | GTEx v8 — gene TPM + eQTL signif-pairs matrices | https://www.gtexportal.org/home/downloads/adult-gtex/overview | GCT/TSV | Functional genomics / eQTL | Genes × tissues (TPM); variant–gene–tissue eQTL pairs → expression, cis/trans regulation |
| 24 | UK Biobank — phenotype/genotype tabular data (showcase) [registered access] | https://www.ukbiobank.ac.uk/enable-your-research/about-our-data | TSV/CSV | Biobank / cohort | Participants × phenotypes/biomarkers/variants → participant–phenotype, variant–trait |
| 25 | All of Us Researcher Workbench — participant survey/EHR/genomic tables [registered] | https://www.researchallofus.org/data-tools/data-snapshots/ | CSV | Biobank / cohort | Participants × measurements/conditions/variants → participant–condition, variant–ancestry |
| 26 | Kaggle — Gene expression dataset (Golub et al. AML/ALL leukemia) | https://www.kaggle.com/datasets/crawford/gene-expression | CSV | Cancer transcriptomics | Patients × 7129 genes + ALL/AML label → patient–gene expression, patient–cancer-class |
| 27 | Kaggle — Brain cancer gene expression (CuMiDa, GSE50161) | https://www.kaggle.com/datasets/brunogrisci/brain-cancer-gene-expression-cumida | CSV | Cancer transcriptomics | Samples × genes + tumor-type label → sample–gene, sample–tumor-class |
| 28 | Kaggle — Genomic Data for Cancer (classification) | https://www.kaggle.com/datasets/brsahan/genomic-data-for-cancer | CSV | Cancer genomics | Samples × genomic features + cancer label → sample–feature, sample–class |
| 29 | Kaggle — Cardiovascular Disease dataset (70k patients) | https://www.kaggle.com/datasets/sulianova/cardiovascular-disease-dataset | CSV | Clinical / cardiology | Patients × 11 features (BP, cholesterol, glucose) + CVD label → patient–risk-factor, patient–disease |
| 30 | Kaggle — Diabetes Clinical Dataset (100k rows) | https://www.kaggle.com/datasets/ziya07/diabetes-clinical-dataset100k-rows | CSV | Clinical / endocrinology | Patients × clinical/lab features + diabetes label → patient–biomarker, patient–diagnosis |
| 31 | Kaggle — Parkinson's Disease (biomedical voice measurements) | https://www.kaggle.com/datasets/vikasukani/parkinsons-disease-data-set | CSV | Clinical / neurology | Voice recordings × acoustic features (jitter/shimmer/HNR) + status → recording–feature, subject–disease |
| 32 | Kaggle — Parkinson's Disease (PD) speech signal features (Sakar) | https://www.kaggle.com/datasets/dipayanbiswas/parkinsons-disease-speech-signal-features | CSV | Clinical / neurology | Subjects × 753 speech features + class → subject–feature, subject–PD |
| 33 | Kaggle — Alzheimer's Disease Dataset (2,149 patients) | https://www.kaggle.com/datasets/rabieelkharoua/alzheimers-disease-dataset | CSV | Clinical / neurology | Patients × demographic/lifestyle/cognitive features + diagnosis → patient–assessment, patient–disease |
| 34 | Kaggle — Alzheimer Features (clinical/MRI summary) | https://www.kaggle.com/datasets/brsdincer/alzheimer-features | CSV | Clinical / neurology | Subjects × MMSE/CDR/eTIV/nWBV + group → subject–measure, subject–dementia-stage |
| 35 | Kaggle — MIMIC-IV style ICU dataset for sepsis prediction | https://www.kaggle.com/datasets/sinanshereef/mimic-iv-style-icu-dataset-for-sepsis-prediction | CSV | Clinical / ICU | ICU stays × vitals/labs + sepsis label → patient–measurement, patient–sepsis |
| 36 | Kaggle — Prediction of Sepsis (PhysioNet 2019 Challenge) | https://www.kaggle.com/datasets/salikhussaini49/prediction-of-sepsis | CSV | Clinical / ICU | Hourly patient records × vitals/labs + SepsisLabel → patient–timepoint, patient–sepsis-onset |
| 37 | UCI ML Repository — Breast Cancer Wisconsin (Diagnostic) | https://archive.ics.uci.edu/dataset/17/breast+cancer+wisconsin+diagnostic | CSV | Clinical / oncology | Tumor FNA samples × 30 nuclear features + benign/malignant → sample–feature, sample–diagnosis |
| 38 | UCI ML Repository — Heart Disease (Cleveland et al.) | https://archive.ics.uci.edu/dataset/45/heart+disease | CSV | Clinical / cardiology | Patients × 13 clinical features + disease label → patient–feature, patient–CHD |
| 39 | UCI ML Repository — Pima Indians Diabetes | https://archive.ics.uci.edu/dataset/34/diabetes | CSV | Clinical / endocrinology | Women × 8 clinical features + diabetes label → patient–biomarker, patient–diagnosis |
| 40 | UCI ML Repository — Health datasets index | https://archive.ics.uci.edu/datasets?Keywords=health | CSV | Clinical / multi | Many medical sets: patients/samples × features → entity–feature–label tables |
| 41 | PhysioNet — MIMIC-IV-ECG diagnostic ECG subset (machine_measurements.csv) | https://physionet.org/content/mimic-iv-ecg/1.0/ | CSV/WFDB | Clinical / cardiology | ~800k ECGs × machine measurements + subject/study IDs → ECG–measurement, patient–ECG link |
| 42 | PhysioNet — MIMIC-IV-ECG record_list.csv (ECG↔patient linking) | https://physionet.org/content/mimic-iv-ecg/1.0/record_list.csv | CSV | Clinical / cardiology | Records × subject_id/study_id → ECG–patient, ECG–note links |
| 43 | PhysioNet — MIMIC-IV-ECG Demo (open, 659 ECGs / 92 subjects) | https://physionet.org/content/mimic-iv-ecg-demo/0.1/ | CSV/WFDB | Clinical / cardiology | ECGs × leads/measurements + subjects → ECG–measurement, patient–ECG |
| 44 | PhysioNet — databases index (ECG, EEG, ICU, waveform) | https://physionet.org/about/database/ | CSV/WFDB | Clinical / signals | Patients × signals/labs/annotations → patient–measurement, patient–outcome |
| 45 | figshare — Gene expression CSV files (all detectable genes) | https://figshare.com/articles/dataset/Gene_expression_csv_files/21861975 | CSV | Functional genomics / supplement | Genes × samples expression → gene–sample expression |
| 46 | figshare — Supplementary Table 1.csv (sequencing experiments) | https://figshare.com/articles/dataset/Supplementary_Table_1_csv/16818505 | CSV | Genomics / supplement | Experiments × metadata → sample–experiment provenance |
| 47 | figshare — Proteomics and RNAseq Data.xlsx (cancer vaccine study) | https://figshare.com/articles/dataset/Proteomics_and_RNAseq_Data_xlsx/28872284 | XLSX | Proteogenomics / supplement | Proteins/genes × samples abundance → protein–sample, gene–sample |
| 48 | figshare — Supplementary Table 11.xlsx (omics supplement) | https://figshare.com/articles/dataset/Supplementary_Table_11_xlsx/22233316 | XLSX | Omics / supplement | Features × samples/conditions → feature–condition measurement |
| 49 | figshare — KMDATA reconstructed oncology trial IPD (153 trials) | https://doi.org/10.6084/m9.figshare.14642247.v1 | CSV | Clinical trials / survival | Patients × time-to-event (OS/PFS) + arm → patient–outcome, treatment–survival |
| 50 | Zenodo — Gene Expression (RNA-seq) count matrix + FASTQ | https://zenodo.org/records/6908427 | CSV/count | Functional genomics | Genes × samples counts → gene–sample expression, sample–condition |
| 51 | Zenodo — Predicting Phenotypic Traits Using a Massive RNA-seq Dataset | https://zenodo.org/records/10183151 | CSV | Functional genomics | Samples × gene counts + BioProject annotation → sample–gene, sample–phenotype |
| 52 | Zenodo — Single-cell RNA-seq UltraMarathonRT (featureCounts tables + RDS) | https://zenodo.org/records/17099015 | CSV/RDS | Single-cell genomics | Cells/samples × genes counts → cell–gene expression |
| 53 | Zenodo — Raw Data for IntoValue (ClinicalTrials.gov registry export) | https://zenodo.org/records/7590083 | CSV | Clinical trials / registry | Trials × registration/results metadata → trial–intervention, trial–outcome reporting |
| 54 | Dryad — MERFISH mouse GI tract ± microbiome (spatial + ligand-receptor CSV) | https://datadryad.org/dataset/doi:10.5061/dryad.p5hqbzm0z | CSV | Spatial transcriptomics / microbiome | Cells × genes/coordinates + ligand-receptor → cell–gene, cell–cell interaction |
| 55 | Dryad — ancient dental calculus oral microbiome (174 samples) | https://datadryad.org/dataset/doi:10.5061/dryad.jdfn2z3mk | HTML/CSV | Microbiome / metagenomics | Samples × taxa/QC metrics → sample–taxon abundance, sample–site |
| 56 | Dryad — microbiome metabolomics in irradiated mice (BIO 300) | https://datadryad.org/dataset/doi:10.5061/dryad.hhmgqnkhb | CSV | Microbiome / metabolomics | Samples × metabolites/taxa → sample–metabolite, sample–taxon, treatment–phenotype |
| 57 | Dryad — host genetics & phenotype microbiome of seaweed (160 individuals) | https://datadryad.org/dataset/doi:10.5061/dryad.qz612jmd4 | CSV | Microbiome / host-genetics | Individuals × 16 traits + microbial communities → host–microbe, host–trait |
| 58 | Mendeley Data — blood cancer hematological/clinical multivariate set (2,296×15) | https://data.mendeley.com/datasets/d3vs7sbpt2/2 | CSV | Clinical / hematology | Patients × hematology + SPEP + mutations → patient–biomarker, patient–blood-cancer |
| 59 | Mendeley Data — CBC complete blood count dataset | https://data.mendeley.com/datasets/28s2bhdjfd/1 | CSV | Clinical / hematology | Patients × CBC indices → patient–blood-count, patient–condition |
| 60 | Mendeley Data — Dengue Fever Hematological Dataset | https://data.mendeley.com/datasets/6fsrsk3mb8/2 | CSV | Clinical / infectious disease | Patients × blood indices + diagnosis → patient–lab, patient–dengue |
| 61 | Mendeley Data — Antibiotic Resistance Tracking Dataset | https://data.mendeley.com/datasets/h4byb28gcv/2 | CSV | Clinical microbiology / AMR | Isolates × antibiotics susceptibility → organism–antibiotic resistance |
| 62 | Metabolomics Workbench — study data (mwTab tabular / REST CSV) | https://www.metabolomicsworkbench.org/data/browse.php | mwTab/CSV | Metabolomics | Samples × metabolite concentrations → sample–metabolite, sample–condition |
| 63 | MetaboLights — open metabolomics studies (EMBL-EBI) | https://www.ebi.ac.uk/metabolights/ | TSV (ISA-Tab) | Metabolomics | Samples × metabolites + assay metadata → sample–metabolite, metabolite–pathway |
| 64 | PRIDE / ProteomeXchange — MS proteomics datasets (protein expr matrices) | https://www.ebi.ac.uk/pride/archive | TSV/CSV | Proteomics | Samples × proteins/peptides abundance → sample–protein, protein–PTM |
| 65 | STITCH — chemical–protein interactions (9606.actions / protein_chemical) | http://stitch.embl.de/cgi/download.pl | TSV (gz) | Chemoinformatics / interactions | Compounds × proteins + score/mode → drug–protein binding/activation/inhibition |
| 66 | DrugBank — bulk target/enzyme/transporter & vocabulary downloads | https://go.drugbank.com/releases/latest | CSV/XML | Pharmacology | Drugs × targets/enzymes/transporters → drug–target, drug–drug, drug–indication |
| 67 | SEER — U.S. cancer incidence & population datasets (research data) | https://seer.cancer.gov/data-software/datasets.html | CSV/TXT | Cancer epidemiology | Cases × age/sex/race/site/stage + incidence/survival → patient–cancer, population–incidence rate |
| 68 | SEER*Explorer — incidence/mortality statistics CSV archive | https://seer.cancer.gov/explorer/ | CSV | Cancer epidemiology | Cancer sites × demographic strata × rates → site–population incidence/mortality |
| 69 | ClinicalTrials.gov — trial records CSV download / AACT database | https://clinicaltrials.gov/data-api/about-api/csv-download | CSV | Clinical trials / registry | Trials × conditions/interventions/outcomes → trial–condition, trial–intervention–outcome |
| 70 | Qiita — 16S/metagenomic feature (OTU/ASV) tables | https://qiita.ucsd.edu/ | BIOM/TSV | Microbiome | Samples × taxa abundance + metadata → sample–taxon, sample–host/condition |
| 71 | MGnify (EBI) — metagenomics taxonomic/functional abundance tables | https://www.ebi.ac.uk/metagenomics/ | TSV | Microbiome / metagenomics | Samples × taxa/GO/KO abundance → sample–taxon, sample–function |
| 72 | GTEx-style + ISB-CGC PanCancer Atlas BigQuery tables (SQL-queryable omics) | https://isb-cgc.appspot.com/ | BigQuery/CSV | Cancer genomics / clinical | Samples × genes/mutations + patient clinical → sample–gene, patient–outcome (queryable) |

## Entity & relation patterns observed

This source class is the most direct "ground truth" for BioOKF node/edge typing
because every item literally *is* a typed adjacency matrix or feature table. The
recurring schema is **rows = primary entity, columns = either a measurement axis
or a second entity**, so the table itself names both the node types and the edge.

### Recurring ENTITY (node) types

- **Sample / Specimen** — the universal row entity in omics matrices (GEO GSM,
  TCGA aliquot, cell-line lysate, biopsy). The hub that links assays to biology.
- **Patient / Participant / Subject** — row entity in clinical, biobank, trial,
  and registry tables (UK Biobank, All of Us, MIMIC, Kaggle clinical sets, SEER
  cases, KMDATA). Carries demographics, outcomes, survival time.
- **Cell** — row entity in single-cell matrices (CELLxGENE, HCA, scRNA-seq GEO
  series); annotated to a **Cell type** and a donor.
- **Gene / Transcript** — the dominant *column* entity in expression matrices;
  also a row entity in gene×tissue/gene×cellline tables.
- **Protein / Peptide / PTM site** — column entity in proteomics and
  phosphoproteomics abundance matrices.
- **Variant / SNP** — row entity in GWAS, ClinVar, gnomAD, eQTL tables.
- **Metabolite** — column entity in metabolomics concentration matrices.
- **Taxon / OTU / ASV** — column entity in microbiome abundance tables.
- **Compound / Drug** — entity in drug-sensitivity (column) and
  drug–target/chemical-protein (row) tables.
- **Cell line** — row entity in DepMap/CCLE/GDSC pharmacogenomic matrices.
- **Tissue / Organ / Anatomical site** — grouping/column entity (GTEx, SEER site,
  Expression Atlas).
- **Disease / Phenotype / Condition / Trait** — the label column that turns a
  feature table into a supervised relationship, and a first-class entity in
  GWAS/ClinVar/SEER/ClinicalTrials.
- **Clinical feature / Biomarker / Lab measurement / Vital sign** — column
  entities in clinical tables (often themselves measurable quantities/LOINC-like).
- **Study / Dataset / Trial / Registry record** — provenance/grouping entity
  (GEO series, recount3 study, ClinicalTrials NCT, trial arm).
- **Population / Ancestry group / Cohort / Demographic stratum** — grouping
  entity in gnomAD, SEER, biobank frequency and rate tables.

### Recurring RELATION (edge) types

- **sample/cell –expresses→ gene** (quantity) — the canonical expression-matrix
  edge (counts/TPM/intensity); applies to bulk + single-cell.
- **sample –has-abundance-of→ protein / metabolite / taxon** — the proteomics,
  metabolomics, and microbiome analogue of the expression edge.
- **patient/sample –has-measurement→ feature/biomarker** (value) — clinical
  feature tables; the value is the edge weight.
- **patient/sample –has-diagnosis / has-phenotype→ disease** (the supervised
  label column) — the single most common edge across clinical/Kaggle/UCI sets.
- **patient –has-outcome / has-survival→ time-to-event** — trial & cancer-registry
  tables (OS/PFS, mortality, status).
- **cell –is-of-type→ cell type** and **cell –from→ donor/tissue** — single-cell
  annotation edges.
- **variant –associated-with→ trait/disease** (effect size, p-value, OR) — GWAS,
  ClinVar (pathogenicity), eQTL (variant–gene–tissue) edges.
- **variant –has-frequency-in→ population** — gnomAD/biobank allele-frequency edges.
- **cell line –is-sensitive-to→ drug** (IC50/AUC) and **cell line –depends-on→
  gene** (CRISPR effect) — pharmacogenomic dependency edges.
- **compound –interacts-with / binds / inhibits / activates→ protein** — STITCH /
  DrugBank chemical-protein edges (typed by mode).
- **drug –targets→ gene/protein**, **drug –treats→ indication** — DrugBank edges.
- **sample/case –belongs-to→ study / cohort / population** — provenance and
  grouping edges that carry metadata.
- **cancer site/population –has-rate→ incidence/mortality value** — epidemiology
  aggregate edges (SEER), where the entity is a population stratum, not an
  individual.
- **gene –regulated-by→ variant in tissue** (cis/trans eQTL) — three-way
  regulatory edge encoded in GTEx signif-pairs tables.

### Schema takeaway for BioOKF

The dataset/table class contributes **the quantitative, weighted edges** that
papers describe qualitatively. Two structural patterns dominate and should both be
first-class in the type system: (1) the **entity × entity adjacency matrix**
(sample×gene, cell×gene, compound×protein, sample×taxon) → directly a typed,
weighted edge with the measurement value as the weight; and (2) the **entity ×
feature table with a label column** (patient × clinical-features + disease) →
many `entity–has-measurement→feature` edges plus one `entity–has-label→class`
edge. A robust BioOKF must therefore (a) treat measurement values/units as edge
attributes (count, TPM, IC50, allele frequency, concentration, OR, p-value,
survival time), (b) carry **provenance** (study/dataset/registry node) on every
edge since these tables are dataset-scoped, and (c) distinguish
**individual-level** entities (patient, sample, cell) from **aggregate/population**
entities (ancestry group, cancer-site cohort) because the same edge type
("incidence", "frequency") attaches to different node granularities. Disease /
phenotype recurs as the universal sink node that ties molecular, clinical, and
epidemiological tables together.
