# Attributes, Provenance & Evidence on Biomedical Nodes and Edges

> Reference for the BioKF type-system design effort. Where the prior taxonomy
> docs (`taxonomy-ontologies.md`, `taxonomy-umls.md`, `taxonomy-spoke-hetionet.md`,
> `taxonomy-semmeddb.md`) enumerate the **node-type** and **edge-type** universes,
> this document enumerates the **third axis**: the *attributes* (properties /
> metadata fields) that biomedical knowledge resources attach to nodes and — far
> more importantly — to **edges (associations)**. It covers four sub-systems:
>
> 1. **Identifiers / CURIEs** — the standard namespace prefixes every node carries.
> 2. **Provenance** — *who/what said it, and where it came from* (knowledge sources,
>    agent type, publications, retrieval source).
> 3. **Evidence** — *what supports it* (ECO, GO evidence codes, evidence ontology
>    terms, evidence counts).
> 4. **Statistical / quantitative attributes** — *how strong / how measured*
>    (p-value, effect size, OR, HR, CI, sample size, fold-change, correlation).
> 5. **Qualifiers** — *under what context* (species, anatomy, aspect, direction,
>    form/variant, sex, onset, severity, frequency).
>
> Type names are quoted **verbatim** from the source schemas (case/underscores
> preserved). The two authoritative cross-cutting prior-art schemas are the
> **Biolink Model** (the `Association` class and its slots) and the **Evidence &
> Conclusion Ontology (ECO)**. Compiled 2026-06-25.

---

## 0. The "attributes" axis in one picture

A typical biomedical KG edge is not just `(subject, predicate, object)`. The
edge carries a *bundle of attributes* answering five orthogonal questions:

| Question | Sub-system | Canonical slot examples |
|----------|-----------|-------------------------|
| **What/who is it?** (node identity) | Identifiers / CURIEs | `id`, `xref`, `provided_by`, `category` |
| **Where did the claim come from?** | Provenance | `primary_knowledge_source`, `aggregator_knowledge_source`, `publications`, `agent_type` |
| **How is the claim known?** | Knowledge level / evidence | `knowledge_level`, `has_evidence`, `has_evidence_of_type` (ECO), GO evidence codes |
| **How strong / how measured?** | Statistics | `p_value`, `adjusted_p_value`, effect size, OR/HR, CI, sample size, fold-change, correlation |
| **In what context?** | Qualifiers | `species_context_qualifier`, `anatomical_context_qualifier`, aspect/direction qualifiers, `sex_qualifier`, `frequency_qualifier` |

The rest of this document enumerates each.

---

## 1. Standard identifiers / CURIEs (node identity attributes)

Every node in a federated biomedical KG is keyed by a **CURIE**
(`prefix:localID`). The prefix encodes the source namespace; the local ID matches
a namespace-specific pattern. Prefixes below are the **Biolink Model / Bioregistry
canonical** spellings (case-sensitive); the expansion URLs are from the Biolink
prefix map; the ID patterns are from identifiers.org / Bioregistry.

### 1.1 Core requested namespaces

| Namespace | Canonical CURIE prefix | Entity kind | ID pattern / example | Expansion base URL |
|-----------|------------------------|-------------|----------------------|--------------------|
| HGNC | `HGNC` | Gene (human symbol authority) | `^\d{1,5}$` → `HGNC:11998` (TP53) | `http://identifiers.org/hgnc/` |
| Ensembl | `ENSEMBL` | Gene/transcript/protein | `ENSG…`/`ENST…`/`ENSP…` → `ENSEMBL:ENSG00000139618` | `http://identifiers.org/ensembl/` |
| UniProt | `UniProtKB` | Protein | 6 or 10 char accession → `UniProtKB:P04637` | `http://purl.uniprot.org/uniprot/` |
| NCBI Gene (Entrez) | `NCBIGene` | Gene | `^\d+$` → `NCBIGene:7157` | `http://identifiers.org/ncbigene/` |
| ChEBI | `CHEBI` | Chemical entity | `^CHEBI:\d+$` → `CHEBI:16236` | `http://purl.obolibrary.org/obo/CHEBI_` |
| DrugBank | `DRUGBANK` | Drug | `^DB\d{5}$` → `DRUGBANK:DB00945` | `http://identifiers.org/drugbank/` |
| RxNorm (RXCUI) | `RXCUI` (also `RXNORM`) | Clinical drug concept | `^\d+$` → `RXCUI:198440` | `https://mor.nlm.nih.gov/RxNav/search?...` / `http://purl.bioontology.org/ontology/RXNORM/` |
| MONDO | `MONDO` | Disease | `^MONDO:\d{7}$` → `MONDO:0005835` | `http://purl.obolibrary.org/obo/MONDO_` |
| Disease Ontology | `DOID` | Disease | `^DOID:\d+$` → `DOID:10652` | `http://purl.obolibrary.org/obo/DOID_` |
| MeSH | `MESH` | Headings (multi-domain) | `^[A-Z]\d+$` / `^[CD]\d{6}$` → `MESH:D000544` | `http://id.nlm.nih.gov/mesh/` |
| Human Phenotype Ontology | `HP` | Phenotype | `^HP:\d{7}$` → `HP:0001250` | `http://purl.obolibrary.org/obo/HP_` |
| UMLS CUI | `UMLS` | Concept Unique Identifier | `^C\d{7}$` → `UMLS:C0027051` | `http://identifiers.org/umls/` |
| PubChem Compound | `PUBCHEM.COMPOUND` | Chemical (CID) | `^\d+$` → `PUBCHEM.COMPOUND:5426` | `http://identifiers.org/pubchem.compound/` |
| ClinicalTrials.gov | `clinicaltrials` / `ClinicalTrials` | Clinical trial | `^NCT\d{8}$` → `clinicaltrials:NCT04178122` | `https://clinicaltrials.gov/ct2/show/` |
| DOI | `doi` (also `DOI`) | Document | `^10\.\d{4,}/.+$` → `doi:10.1111/cts.13302` | `https://doi.org/` |
| PubMed ID | `PMID` | Article | `^\d+$` → `PMID:34986598` | `http://www.ncbi.nlm.nih.gov/pubmed/` |

