# BioOKF Ingestion Workflow

> **What this is.** Step-by-step instructions for an LLM agent ingesting a single source into a
> BioOKF bundle: distilling it into typed concept nodes and typed, provenance-stamped edges, and
> keeping the bundle's catalog and history current. One run = one source. A single source
> typically creates or touches **10 to 15 concept pages**, and that bookkeeping is the agent's job, not
> the human's.
>
> **Primary references (read these first; this workflow does not restate them):**
> - [`schema.md`](../Repo/schema.md), the operating doc: the 28 node types, the 24 edge
>   predicates, required fields, domain/range notes, provenance rules. **Adhere to everything in it.**
> - [`docs/04-classification_guidelines.md`](../Repo/docs/04-classification_guidelines.md), the
>   classification tiebreakers (TB-1 disambiguate-by-use, TB-2 identity-over-role, TB-3
>   subtype-for-specificity) and the Disease/Phenotype/BiomedicalMeasure trio. **Consult before
>   deciding any ambiguous type.**
> - [`SPEC.md`](../Repo/SPEC.md), the normative format when the operating doc is silent.
>
> **Note on the predicate set.** Follow `schema.md`: **24** positive predicates (the v0.1 to v0.4 core
> of 23 plus `used_to_study`) **plus 11 `not_<X>` negatives** for the negatable effect predicates
> (**35 total**). `SPEC.md` §6 still enumerates the v0.4 core; **`schema.md` is authoritative for
> this workflow** (it is the implemented set in `bokf-core`).

## Operating principles (apply throughout)

- You **own** `knowledge/`, `index.md`, and `log.md`. **Never edit anything in `raw/`.**
- **Type by identity, not role** (TB-2). Aspirin *is* a `Molecule`; "treats headache" is an edge.
- **Only `type` and `identifier` are mandatory on a node**; `predicate`, `object`, and the
  provenance triplet (`knowledge_level`, `agent_type`, `primary_source`) are mandatory on every
  edge. Always also coin a `subtype` (agent-coined; no controlled set).
- **Mint the right node/edge; never bend** a concept onto the nearest existing node just because
  the correct one is missing. The absence of a node is a reason to create it (docs/04).
- **Source-only.** Every claim, edge, and `xref` comes from **the source document plus your own
  knowledge** of what the named entity is. Do **not** call external tools (PubMed, proto-OKN, web)
  to mine new claims or resolve identifiers during ingestion. An `xref` you cannot fill from the
  source is left for a later enrichment/lint pass (a missing `xref` is never a conformance error).

---

## Step 1: Process the source

> ⚠️ **Stub: tooling to be finalized.** Convert the source to faithful Markdown so the later
> steps can read it.

1. Save the source into `raw/` **unchanged**, and note its modality (PDF, image, slide deck, CSV,
   notebook, tweet, …).
2. Produce a faithful Markdown rendering for working use, preserving text, tables, and figure/caption
   content. *(Planned toolchain: `markitdown` for born-digital documents/Office/PDF; an OCR pass for
   scanned or image sources. None are installed yet; fill in here.)*
3. If the source is already provided as Markdown (e.g. a pre-converted paper), use it as-is.

**Output of Step 1:** the original bytes in `raw/`, and a readable Markdown rendering to extract from.

---

## Step 2: Extract entities (nodes)

### 2.0 Create the source node first

Before extracting entities, create the node **for the source itself** (so every claim has somewhere
to point):

- Type it by identity: a document/artifact (paper, preprint, slide, tweet, bench note) → `Publication`;
  a designed investigation → `Study`; a data file/matrix/collection → `Dataset`.
- Because you just placed it in `raw/`, it is an **ingested-document** source: give it a
  **`raw_source`** listing its `raw/…` path(s). Coin a `subtype` (`article`, `preprint`, `rct`, …).
