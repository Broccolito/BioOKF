# Literature-Mining Edge Universe: SemMedDB / SemRep Predicates & PubTator3 Entity/Relation Types

> Reference document for the canonical "edge universe" produced by biomedical literature mining.
> Covers (1) **SemMedDB / SemRep** semantic predications (subject–PREDICATE–object triples over
> UMLS concepts) and (2) **PubTator / PubTator3** entity types and relation types.
> Compiled 2026-06-25 from primary sources (see [Sources](#sources)).

---

## Part 1 — SemMedDB / SemRep Semantic Predications

### 1.1 What it is

**SemRep** is a UMLS-based rule + lexicon NLP system (US National Library of Medicine, Kilicoglu,
Rindflesch et al.) that extracts **three-part semantic predications** from biomedical text in the form:

```
SUBJECT — PREDICATE — OBJECT
```

where the **subject and object are UMLS Metathesaurus concepts** (each carrying a CUI and one or more
UMLS **semantic types**), and the **predicate is drawn from a small controlled vocabulary** loosely based
on (and extending) the relations of the **UMLS Semantic Network**.

**SemMedDB** (Semantic MEDLINE Database) is the PubMed-scale repository of these predications: SemRep is
run over all of PubMed/MEDLINE (titles + abstracts) and the resulting predications are stored in a MySQL
database. As of recent releases it holds well over 100 million predication instances (~10-13 M *unique*
predications), keyed back to the sentence and PMID they came from.

### 1.2 How many predicates?

The phrasing in the literature is slightly inconsistent, so to be precise:

- **~30 predicate types** is the figure most often quoted in summaries.
- The 2020 broad-coverage SemRep paper states: **"In all, 25 relations (excluding ISA and comparative
  predicates) are used in the SemRep ontology."** These 25 are the **associative predicates** below.
- Add **`ISA`** (the single hypernymic/taxonomic predicate) and the **4 comparative predicates**
  (`COMPARED_WITH`, `HIGHER_THAN`, `LOWER_THAN`, `SAME_AS`) → the full positive set.
- At the database level SemRep emits roughly **~58 distinct predicate strings**, because **every predicate
  has a negated counterpart** prefixed `NEG_` (e.g. `NEG_TREATS`, `NEG_CAUSES`) used when the sentence
  asserts the relation does *not* hold. (`PREDICATE` column stores the bare name; `NEG_` flags negation.)

So the "canonical ~30" = **25 associative + ISA + 4 comparative**. The "edge universe" for a knowledge
graph is normally the **25–30 positive predicates** (NEG_* are the negative-polarity duals).

### 1.3 The associative predicates (25) — with verbatim ontological definitions

Definitions below are quoted from the **"Ontological Predicate Definitions"** appendix of Kilicoglu et al.,
*"Constructing a semantic predication gold standard from the biomedical literature"* (BMC Bioinformatics
2011, PMC3281188), which is the authoritative definition source. The "domain → range" column gives the
typical UMLS **semantic group** constraints (CHEM = Chemicals & Drugs, DISO = Disorders, GENE = Genes &
Molecular Sequences, ANAT = Anatomy, PHYS = Physiology, PROC = Procedures, LIVB = Living Beings,
ACTI/PHEN/CONC = Activities/Phenomena/Concepts). These are typical, not strict.

| Predicate | Definition (verbatim) | Typical domain → range |
|---|---|---|
| **TREATS** | "Applies a remedy with the object of effecting a cure or managing a condition." | CHEM/PROC/LIVB → DISO/LIVB |
| **PREVENTS** | "Stops, hinders or eliminates an action or condition." | CHEM/PROC → DISO |
| **DIAGNOSES** | "Distinguishes or identifies the nature or characteristics of." | PROC/CHEM → DISO |
| **ADMINISTERED_TO** | "Given to an entity, when no assertion is made that the substance or procedure is being given as treatment." | CHEM/PROC → LIVB |
| **CAUSES** | "Brings about a condition or an effect. Implied here is that an agent, such as for example, a pharmacologic substance or an organism, has brought about the effect." | CHEM/DISO/GENE/LIVB → DISO/PHYS |
| **PREDISPOSES** | "To be a risk to a disorder, pathology, or condition." | CHEM/DISO/GENE → DISO |
| **AFFECTS** | "Produces a direct effect on. Implied here is the altering or influencing of an existing condition, state, situation, or entity." | CHEM/DISO/PHYS → PHYS/DISO |
| **COMPLICATES** | "Causes to become more severe or complex, or results in adverse effects." | DISO/CHEM → DISO |
| **DISRUPTS** | "Alters or influences an already existing condition, state, or situation. Produces a negative effect on." | CHEM/DISO → PHYS/ANAT |
| **AUGMENTS** | "Expands or stimulates a process." | CHEM/DISO → PHYS |
| **ASSOCIATED_WITH** | "Has a relationship to (gene-disease relation)." | GENE/CHEM → DISO |
| **INTERACTS_WITH** | "Substance interaction." | CHEM/GENE → CHEM/GENE |
| **INHIBITS** | "Decreases, limits, or blocks the action or function of (substance interaction)." | CHEM → CHEM/GENE/PHYS |
| **STIMULATES** | "Increases or facilitates the action or function of (substance interaction)." | CHEM → CHEM/GENE/PHYS |
| **CONVERTS_TO** | "Changes from one form to another (both substances)." | CHEM → CHEM |
| **PRODUCES** | "Brings forth, generates or creates. This includes yields, secretes, emits, biosynthesizes, generates, releases, discharges, and creates." | LIVB/ANAT/CHEM → CHEM |
| **COEXISTS_WITH** | "Occurs together with, or jointly." | any → any (symmetric) |
| **LOCATION_OF** | "The position, site, or region of an entity or the site of a process." | ANAT → DISO/CHEM/PHYS |
| **PART_OF** | "Composes, with one or more other physical units, some larger whole. This includes component of, division of, portion of, fragment of, section of, and layer of." | ANAT/GENE → ANAT/LIVB |
| **PROCESS_OF** | "Disorder occurs in (higher) organism." (process/disorder takes place in an organism) | DISO/PHYS → LIVB |
| **OCCURS_IN** | "Has incidence in a group or population." | DISO/PHYS → LIVB/group |
| **MANIFESTATION_OF** | "That part of a phenomenon which is directly observable or concretely or visibly expressed, or which gives evidence to the underlying process." | PHYS/DISO → DISO/PHYS |
| **PRECEDES** | "Occurs earlier in time. This includes antedates, comes before, is in advance of, predates, and is prior to." | PHEN/PROC/DISO → PHEN/PROC/DISO |
| **METHOD_OF** | "The manner and sequence of events in performing an act or procedure." | PROC → PROC/ACTI |
| **USES** | "Employs in the carrying out of some activity. This includes applies, utilizes, employs, and avails." | LIVB/PROC → CHEM/PROC/DEVI |

> **Note on `MEASURES` / `MEASUREMENT_OF`:** The predicate name in the SemRep ontology is **`MEASURES`**
> ("a procedure/finding measures a quantity or substance"). It appears in the 25-predicate enumeration in
> the 2020 SemRep paper. There is **no `MEASUREMENT_OF`** predicate — that is a common mis-remembering;
> the correct name is `MEASURES`. (Counting `MEASURES`, the associative set is the canonical 25.)

### 1.4 The hierarchical predicate (1)

| Predicate | Definition (verbatim) |
|---|---|
| **ISA** | "The basic hierarchical link in the UMLS Semantic Network. If one item isa another item then the first item is more specific in meaning than the second item." |

### 1.5 The comparative predicates (4)

These come from SemRep's comparative-analysis module (drug/measurement comparison) and are stored in
SemMedDB but are usually filtered out for KG building.

| Predicate | Meaning |
|---|---|
| **COMPARED_WITH** | The two arguments are being explicitly compared. |
| **HIGHER_THAN** | Subject quantity/effect is greater than object. |
| **LOWER_THAN** | Subject quantity/effect is less than object. |
| **SAME_AS** | Subject and object are asserted equivalent. |

### 1.6 Negated duals (`NEG_*`)

Every positive predicate has a negated form indicating the sentence denies the relation:
`NEG_TREATS`, `NEG_PREVENTS`, `NEG_CAUSES`, `NEG_AFFECTS`, `NEG_ASSOCIATED_WITH`, `NEG_INHIBITS`,
`NEG_STIMULATES`, `NEG_PART_OF`, `NEG_PROCESS_OF`, `NEG_COEXISTS_WITH`, `NEG_LOCATION_OF`,
`NEG_ADMINISTERED_TO`, `NEG_INTERACTS_WITH`, `NEG_DISRUPTS`, `NEG_AUGMENTS`, `NEG_PREDISPOSES`,
`NEG_DIAGNOSES`, `NEG_PRODUCES`, `NEG_USES`, `NEG_METHOD_OF`, `NEG_PRECEDES`, `NEG_COMPLICATES`,
`NEG_CONVERTS_TO`, `NEG_MANIFESTATION_OF`, `NEG_OCCURS_IN`, `NEG_MEASURES`, `NEG_ISA`, etc. This is what
brings the distinct predicate-string count in the DB to ~58.

### 1.7 Predicate domains (semantic groupings used in SemRep documentation)

SemRep documentation groups the predicates by application domain:

| Domain group | Predicates |
|---|---|
| **Clinical medicine** | TREATS, PREVENTS, DIAGNOSES, ADMINISTERED_TO, MANIFESTATION_OF, PROCESS_OF, OCCURS_IN, COMPLICATES |
| **Substance / molecular interactions** | INTERACTS_WITH, INHIBITS, STIMULATES, CONVERTS_TO |
| **Genetic etiology of disease** | ASSOCIATED_WITH, CAUSES, PREDISPOSES |
| **Pharmacogenomics / effect** | AFFECTS, AUGMENTS, DISRUPTS, PRODUCES |
| **Anatomy / structure / static** | LOCATION_OF, PART_OF, ISA |
| **Procedure / methodology** | USES, METHOD_OF, MEASURES, DIAGNOSES |
| **Temporal / co-occurrence** | PRECEDES, COEXISTS_WITH |
| **Causal subset (for cause–effect mining)** | AFFECTS, CAUSES, STIMULATES, INHIBITS, DISRUPTS, PRODUCES, PRECEDES, COMPLICATES, PREDISPOSES, PREVENTS |

### 1.8 Node side — UMLS concepts as nodes

SemMedDB nodes are **UMLS Metathesaurus concepts**, each carrying:

- a **CUI** (Concept Unique Identifier, e.g. `C0011849` for Diabetes Mellitus),
- a **preferred name**, and
- one or more **UMLS semantic types** (135 leaf types, e.g. `Disease or Syndrome` (`dsyn`),
  `Pharmacologic Substance` (`phsu`), `Gene or Genome` (`gngm`), `Neoplastic Process` (`neop`),
  `Amino Acid, Peptide, or Protein` (`aapp`), `Body Part, Organ, or Organ Component` (`bpoc`)),
- which roll up into **UMLS semantic groups** (the high-level buckets used in predicate constraints):
  **ANAT** (Anatomy), **CHEM** (Chemicals & Drugs), **DISO** (Disorders), **GENE/GenMolSeq** (Genes &
  Molecular Sequences), **PHYS** (Physiology), **PROC** (Procedures), **LIVB** (Living Beings),
  **ACTI** (Activities & Behaviors), **PHEN** (Phenomena), **OBJC** (Objects), **CONC** (Concepts & Ideas),
  **DEVI** (Devices), **GEOG** (Geographic Areas), **OCCU** (Occupations), **ORGA** (Organizations).

So at the schema level the **"node type" of a SemMedDB node is its UMLS semantic type / semantic group**,
and the **"edge type" is the predicate**.

### 1.9 SemMedDB database schema (edge/node attributes)

| Table | Purpose | Key columns |
|---|---|---|
| **PREDICATION** | One row per unique predication (the edges) | `PREDICATION_ID` (PK), `SENTENCE_ID` (FK), `PMID`, `PREDICATE`, `SUBJECT_CUI`, `SUBJECT_NAME`, `SUBJECT_SEMTYPE`, `SUBJECT_NOVELTY`, `OBJECT_CUI`, `OBJECT_NAME`, `OBJECT_SEMTYPE`, `OBJECT_NOVELTY` |
| **PREDICATION_AUX** | Mention-level provenance for a predication | text spans, subject/object start indices, predicate token, distance between args, indicator type, negation/confidence |
| **SENTENCE** | Source sentences | `SENTENCE_ID`, `PMID`, sentence type (`ti`/`ab`), section header, sentence number, normalized sentence text |
| **CITATIONS** | PubMed metadata | `PMID`, `ISSN`, `DP` (date of publication), `EDAT` (entrez date), `PYEAR` (publication year) |
| **ENTITY** | Recognized concept mentions | `ENTITY_ID`, `SENTENCE_ID`, `CUI`, `NAME`, `SEMTYPE`, `GENE_ID`, `GENE_NAME`, text span, score |
| **GENERIC_CONCEPT** | Overly broad/non-novel concepts (flagged so they can be excluded) | `CUI`, `NAME` |

**Per-edge attributes worth keeping for a KG:** `PREDICATE`, `SUBJECT_CUI/NAME/SEMTYPE`,
`OBJECT_CUI/NAME/SEMTYPE`, `PMID` (provenance), `SUBJECT_NOVELTY` / `OBJECT_NOVELTY` (1 = a novel,
non-generic concept), negation flag (`NEG_*`), and the aux confidence/distance fields.

---

## Part 2 — PubTator / PubTator3 Entity & Relation Types

### 2.1 What it is

**PubTator3** (NCBI/NLM; Wei, Allot, Lu et al., *Nucleic Acids Research* 2024) is an AI-powered annotation
resource over all of PubMed + PMC. It performs **named-entity recognition, entity normalization/linking,
and relation extraction** on the full literature. Where SemMedDB uses UMLS+rules, PubTator3 uses
transformer deep-learning models:

- **AIONER** — unified all-in-one NER, tags the 6 entity types.
- **GNorm2** — gene normalization → NCBI Gene IDs (and species assignment).
- **tmVar3** — variant recognition/normalization → dbSNP rsIDs / HGVS.
- **NLM-Chem tagger** — chemical normalization → MeSH.
- **TaggerOne** — disease + cell-line normalization → MeSH / Cellosaurus.
- **BioREx** — unified relation-extraction model producing the relation edges.

### 2.2 Entity types (6) — nodes

| Entity type | API/bioconcept token | Normalization vocabulary (identifier) |
|---|---|---|
| **Gene** (genes/proteins) | `@GENE_…` | **NCBI Gene** identifiers |
| **Disease** | `@DISEASE_…` (MeSH) | **MeSH** identifiers |
| **Chemical** | `@CHEMICAL_…` (MeSH) | **MeSH** identifiers |
| **Variant** (genetic variant / mutation) | `@VARIANT_…` / `@MUTATION_…` | **dbSNP** rsIDs, or **HGVS** / tmVar notation (and COSMIC for some) |
| **Species** | `@SPECIES_…` | **NCBI Taxonomy** |
| **CellLine** | `@CELLLINE_…` | **Cellosaurus** |

> Lineage note: PubTator 2.0 covered Gene, Disease, Chemical, Species, **Mutation**, and (added later)
> **CellLine**. PubTator3 standardizes the variant type as **Variant** (genetic variant), normalized by
> tmVar3 to dbSNP/HGVS. "Mutation" and "Variant" refer to the same node category across versions.

Nodes carry: the surface text, the entity type, and the **normalized concept ID** (the prefix tokens
above are how you query a specific normalized concept, e.g. `@CHEMICAL_Doxorubicin`,
`@DISEASE_Neoplasms`, `@GENE_…`).

### 2.3 Relation types (13) — edges

PubTator3's BioREx model extracts a controlled set of relation types ("12 common types" in the headline
count; the full machine-name list is **13** including the explicit `mention` plus the 12 semantic ones —
the paper-reported set is enumerated below). Machine names are lowercase tokens.

