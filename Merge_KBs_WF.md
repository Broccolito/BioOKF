# BioOKF Knowledge-Base Merge Workflow

> **What this is.** Step-by-step instructions for an LLM agent merging two existing BioOKF bundles
> into one. The user's primary/pre-existing KB is the **Main KB (MKB)**; the other is the
> **Secondary KB (SKB)**. The SKB is merged **onto** the MKB, so **as much as possible about the
> MKB stays unchanged** — its identifiers, file paths, `raw/` locations, and subtype names all win
> on every collision. The SKB is the side that gets renamed, moved, or collapsed.
>
> **Primary references** (adhere to them, as in ingestion):
> [`schema.md`](../Repo/schema.md) (the operating doc — 28 node types, 24 edge predicates,
> provenance rules) and
> [`docs/04-classification_guidelines.md`](../Repo/docs/04-classification_guidelines.md) (identity
> vs role, the facet rules) for judging when two entities are truly the same concept.
>
> **Note on the predicate set.** As in ingestion, follow `schema.md`'s **24** positive predicates +
> **11** `not_<X>` negatives (**35 total**); `SPEC.md` §6 still lists the v0.4 core — `schema.md` is
> authoritative.

## Governing rules (apply throughout)

- **MKB is canonical.** On any conflict — identifier collision, subtype name, `raw/` path — keep the
  MKB value and change the SKB side.
- **Within-KB entities are never merge candidates.** Only look for matches **across** the two KBs.
  Two distinct entities inside one KB are assumed distinct for valid reasons and must not be
  collapsed.
- **Lose no information, break no links.** Combine rather than discard; whenever an SKB identifier or
  `raw/` path changes (by collapse or rename), every reference to it must be rewritten so nothing
  dangles (see the reference-rewriting note in Step 1).

---

## Step 1 — Identifier Resolution

### 1.1 Find candidate matches across the two KBs

Compare the MKB `index.md` against the SKB `index.md` and surface both **exact** identifier matches
and **semantically similar** matches (only across KBs, never within one).

The index carries the identifier/name and a short human-readable description, which is enough to
*propose* candidates. **When that is not enough to decide, open the actual node `.md` files** and
reason over their `synonyms`, `xref`, and body to judge whether they refer to the same concept.
There is no hardcoded matching rule — use judgment (a shared `xref` CURIE, overlapping synonyms, and
matching descriptions are all strong evidence; reference `docs/04` for identity-vs-role and the
facet rules, e.g. a `Disease` facet and its `Phenotype` facet are **distinct** concepts and must
**not** be collapsed).

### 1.2 Review each candidate, and collapse only true matches

For each candidate pair, decide whether they are **genuinely the same concept**. **If and only if**
they are, collapse the SKB node onto the MKB node:

1. **Retain the MKB identifier.** Concatenate the SKB node's `edges:` into the MKB node.

2. **Collapse the bodies/frontmatter into the MKB node.** Remove redundancies and combine
   information, but **discard nothing**. If the two KBs directly **contradict** each other, **keep
   both** statements, each tagged with its source.

3. **Remove the collapsed node from the SKB `index.md`** (bookkeeping — Step 1.3 depends on what
   remains there).

> **Reference rewriting (required on every collapse).** The SKB identifier is going away, so update
> **every reference to it** to the retained MKB identifier — anywhere it appears as an edge
> `object`, a `primary_source`, or a `reported_in` object, across all SKB files (and any
> already-merged content). Otherwise those edges dangle.

### 1.3 Carry over the SKB nodes that remain

Every node **still** in the SKB `index.md` after Step 1.2 is either (1) a node with no MKB match, or
(2) a candidate that was reviewed and judged a **different** entity. For each:

1. **Append it to the MKB `index.md`**, ensuring **no duplicate identifiers**. On a collision,
   **rename the SKB entity** (never the MKB one) — then **rewrite every reference to the old SKB
   identifier** (edge `object`, `primary_source`, `reported_in` object) to the new name, as above.

2. **Move the SKB node `.md` file into the MKB filesystem**, following the BioOKF layout
   (`knowledge/<type-lowercased>/<slug>.md`).

### 1.4 Source nodes merge by the same logic

Provenance nodes (`Publication`/`Study`/`Dataset`/`Agent`) are ordinary nodes that appear in the
index too, so they go through 1.1–1.3 like any other node. When **collapsing two source nodes** that
represent the same source, keep the MKB identifier and **union** their `raw_source` paths and `xref`
lists; rewrite all `primary_source`/`reported_in` references accordingly.

### 1.5 Merge the `raw/` directory

The sources themselves must travel with their nodes:

1. Move the SKB's `raw/` files into the **MKB's `raw/`** (keep the MKB location). On a filename
   collision, **rename the incoming SKB file** (keep the MKB file) — unless it is a true duplicate of
   an already-collapsed source (Step 1.4), in which case keep the single MKB copy and drop the SKB
   duplicate.
2. **Update `raw_source` paths** on every moved or collapsed source node to the new MKB-relative
   `raw/…` path, so every provenance chain still terminates at a real file in the merged bundle.

> **Edge-level duplicate pruning is out of scope here.** Concatenating edges (Step 1.2) can leave two
> edges with the same `predicate` + `object` after rewriting. That is expected — **do not dedupe
> edges in this workflow**; near-identical edges are pruned by a future **linting workflow**.

---

## Step 2 — Subtype Resolution

Subtypes are agent-coined and have no controlled vocabulary, so the two KBs may name the same
distinction differently. Harmonize them after the nodes are merged:

1. Walk `index.md` and compare the **subtypes in use per `type`** across the SKB-origin and MKB
   nodes.
2. **Merge equivalent-but-differently-named subtypes**, keeping the **MKB** subtype name.
3. When a subtype name changes, **find-and-replace every occurrence of the old token** in the
   affected markdown files — on **both nodes and edges** (edges carry subtypes too, per TB-3 in
   `docs/04`).
4. Update the subtypes-in-use list in `index.md` to the harmonized set.

---

## Step 3 — Log and integrity check

1. **Append a merge report to the MKB `log.md`** under a `## YYYY-MM-DD` heading: the SKB name, how
   many candidate matches were reviewed, how many were collapsed, how many SKB entities were carried
   over (and how many renamed on collision), how many source nodes were merged, how many `raw/` files
   moved/renamed, and which subtypes were harmonized.

2. **Final integrity pass** (fix anything that fails, then re-check):
   - [ ] No duplicate identifiers in the merged `index.md`.
   - [ ] No dangling references — every edge `object`, `primary_source`, and `reported_in` object
         resolves to a node that exists in the merged bundle.
   - [ ] Every source node's `raw_source` points to a real file under the MKB `raw/`.
   - [ ] Subtypes are harmonized (no equivalent-but-differently-named pairs remain).
   - [ ] The MKB's own identifiers, file paths, and `raw/` locations are **unchanged** except where a
         genuine merge required it.
