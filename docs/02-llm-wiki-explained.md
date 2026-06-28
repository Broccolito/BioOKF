# Understanding the LLM Wiki (and what BioOKF specializes)

> Andrej Karpathy's *LLM Wiki* is the seed idea behind OKF and BioOKF. This doc explains
> what it is, the universal steps it defines, why it works, and which steps BioOKF keeps
> verbatim vs specializes for biomedicine. Source notes:
> [../research/llm-wiki.md](../research/llm-wiki.md).
> Original gist: <https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f>.

## 1. The core idea

> *"Incrementally build and maintain a persistent wiki, a structured, interlinked
> collection of Markdown files that sits between you and the raw sources."*

Instead of RAG (retrieve-augment-generate), where the LLM **re-discovers connections on
every query**, the LLM Wiki has the model build a **persistent, compounding artifact**. The
key line: *"the cross-references are already there; the contradictions have already been
flagged."* The work of understanding is done **once** and **kept**.

Why now: the tedious part of maintaining a knowledge base "is not the reading or the
thinking, it's the **bookkeeping**." LLMs are extraordinarily good at exactly that
bookkeeping (cross-referencing, summarizing, touching many files in one pass), so the
cost of keeping a wiki current drops toward zero.

## 2. The three-layer architecture (universal)

| Layer | What it holds | Who owns it |
|---|---|---|
| **Raw sources** | Immutable source documents (articles, papers, images, data files). The source of truth, never modified. | The human curates *what* enters. |
| **Wiki** | LLM-generated Markdown: summaries, **entity pages**, **concept pages**, comparisons, overviews, fully cross-referenced. | The **LLM** owns this entirely. |
| **Schema** | A config doc (à la `CLAUDE.md`) describing the wiki's structure, conventions, and the workflows to follow. | The human + LLM agree on it. |

BioOKF maps these to `raw/`, `knowledge/`, and `schema.md` respectively
([SPEC.md §2](../SPEC.md#2-relationship-to-okf-and-to-the-llm-wiki)).

## 3. The three operations (universal)

### Ingest
Add a source → the LLM **reads it, discusses takeaways, writes summary pages, updates the
index, revises relevant entity & concept pages, and logs the activity.** *A single source
typically affects **10-15 wiki pages.*** This multi-file fan-out is the whole point: the
human never does the bookkeeping.

### Query
Ask a question → the LLM **searches relevant pages, synthesizes a cited answer, and can
emit varied outputs** (tables, slides, charts). Crucially: *"good answers can be filed back
into the wiki as new pages,"* so querying also grows the wiki.

### Lint / maintenance
Periodic health check identifying **contradictions, stale claims, orphan pages, missing
cross-references, and data gaps** to investigate.

### Navigation infrastructure
- **`index.md`**: a content catalog listing every page with a one-line summary, organized
  by category. Read first when answering. Works well at moderate scale (~100 sources,
  hundreds of pages).
- **`log.md`**: an append-only, dated, parseable record of ingests/queries/maintenance.

## 4. Division of labor

| Human | LLM |
|---|---|
| Curate sources, direct analysis, ask insightful questions, interpret results | **All** bookkeeping: summarization, cross-referencing, consistency, contradiction-flagging, multi-file updates |

## 5. What is intentionally left open

Karpathy frames the pattern as **modular and abstract**: directory structure, schema
conventions, page formats, and tooling **vary by domain and preference**. You instantiate a
concrete version with your agent. OKF made one such instantiation (generic, open `type`).
**BioOKF makes the biomedical one.**

## 6. Universal vs BioOKF-specific

| Universal LLM-Wiki step | Kept verbatim in BioOKF? | BioOKF specialization |
|---|---|---|
| Raw / Wiki / Schema layers | ✅ | `raw/` · `knowledge/` · `schema.md` |
| Ingest: read → summarize → write/update pages → update index → log | ✅ | each page is one of **20 typed biomedical nodes**; a `Publication`/`Study`/`Dataset` node is created for the source; every claim gets a `reported_in` provenance edge |
| Entity pages + concept pages | ✅ (becomes the node) | entity = a typed node; its `type` is controlled |
| Cross-references between pages | ⛌ → **typed** | links become **23 typed, attributed edges** with domain/range and a provenance triplet |
| `index.md` catalog + `log.md` history | ✅ | unchanged |
| Query: search → synthesize → cite → (file back) | ✅ | **graph-shaped** queries over typed edges; filter by `knowledge_level` |
| Lint: contradictions / staleness / orphans | ✅ + **more** | also validates `type`/`predicate`, CURIE resolution, and edge domain/range |
| "Capture quantitative facts" | implicit | **first-class**: p-value, OR/HR, IC50, fold-change live as named edge attributes |

## 7. Why this matters for an AI agent's memory

A BioOKF bundle is **persistent agent memory with structure**. Because entities are typed
and relationships are typed + provenance-stamped, an agent can:
- recall *what it knows about a subject* by reading a few typed pages instead of
  re-embedding a corpus;
- reason over a **graph** ("drugs that `treat` diseases `associated_with` gene X");
- **trust-rank** its own memory (curated assertion vs tweet) via `knowledge_level`;
- improve monotonically: each ingested source *compounds* the graph rather than adding
  another opaque chunk to a vector store.

This is the line of value that runs from Karpathy's gist → OKF → **BioOKF for biomedicine**.