| Relation type (machine name) | Meaning / description |
|---|---|
| **treat** | Subject (chemical/drug) treats the object condition/disease. |
| **cause** | Subject causes the object (e.g. chemical/variant causes disease). |
| **cotreat** | Co-treatment — two chemicals used together in treatment. |
| **convert** | One entity is converted/metabolized into another (chemical→chemical). |
| **compare** | A comparative relation between the two entities. |
| **interact** | General interaction between the two entities (e.g. gene–gene, chemical–gene). |
| **drug_interact** | Drug–drug interaction (chemical–chemical pharmacological interaction). |
| **inhibit** | Subject inhibits / down-regulates / blocks the object. |
| **stimulate** | Subject stimulates / up-regulates / activates the object. |
| **associate** | General (non-directional) statistical/biological association. |
| **positive_correlate** | Positive correlation between the two entities. |
| **negative_correlate** | Negative correlation between the two entities. |
| **prevent** | Subject prevents the object condition/disease. |

> The directional / functional relations (`treat`, `cause`, `inhibit`, `stimulate`, `prevent`,
> `convert`, `cotreat`, `drug_interact`) plus the correlation/association relations (`associate`,
> `positive_correlate`, `negative_correlate`, `compare`) make up the searchable relation universe.

### 2.4 Entity-type pairs covered by relation extraction (8)