Notes:
- **HGNC** is the *symbol authority* for human genes; many KGs key genes on
  `NCBIGene` (Entrez) and use `HGNC`/`ENSEMBL`/`UniProtKB` as `xref`/equivalent IDs.
- **PubChem CID** pattern is `[1-9]\d{0,8}` (1–9 digits). PubChem also has the
  separate `PUBCHEM.SUBSTANCE` namespace (SIDs).
- **MeSH** spans many domains (it appears in `taxonomy-ontologies.md` §1 as a
  16-category vocabulary); the same `MESH:` prefix tags Diseases (`C…`),
  Chemicals (`D…`), and supplementary concept records.
- **NCT** numbers (`clinicaltrials`) and **PMID/DOI** are the canonical
  *provenance* identifiers (see §3): they appear as the `publications` of an edge,
  not (usually) as subject/object nodes — though a trial *can* be a node.

### 1.2 Other high-frequency namespaces (commonly co-occur)

| Namespace | Prefix | Entity kind | Example | Expansion base URL |
|-----------|--------|-------------|---------|--------------------|
| Gene Ontology | `GO` | BP/MF/CC term | `GO:0008150` | `http://purl.obolibrary.org/obo/GO_` |
| Uberon | `UBERON` | Anatomy | `UBERON:0002107` | `http://purl.obolibrary.org/obo/UBERON_` |
| Cell Ontology | `CL` | Cell type | `CL:0000540` | `http://purl.obolibrary.org/obo/CL_` |
| NCBI Taxonomy | `NCBITaxon` | Organism/species | `NCBITaxon:9606` (human) | `http://purl.obolibrary.org/obo/NCBITaxon_` |
| ECO | `ECO` | Evidence term | `ECO:0000269` | `http://purl.obolibrary.org/obo/ECO_` |
| OMIM | `OMIM` / `MIM` | Mendelian disease/gene | `OMIM:114480` | `http://purl.obolibrary.org/obo/OMIM_` |
| Orphanet | `orphanet` / `Orphanet` | Rare disease | `Orphanet:558` | `http://www.orpha.net/ORDO/Orphanet_` |
| SNOMED CT | `SNOMEDCT` | Clinical concept | `SNOMEDCT:73211009` | `http://snomed.info/id/` |
| ICD-10(-CM) | `ICD10` / `ICD10CM` | Diagnosis code | `ICD10:E11` | (ICD code resolvers) |
| ICD-11 | `ICD11` | Diagnosis code | `ICD11:5A11` | (WHO ICD-11 API) |
| LOINC | `LOINC` | Lab/observable | `LOINC:4548-4` | `http://loinc.org/rdf/` |
| Reactome | `REACT` | Pathway | `REACT:R-HSA-109582` | `http://www.reactome.org/...` |
| Protein Ontology | `PR` | Protein form | `PR:000003035` | `http://purl.obolibrary.org/obo/PR_` |
| EFO | `EFO` | Experimental factor / trait | `EFO:0000400` | `http://www.ebi.ac.uk/efo/EFO_` |
| NCI Thesaurus | `NCIT` | Cancer/clinical concept | `NCIT:C3262` | `http://purl.obolibrary.org/obo/NCIT_` |
| dbSNP | `DBSNP` | Variant | `DBSNP:rs7412` | `https://www.ncbi.nlm.nih.gov/snp/` |
| ClinVar | `CLINVAR` | Variant interpretation | `CLINVAR:12345` | `https://www.ncbi.nlm.nih.gov/clinvar/` |
| CHEMBL | `CHEMBL.COMPOUND` | Bioactive molecule | `CHEMBL.COMPOUND:CHEMBL25` | `https://www.ebi.ac.uk/chembl/` |
| PubChem Substance | `PUBCHEM.SUBSTANCE` | Substance (SID) | `PUBCHEM.SUBSTANCE:347827423` | `http://identifiers.org/pubchem.substance/` |

