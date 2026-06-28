# Biomedical Images, Figures & Visual-Knowledge Source Catalog (BioOKF)

Openly accessible biomedical sources in which **knowledge is encoded visually** —
pathway/wiring diagrams (KEGG/Reactome/WikiPathways and other curated maps),
interaction-network figures, 3D molecular-structure renderings, histology /
microscopy / radiology image archives, single-cell atlas plots (UMAP/heatmaps),
and example paper figures. The defining property of this source class: the
biomedical *entities* are drawn as **nodes / shapes / image regions** and the
*relationships* are drawn as **arrows / edges / spatial adjacency / co-occurrence
in a plot**. This makes the class a primary anchor for visual-relation extraction
into BioOKF.

Sources are repositories/databases (image + diagram archives), pathway-map
collections, structure databanks, and representative open-access figure examples.
URLs verified June 2026; all resolve to public landing pages, open-access full
text (PMC OA / Nature OA / Oxford NAR OA / bioRxiv), or public data registries.

Total items: 84

## Catalog

| # | Title | URL | Format | Subfield | Key entities & relationships |
|---|-------|-----|--------|----------|------------------------------|
| 1 | KEGG PATHWAY Database (manually drawn pathway maps) | https://www.genome.jp/kegg/pathway.html | HTML (KGML/KGML+PNG/SVG) | Pathway diagrams / metabolism+signaling | Genes/KOs, compounds, reactions, drugs, diseases as nodes; arrows = reaction/relation (activation, inhibition, expression) → gene–reaction–compound, pathway membership |
| 2 | KEGG Atlas — global metabolic pathway map | https://pmc.ncbi.nlm.nih.gov/articles/PMC2447737/ | HTML (PMC OA) | Pathway atlas / metabolism | Enzymes, metabolites, reaction edges across the whole metabolic network → compound–enzyme–compound flux topology |
| 3 | KEGGtranslator: visualizing/converting KEGG PATHWAY | https://academic.oup.com/bioinformatics/article/27/16/2314/255063 | HTML (OA) | Pathway diagram tooling | KGML graph (entries=genes/compounds, relations=edges) → reproducible node–edge pathway encoding |
| 4 | Reactome Pathway Browser (interactive diagrams) | https://reactome.org/userguide/pathway-browser | HTML (SVG/SBGN diagrams) | Pathway diagrams / reactions | Physical entities (proteins, complexes, small molecules), reactions, compartments; event hierarchy → entity–reaction–entity, input/output/catalyst/regulator edges |
| 5 | Reactome: database of reactions, pathways & biological processes | https://pmc.ncbi.nlm.nih.gov/articles/PMC3013646/ | HTML (PMC OA) | Pathway database | Reactions, pathways, catalysts, regulators → ordered network of molecular transformations |
| 6 | The Reactome Pathway Knowledgebase (NAR) | https://pmc.ncbi.nlm.nih.gov/articles/PMC5753187/ | HTML (PMC OA) | Pathway knowledgebase | Proteins/complexes, reactions, disease variants overlaid → entity–reaction, normal-vs-disease pathway |
| 7 | Reactome Pathway Diagrams (developer/diagram spec) | https://reactome.org/dev/diagram | HTML | Pathway diagram format | SBGN-style nodes/edges, compartments → standardized graphical encoding of reactions |
| 8 | Reactome enhanced pathway visualization | https://pmc.ncbi.nlm.nih.gov/articles/PMC5860170/ | HTML (PMC OA) | Pathway diagram rendering | EHLD illustrations, subpathway regions → high-level pathway-to-subpathway containment |
| 9 | Reactome diagram viewer: data structures & performance | https://pubmed.ncbi.nlm.nih.gov/29186351/ | HTML (abstract) | Pathway diagram tooling | Diagram nodes (entities) + reaction edges as graph DB → entity–reaction graph traversal |
| 10 | WikiPathways 2024: next generation pathway database (NAR) | https://academic.oup.com/nar/article/52/D1/D679/7369835 | HTML (OA) | Community pathway diagrams | GPML pathways: genes/proteins/metabolites as datanodes, interactions as edges → curated node–edge biological pathways |
| 11 | WikiPathways 2024 (PMC OA mirror) | https://pmc.ncbi.nlm.nih.gov/articles/PMC10767877/ | HTML (PMC OA) | Community pathway diagrams | 1,913 human pathways, 27 species; GPML/SVG/RDF → gene/metabolite interaction diagrams |
| 12 | WikiPathways Download (GPML/GMT/SVG/RDF) | https://www.wikipathways.org/download.html | HTML (GPML/SVG/RDF) | Pathway diagram corpus | Machine-readable pathway graphs → datanode (entity) + line (interaction) encodings |
| 13 | WikiPathways Semantic Web / RDF portal | https://www.wikipathways.org/rdf.html | HTML (RDF) | Pathway diagrams as triples | Pathway elements + interactions as RDF → entity–interaction triples derived from diagrams |
| 14 | Reactome from a WikiPathways Perspective | https://pmc.ncbi.nlm.nih.gov/articles/PMC4874630/ | HTML (PMC OA) | Pathway interoperability | Cross-mapped pathway entities/reactions → equivalence + provenance of pathway nodes/edges |
| 15 | PANTHER pathway database (NAR) | https://academic.oup.com/nar/article/33/suppl_1/D284/2505352 | HTML (OA) | Signaling pathway diagrams | CellDesigner/SBGN-PD maps: proteins, complexes, processes → activation/inhibition signaling edges |
| 16 | PANTHER Pathway Diagram Help (SBGN-PD spec) | https://www.pantherdb.org/tips/tips_diagram.jsp | HTML | Pathway diagram notation | SBGN glyphs (state transitions, modulation) → process-description node/edge grammar |
| 17 | Small Molecule Pathway Database (SMPDB) | https://smpdb.ca/ | HTML (image/SVG) | Metabolic/drug pathway diagrams | Metabolites, enzymes, cofactors, organelles, tissues → metabolite–enzyme reactions with subcellular location edges |
| 18 | SMPDB 2.0: Big Improvements (NAR) | https://pmc.ncbi.nlm.nih.gov/articles/PMC3965088/ | HTML (PMC OA) | Pathway diagrams / small molecule | 30,000+ pathway diagrams; drug action/metabolism → compound–protein–pathway |
| 19 | PathBank: comprehensive pathway database (PMC OA) | https://pmc.ncbi.nlm.nih.gov/articles/PMC6943071/ | HTML (PMC OA, SVG/PNG) | Pathway diagrams / multi-organism | 110,000+ visual pathways; proteins, metabolites, reactions → protein/metabolite–pathway map per organism |
| 20 | PathBank 2.0 (NAR) | https://academic.oup.com/nar/article/52/D1/D654/7420099 | HTML (OA) | Pathway diagrams / metabolomics | Metabolites, enzymes, reactions, organelles → metabolite-centric reaction maps |
| 21 | INOH: ontology-based signal transduction pathway DB | https://academic.oup.com/database/article/doi/10.1093/database/bar052/469742 | HTML (OA) | Signaling pathway diagrams | 73 signal-transduction + 29 metabolic diagrams, 6,155 interactions, 3,395 proteins → protein–interaction signaling graphs (BioPAX) |
| 22 | SignaLink 2: multi-layered signaling pathway resource | https://pmc.ncbi.nlm.nih.gov/articles/PMC3599410/ | HTML (PMC OA) | Signaling network diagrams | Pathway proteins, scaffolds, TFs, miRNAs in layers → directed signaling + regulatory edges |
| 23 | SignaLink3: tissue-specific signaling networks | https://pmc.ncbi.nlm.nih.gov/articles/PMC8728204/ | HTML (PMC OA) | Signaling network diagrams | Proteins, pathways, tissues → tissue-specific directed signaling edges |
| 24 | BioModels (curated SBML models w/ SBGN maps) | https://www.ebi.ac.uk/biomodels/ | HTML (SBML/SBGN/PNG/SVG) | Systems-biology model diagrams | Species (molecules), reactions, kinetic edges; SBGN maps → quantitative reaction-network nodes/edges |
| 25 | Path2Models: models from biochemical pathway maps | https://bmcsystbiol.biomedcentral.com/articles/10.1186/1752-0509-7-116 | HTML (OA) | Pathway-to-model diagrams | Pathway map glyphs → reactions/species → executable reaction-network graph |
| 26 | COVID-19 Disease Map (curated mechanism diagrams) | https://covid.pages.uni.lu/ | HTML (MINERVA/SBGN/CellDesigner) | Disease mechanism maps | Viral + host proteins, genes, complexes, processes; replication/immune pathways → virus–host interaction + signaling edges |
| 27 | COVID-19 Disease Map (Scientific Data, building the repo) | https://www.nature.com/articles/s41597-020-0477-8 | HTML (Nature OA) | Disease mechanism maps | SARS-CoV-2 proteins, host pathways, drugs → host–pathogen interaction diagrams, drug-target overlays |
| 28 | COVID-19 Disease Map (knowledge repository, FEBS/PMC) | https://pmc.ncbi.nlm.nih.gov/articles/PMC8524328/ | HTML (PMC OA) | Disease mechanism maps | Curated SBGN diagrams of virus replication, IFN/PAMP signaling → mechanism-level entity–process edges |
| 29 | MINERVA Platform guide (disease-map webserver) | https://covid.pages.uni.lu/minerva-guide/ | HTML | Disease-map visualization | Molecular networks (SBGN) as explorable maps → entity–interaction overlays with data |
| 30 | FAIR assessment of Disease Maps (Scientific Data) | https://www.nature.com/articles/s41597-025-05147-w | HTML (Nature OA) | Disease-map ecosystem | Disease-specific maps (proteins, genes, processes) → standardized mechanism diagrams across diseases |
| 31 | SARS-CoV-2 signaling pathway map (functional landscape) | https://pmc.ncbi.nlm.nih.gov/articles/PMC8237035/ | HTML (PMC OA) | Disease signaling map | Viral proteins, host kinases/TFs, processes → directed COVID-19 signaling edges |
| 32 | STRING v12/2025: protein networks w/ directionality (NAR) | https://pmc.ncbi.nlm.nih.gov/articles/PMC11701646/ | HTML (PMC OA) | Interaction-network figures | Proteins as nodes; functional/physical/regulatory edges with directionality → protein–protein association graphs |
| 33 | STRING v11 (PMC OA) | https://pmc.ncbi.nlm.nih.gov/articles/PMC6323986/ | HTML (PMC OA) | Interaction-network figures | Proteins, scored associations (text-mining/coexpression/experiment) → weighted PPI network nodes/edges |
| 34 | BioGRID database (Protein Science 2021, PMC OA) | https://pmc.ncbi.nlm.nih.gov/articles/PMC7737760/ | HTML (PMC OA) | Interaction-network figures | Genes/proteins/chemicals; colored edges (protein=yellow, genetic=green, chemical=blue) → typed interaction network |
| 35 | BioGRID interaction database: 2019 update (NAR) | https://academic.oup.com/nar/article/47/D1/D529/5204333 | HTML (OA) | Interaction database / network viewer | 1.9M curated interactions; node size = connectivity → protein/genetic/chemical interaction edges |
| 36 | Cytoscape: software for biomolecular interaction networks | https://pmc.ncbi.nlm.nih.gov/articles/PMC403769/ | HTML (PMC OA) | Network visualization (figures) | Molecular species = nodes, interactions = edges; data-mapped visual attributes → integrated network graphs |
| 37 | Ten simple rules to create biological network figures | https://pmc.ncbi.nlm.nih.gov/articles/PMC6762067/ | HTML (PMC OA) | Network-figure design | Node/edge encoding conventions → how entities & relations are visually communicated |
| 38 | SPOKE: massive biomedical knowledge graph (Bioinformatics OA) | https://pmc.ncbi.nlm.nih.gov/articles/PMC9940622/ | HTML (PMC OA) | Knowledge-graph visualization | 42M nodes / 28 types, 160M edges / 91 types; Neighborhood Explorer map → disease–gene–compound–protein typed edges |
| 39 | Building a knowledge graph for precision medicine (SPOKE, Sci Data) | https://www.nature.com/articles/s41597-023-01960-3 | HTML (Nature OA) | Knowledge-graph schema/figures | Node types (Disease, Gene, Compound, Symptom, Pathway…) + edge types (TREATS, BINDS, UPREGULATES) → schema diagram |
| 40 | RCSB Protein Data Bank (3D structure visualization) | https://www.rcsb.org/ | HTML (PDB/mmCIF + Mol*/NGL render) | Molecular-structure images | Proteins, nucleic acids, ligands, ions; chains, residues, bonds → 3D structure, ligand–binding-site spatial relations |
| 41 | RCSB PDB: integrative view of protein/gene/3D structure (PMC OA) | https://pmc.ncbi.nlm.nih.gov/articles/PMC5210513/ | HTML (PMC OA) | Structure database | Structures, genes, sequence features → structure–gene–annotation integration |
| 42 | RCSB PDB: visualizing experimental + computed models (NAR) | https://pmc.ncbi.nlm.nih.gov/articles/PMC10726007/ | HTML (PMC OA) | Structure visualization | Experimental structures + computed models, ligands → grouped structural-proteome views |
| 43 | Mol* at RCSB.org (H5N1 proteome case study) | https://pmc.ncbi.nlm.nih.gov/articles/PMC11915458/ | HTML (PMC OA) | 3D structure rendering | Viral proteins, domains, ligands → 3D residue-level + binding-pocket spatial relationships |
| 44 | PDB-101 (educational structure images, RCSB) | https://pdb101.rcsb.org/ | HTML (image, public domain) | Molecular illustrations | Molecules-of-the-month renderings → protein–ligand, complex assembly visual relationships |
| 45 | AlphaFold Protein Structure Database | https://alphafold.ebi.ac.uk/ | HTML (PDB/mmCIF + 3D render) | Predicted-structure images | 214M+ predicted protein structures, per-residue pLDDT, PAE plots → sequence→3D-fold, confidence-colored renders |
| 46 | AlphaFold DB: massively expanding structural coverage (NAR) | https://pmc.ncbi.nlm.nih.gov/articles/PMC8728224/ | HTML (PMC OA) | Predicted-structure database | Proteins, predicted folds, confidence → structure prediction with PAE/pLDDT visual annotations |
| 47 | PDBe-KB: aggregated views of proteins (NAR) | https://academic.oup.com/nar/article/50/D1/D534/6424755 | HTML (OA) | Structure annotation figures | Proteins, ligands, binding sites, annotations on sequence + 3D (Mol*) → ligand–environment, functional-site spatial edges |
| 48 | EMDB — Electron Microscopy Data Bank (3D maps) | https://www.ebi.ac.uk/emdb/ | HTML (MRC maps + renders) | Cryo-EM density maps | Macromolecular complexes, viruses, organelles as 3D density → complex assembly, subunit spatial arrangement |
| 49 | EMDB — the Electron Microscopy Data Bank (NAR PMC OA) | https://pmc.ncbi.nlm.nih.gov/articles/PMC10767987/ | HTML (PMC OA) | Cryo-EM maps | 30,000+ 3DEM maps; single-particle/tomography → 3D structure of complexes/cells |
| 50 | EMPIAR — Electron Microscopy Public Image Archive | https://www.ebi.ac.uk/empiar/ | HTML (raw image stacks/TIFF/MRC) | Raw cryo-EM / volume-EM images | Raw 2D micrographs + 3D bioimaging (cryo-ET, X-ray tomography) → image-to-3D-structure provenance |
| 51 | EMPIAR: the Electron Microscopy Public Image Archive (NAR PMC OA) | https://pmc.ncbi.nlm.nih.gov/articles/PMC9825465/ | HTML (PMC OA) | Raw EM image archive | Specimens, datasets, EMDB/PDB links → raw images underpinning 3D maps/models |
| 52 | BioImage Archive (EMBL-EBI) | https://www.ebi.ac.uk/bioimage-archive/ | HTML (multi-modal images) | Biological image repository | Cells, tissues, organisms across all imaging modalities → publication-linked reference image datasets |
| 53 | The BioImage Archive — home for life-sciences microscopy (bioRxiv) | https://www.biorxiv.org/content/10.1101/2021.12.17.473169v2.full | HTML (preprint) | Image archive description | Microscopy datasets, study metadata → image–study–publication links across modalities |
| 54 | Image Data Resource (IDR) | https://idr.openmicroscopy.org/ | HTML (OME-TIFF + OMERO viewer) | Published bioimage datasets | Cells, phenotypes (ontology-tagged), genes/chemicals, high-content screens → perturbation–phenotype image associations |
| 55 | IDR: bioimage data integration & publication platform (PMC OA) | https://pmc.ncbi.nlm.nih.gov/articles/PMC5536224/ | HTML (PMC OA) | Bioimage data platform | Imaging modalities, genetic/chemical perturbations, phenotypes → gene/compound–cell-phenotype edges |
| 56 | Human Protein Atlas (proteinatlas.org) | https://www.proteinatlas.org/ | HTML (IHC/IF images) | Tissue & subcellular imaging | Genes/proteins, 48 tissues, 35 organelles, cancers; antibody staining → protein–tissue expression, protein–organelle localization |
| 57 | The Human Protein Atlas — spatial localization (Mol Cell Prot, PMC OA) | https://pmc.ncbi.nlm.nih.gov/articles/PMC7737765/ | HTML (PMC OA) | Tissue/subcellular imaging | 5M+ IHC images, antibodies, cell types → protein spatial-expression map (tissue/cell/subcellular) |
| 57b | HPA: integrated omics for single-cell mapping (PMC OA) | https://pmc.ncbi.nlm.nih.gov/articles/PMC9850435/ | HTML (PMC OA) | Single-cell / proteomics imaging | Genes, proteins, cell types, tissues → gene–protein–cell-type–tissue spatial expression |
| 58 | GTEx Histology Viewer | https://gtexportal.org/home/histologyPage | HTML (whole-slide images) | Histopathology imaging | 50+ tissue types, ~1000 donors, H&E slides → tissue–donor histology, pathology annotations |
| 59 | GTEx Portal (expression + histology integration) | https://gtexportal.org/home/ | HTML (images + plots) | Multi-modal tissue atlas | Genes, tissues, eQTLs, histology → gene-expression-by-tissue plots linked to slide images |
| 60 | The Cancer Imaging Archive (TCIA) | https://www.cancerimagingarchive.net/ | HTML (DICOM/SVS/NDPI) | Cancer radiology + pathology | Patients, tumors, modalities (CT/MRI/PET/histopath), genomics → image–diagnosis–outcome–genomics collections |
| 61 | TCIA: public cancer radiology imaging collections (PMC OA) | https://pmc.ncbi.nlm.nih.gov/articles/PMC5827108/ | HTML (PMC OA) | Cancer imaging collections | Collections by disease/modality, patient outcomes → tumor–image–outcome associations |
| 62 | CAMELYON16 Grand Challenge (lymph-node WSI) | https://camelyon16.grand-challenge.org/ | HTML (WSI + XML masks) | Histopathology / digital pathology | H&E sentinel-lymph-node slides, metastasis regions, masks → tumor-region annotation, normal-vs-metastasis labels |
| 63 | CAMELYON dataset: 1399 H&E lymph-node sections (GigaScience OA) | https://academic.oup.com/gigascience/article/7/6/giy065/5026175 | HTML (OA) | Histopathology dataset | Patients, slides, metastasis contours (pN-stage) → slide–metastasis–stage labels |
| 64 | MIMIC-CXR Database (PhysioNet) | https://physionet.org/content/mimic-cxr/2.1.0/ | HTML (DICOM + reports) | Chest radiography | 377K chest X-rays, free-text reports, 14 finding labels → image–finding–report associations |
| 65 | MIMIC-CXR (Scientific Data) | https://www.nature.com/articles/s41597-019-0322-0 | HTML (Nature OA) | Chest radiography dataset | Radiographs, studies, NLP-derived findings → image–pathology label edges |
| 66 | Cell Painting Gallery (JUMP morphological profiling) | https://registry.opendata.aws/cellpainting-gallery/ | HTML (multichannel images) | High-content phenotypic imaging | U2OS cells, 116K compounds, gene OE/KO, 8 stained organelles → perturbation–morphology phenotype images |
| 67 | JUMP Cell Painting dataset (bioRxiv) | https://www.biorxiv.org/content/10.1101/2023.03.23.534023v1 | HTML (preprint) | Morphological profiling | 136K chemical+genetic perturbations, single-cell profiles → compound/gene–cell-morphology edges |
| 68 | The Cell Image Library (CIL/CCDB) | https://www.cellimagelibrary.org/ | HTML (microscopy images/video) | Cell-biology microscopy | Cells, organelles, organisms, processes → cell-architecture + cellular-process visual annotations |
| 69 | The cell: an image library-CCDB (NAR PMC OA) | https://pmc.ncbi.nlm.nih.gov/articles/PMC3531121/ | HTML (PMC OA) | Microscopy repository | Light/EM images, cell types, structures → annotated cell/subcellular image records |
| 70 | Allen Brain Atlas (brain-map.org) | https://portal.brain-map.org/ | HTML (ISH images + 3D viewer) | Brain gene-expression imaging | Genes, brain regions, ISH expression energy, 3D atlas → gene–brain-region expression, region co-expression edges |
| 71 | Allen Brain Atlas: integrated spatio-temporal portal (NAR OA) | https://academic.oup.com/nar/article/41/D1/D996/1052578 | HTML (OA) | Brain atlas resource | ISH/microarray/RNA-seq, connectivity, neuroanatomy → gene–region expression + projection connectivity |
| 72 | Brain Image Library (BIL) | https://www.brainimagelibrary.org/ | HTML (whole-brain microscopy) | Neuroscience microscopy archive | Whole-brain images, neuron morphologies, connectivity, spatial transcriptomics → cell–cell connectivity, region–transcript spatial maps |
| 73 | Brain Image Library: community microscopy resource (Sci Data) | https://www.nature.com/articles/s41597-024-03761-8 | HTML (Nature OA) | Brain microscopy repository | Species, modalities (STPT/fMOST/LSFM), neurons → brain-wide image + neuron-morphology datasets |
| 74 | NeuroMorpho.Org (3D neuron reconstructions) | https://neuromorpho.org/ | HTML (SWC morphology + 2D/3D render) | Neuron morphology archive | 150K neurons, 78 species, 1,330 cell types, 381 brain regions → neuron–cell-type–region morphology, branch topology |
| 75 | NeuroMorpho.Org: central resource for neuronal morphologies (J Neurosci) | https://www.jneurosci.org/content/27/35/9247 | HTML (OA) | Neuron morphology | Reconstructed dendritic/axonal trees, soma → branch-point/segment graph per neuron |
| 76 | HuBMAP Data Portal (spatial single-cell tissue atlas) | https://portal.hubmapconsortium.org/ | HTML (Vitessce spatial images + UMAP) | Spatial multi-omics imaging | Organs, cells, genes, biomarkers; spatial + single-cell → cell–biomarker, cell–spatial-neighborhood edges |
| 77 | HuBMAP: 3D Human Reference Atlas construction (PMC OA) | https://pmc.ncbi.nlm.nih.gov/articles/PMC11142047/ | HTML (PMC OA) | Reference-atlas figures | Anatomical structures, cell types, biomarkers (ASCT+B) → organ–tissue–cell-type–biomarker hierarchy diagrams |
| 78 | Tabula Sapiens: single-cell atlas of human organs (Science) | https://www.science.org/doi/10.1126/science.abl4896 | HTML (abstract; figures; preprint OA) | Single-cell atlas figures | 400+ cell types, 24 tissues, ~500K cells; UMAP clusters, marker dot-plots → cell-type–tissue, gene–cell-type marker edges |
| 79 | Tabula Sapiens (bioRxiv full text + UMAP figures) | https://www.biorxiv.org/content/10.1101/2021.07.19.452956v2.full | HTML (preprint) | Single-cell atlas figures | Cells, tissues, marker genes; tissue-specific UMAP clustering → cell-type clustering + tissue-of-origin separation |
| 80 | Open-i (NLM open-access biomedical image search) | https://openi.nlm.nih.gov/ | HTML (figures + captions) | Figure/image search engine | 3.7M+ figures (charts, micrographs, radiographs) from PMC + collections → figure–caption (entity-mention) pairs |
| 81 | MultiCaRe: clinical cases/images/captions from PMC (ScienceDirect OA) | https://www.sciencedirect.com/science/article/pii/S2352340923010351 | HTML (OA) | Figure-caption dataset | 135K images, labels, captions from 75K case reports → image–label–caption (diagnosis) triples |
| 82 | Open-PMC-18M: large medical image-caption dataset (arXiv) | https://arxiv.org/pdf/2506.02738 | PDF (preprint) | Figure-caption corpus | 18M+ subfigure–caption pairs from PMC-OA → biomedical figure–text entity/relation alignment |
| 83 | NCI Visuals Online (open biomedical illustrations) | https://visualsonline.cancer.gov/ | HTML (image, public domain) | Biomedical illustrations | Normal vs. cancer cells, mechanisms, anatomy → labeled-diagram cell/structure relationships |
| 84 | MedPix (NLM teaching radiology cases) | https://medpix.nlm.nih.gov/home | HTML (images + case text) | Radiology teaching images | 59K images, 12K cases; modality, diagnosis, differential → image–finding–diagnosis teaching associations |