BioREx simultaneously extracts the 12 relation types across **eight entity-type pair combinations**:

| # | Entity pair |
|---|---|
| 1 | **Chemical – Chemical** |
| 2 | **Chemical – Disease** |
| 3 | **Chemical – Gene** |
| 4 | **Chemical – Variant** |
| 5 | **Disease – Gene** |
| 6 | **Disease – Variant** |
| 7 | **Gene – Gene** |
| 8 | **Variant – Variant** |

### 2.5 API / search syntax for relations

PubTator3 exposes relation search via its REST API and web UI. The query-text grammar is:

```
relations:{RELATION}|{ENTITY_ID_1}|{ENTITY_ID_2}
```

- `{RELATION}` is one of the machine names above, or **`ANY`** to match any relation type.
- `{ENTITY_ID}` is a normalized concept token with the `@TYPE_` prefix, **or** a bare type name to mean
  "any entity of this type."

Examples (verbatim form from the docs):

```
relations:treat|@CHEMICAL_Doxorubicin|@DISEASE_Neoplasms        # specific drug → specific disease
relations:ANY|@CHEMICAL_Doxorubicin|@DISEASE_Neoplasms          # any relation between the two concepts
relations:ANY|@CHEMICAL_Doxorubicin|DISEASE                     # any relation to any disease
```

