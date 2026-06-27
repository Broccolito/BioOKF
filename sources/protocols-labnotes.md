# Catalog: Bench-side Notes, Lab Protocols & Open Lab Notebooks

**Source class:** Non-article knowledge sources — step-by-step experimental/computational
**protocols**, bench-side notes, lab-notebook entries, and reproducible analysis notebooks.
**Platforms covered:** protocols.io, Nature Protocols, Bio-protocol, OpenWetWare, Open Notebook
Science (UsefulChem), STAR Protocols (Cell Press), and example Jupyter analysis notebooks
(GitHub / training portals).

**Purpose:** Feed the BioOKF type universe by cataloging real, openly-accessible items and
analyzing the recurring biomedical **entity (node)** and **relationship (edge)** patterns that
characterize procedural/experimental knowledge as distinct from published-article claims.

**Compiled:** 2026-06-25 · **Items:** 80

> Note on access/verification: All URLs were surfaced from live web search of the named
> platforms. protocols.io view pages serve HTTP 403 to automated fetchers (anti-bot) but
> resolve normally in a browser; DOIs (10.17504/protocols.io.*) are the durable handles.
> STAR Protocols, Nature Protocols, Bio-protocol, OpenWetWare and the PMC mirrors are openly
> readable. Format column reflects the canonical landing page (HTML) plus the commonly
> available export (PDF) where applicable.

---

## Catalog Table