### 1.3 Node-identity attribute slots (Biolink `NamedThing`)

Beyond the CURIE in the `id` slot, nodes carry these standard attribute slots
(Biolink `NamedThing` / `entity`):

`id` · `name` · `category` (the Biolink class, e.g. `biolink:Gene`) ·
`description` · `xref` (cross-references / equivalent CURIEs) ·
`synonym` · `iri` · `type` · `provided_by` (source KG/database) ·
`in_taxon` + `in_taxon_label` (species) · `has_attribute` · `deprecated`.

---

## 2. The Biolink `Association` — the master edge-attribute schema

The single most important prior art for "what attributes go on an edge" is the
**Biolink Model** `Association` class. An `Association` reifies a `(subject,
predicate, object)` triple and hangs **all metadata** off it. Below is the full
enumerated slot set, grouped by sub-system. (Source: Biolink `Association` class
docs; ranges shown as the Biolink range type.)

### 2.1 Core triple slots

| Slot | Range | Notes |
|------|-------|-------|
| `subject` | NamedThing | edge source node |
| `predicate` | uriorcurie | the Biolink relation, e.g. `biolink:treats` |
| `object` | NamedThing | edge target node |
| `negated` | boolean | the statement is **explicitly false** (e.g. "X does NOT treat Y") |
| `qualifier` | string | (legacy) single qualifier |
| `qualifiers` | OntologyClass [] | (legacy multivalued) — superseded by typed qualifier slots (§6) |
| `subject_category` / `object_category` | OntologyClass | the category of each endpoint |
| `subject_category_closure` / `object_category_closure` | OntologyClass [] | ancestor categories (for query closure) |

### 2.2 Provenance slots (who said it / where from) — see §3

| Slot | Range | Cardinality | Meaning |
|------|-------|-------------|---------|
| `knowledge_source` | string | — | (generic) source of the assertion |
| `primary_knowledge_source` | string | **1 per edge** | the original resource that *first* asserted the edge |
| `aggregator_knowledge_source` | string | * | resources that aggregated/relayed it |
| `supporting_data_source` | string | * | underlying data sources |
| `provided_by` | string | * | the KG/ingest that supplied the record |
| `sources` / `retrieval_source_ids` | RetrievalSource [] | * | structured provenance chain (TRAPI `RetrievalSource`, §3.3) |
| `publications` | Publication [] | * | PMIDs/DOIs/NCTs backing the edge |
| `original_subject` / `original_object` | string | — | pre-normalization IDs (provenance of ID mapping) |
| `original_predicate` | uriorcurie | — | pre-normalization predicate |

### 2.3 Knowledge-level & agent slots (how it's known) — see §3.1

| Slot | Range | Required? | Meaning |
|------|-------|-----------|---------|
| `knowledge_level` | `KnowledgeLevelEnum` | **required** | strength/scope of the claim (§3.1) |
| `agent_type` | `AgentTypeEnum` | **required** | human vs machine origin (§3.2) |

### 2.4 Evidence slots (what supports it) — see §4

| Slot | Range | Meaning |
|------|-------|---------|
| `has_evidence` | InformationContentEntity [] | evidence artifacts supporting the edge |
| `has_evidence_of_type` | **EvidenceType** (ECO) [] | the ECO evidence-type term(s) |
| `has_supporting_studies` | Study [] | studies backing the edge |
| `supporting_text` | string [] | text snippets (literature evidence) |
| `evidence_count` | integer | number of evidence instances |
| `semmed_agreement_count` | integer | (SemMedDB-specific) supporting-predication count |
| `elevate_to_prediction` | boolean | mark a statistical edge as a predictive claim |

### 2.5 Statistical slots (how strong) — see §5

| Slot | Range | Meaning |
|------|-------|---------|
| `p_value` | float | unadjusted significance |
| `adjusted_p_value` | float | FDR / multiple-testing-corrected p |
| `has_confidence_score` | float | a normalized confidence (0–1) |
| `has_count` | integer | # things with the property (FrequencyQuantifier mixin) |
| `has_total` | integer | # things in the reference set |
| `has_quotient` | double | `has_count / has_total` |
| `has_percentage` | double | `has_quotient × 100` |
| `frequency_qualifier` | FrequencyValue | qualitative frequency (HP frequency terms) |

### 2.6 Context / housekeeping slots

`subject_feature_name` · `object_feature_name` · `timepoint` (TimeType) ·
`update_date` · plus the typed **qualifier** slots in §6.

> **Design takeaway for BioKF.** Biolink makes exactly **two** edge attributes
> *required* — `knowledge_level` and `agent_type` — and everything else
> (publications, evidence, p-values, qualifiers) *optional*. This is the cleanest
> precedent for a "required-vs-optional" split: require provenance-of-reasoning
> (`knowledge_level`, `agent_type`, `primary_knowledge_source`), keep quantitative
> and evidence detail optional.

---

## 3. Provenance models