Endpoints: `/research/pubtator3-api/search/?text=…` (search) and the entity-annotation/export endpoints
return BioC-JSON/PubTator format with the entity spans, normalized IDs, and relation records.

### 2.6 Node/edge attributes (PubTator3)

- **Node attributes:** entity type, surface mention text, character offsets, normalized concept ID
  (NCBI Gene / MeSH / dbSNP-HGVS / NCBI Taxonomy / Cellosaurus).
- **Edge attributes:** relation type (one of 13), the two participant entity IDs and their types,
  and provenance (PMID/PMCID + the sentence/passage where the relation was found). Relations are
  document/abstract-level assertions aggregated across the corpus.

---

## Part 3 — Cross-walk: SemMedDB vs PubTator3 (the combined edge universe)

| Aspect | SemMedDB / SemRep | PubTator3 |
|---|---|---|
| Method | UMLS + rules/lexicon | Transformer deep-learning (AIONER, GNorm2, tmVar3, BioREx) |
| Node identity | UMLS CUI + semantic type | NCBI Gene / MeSH / dbSNP / NCBI Tax / Cellosaurus |
| Node typing | 135 UMLS semantic types → 15 semantic groups | 6 entity types |
| # edge types | ~30 (25 associative + ISA + 4 comparative; ~58 with NEG_*) | 13 relation types (across 8 entity-pair combos) |
| Polarity | explicit negation via `NEG_*` predicates | (no first-class negation token) |
| Provenance | PMID + sentence | PMID/PMCID + passage |
| Directionality | predicate-specific (most directional; COEXISTS_WITH/ASSOCIATED_WITH symmetric) | most directional; associate/correlate/compare symmetric |