| # | Title | URL | Format | Subfield | Key entities & relationships |
|---|-------|-----|--------|----------|------------------------------|
| 1 | Genomic DNA extraction and PCR (protocols.io v1) | https://www.protocols.io/view/Genomic-DNA-extraction-and-PCR-eq2ly38pgx9k/v1 | HTML/PDF | Molecular biology / genomics | Reagents (proteinase K, ethanol) → extract → genomic DNA → amplify target locus via PCR; reagent→sample→product chain |
| 2 | Protocol of DNA extraction for Nanopore long-reads genome sequencing | https://www.protocols.io/view/protocol-of-dna-extraction-for-nanopore-long-reads-bzbdp2i6 | HTML/PDF | Genomics / sequencing | HMW DNA → ONT library → long-read sequencing; sample→reagent→instrument(MinION) workflow |
| 3 | High molecular weight DNA extraction for long read sequencing | https://www.protocols.io/view/high-molecular-weight-dna-extraction-for-long-read-uppevmn.html | HTML/PDF | Genomics / sequencing | Tissue/cells → lysis buffer → HMW DNA quality (A260/A280) → sequencer-ready input |
| 4 | OmniPrep Protocol Collection for High Quality Genomic DNA Extraction | https://www.protocols.io/view/OmniPrep-Protocol-Collection-for-High-Quality-Geno-e7rbhm6 | HTML/PDF | Molecular biology | Kit (reagent set) applied-to multiple sample types → genomic DNA; collection groups variant protocols |
| 5 | 16S rRNA gene Library Preparation Protocol (Wellcome Sanger) | https://www.protocols.io/view/16s-rrna-gene-library-preparation-protocol-cvz9w796.html | HTML/PDF | Microbiome / metagenomics | Bacterial DNA → 16S indexed primer PCR → bead cleanup → Qubit quant → Illumina library; primer→gene region→organism (bacteria) |
| 6 | 16S rRNA Library Preparation Protocol (v1) | https://www.protocols.io/view/16s-rrna-library-preparation-protocol-5qpvo86zl4o1/v1 | HTML/PDF | Microbiome | Variable-region (V3-V4) amplicon → MiSeq library; primer targets gene→ taxonomically classifies organism |
| 7 | Library prep to sequence V3-V4 region of 16S rRNA on Illumina MiSeq | https://www.protocols.io/view/library-preparation-protocol-to-sequence-v3-v4-reg-6i7hchn | HTML/PDF | Microbiome | 16S V3-V4 primers → 2x300bp library; reagent→gene-region→instrument |
| 8 | 16S Metagenomic Sequencing Library Preparation (modified Illumina) | https://www.protocols.io/view/16s-metagenomic-sequencing-library-preparation-mod-nb5daq6 | HTML/PDF | Microbiome | Two-step PCR (amplicon + index) → pooled library; contamination-control steps as procedure constraints |
| 9 | Earth Microbiome Project 16S Illumina Amplicon Protocol | https://earthmicrobiome.ucsd.edu/protocols-and-standards/16s/ | HTML | Microbiome / standards | Standardized 515F/806R primers → V4 amplicon; defines reference standard across consortium |
| 10 | Western Blotting Protocol (protocols.io) | https://www.protocols.io/view/western-blotting-protocol-hk8xb4zxp.html | HTML/PDF | Protein biochemistry | Cells → RIPA lysate → BCA quant → SDS-PAGE → transfer → primary/secondary antibody → chemiluminescence detects protein; antibody→targets→protein |
| 11 | Flow Cytometry Protocol (protocols.io v1) | https://www.protocols.io/view/flow-cytometry-protocol-ewov1127vr24/v1 | HTML/PDF | Immunology / cytometry | Cell suspension + fluorophore-conjugated antibodies → markers identify cell type; antibody→binds→surface marker→defines→cell population |
| 12 | Introduction & Lineage Assignment of Assembled SARS-CoV-2 Sequences | https://www.protocols.io/view/introduction-and-lineage-assignment-of-assembled-s-cgftttnn.html | HTML/PDF | Genomic epidemiology | Reads → consensus genome → Pangolin lineage; sequence→assigned-to→viral lineage (variant) |
| 13 | Extraction-free SARS-CoV-2 detection by RT-qPCR (proteinase K + heat) | https://www.ncbi.nlm.nih.gov/pmc/articles/PMC7909620/ | HTML/PDF | Clinical diagnostics / virology | Swab → PK+heat → RT-qPCR detects viral RNA; reagent→inactivates→virus; assay→detects→pathogen |
| 14 | SARS-CoV-2 RNA extraction using magnetic beads (RT-qPCR / RT-LAMP) | https://www.ncbi.nlm.nih.gov/pmc/articles/PMC7472728/ | HTML/PDF | Clinical diagnostics | Magnetic-bead RNA capture → amplification; reagent→binds→nucleic acid; assay→quantifies→pathogen load |
| 15 | Fast SARS-CoV-2 detection via RNA precipitation + RT-qPCR (nasopharyngeal) | https://www.medrxiv.org/content/10.1101/2020.04.26.20081307.full.pdf | PDF | Clinical diagnostics | Isopropanol precipitation → one-step RT-qPCR; sample(swab)→reagent→pathogen detection |
| 16 | Immunoprecipitation & Western blot detection (STAR Protocols #1363) | https://star-protocols.cell.com/protocols/1363 | HTML | Protein biochemistry | Antibody→pulls-down→target protein complex → blot; protein-protein interaction capture |
| 17 | Analysis of bacterial-surface-specific antibodies via bacterial flow cytometry (Nature Protocols) | https://www.nature.com/articles/nprot.2016.091 | HTML/PDF | Immunology / microbiology | Body-fluid antibody→binds→bacterial surface antigen → cytometry quant; antibody→targets→organism |
| 18 | Flow cytometry method to quantify protein association to chromatin (Nature Protocols) | https://www.nature.com/articles/nprot.2015.066 | HTML/PDF | Cell biology / epigenetics | Protein→binds→chromatin; IF + cytometry measures protein-DNA association in cells |
| 19 | Single cell-resolution western blotting (Nature Protocols) | https://www.nature.com/articles/nprot.2016.089 | HTML/PDF | Single-cell proteomics | Single cell → blot → protein-state per cell; measures cell-to-cell protein expression variation |
| 20 | FACS isolation of endothelial cells and pericytes from mouse brain microregions (Nature Protocols) | https://www.nature.com/articles/nprot.2017.158 | HTML/PDF | Vascular biology / neuroscience | Marker antibodies→sort→cell types (endothelial, pericyte) from anatomical region (brain) |
| 21 | Flow-cytometry isolation of neurovascular unit cells (mouse & human) (Nature Protocols) | https://www.nature.com/articles/s41596-023-00805-y | HTML/PDF | Neuroscience | Simultaneous sort of endothelial/pericyte/astrocyte/microglia; markers→define→cell types in tissue |
| 22 | Targeted single-cell RNA & perturbation sequencing with TAP-seq (Nature Protocols) | https://www.nature.com/articles/s41596-026-01367-5 | HTML/PDF | Functional genomics / single-cell | CRISPR perturbation→affects→gene expression; sgRNA→targets→gene; perturbation→phenotype edge |
| 23 | Small-seq for single-cell small-RNA sequencing (Nature Protocols) | https://www.nature.com/articles/s41596-018-0049-y | HTML/PDF | Single-cell genomics | Single cell → small-RNA library; cell→expresses→miRNA/small-RNA |
| 24 | MARS-seq2.0: indexed sorting + single-cell RNA-seq (Nature Protocols) | https://www.nature.com/articles/s41596-019-0164-4 | HTML/PDF | Single-cell genomics | Index-sorted cells → scRNA-seq; cell→profiled-by→transcriptome; sorting marker→links→cell index |
| 25 | High-throughput full-length single-cell RNA-seq automation (Nature Protocols) | https://www.nature.com/articles/s41596-021-00523-3 | HTML/PDF | Single-cell genomics | Automated Smart-seq2 workflow; reagent→step→library; instrument automates protocol |
| 26 | Protocol for high-quality scRNA-seq from tissue sections with DRaqL (STAR Protocols) | https://www.cell.com/star-protocols/fulltext/S2666-1667(24)00215-6 | HTML/PDF | Single-cell genomics | Tissue section → dissociation → scRNA-seq; tissue→source-of→cells→profiled |
| 27 | Dissociate, process & analyze human lung tissue using scRNA-seq (PMC) | https://www.ncbi.nlm.nih.gov/pmc/articles/PMC9597186/ | HTML/PDF | Single-cell / pulmonary | Lung tissue→dissociation→single-cell suspension→sequencing; tissue→yields→cell types |
| 28 | High-quality scRNA-seq with cell surface protein quantification (CITE-seq) (PMC) | https://pmc.ncbi.nlm.nih.gov/articles/PMC12757190/ | HTML/PDF | Multi-omics single-cell | Antibody-oligo→tags→surface protein + transcriptome; gene expression ↔ protein co-measured per cell |
| 29 | Multi-modal scRNA-seq on M. tuberculosis-infected mouse lungs (PMC) | https://www.ncbi.nlm.nih.gov/pmc/articles/PMC9937979/ | HTML/PDF | Host-pathogen / single-cell | Pathogen (M. tuberculosis)→infects→host cells; captures host transcriptome ↔ infection state |
| 30 | Protocol for single-cell spatial transcriptomic profiling (Visium HD, cultured cells/engineered tissue) (STAR Protocols) | https://www.cell.com/star-protocols/fulltext/S2666-1667(25)00480-0 | HTML/PDF | Spatial transcriptomics | Gene expression mapped-to→spatial coordinate; transcript→located-in→tissue region |
| 31 | Localization of T cell clonotypes using Visium spatial transcriptomics (STAR Protocols #1711) | https://star-protocols.cell.com/protocols/1711 | HTML | Spatial / immunology | T-cell clonotype→localized-to→tissue position; TCR sequence ↔ spatial location |
| 32 | High-resolution 3D spatial transcriptomics using Open-ST (STAR Protocols #3922) | https://star-protocols.cell.com/protocols/3922 | HTML | Spatial transcriptomics | Subcellular barcoded capture; transcript→spatial-coordinate at 0.6µm resolution |
| 33 | Establishing inducible CRISPRi for multi-gene silencing in human PSCs (STAR Protocols) | https://www.cell.com/star-protocols/fulltext/S2666-1667(24)00386-1 | HTML/PDF | Functional genomics / stem cells | dCas9-sgRNA→silences→gene; inducible system→regulates→gene expression in cell line |
| 34 | Isogenic disease models from adult stem cell-derived organoids via CRISPR (STAR Protocols) | https://www.cell.com/star-protocols/fulltext/S2666-1667(24)00354-X | HTML/PDF | Disease modeling / organoids | CRISPR edit→introduces→mutation→models→disease; gene variant ↔ phenotype |
| 35 | Gene-of-interest knockouts in murine organoids using CRISPR-Cas9 (STAR Protocols) | https://www.cell.com/star-protocols/fulltext/S2666-1667(23)00034-5 | HTML/PDF | Functional genomics / organoids | RNP→knocks-out→gene; sgRNA→targets→locus; clonal organoid pairs (WT vs KO) |
| 36 | Intestinal organoid co-culture to study cell competition in vitro (STAR Protocols) | https://www.cell.com/star-protocols/fulltext/S2666-1667(21)00756-5 | HTML/PDF | Organoids / cell biology | Two cell populations→compete-in→co-culture; cell type↔cell type interaction |
| 37 | CRISPR: questions and answers (STAR Protocols #2555) | https://star-protocols.cell.com/protocols/2555 | HTML | Methods reference | sgRNA design ↔ off-target; Cas9→cuts→genomic target; troubleshooting knowledge |
| 38 | Whole-cell patch clamp + extracellular electrophysiology in mouse brain slices (STAR Protocols) | https://www.cell.com/star-protocols/fulltext/S2666-1667(25)00414-9 | HTML/PDF | Neuroscience / electrophysiology | Electrode→records→neuron/astrocyte activity; cell type→exhibits→electrophysiological property |
| 39 | Simultaneous patch-clamp from tanycytes and neurons in mouse brain slices (STAR Protocols) | https://www.cell.com/star-protocols/fulltext/S2666-1667(23)00538-5 | HTML/PDF | Neuroscience | Paired recording probes metabolic coupling; cell type↔cell type functional link |
| 40 | Patch-clamp & multi-electrode array analysis in acute mouse brain slices (STAR Protocols) | https://www.cell.com/star-protocols/fulltext/S2666-1667(21)00149-0 | HTML/PDF | Neuroscience | MEA→measures→network activity; neuron→synaptic-parameter→neurotransmission |
| 41 | Isolation, culture & patch-clamp of dorsal root ganglion neurons (PMC) | https://www.ncbi.nlm.nih.gov/pmc/articles/PMC12035743/ | HTML/PDF | Neuroscience / sensory | DRG tissue→dissociation→neuron culture→recording; anatomical structure→source-of→cell |
| 42 | Flow cytometry immunophenotyping of antigen-specific T cells (AIM + Th1 cytokine) (STAR Protocols) | https://www.cell.com/star-protocols/fulltext/S2666-1667(24)00508-2 | HTML/PDF | Immunology | Activation marker + cytokine→define→antigen-specific T-cell; antigen→activates→T cell→produces→cytokine |
| 43 | Murine multi-tissue deep immunophenotyping (40-color full-spectrum flow) (STAR Protocols) | https://www.sciencedirect.com/science/article/pii/S2666166724006579 | HTML/PDF | Immunology | 40-marker panel→resolves→immune cell types across tissues; marker combination→identifies→population |
| 44 | Tumor dissociation for single-cell omics in mouse breast cancer models (STAR Protocols) | https://www.cell.com/star-protocols/fulltext/S2666-1667(21)00547-5 | HTML/PDF | Cancer / single-cell | Tumor→dissociated-to→immune (CD45+)/tumor/stromal cells; marker→sorts→population |
| 45 | FACS-based isolation of fixed mouse neuronal nuclei for ATAC-seq and Hi-C (STAR Protocols #790) | https://star-protocols.cell.com/protocols/790 | HTML | Epigenomics / neuroscience | Nuclei→sorted→ATAC-seq (open chromatin) + Hi-C (3D contacts); locus→accessibility/contact edge |
| 46 | Isolation of mouse muscle stem cells (STAR Protocols #3060) | https://star-protocols.cell.com/protocols/3060 | HTML | Stem cell biology | Surface markers→isolate→satellite/muscle stem cell from tissue |
| 47 | Drug screening of patient-derived tumor organoids via high-content imaging (STAR Protocols) | https://www.cell.com/star-protocols/fulltext/S2666-1667(22)00287-8 | HTML/PDF | Cancer pharmacology | Drug→treats→organoid→viability readout; compound→affects→cell viability (dose-response) |
| 48 | 3D high-content γH2AX DNA-damage imaging in ovarian cancer organoids (PMC) | https://pmc.ncbi.nlm.nih.gov/articles/PMC9841773/ | HTML/PDF | Cancer / DNA damage | Drug→induces→DNA damage (γH2AX foci); marker→quantifies→damage response |
| 49 | High-content drug screening using tumor organoids on 384-pillar plate (PMC) | https://www.ncbi.nlm.nih.gov/pmc/articles/PMC12718461/ | HTML/PDF | Drug screening | Compound library→screened-against→organoids; drug→phenotype mapping at scale |
| 50 | Cortical brain organoid slices culture (STAR Protocols) | https://www.cell.com/star-protocols/fulltext/S2666-1667(24)00377-0 | HTML/PDF | Neuroscience / organoids | iPSC→differentiates→cortical organoid→sliced for long-term culture; stem cell→becomes→tissue |
| 51 | Generating human cerebral organoids from 2D PSC cultures (bypass EB) (ScienceDirect) | https://www.sciencedirect.com/science/article/pii/S266616672500084X | HTML | Neuroscience / organoids | PSC→neural identity→3D self-organization→cerebral organoid; differentiation lineage chain |
| 52 | iPSC differentiation into retinal pigment epithelial cells (PMC) | https://www.ncbi.nlm.nih.gov/pmc/articles/PMC12741457/ | HTML/PDF | Stem cells / ophthalmology | iPSC→differentiates→RPE (marker-defined); growth factor→drives→cell fate |
| 53 | In vitro differentiation of human iPSCs into colon organoids (Sigma) | https://www.sigmaaldrich.com/US/en/technical-documents/protocol/cell-culture-and-cell-culture-analysis/3d-cell-culture/human-colon-organoids | HTML | Stem cells / GI | iPSC→3-step differentiation→colon organoid expressing mature markers |
| 54 | Microinjection of free fatty acids & triacylglycerol in zebrafish embryos (STAR Protocols) | https://www.sciencedirect.com/science/article/pii/S266616672400251X | HTML | Model organism / metabolism | Lipid→injected-into→zebrafish embryo→metabolic readout; reagent→delivered-to→organism |
| 55 | Minimally invasive fin scratching for fast zebrafish genotyping (PMC) | https://www.ncbi.nlm.nih.gov/pmc/articles/PMC9803660/ | HTML/PDF | Model organism / genetics | Tissue sample→genotype→embryo selection; organism→has→genotype |
| 56 | Endy Lab: RNA extraction (acid-phenol, E. coli) — OpenWetWare | https://openwetware.org/wiki/Endy:RNA_extraction | HTML | Molecular biology (lab notebook) | E. coli culture→acid-phenol→RNA; organism→growth condition→nucleic acid; lab-specific procedure |
| 57 | RNA extraction (general) — OpenWetWare | https://openwetware.org/wiki/RNA_extraction | HTML | Molecular biology | TRI Reagent→lyses→cells→RNA; reagent ratio→per-cell-count constraint |
| 58 | Sauer: RNA Purification from E. coli — OpenWetWare | https://openwetware.org/wiki/Sauer:RNA_Purification_from_E._coli | HTML | Molecular biology (lab notebook) | E. coli→lysozyme buffer (OD-scaled)→purified RNA; quantitative reagent-to-sample relationship |
| 59 | Wiese Lab: Plasmid DNA Miniprep — OpenWetWare | https://openwetware.org/wiki/Wiese_Lab:Plasmid_DNA_Miniprep | HTML | Molecular biology (lab notebook) | Bacterial culture→alkaline lysis→plasmid DNA; reagent→step→product |
| 60 | Bacteria Transformation — OpenWetWare | https://openwetware.org/wiki/Bacteria_Transformation | HTML | Molecular biology | Competent cells + plasmid DNA→heat shock→transformants; β-ME→increases→efficiency (parameter→effect) |
| 61 | Lidstrom: Miniprep — OpenWetWare | https://openwetware.org/wiki/Lidstrom:Miniprep | HTML | Molecular biology (lab notebook) | Alkaline-lysis miniprep variant; copy-number→determines→culture volume |
| 62 | Smolke: Protocols/Plasmid prep — OpenWetWare | https://openwetware.org/wiki/Smolke:Protocols/Plasmid_prep | HTML | Synthetic biology (lab notebook) | Culture→column purification→plasmid; A260/A280→indicates→purity |
| 63 | Wittrup: Plasmid miniprep — OpenWetWare | https://openwetware.org/wiki/Wittrup:_Plasmid_miniprep | HTML | Protein engineering (lab notebook) | Standard miniprep; lab-specific reagent recipe and yield notes |
| 64 | IGEM Harvard 2007: Plasmid MiniPrep Protocol — OpenWetWare | https://openwetware.org/wiki/IGEM:Harvard/2007/Protocols/Plasmid_MiniPrep_Protocol | HTML | Synthetic biology (iGEM) | Student-team protocol; plasmid prep as part of part-assembly workflow |
| 65 | Janet B. Matsen: Lab Tips & Tricks — OpenWetWare | https://openwetware.org/wiki/Janet_B._Matsen:Lab_Tips_&_Tricks | HTML | Bench notes | Free-text bench-side tips: media (TB vs LB) yield trade-offs; informal procedural knowledge |
| 66 | Moore: Protocols (silver stain, SDS-PAGE, western blot, HRP) — OpenWetWare | https://openwetware.org/wiki/Moore:Protocols | HTML | Protein biochemistry (lab notebook) | Protein→SDS-PAGE→transfer→HRP detection; index of protein-detection procedures |
| 67 | McClean: Protocols (yeast genetics, Cre/LoxP) — OpenWetWare | https://openwetware.org/wiki/McClean:Protocols | HTML | Yeast genetics (lab notebook) | S. cerevisiae transformation, Cre/LoxP recombination; enzyme→acts-on→DNA site |
| 68 | Methods and Protocols (index hub) — OpenWetWare | https://openwetware.org/wiki/Methods_and_Protocols | HTML | Methods hub | Cross-lab index linking many bench protocols by technique |
| 69 | UsefulChem Exp098 (open lab notebook entry) | http://usefulchem.blogspot.com/2008/11/what-is-solubility-of-vanillin-in.html | HTML | Open Notebook Science / chemistry | Compound (vanillin)→solubility-in→solvent (methanol); real-time recorded measurement, raw data |
| 70 | Open Notebook Science Using Blogs and Wikis (Nature Precedings, Bradley) | https://www.nature.com/articles/npre.2007.39.1 | HTML/PDF | Open Notebook Science | Defines bliki notebook structure; experiment→links-to→prior experiment (Exp064→Exp098 provenance) |
| 71 | Open-notebook science (overview, UsefulChem antimalarial project) | https://en.wikipedia.org/wiki/Open-notebook_science | HTML | Open Notebook Science | Candidate compound→tested-against→malaria target; documents synthesis + NMR raw data openly |
| 72 | GATK4 Jupyter Notebook Tutorials (germline + somatic) — GitHub | https://github.com/gatk-workflows/gatk4-jupyter-notebook-tutorials | ipynb / HTML | Variant calling / bioinformatics | Reads→aligned→variants (SNV/indel); Mutect2→calls→somatic variant; sample→has→variant |
| 73 | GATK4 Somatic Mutect2 tutorial notebook — GitHub | https://github.com/gatk-workflows/gatk4-jupyter-notebook-tutorials/blob/master/notebooks/Day3-Somatic/1-somatic-mutect2-tutorial.ipynb | ipynb | Cancer genomics | Tumor/normal→Mutect2→somatic mutation; variant→annotated/filtered |
| 74 | GWAS lecture notebooks (Python, Limix) — GitHub | https://github.com/timeu/gwas-lecture | ipynb | Statistical genetics | Genotype (SNP)→associated-with→phenotype/trait; variant→p-value→trait edge |
| 75 | Scanpy single-cell analysis toolkit (tutorials) — GitHub / RTD | https://github.com/scverse/scanpy | ipynb / HTML | Single-cell bioinformatics | Cells→clustered→cell types; gene→marker-of→cluster; trajectory/DE edges |
| 76 | Galaxy: Filter, plot & explore scRNA-seq with Scanpy (hands-on notebook) | https://training.galaxyproject.org/training-material/topics/single-cell/tutorials/scrna-case-jupyter_basic-pipeline/tutorial.html | ipynb / HTML | Single-cell bioinformatics | QC→filter→normalize→cluster; cell→expresses→gene; cluster→annotated-as→cell type |
| 77 | EBI Single-cell RNA-seq analysis using Python (course notebooks) — GitHub | https://github.com/Functional-Genomics/scrnaseq_python_2023 | ipynb | Single-cell bioinformatics | Reproducible QC/normalization/clustering notebooks; gene↔cell expression matrix |
| 78 | Pyteomics tutorial (mass spectrometry data analysis) | https://pyteomics.readthedocs.io/ | HTML / ipynb | Proteomics bioinformatics | Spectrum→peptide→protein identification; m/z→matches→sequence; PSM edges |
| 79 | R-proteomics-Nrf1 reproducible MS analysis (Jupyter + R Markdown) — GitHub | https://github.com/br3ndonland/R-proteomics-Nrf1 | ipynb / Rmd | Proteomics | MS intensity→protein abundance→condition comparison; protein↔treatment differential edge |
| 80 | A Protocol for Untargeted Metabolomic Analysis: Sample Prep to Data Processing (PMC) | https://pmc.ncbi.nlm.nih.gov/articles/PMC9284939/ | HTML/PDF | Metabolomics | Sample→extraction→LC-MS feature→metabolite ID; feature→annotated-as→metabolite; metabolite↔condition |

---

## Entity & Relation Patterns Observed

This source class (procedural / experimental knowledge) is structurally different from the
claim-centric content of research articles. Instead of asserting biological *facts* ("gene X
causes disease Y"), these sources encode **how to produce, measure, or transform** biomedical
material. The dominant relationships are **procedural and operational** rather than causal-biological.
Both flavors matter for BioOKF: the procedural graph is what lets the KG represent *provenance,
methods, and reproducibility*, while the embedded biological entities (genes, organisms, cell
types, compounds) overlap heavily with the article-derived ontology.

### Recurring ENTITY (node) types

**Biological / biomedical entities (overlap with article ontology):**
- **Organism / model system** — E. coli, S. cerevisiae (yeast), mouse, zebrafish, Drosophila,
  C. elegans, Arabidopsis, human; pathogens (M. tuberculosis, SARS-CoV-2).
- **Gene / locus / gene region** — target genes for KO/KD, 16S rRNA gene, V3-V4 region, sgRNA targets.
- **Variant / mutation** — SNVs, indels, somatic mutations, CRISPR-induced frameshifts, lineages/variants.
- **Protein** — blot targets, purified recombinant proteins, antibody antigens, γH2AX, GroEL.
- **Cell type / cell population** — T cells (clonotypes, antigen-specific), endothelial cells,
  pericytes, astrocytes, microglia, neurons, tanycytes, muscle/satellite stem cells, immune
  (CD45+) / tumor / stromal cells, iPSC/PSC, RPE.
- **Cell-surface marker / antigen** — CD45, CD3, CD4, CD45RA, CCR7, CXCR5 (define populations).
- **Tissue / anatomical structure** — brain (microregions, cortex), lung, intestine/colon,
  dorsal root ganglion, tumor, embryo, retina.
- **Organoid / 3D culture** — cerebral, cortical, intestinal, colon, ovarian/tumor, retinal.
- **Small molecule / compound / drug** — screening libraries, vanillin, reagents (proteinase K,
  TRI Reagent, lysozyme, paraformaldehyde, Triton X-100, MSTFA), fluorophores.
- **Metabolite / lipid** — fatty acids, triacylglycerol, untargeted-metabolomics features.
- **Phenotype / readout** — viability, DNA-damage foci, electrophysiological properties, cytokine
  production, solubility.

**Procedural / methods entities (distinctive to this source class):**
- **Protocol / step / substep** — the ordered procedure itself (often versioned, DOI'd).
- **Reagent / kit / buffer / consumable** — with concentrations, ratios, catalog identity.
- **Instrument / platform** — MinION, Illumina MiSeq/HiSeq, flow cytometer, patch-clamp rig,
  MEA, confocal microscope, LC-MS/GC-MS, NMR, micromanipulator.
- **Assay / technique** — PCR, RT-qPCR, RT-LAMP, SDS-PAGE/western, IP, IHC/IF, FACS,
  scRNA-seq/CITE-seq, ATAC-seq, Hi-C, ChIP-seq, patch-clamp, mass spec.
- **Sample / specimen** — swab, lysate, single-cell suspension, nuclei, tissue section, library.
- **Library / sequencing product / data file** — amplicon library, FASTQ reads, consensus genome,
  count matrix, spectra.
- **Parameter / condition** — temperature, OD600, incubation time, dose, A260/A280, fluorophore panel.
- **Software / pipeline / package** — GATK/Mutect2, Scanpy, Pangolin, Pyteomics, edgeR, Limix.
- **Author / lab / consortium** — lab-namespaced protocols (Endy, Sauer, Smolke, Wittrup;
  Earth Microbiome Project; iGEM teams) — a strong **provenance/attribution** signal.
- **Control / standard** — isotype controls, FMO, WT-vs-KO clonal pairs, reference primers.

### Recurring RELATION (edge) types

**Procedural / transformation edges (the backbone of this source class):**
- **protocol → has_step → step** (and **step → precedes → step**: ordered workflow / sequence).
- **reagent / kit → applied_to → sample** (often with quantitative ratio, e.g. "2 mL buffer per 10 mL culture").
- **sample → transformed_into → product** (cells → lysate → DNA/RNA → library → reads → genome).
- **assay / instrument → measures / detects / quantifies → entity** (RT-qPCR detects pathogen RNA;
  cytometer quantifies marker+ population; patch-clamp records neuron activity).
- **antibody / primer / probe → targets / binds → antigen / gene region** (specificity edge).
- **enzyme → acts_on → substrate / DNA site** (Cas9 cuts locus; Cre recombines LoxP; Tn5 inserts into open chromatin).
- **parameter → affects / optimizes → outcome** (β-ME increases transformation efficiency;
  media choice affects yield; dose affects viability) — a **conditional/quantitative** edge.
- **protocol → uses → instrument / software / reagent** (resource dependency).
- **protocol → produces → data_product** (library, count matrix, spectrum, structure).

**Provenance / organizational edges (strong in lab notebooks & versioned protocols):**
- **protocol / notebook_entry → authored_by → person / lab / consortium.**
- **protocol → has_version → version** (protocols.io v1/v2; DOI per version).
- **experiment → references / derived_from → prior_experiment** (Open Notebook Science:
  Exp098 → Exp064 — explicit provenance chain).
- **protocol → variant_of / adapts → another_protocol** (e.g. "modified version of Illumina").
- **protocol → conforms_to → standard** (Earth Microbiome 16S; ARTIC).
- **protocol → bundled_in → collection** (OmniPrep collection; iGEM team protocol sets).

**Biological/contextual edges embedded in procedures (overlap with article KG):**
- **marker → defines / identifies → cell_type** (CD45+ → immune cell; surface-panel → population).
- **cell_type → isolated_from / resides_in → tissue / anatomical_region.**
- **stem_cell → differentiates_into → cell_type / organoid** (iPSC → RPE; PSC → cerebral organoid).
- **perturbation (CRISPR KO/KD, drug) → causes / induces → phenotype / expression_change**
  (the closest thing here to an article-style causal edge — and the most KG-valuable).
- **pathogen → infects → host_cell / tissue** (M. tuberculosis → lung cells; SARS-CoV-2 → swab specimen).
- **gene / transcript → expressed_in / located_at → cell / spatial_coordinate** (single-cell & spatial).
- **compound → has_property → value** (vanillin solubility in methanol; metabolite abundance).

### Implications for the BioOKF type universe

1. **A dual node universe is needed.** Article-derived KGs are dominated by biological entities;
   protocol/notebook sources add a parallel **methods/provenance** layer (Protocol, Step, Reagent,
   Instrument, Assay, Sample, DataProduct, Lab/Author, Version, Standard). Modeling these makes the
   KG *method-aware and reproducible*, and lets evidence be traced to "how it was generated."
2. **Procedural edges (precedes, transforms_into, applied_to, measures, produces) are first-class**
   and largely absent from article-claim extraction — they are the defining signal of this class.
3. **Quantitative/conditional edges** (parameter→affects→outcome, reagent→ratio→sample) carry
   numeric attributes; BioOKF edges should support typed quantitative properties (concentration,
   temperature, dose, time, OD, A260/A280).
4. **Provenance edges are unusually rich** here (versioned DOIs, lab-namespaced protocols,
   experiment→prior-experiment links in Open Notebook Science). This source class is the natural
   home for the KG's **attribution, versioning, and reproducibility** subgraph.
5. **The bridge to the biological ontology** is the perturbation→phenotype and
   marker→cell_type→tissue→organism chains embedded in protocols. These overlap directly with
   article-derived entities (genes, cell types, drugs, organisms, variants), so protocol nodes
   can be linked into the same biomedical entity space rather than living in isolation.