### 3.1 `KnowledgeLevelEnum` — strength/scope of the claim (Biolink, **required**)

The level of knowledge expressed, based on the reasoning/analysis used to
generate it. All 7 permissible values, verbatim:

| Value | Definition |
|-------|------------|
| `knowledge_assertion` | A statement of purported fact put forth by an agent as true, based on assessment of **direct evidence**. (Strongest.) |
| `logical_entailment` | A conclusion that follows **logically** from premises representing established facts or knowledge assertions. |
| `prediction` | A statement of a *possible* fact based on **probabilistic** reasoning over more indirect evidence. |
| `statistical_association` | Reports that variables in a dataset are **statistically associated** within a particular cohort. |
| `text_co_occurrence` | Reports that mentions of two concepts **co-occur** in a text corpus at statistically significant frequency. |
| `observation` | Reports (and possibly quantifies) a phenomenon **observed** to occur — absent analysis/interpretation. |
| `not_provided` | Knowledge level cannot be determined from available information. |

General strength ordering: *assertions > entailments > predictions*; specificity
ranges from context-specific data-analysis results up to generalized assertions.

### 3.2 `AgentTypeEnum` — human vs machine origin (Biolink, **required**)

The high-level category of agent that *originally generated* the statement (not
necessarily the source of its supporting evidence). All 8 values, verbatim:

| Value | Definition |
|-------|------------|
| `manual_agent` | A **human** agent responsible for generating the statement. |
| `automated_agent` | An automated agent (software/tool) responsible for generating the statement. |
| `data_analysis_pipeline` | An automated agent that executes an analysis workflow over data and reports the **direct results**. |
| `computational_model` | An automated agent that generates statements (typically predictions) from rules/logic encoded in an algorithm. |
| `text_mining_agent` | An automated agent using **NLP** to recognize concepts/relationships in text. |
| `image_processing_agent` | An automated agent that processes **images** to generate textual knowledge statements. |
| `manual_validation_of_automated_agent` | A **human** reviews/validates knowledge initially generated by an automated agent. |
| `not_provided` | Cannot determine whether the generating agent was manual or automated. |

### 3.3 Knowledge-source provenance: `ResourceRoleEnum` + `RetrievalSource` (Biolink / TRAPI)

NCATS **Translator / TRAPI** introduced a structured provenance chain. Each edge
records one or more `RetrievalSource` objects; each names an `InformationResource`
and its **role**. `ResourceRoleEnum` (all 3 values, verbatim):

| Value | Meaning | Cardinality rule |
|-------|---------|------------------|
| `primary_knowledge_source` | the resource that **originally produced** the knowledge. | **exactly one** per edge |
| `aggregator_knowledge_source` | a resource that **collected/relayed** knowledge from other sources. | any number |
| `supporting_data_source` | a resource providing the **underlying data** an analysis ran over. | any number |

`RetrievalSource` slots: `resource_id` (InformationResource CURIE, `infores:…`),
`resource_role` (one of the above), `upstream_resource_ids` (the chain of sources
this one drew from), `source_record_urls` (deep links to the original record).
Information resources are themselves identified with the **`infores:`** CURIE
namespace (e.g. `infores:disgenet`, `infores:ctd`, `infores:semmeddb`).

### 3.4 Publications & literature provenance (Biolink `Publication` node)

The `publications` edge slot points at `Publication` nodes (keyed by `PMID`,
else `DOI`, else another CURIE — that precedence is the Biolink convention).
`Publication` node attributes:

`id` · `name` · `authors` [] · `pages` · `summary` · `keywords` [] ·
`mesh_terms` [] · `publication_type` [] · `creation_date` · `xref` [] ·
`provided_by` · plus subtype classes `Article`, `Book`, `Serial`,
`JournalArticle`, `Patent`, `WebPage`, `PreprintPublication`, `DrugLabel`.

---

## 4. Evidence models

### 4.1 ECO — Evidence & Conclusion Ontology (`ECO:` / Biolink `EvidenceType`)