**Rough predicate ↔ relation alignment** (for KG harmonization):

| SemMedDB predicate | ≈ PubTator3 relation |
|---|---|
| TREATS | treat |
| PREVENTS | prevent |
| CAUSES / PREDISPOSES | cause |
| INHIBITS | inhibit |
| STIMULATES / AUGMENTS | stimulate |
| INTERACTS_WITH | interact / drug_interact |
| CONVERTS_TO | convert |
| ASSOCIATED_WITH / COEXISTS_WITH | associate / positive_correlate / negative_correlate |
| AFFECTS | (no clean 1:1; maps to inhibit/stimulate/associate depending on polarity) |
| ADMINISTERED_TO, LOCATION_OF, PART_OF, PROCESS_OF, MANIFESTATION_OF, DIAGNOSES, METHOD_OF, MEASURES, USES, OCCURS_IN, PRECEDES, COMPLICATES, DISRUPTS, PRODUCES | (no PubTator3 equivalent — SemMedDB is richer on structural/clinical predicates) |
| — | cotreat, compare (no SemMedDB equivalent; comparative handled by COMPARED_WITH/HIGHER_THAN/LOWER_THAN/SAME_AS) |

---

## Sources

- Kilicoglu H. et al. **SemMedDB: a PubMed-scale repository of biomedical semantic predications.**
  *Bioinformatics* 28(23):3158-3160, 2012. https://pmc.ncbi.nlm.nih.gov/articles/PMC3509487/ ·
  https://academic.oup.com/bioinformatics/article/28/23/3158/195282