## Entity & relation patterns observed

This source class is fundamentally **graphical**: entities are rendered as nodes,
glyphs, or image regions, and relationships are rendered as arrows, edges, spatial
adjacency, or co-presence in a plot. Three structural sub-genres recur, each with a
distinct entity/relation profile.

### A. Pathway / mechanism diagrams (KEGG, Reactome, WikiPathways, PANTHER, SMPDB/PathBank, INOH, SignaLink, Disease Maps/MINERVA, BioModels)

**Node (entity) types:**
- **Gene / protein / enzyme** (often as KO or ortholog group)
- **Protein complex / molecular machine** (composed-of subunits)
- **Small molecule / metabolite / compound / cofactor / ion**
- **Reaction / process** (a first-class node in SBGN process-description)
- **Drug** and **drug target**
- **Cellular compartment / organelle** (a containment context, not just decoration)
- **Phenotype / disease** (as pathway endpoints or disease-map foci)
- **Pathway / subpathway** (an aggregate node containing reactions)
- **Tissue / cell type / organism** (context qualifiers on a map)
- **Virus / pathogen protein** (in host–pathogen disease maps)

**Edge (relation) types:**
- **catalyzes / enzyme-of** (enzyme → reaction)
- **input-of / output-of / consumes / produces** (metabolite ↔ reaction)
- **activates / inhibits / phosphorylates / ubiquitinates** (directed signaling)
- **regulates / modulates** (positive/negative modulation glyphs)
- **expresses / transcribes / translates** (gene → product)
- **binds / forms-complex-with** (assembly)
- **transports / translocates** (across compartments)
- **part-of / contained-in** (reaction → pathway; entity → compartment)
- **virus-protein interacts-with host-protein** (host–pathogen)
- **drug targets / inhibits target** (overlay edges)