ECO is the **community standard for evidence information** (>1,500 terms; used by
GO, UniProt, and most model-organism DBs). It is what the Biolink
`has_evidence_of_type` slot ranges over (`EvidenceType`). Root =
**`ECO:0000000` "evidence"** ("a type of information that is used to support an
assertion"). Two conceptual top branches: **evidence** and **assertion method**
(which combine to say *what supports the claim* and *whether a human or machine
asserted it*).

#### 4.1a Direct children of `ECO:0000000` (the 9 top evidence branches)

| ECO ID | Class |
|--------|-------|
| `ECO:0000006` | experimental evidence |
| `ECO:0007672` | computational evidence |
| `ECO:0006055` | high throughput evidence |
| `ECO:0000041` | similarity evidence |
| `ECO:0000361` | inferential evidence |
| `ECO:0000212` | combinatorial evidence |
| `ECO:0006151` | documented statement evidence |
| `ECO:0000352` | evidence used in **manual** assertion |
| `ECO:0000501` | evidence used in **automatic** assertion |

The last two are the **assertion-method axis**: every leaf evidence term has a
"…used in manual assertion" vs "…used in automatic assertion" variant, which is
how ECO encodes the manual/automatic (curator vs pipeline) distinction.

#### 4.1b Commonly-used ECO terms (the ones KGs actually attach to edges)

| ECO ID | Term | Typical use |
|--------|------|-------------|
| `ECO:0000269` | experimental evidence used in manual assertion | gold-standard curated experimental support |
| `ECO:0000314` | direct assay evidence used in manual assertion | a direct assay result |
| `ECO:0000353` | physical interaction evidence used in manual assertion | PPI / binding |
| `ECO:0000315` | mutant phenotype evidence used in manual assertion | knockout/mutation phenotype |
| `ECO:0000316` | genetic interaction evidence used in manual assertion | epistasis / genetic interaction |
| `ECO:0000270` | expression pattern evidence used in manual assertion | expression-based support |
| `ECO:0000250` | sequence similarity evidence used in manual assertion | "by similarity" |
| `ECO:0000266` | sequence orthology evidence | orthology transfer |
| `ECO:0000305` | curator inference used in manual assertion | inferred by curator (IC) |
| `ECO:0000303` | author statement without traceable support | NAS |
| `ECO:0000304` | author statement supported by traceable reference | TAS |
| `ECO:0000307` | no evidence data found | ND |
| `ECO:0000501` | evidence used in automatic assertion | umbrella for electronic/automatic |
| `ECO:0007669` | computational evidence used in automatic assertion | IEA-style pipeline annotation |

### 4.2 GO evidence codes — the canonical 3-letter evidence codes

Gene Ontology Annotation (GAF) attaches a **3-letter evidence code** to every
gene–term edge; each maps to an ECO term. The full set, grouped:

| Group | Codes (abbrev → name) |
|-------|-----------------------|
| **Experimental** | `EXP` Inferred from Experiment · `IDA` Direct Assay · `IPI` Physical Interaction · `IMP` Mutant Phenotype · `IGI` Genetic Interaction · `IEP` Expression Pattern |
| **High-throughput** | `HTP` High Throughput Experiment · `HDA` HT Direct Assay · `HMP` HT Mutant Phenotype · `HGI` HT Genetic Interaction · `HEP` HT Expression Pattern |
| **Phylogenetic** | `IBA` Biological aspect of Ancestor · `IBD` Biological aspect of Descendant · `IKR` Key Residues · `IRD` Rapid Divergence |
| **Computational analysis** | `ISS` Sequence/structural Similarity · `ISO` Sequence Orthology · `ISA` Sequence Alignment · `ISM` Sequence Model · `IGC` Genomic Context · `RCA` Reviewed Computational Analysis |
| **Author statement** | `TAS` Traceable Author Statement · `NAS` Non-traceable Author Statement |
| **Curatorial** | `IC` Inferred by Curator · `ND` No biological Data available |
| **Electronic (uncurated)** | `IEA` Inferred from Electronic Annotation |

(`EXP` is the parent of `IDA`/`IPI`/`IMP`/`IGI`/`IEP`. `IEA` is the only code
**not** assigned by a curator — it is the machine-inferred bucket and dominates
GO by volume.)

### 4.3 UniProt evidence categories (4 legacy + ECO)

UniProt tags every annotation with an ECO term and, historically, one of four
categories: **Experimental** (experimental support), **Probable** (curator-inferred
with good confidence), **By similarity** (`ECO:0000250`, inferred from high
sequence similarity), **Potential** (predicted by a sequence-analysis tool).
Each annotation also stores the source (PMID, the curator, or the prediction tool).

### 4.4 Domain-specific evidence/confidence scales (for context)

Several resources ship their own ordinal evidence ladders, which KG builders map
into `knowledge_level` + `has_confidence_score`:

| Resource | Scale | Levels (high → low) |
|----------|-------|---------------------|
| **ClinVar** (variant pathogenicity) | review status (★) + ACMG class | 4★ practice guideline → 3★ expert panel → 2★ multiple submitters, criteria → 1★ single submitter → 0★ no criteria; classes: `Pathogenic`, `Likely pathogenic`, `Uncertain significance`, `Likely benign`, `Benign` |
| **PharmGKB** (clinical annotation) | Level of Evidence | `1A` → `1B` → `2A` → `2B` → `3` → `4` |
| **DisGeNET** | numeric `score` (0–1) + indices | `score`, `EI` (Evidence Index, contradiction flag), `DSI` (Disease Specificity Index), `DPI` (Disease Pleiotropy Index) |
| **CPIC / DPWG** | guideline strength | strong / moderate / optional / no recommendation |
| **OpenTargets** | association/target-disease `score` (0–1) | per-datatype scores aggregated to an overall score |

---

## 5. Statistical / quantitative edge attributes

These are the numeric properties biomedical edges carry when they come from a
statistical analysis (GWAS, DE, eQTL, survival, enrichment, correlation). Listed
with the resource/format field names that supply them.

### 5.1 Significance

| Attribute | Field names seen in the wild | Biolink slot |
|-----------|------------------------------|--------------|
| p-value (raw) | `p_value`, `pvalue`, `p-value`, GWAS `p_value` | `p_value` |
| adjusted / corrected p-value | `adjusted_p_value`, `padj`, `FDR`, `q_value`, `BH`, `Bonferroni` | `adjusted_p_value` |
| −log10(p) | `neg_log10_p`, `log10p` | (custom attribute) |
| significance threshold flag | `genome_wide_significant` (p < 5×10⁻⁸) | (custom) |

### 5.2 Effect size & direction

| Attribute | Field names | Domain |
|-----------|-------------|--------|
| beta / regression coefficient | `beta`, `effect_size`, `coefficient` | GWAS continuous traits, eQTL, regression |
| odds ratio | `odds_ratio`, `OR` | case-control / GWAS binary traits |
| hazard ratio | `hazard_ratio`, `HR` | survival / Cox models |
| relative risk / risk ratio | `relative_risk`, `RR` | cohort studies |
| standardized mean difference | `SMD`, `Cohen_d`, `Hedges_g` | meta-analysis |
| log2 fold-change | `log2FoldChange`, `logFC`, `fold_change`, `FC` | differential expression (RNA-seq/microarray) |
| standard error | `standard_error`, `SE`, `se`, `standard_error` | accompanies every estimate above |

### 5.3 Uncertainty intervals

| Attribute | Field names |
|-----------|-------------|
| confidence interval bounds | `ci_lower` / `ci_upper`, `lower_CI` / `upper_CI`, `95%_CI` |
| credible interval | `credible_interval` |

### 5.4 Sample / cohort sizing

| Attribute | Field names |
|-----------|-------------|
| sample size | `sample_size`, `N`, `n_cases` / `n_controls`, `initial_sample_size` (GWAS) |
| number of studies / cohorts | `n_studies`, `evidence_count`, `replication_count` |
| allele/effect frequency | `effect_allele_frequency`, `MAF`, `risk_allele_frequency` |
| has_count / has_total / has_quotient / has_percentage | Biolink FrequencyQuantifier (§2.5) |

### 5.5 Correlation / similarity

| Attribute | Field names | Domain |
|-----------|-------------|--------|
| Pearson r / Spearman ρ | `correlation_coefficient`, `r`, `rho`, `pearson_r`, `spearman` | co-expression, covariation (Hetionet `GcG`) |
| R² / variance explained | `r_squared`, `variance_explained` | regression, heritability |
| mutual information / Jaccard / cosine | `mutual_information`, `jaccard`, `cosine_similarity` | network / similarity edges |
| z-score | `z_score`, `zscore` | enrichment / standardized effect |
| enrichment score / NES | `enrichment_score`, `NES`, `combined_score` (STRING) | GSEA, STRING PPI |

### 5.6 Worked example — GWAS Catalog edge fields

A GWAS Catalog `variant–associated_with–trait` edge typically carries:
`chromosome`, `base_pair_location`, `effect_allele`, `other_allele`,
`effect_allele_frequency`, `p_value`, `odds_ratio`, `beta`, `standard_error`,
`ci_lower`, `ci_upper`, `risk_allele_frequency`, `initial_sample_size`,
`replication_sample_size`, `mapped_trait` (EFO), `study_accession`, `pubmed_id`.

---

## 6. Qualifiers — species, context, condition, aspect, direction

Biolink's **qualifier** slots refine the *meaning/context* of an edge without
changing the predicate. These are how a KG says "in human", "in liver", "of the
mutant form", "increased activity", "in females", "rare". Grouped:

### 6.1 Context qualifiers (species / anatomy / condition)

| Qualifier slot | Range | Meaning |
|----------------|-------|---------|
| `species_context_qualifier` | OrganismTaxon (`NCBITaxon:`) | taxonomic context the relationship holds in |
| `anatomical_context_qualifier` | AnatomicalEntity (`UBERON:`/`CL:`) | tissue / cell type / subcellular location |
| `context_qualifier` | OntologyClass | general condition/context |
| `qualified_predicate` | uriorcurie | the actual predicate when aspect/direction qualifiers are present (e.g. base predicate `affects` + qualifiers → `causes increased activity of`) |
| `causal_mechanism_qualifier` | CausalMechanismQualifierEnum | mechanism (e.g. activation, inhibition, agonism, antagonism) |

### 6.2 Subject/object aspect & direction qualifiers (the chemical–gene model)

For statements like "drug X increases the **activity** of protein Y", Biolink
splits the meaning into an **aspect** and a **direction** on subject and object:

| Slot | Range |
|------|-------|
| `subject_aspect_qualifier` / `object_aspect_qualifier` | `GeneOrGeneProductOrChemicalEntityAspectEnum` |
| `subject_direction_qualifier` / `object_direction_qualifier` | `DirectionQualifierEnum` |
| `subject_form_or_variant_qualifier` / `object_form_or_variant_qualifier` | FormOrVariant enum (e.g. `mutant_form`, `wild_type`, `polymorphic_form`) |
| `subject_part_qualifier` / `object_part_qualifier` | part of the entity affected |
| `subject_derivative_qualifier` / `object_derivative_qualifier` | a derivative (metabolite, etc.) |
| `subject_context_qualifier` / `object_context_qualifier` | endpoint-specific context |
| `specialization_qualifier` | a more specific sub-type |

**`DirectionQualifierEnum`** (4 values): `increased`, `upregulated`
(under increased), `decreased`, `downregulated` (under decreased).

**`GeneOrGeneProductOrChemicalEntityAspectEnum`** (57 values — the affected
biological aspect). Top-level/common ones first, then the
molecular-modification leaves:

`activity_or_abundance` · `abundance` · `activity` · `expression` · `synthesis` ·
`degradation` · `cleavage` · `hydrolysis` · `metabolic_processing` ·
`mutation_rate` · `stability` · `folding` · `localization` · `transport` ·
`absorption` · `aggregation` · `interaction` · `release` · `isomerization` ·
`secretion` · `uptake` · `splicing` · `molecular_interaction` ·
`guanyl_nucleotide_exchange` · `adenyl_nucleotide_exchange` ·
`molecular_modification` · `acetylation` · `acylation` · `alkylation` ·
`amination` · `carbamoylation` · `ethylation` · `glutathionylation` ·
`glycation` · `glycosylation` · `glucuronidation` · `n_linked_glycosylation` ·
`o_linked_glycosylation` · `hydroxylation` · `lipidation` · `farnesylation` ·
`geranoylation` · `myristoylation` · `palmitoylation` · `prenylation` ·
`methylation` · `nitrosation` · `nucleotidylation` · `phosphorylation` ·
`ribosylation` · `ADP-ribosylation` · `sulfation` · `sumoylation` ·
`ubiquitination` · `oxidation` · `reduction` · `carboxylation`.

### 6.3 Clinical / phenotypic qualifiers (disease ↔ phenotype edges)

| Slot | Range | Meaning |
|------|-------|---------|
| `frequency_qualifier` | FrequencyValue (HP frequency terms / `has_count`…`has_percentage`) | how often the phenotype occurs in the subject |
| `severity_qualifier` *(deprecated)* | SeverityValue | how severe (e.g. `HP:0012824` Severity terms: mild/moderate/severe/profound) |
| `onset_qualifier` | Onset | when the phenotype appears (HP onset terms: congenital, neonatal, juvenile, adult…) |
| `sex_qualifier` | BiologicalSex | sex specificity of the association |
| `clinical_modifier_qualifier` | ClinicalModifier | HP `Clinical modifier` axis (laterality, pace, triggers…) |
| `stage_qualifier` | LifeStage | developmental/life stage |
| `temporal_context_qualifier` | TimeType | time window |

These map directly to the **HPO modifier subontologies** in
`taxonomy-ontologies.md` §7 (Mode of inheritance, Clinical modifier, Frequency).

---

## 7. Synthesis — recommended required-vs-optional ATTRIBUTE design for BioKF

Combining the prior art (Biolink's 2 required slots, ECO, TRAPI provenance,
the statistical conventions), a defensible split for BioKF:

### 7.1 Node attributes

| Tier | Attribute | Source/precedent |
|------|-----------|------------------|
| **Required** | `id` (CURIE), `category` (node type), `name` | Biolink NamedThing |
| **Strongly recommended** | `xref`/equivalent IDs, `provided_by`, `in_taxon` (for species-bearing types) | Biolink |
| **Optional** | `description`, `synonym`, `iri`, `deprecated`, domain-specific properties | Biolink |

### 7.2 Edge attributes

| Tier | Attribute | Precedent / rationale |
|------|-----------|-----------------------|
| **Required** | `subject`, `predicate`, `object` | the triple |
| **Required** | `knowledge_level` (`KnowledgeLevelEnum`, 7 values) | Biolink makes this required |
| **Required** | `agent_type` (`AgentTypeEnum`, 8 values) | Biolink makes this required |
| **Required** | `primary_knowledge_source` (exactly one, `infores:`) | TRAPI rule: one primary source per edge |
| **Strongly recommended** | `publications` [] (PMID/DOI/NCT), `aggregator_knowledge_source` [], `provided_by` | provenance/traceability |
| **Recommended when applicable** | `has_evidence_of_type` (ECO), GO evidence code, `negated` | evidence axis |
| **Optional (analysis edges)** | `p_value`, `adjusted_p_value`, effect size (`beta`/`OR`/`HR`/`logFC`), `ci_lower`/`ci_upper`, `sample_size`, `correlation_coefficient`, `has_confidence_score` | statistical axis |
| **Optional (context)** | `species_context_qualifier`, `anatomical_context_qualifier`, aspect/direction qualifiers, `sex_qualifier`, `frequency_qualifier` | qualifier axis |
| **Housekeeping** | `original_subject`/`original_object`/`original_predicate`, `update_date` | ID-mapping provenance |

The clean rule of thumb mirrored from Biolink: **require the provenance-of-reasoning
triplet (`knowledge_level`, `agent_type`, `primary_knowledge_source`)** so every
edge is at minimum attributable and interpretable; make the **evidence detail,
statistics, and qualifiers optional** so simple curated edges aren't over-burdened
while analysis-derived edges can carry full quantitative detail.

---

## Sources

**Biolink Model (master edge-attribute schema, provenance, qualifiers):**
- Association class — https://biolink.github.io/biolink-model/Association/
- KnowledgeLevelEnum — https://biolink.github.io/biolink-model/KnowledgeLevelEnum/ ; slot https://biolink.github.io/biolink-model/knowledge_level/
- AgentTypeEnum — https://biolink.github.io/biolink-model/AgentTypeEnum/ ; slot https://biolink.github.io/biolink-model/agent_type/
- ResourceRoleEnum / RetrievalSource — https://biolink.github.io/biolink-model/ResourceRoleEnum/ ; https://biolink.github.io/biolink-model/RetrievalSource/
- DirectionQualifierEnum — https://biolink.github.io/biolink-model/DirectionQualifierEnum/
- GeneOrGeneProductOrChemicalEntityAspectEnum — https://biolink.github.io/biolink-model/GeneOrGeneProductOrChemicalEntityAspectEnum/
- FrequencyQuantifier (has_count/total/quotient/percentage) — https://biolink.github.io/biolink-model/FrequencyQuantifier/
- anatomical_context_qualifier — https://biolink.github.io/biolink-model/anatomical_context_qualifier/
- Publication class — https://biolink.github.io/biolink-model/Publication/
- Association examples with qualifiers — https://biolink.github.io/biolink-model/association-examples-with-qualifiers/
- Biolink paper (Unni et al., CTS 2022) — https://ascpt.onlinelibrary.wiley.com/doi/10.1111/cts.13302 ; preprint https://arxiv.org/pdf/2203.13906
- Biolink prefix map — https://github.com/biolink/biolink-model/blob/master/project/prefixmap/biolink_model_prefix_map.json

**TRAPI / NCATS Translator (knowledge-source provenance):**
- ReasonerAPI (TRAPI) — https://github.com/NCATSTranslator/ReasonerAPI
- Translator technical docs — https://ncatstranslator.github.io/TranslatorTechnicalDocumentation/

**Evidence ontologies & codes:**
- ECO home / OBO — https://obofoundry.org/ontology/eco.html ; repo https://github.com/evidenceontology/evidenceontology
- ECO 2022 update (NAR) — https://academic.oup.com/nar/article/50/D1/D1515/6431816 ; PMC https://pmc.ncbi.nlm.nih.gov/articles/PMC8728134/
- ECO community-standard paper — https://www.ncbi.nlm.nih.gov/pmc/articles/PMC6323956/
- ECO root (OLS4 / QuickGO) — https://www.ebi.ac.uk/ols4/ontologies/eco ; https://www.ebi.ac.uk/QuickGO/services/ontology/eco/terms/ECO:0000000/children
- GO Evidence Codes guide — https://geneontology.org/docs/guide-go-evidence-codes/ ; legacy http://www-legacy.geneontology.org/GO.evidence.shtml
- GAF format — https://geneontology.org/docs/go-annotation-file-gaf-format-2.1/
- UniProt evidence — https://www.ebi.ac.uk/training/online/courses/uniprot-exploring-protein-sequence-and-functional-info/where-does-the-data-come-from/data-evidence/

**Identifiers / CURIEs:**
- Bioregistry (metaregistry) — https://bioregistry.io ; paper https://www.nature.com/articles/s41597-022-01807-3
- identifiers.org registry API — https://registry.api.identifiers.org
- UniProt accession format — https://www.uniprot.org/help/accession_numbers
- HGNC (genenames.org) — https://www.genenames.org/help/faq/
- ChEBI — https://www.ebi.ac.uk/chebi/ ; bioregistry https://bioregistry.io/chebi
- MONDO — https://mondo.monarchinitiative.org/ ; DOID — https://obofoundry.org/ontology/doid.html
- PubChem CID — https://pubchem.ncbi.nlm.nih.gov/docs/

**Statistical attributes & domain evidence scales:**
- GWAS Catalog summary-statistics fields (GWAS-SSF) — https://ebispot.github.io/gwas-blog/gwas-ssf-release/
- DisGeNET score / DSI / DPI / EI — https://academic.oup.com/nar/article/45/D1/D833/2290909 ; https://blog.disgenet.com/disease-specificity-index-dsi-disease-pleiotropy-index-dpi/
- PharmGKB Levels of Evidence — https://blog.clinpgx.org/pharmgkb-clinical-annotations-update/ ; https://pmc.ncbi.nlm.nih.gov/articles/PMC8457105/
- ClinVar clinical significance / review status — https://www.ncbi.nlm.nih.gov/clinvar/docs/clinsig/ ; https://www.ncbi.nlm.nih.gov/clinvar/docs/review_guidelines/
</content>
</invoke>