- Kilicoglu H. et al. **Constructing a semantic predication gold standard from the biomedical literature.**
  *BMC Bioinformatics* 12:486, 2011 (Ontological Predicate Definitions appendix — verbatim definitions).
  https://pmc.ncbi.nlm.nih.gov/articles/PMC3281188/ · https://link.springer.com/article/10.1186/1471-2105-12-486
- Kilicoglu H., Rosemblat G., Fiszman M., Shin D. **Broad-coverage biomedical relation extraction with
  SemRep.** *BMC Bioinformatics* 21:188, 2020 ("25 relations excluding ISA and comparative predicates").
  https://pmc.ncbi.nlm.nih.gov/articles/PMC7222583/ · https://link.springer.com/article/10.1186/s12859-020-3517-7
- **SemMedDB Database Details (schema/version docs).** NLM/LHNCBC.
  https://lhncbc.nlm.nih.gov/temp/SemRep_SemMedDB_SKR/dbinfo.html ·
  https://ii.nlm.nih.gov/SemRep_SemMedDB_SKR/dbinfo20.shtml · https://skr3.nlm.nih.gov/SemMedDB/dbinfo.html
- Wei C-H., Allot A., Lu Z. et al. **PubTator 3.0: an AI-powered literature resource for unlocking
  biomedical knowledge.** *Nucleic Acids Research* 52(W1):W540-W546, 2024.
  https://academic.oup.com/nar/article/52/W1/W540/7640526 · https://arxiv.org/abs/2401.11048 (PDF: https://arxiv.org/pdf/2401.11048)
- **PubTator3 API & Tutorial** (relation search syntax, bioconcept prefixes). NCBI.
  https://www.ncbi.nlm.nih.gov/research/pubtator3/api · https://www.ncbi.nlm.nih.gov/research/pubtator3/tutorial
- **PubTator (KG-Registry entry).** https://kghub.org/kg-registry/resource/pubtator/pubtator.html
- **UMLS Semantic Network** (semantic types/groups for SemMedDB nodes).
  https://www.nlm.nih.gov/research/umls/knowledge_sources/semantic_network/index.html
