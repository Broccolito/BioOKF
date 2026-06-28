---
name: biookf-query
description: Use when answering a question from a BioOKF knowledge base: graph-shaped, provenance-cited retrieval over the typed concept docs.
---

# Skill: biookf-query

Answer from the bundle, never from memory. Cite node identifiers and their sources.

## The loop
1. Read `index.md` for orientation (`bokf_read_page`).
2. **Search broadly, then traverse.** `bokf_search` (CLI: `bokf search`) ranks by BM25; it finds
   pages whose *text contains your words*, so it will MISS entities the question doesn't name
   (e.g. searching "drug resistance mechanisms" won't surface `BRAF`). So: issue several searches
   (the disease, the process, key nouns), open the hits, then **follow their edges** to reach the
   entities the question is really about. Don't rely on one search of the whole question string.
3. `bokf_read_page` the top hits; read their `edges:`.
4. Traverse with `bokf_graph` (CLI: `bokf graph`) to find neighbors (graph-shaped reasoning, e.g.
   *"what `treats` a `Disease` `associated_with` this `Gene`?"*). This is where most answers live.
5. Synthesize a **cited** answer: for every claim, name the concept doc and its `primary_source`. Include the numbers from the edges (effect sizes, p-values).
6. For clinical questions, prefer edges with `knowledge_level: knowledge_assertion` or `statistical_association` over `prediction`; say so.
7. If the answer is durable, you MAY file it back as a new `Concept`/note page (with edges + provenance) so the base learns.

## Rules
- If the graph doesn't support a claim, say it's unknown; do not invent.
- Distinguish association from causation by the `predicate` (`associated_with` ≠ `causes`).
- Surface contradictions if `bokf_lint` reports them for the entities involved.