### B. Image / microscopy / histology / radiology archives (BioImage Archive, IDR, EMPIAR/EMDB, HPA, GTEx, TCIA, CAMELYON, MIMIC-CXR, Cell Painting, Cell Image Library, Allen/Brain Image Library, HuBMAP, NeuroMorpho)

**Node types:** **specimen / patient / donor**, **tissue**, **cell / cell type**,
**organelle / subcellular structure**, **gene / protein** (the imaged target),
**antibody / stain / channel**, **anatomical region** (brain region, organ),
**imaging modality** (IHC, IF, cryo-EM, CT, MRI, H&E WSI, light-sheet),
**phenotype** (ontology-tagged), **perturbation** (compound / gene KO/OE),
**study / dataset / publication** (provenance), **3D structure / density map**.

**Edge types:**
- **protein/gene — expressed-in / localized-to → tissue / cell / organelle**
  (HPA, Allen ISH, GTEx) — the core "molecule has a spatial address" edge.
- **perturbation (compound/gene) — induces → cell phenotype / morphology**
  (IDR, Cell Painting/JUMP) — image-based mechanism-of-action edges.
- **specimen / image — has-diagnosis / has-finding → disease / pathology**
  (TCIA, MIMIC-CXR, CAMELYON, MedPix) — image → clinical-label edges.