- If the source *cites* an external database/ontology you did **not** ingest (HGNC, GO, DrugBank,
  SemMedDB, a referenced trial), create a lightweight **external-reference** source node **once**
  (no `raw_source`, its `infores:`/registry CURIE in `xref`) and reuse it across every claim it
  supports. Name it canonically (`HGNC`, `Gene Ontology`, `DrugBank`). An identity/naming authority
  → `Agent` (`subtype: organization`); a citable data/knowledge artifact → `Dataset`
  (`subtype: knowledge_base`).

### 2.1 The entity-vs-relationship test (decide what is a node at all)

Create a node only for something with **standalone identity**, a thing you can point at
independent of what it relates to. If the concept is inherently a *relationship between two other
concepts*, it is an **edge** (Step 3), not a node.

- "Interleukin-6", "COVID-19", "tocilizumab", "RECOVERY trial" → **nodes** (they exist on their own).
- "IL6 is elevated in COVID-19", "tocilizumab treats COVID-19", "TCF7L2 predisposes to T2D" →
  **edges** (Step 3), not nodes.
- A measure's **value** ("183 cm", "HR 2.9") is **edge data, never a node**. Variant **consequence**
  (missense, 5′UTR) is an **attribute**, never a node.

### 2.2 For each entity: classify, deduplicate, then write

1. **Classify the `type`** (one of the 28) using the `schema.md` cheatsheet and the `docs/04`
   tiebreakers, especially TB-1 (a word whose referent changes → classify by use here), the
   Disease vs Phenotype vs BiomedicalMeasure trio (one node per facet, linked by edges), and the
   §5.D boundary tests. When genuinely nothing fits, `Other` + a `note:`, but never invent a type.

2. **Check for an existing node before creating one.** Consult `index.md`'s **identifier registry**
   first. Treat it as a candidate match on exact identifier, on a synonym, or on a recognizably-same
   concept; the registry carries the identifier/name/description, so when that is not enough to
   decide, **open the candidate node's `.md`** (its `synonyms`, `xref`, body) and reason about
   whether it is the same entity.
   - **If it already exists:** do **not** create a duplicate. Update the existing node instead:
     add this source to its provenance (a `reported_in` edge to the source node from Step 2.0) and
     **merge in any genuinely new/distinct information** (synonyms, `xref`, a fuller description,
     body detail) **without discarding** what is already there.
   - **If it is new:** create it (next sub-steps).

3. **Choose the `subtype`.** Consult `index.md`'s **subtypes-in-use list** for that `type` and
   **reuse an existing subtype** when one fits, since consistent subtypes are what make subtype-filtered
   querying meaningful. Coin a new lowercase token only when none fits (then it joins the list in
   Step 2.3). Use the coarsest granularity that is still a useful query filter (TB-3 /
   docs/04 "what subtypes are NOT").

4. **Set the `identifier`**, human-readable **and** unique across the bundle; avoid `:` (reserved
   for CURIEs); disambiguate collisions with a parenthetical facet (`IL6 (gene)` vs `IL6 (protein)`).

5. **Curate `xref`** from the source + your knowledge only (source-only rule). What you cannot
   resolve now is left blank for a later pass.

6. **Add the back-link to provenance:** a `reported_in` edge from this node to the source node. (By
   convention a `reported_in` edge's own `primary_source` is its own `object`, the terminating
   base case.)

7. **Write the node** at `knowledge/<type-lowercased>/<slug>.md` (the repo convention, e.g.
   `knowledge/methodorprocedure/…`): YAML frontmatter (typed, queryable) + Markdown body
   (human-readable). Put numbers on edges, not the body.

8. **Update `index.md`:** register the node in the identifier registry and the by-type catalog, and
   register any newly coined subtype in the subtypes-in-use list. *(The index file's exact structure
   is maintained separately; this workflow only requires that the registry and subtypes-in-use list
   stay current.)*

---

## Step 3: Extract relationships (edges)

