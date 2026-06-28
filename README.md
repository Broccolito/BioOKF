# BioOKF: Biomedical Open Knowledge Format

**BioOKF is a protocol for turning any biomedical source (a paper, preprint, bench note,
slide deck, CSV, figure, or tweet) into a structured, interlinked, version-controlled
knowledge base that compounds over time and can be queried as a graph.**

It is a biomedical *profile* of Google Cloud's [Open Knowledge Format
(OKF)](https://github.com/GoogleCloudPlatform/knowledge-catalog), itself a formalization of
Andrej Karpathy's [LLM Wiki](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f)
pattern. BioOKF keeps OKF's portable substrate (a Git-shippable tree of Markdown + YAML)
and adds the one thing OKF deliberately leaves open: **meaning**.

1. **A closed universe of 28 biomedical node types.** OKF's `type` field is an open
   vocabulary (any string); BioOKF constrains it to a finite, exhaustive-but-general set
   covering all of biology, medicine, molecular science, chemistry, and biochemistry. An
   optional, agent-coined `subtype` carries finer granularity without enlarging the controlled set.
2. **A closed universe of 23 typed, attributed edge types.** OKF links are untyped prose;
   BioOKF makes relationships first-class, directional (**forward-only**), domain/range-constrained,
   and able to carry quantitative evidence (p-value, odds ratio, IC50, and so on).
3. **Node-based provenance on every claim.** Each edge's `primary_source` names a **source node**
   (a `Publication`/`Study`/`Dataset`/`Agent`) by its `identifier`; ingested sources anchor to the
   immutable bytes in `raw/` via `raw_source`. So knowledge mined from a tweet is never confused
   with a curated assertion from DrugBank, and every claim is traceable to a node and, ultimately,
   to a file.

The goal: a format an **AI agent or human curator** can follow to distill knowledge and
connections (entities = nodes, relationships = edges) from heterogeneous biomedical
sources, and use the result as **persistent agent memory** and a queryable knowledge graph.

## Start here

| File | What it is |
|---|---|
| **[SPEC.md](SPEC.md)** | **The specification.** Node universe, edge universe, attributes, provenance model, conformance, worked example. |
| [schema.md](schema.md) | The agent-facing operating doc: conventions + the ingest/query/lint workflow an LLM follows. |
| [docs/01-okf-explained.md](docs/01-okf-explained.md) | Understanding of OKF: what it is, what it solves, its components, constraints, flexibility, and how the community uses it. |
| [docs/02-llm-wiki-explained.md](docs/02-llm-wiki-explained.md) | The LLM Wiki pattern: the universal ingest/query/lint loop and what BioOKF specializes. |
| [docs/03-rationale.md](docs/03-rationale.md) | Why these 28 nodes and 23 edges: chain-of-thought design, prior-art mapping (UMLS/Biolink/SPOKE/Hetionet/SemMedDB), and the key decisions. |
| [examples/](examples/) | A small worked BioOKF **v0.5** bundle (typed concept docs + source nodes + `raw/` + `index.md` + `log.md`). |
| [sources/](sources/) | **1,100+ cataloged biomedical sources** across 12 modalities/subfields: the empirical grounding for the type design. See [sources/README.md](sources/README.md). |
| [research/](research/) | The raw research the design is built on: verbatim OKF spec, LLM Wiki, and 8 biomedical node/edge taxonomy references. |

## The two universes at a glance

**28 node types.** *Biomedical entities (20):* `Gene` · `Molecule` · `MolecularClass` ·
`Variant` · `SequenceFeature` · `Structure` · `Anatomy` · `CellType` · `Organism` ·
`BiologicalPathway` · `BiologicalFunction` · `Disease` · `Phenotype` · `BiomedicalMeasure` ·
`MethodOrProcedure` · `Exposure` · `SocialFactor` · `Food` · `Device` · `MaterialSample`.
*Provenance & context (8):* `Publication` · `Study` · `Dataset` · `Agent` · `Population` ·
`GeographicLocation` · `Concept` · `Other`.

**23 edge types.** `is_a` · `part_of` · `member_of` · `derives_from` · `located_in` ·
`expressed_in` · `encodes` · `interacts_with` · `binds` · `regulates` · `catalyzes` ·
`converts_to` · `participates_in` · `causes` · `predisposes_to` · `treats` · `prevents` ·
`contraindicated_in` · `affects_response_to` · `has_phenotype` · `measures` ·
`associated_with` · `reported_in`.

Both universes are **exhaustive** (anchored to the UMLS Semantic Groups that partition
99.5% of biomedicine, plus an `Other` closure) yet **not granular** (umbrella types +
an agent-coined `subtype` give large compression vs UMLS's 127 semantic types).

## Identity & provenance in one line

Every node's primary key is a single human-readable, bundle-unique **`identifier`** (external
ontology CURIEs are optional, living in `xref`). Every edge points subject → object by `identifier`,
carries the required provenance triplet (`knowledge_level`, `agent_type`, `primary_source`), and
`primary_source` names a **source node**, never a bare CURIE.

## How it relates to BioRouter / SPOKE

BioOKF is designed for [BioRouter](https://github.com/BaranziniLab/biorouter)'s Knowledge
feature and is deliberately **round-trippable to SPOKE** (the Baranzini Lab knowledge
graph) and the Biolink Model: every BioOKF node/edge maps to a SPOKE metanode/metaedge and
a Biolink category/predicate, so a curated BioOKF bundle can be projected into a SPOKE-style
graph (and vice-versa) with no remainder.

---

*v0.5 (Draft), 2026-06-27. A community profile of OKF for biomedicine; not an official
Google or UCSF product. See [SPEC.md §14](SPEC.md#14-changelog-and-deprecated-aliases) for the
changelog.*

## Authors

Wanjun Gu (<wanjun.gu@ucsf.edu>), Gianmarco Bellucci, Ilan Ladabaum, James Xue, Jonathan Xue, and Xi Zheng.