- **image — depicts → entity at coordinates** (segmentation masks, ROI, binding
  pocket) — spatial-region annotation.
- **cell — connected-to / projects-to → cell / region** (BIL, NeuroMorpho,
  Allen connectivity) — neural connectivity / morphology-as-graph.
- **raw image — underpins → 3D structure → atomic model** (EMPIAR → EMDB → PDB) —
  a provenance chain across imaging resolution scales.
- **anatomical structure — contains → cell type — expresses → biomarker** (HuBMAP
  ASCT+B) — a multi-scale anatomy→cell→molecule hierarchy.

### C. Network & atlas figures (STRING, BioGRID, Cytoscape figures, SPOKE, single-cell UMAP/marker plots)

**Node types:** proteins/genes (PPI), and in property-graph KGs (SPOKE) a rich
typed universe — **Disease, Gene, Compound, Protein, Symptom, Pathway, Anatomy,
Side Effect, Pharmacologic Class, Biological Process**. In single-cell figures:
**cell, cell type/cluster, marker gene, tissue**.

**Edge types:** **physically-interacts / functionally-associates / co-expresses**
(STRING/BioGRID, with confidence scores and evidence channels as edge attributes);
typed biomedical relations **TREATS, BINDS, CONTRAINDICATES, UPREGULATES,
DOWNREGULATES, CAUSES (side effect), ASSOCIATES (gene–disease), PART-OF, EXPRESSES,
LOCALIZES, RESEMBLES** (SPOKE's 91 edge types); **marker-of / clusters-with /
derived-from-tissue** (single-cell plots).

## Implications for the BioOKF type universe

1. **Reactions/processes are first-class nodes, not just edges.** SBGN-PD,
   Reactome, and KEGG model a reaction as a node with typed roles (input, output,
   catalyst, regulator) attaching entities to it. BioOKF likely needs a
   **reified "reaction/process" node type** so a single biochemical event can
   connect ≥3 participants with role-typed edges — a plain binary edge loses the
   stoichiometry and the catalyst/modulator distinction.

2. **Compartment / spatial context is a structural qualifier.** Pathway diagrams
   place entities in organelles; image archives give molecules a literal spatial
   address (tissue → cell → organelle → coordinates). The same gene–"present" edge
   means something different in nucleus vs. membrane vs. a given tissue. BioOKF
   edges need **compartment / tissue / cell-type qualifier slots**, mirroring the
   tissue/cell-type qualifiers already noted in the genomics catalog.

3. **Directed, signed, mechanism-typed edges.** Unlike association edges from
   GWAS, pathway/signaling diagrams encode **direction and sign with a specific
   mechanism** (phosphorylates, ubiquitinates, transcribes, inhibits). The edge
   type vocabulary must be richer than "interacts" — it should carry a
   molecular-action label, which is exactly what visual glyphs disambiguate.

4. **Multi-scale anatomy hierarchy.** HuBMAP's ASCT+B and the Human Reference
   Atlas formalize **anatomical-structure → cell-type → biomarker** containment.
   BioOKF benefits from explicit **part-of / contains** edges spanning
   organ → tissue → cell → organelle → molecule, letting image-derived
   localizations attach at the right scale.

5. **Perturbation → phenotype is an image-native edge.** Cell Painting/IDR encode
   **compound-or-gene → cellular-morphology-phenotype**, a mechanism-of-action edge
   that is *only* observable in images. This argues for a **phenotype node type**
   (morphological, histological, radiological) and a **perturbation→phenotype**
   edge distinct from molecular interaction.

6. **Provenance chains across imaging scales.** EMPIAR→EMDB→PDB is a literal
   "raw image → 3D density → atomic model" chain; TCIA links image→genomics→outcome.
   BioOKF should treat **datasets/images/structures as provenance-bearing nodes**
   with derived-from / underpins edges, so a structural or phenotypic claim can be
   traced back to the primary image.

7. **Knowledge graphs already supply a reusable type schema.** SPOKE (28 node
   types / 91 edge types) and the curated pathway resources are not just data —
   they are **explicit ontologies of entity and relation types** that BioOKF can
   adopt or align to, rather than re-deriving the type universe from scratch.

8. **Figure–caption pairs are the extraction substrate.** Open-i, MultiCaRe, and
   Open-PMC-18M show that the *caption* names the entities and relations a figure
   draws. Visual-relation extraction for BioOKF should jointly use the **image
   (node/arrow geometry) + caption (entity mentions + relation verbs)**, treating
   the diagram as ground truth for structure and the caption as the label source.