For each claim or relationship the source states, add a typed `edges:` entry to the **subject**
node. Mostly the same discipline as Step 2, but you are connecting existing nodes, not minting
new ones (create a target node only when the object is a legitimate entity that does not yet exist).

1. **Direction is `subject (host doc) → object`, forward-only.** There are no inverse predicates: to
   express a reverse relation, author the **forward** edge on the *other* node (a gene's `encodes`,
   never a protein's `encoded_by`; `causes`, never `caused_by`).

2. **`predicate`** is one of the **24** (schema.md). **`object`** is the **target node's
   `identifier`** and must resolve to a real node. If the right target does not exist yet, create
   it per Step 2. Never bend the `object` onto a wrong existing node to avoid creating one.

3. **Required attributes:** `knowledge_level`, `agent_type`, and `primary_source` (= a **source
   node's `identifier`**, a `Publication`/`Study`/`Dataset`/`Agent`, never a bare CURIE).
   `direction` is additionally required for `regulates` and `expressed_in`. `not_provided` is a rare
   escape for genuinely unknown origin, never a default.

4. **Numbers are first-class.** When the source gives a statistic (p-value, OR/HR/RR, IC50, Kd,
   fold-change, sensitivity, …), put it on the edge as a named attribute (§7.3), never only in prose.

5. **Edge `subtype` (TB-3).** When the predicate is correct but drops a distinction the source draws
   (e.g. an ecological `associated_with` whose role is `natural_reservoir`), carry it in the edge's
   `subtype`, only when the bare predicate is already correct on its own.

6. **Respect domain/range** (schema.md "Edges: domain/range notes"), e.g. `treats`/`prevents` →
   `Disease`/`Phenotype`; `member_of` → `MolecularClass`; `used_to_study` runs from the investigative
   resource → the entity under study.

7. **Don't duplicate edges.** If the same `predicate` + `object` already exists on the node from a
   different source, **add this source** (extend `publications` / add a parallel provenance-stamped
   edge) and merge in genuinely distinct statistics rather than creating a redundant edge.
   *(Pruning of near-identical edges is otherwise left to a future linting workflow.)*

8. **Update `index.md` / register edge subtypes** as in Step 2.8 if any new edge subtype was coined.

---

## Step 4: Self-review checklist

Verify the run against every reference document, and confirm **no BioOKF principle in `schema.md`
is violated**. Walk this checklist and **fix anything that fails, then re-check**:

**Source & provenance**
- [ ] A node exists for the source itself; an ingested document carries `raw_source` (→ `raw/`), an
      external reference carries its CURIE in `xref` (no `raw_source`).
- [ ] Every node links back to its source(s) via `reported_in`; every edge's `primary_source`
      resolves to a source node (`Publication`/`Study`/`Dataset`/`Agent`).
- [ ] `not_provided` is not used as a default (rare-exception only).

**Nodes**
- [ ] Every `type` is one of the 28; every node has a human-readable, **bundle-unique** `identifier`
      and a `subtype`.
- [ ] **No duplicate nodes**: the same concept was not forked; the identifier registry is consistent.
- [ ] No relationship was modeled as a node, and no measure **value** or variant **consequence**
      became a node.

**Edges**
- [ ] Every `predicate` is one of the 24; every `object` resolves to an existing node's `identifier`.
- [ ] Every edge carries `knowledge_level`, `agent_type`, `primary_source`; `direction` is present
      on `regulates`/`expressed_in`.
- [ ] Domain/range respected; no inverse predicates used (forward edge authored on the right node).
- [ ] Quantitative facts live on edges as named attributes, not only in prose.
- [ ] No duplicate edges introduced (sources merged onto existing edges instead).

**Bookkeeping**
- [ ] `index.md` updated: identifier registry, by-type catalog, and subtypes-in-use list all current.
- [ ] `log.md` has a new `## YYYY-MM-DD` entry summarizing the ingest: source(s) added; counts of
      nodes by type, edges, and source nodes; key claims captured; any unresolved `xref` left to
      backfill.
