# Annotation schema — the generic fabric contract

The fabric adds one annotation object to every node and every edge, plus a top-level report. The
field **vocabulary is KG-agnostic**: a BioOKF→PrimeKG fabric emits the *same* field names (only the
values and the container key differ). The container key is the target KG's short name (`node.spoke`,
`edge.spoke`; a PrimeKG fabric would use `node.primekg`), and every annotation carries `knowledge_graph`
so it is self-describing out of context.

Design rules:
- **Fixed schema** — every field is always present; N/A holes are `null` or `[]`/`""` (no "sometimes
  a key exists, sometimes not"). This makes the output unambiguous and easy to consume.
- **No invented confidence number.** How a node matched is recorded as `match_method`; the harness —
  not the annotation — owns any accept/reject threshold.

## Node annotation (`node.spoke`)

| field | type | meaning |
|---|---|---|
| `knowledge_graph` | string | which knowledge graph this grounds to (`"spoke"`) |
| `mapping_status` | enum | `mapped` · `ambiguous` · `not_found` · `not_mappable` |
| `candidate_types` | list | the target node-types the crosswalk tried, in priority order (`[]` ⇒ not_mappable) |
| `target_type` | string \| null | the matched node's **type** in the target KG (e.g. `Compound`) |
| `target_identifier` | string/int \| null | the matched node's **unique id** (native scheme) — the key deliverable |
| `target_name` | string \| null | the matched node's **name** in the target KG (may differ from the BioOKF name; `null` if none) |
| `match_method` | enum \| null | how it matched: `exact_identifier` · `exact_name` · `synonym` · `fuzzy` · `reviewed` |
| `candidates` | list | for `ambiguous`/`not_found`: alternatives `{target_type, target_identifier, target_name, score, source}` (else `[]`) |
| `notes` | string | mandatory; human-readable explanation, `""` when there's nothing to say |

`mapped` ⇒ `target_type`, `target_identifier`, `match_method` are non-null (enforced by `validate`).
`target_name` can still be null for a mapped node (e.g. a Variant = bare rsID).

### `match_method` values
- `exact_identifier` — a shared identifier / xref matched (highest trust).
- `exact_name` — the canonical name matched (case-insensitive folding is an internal detail).
- `synonym` — a known synonym / alias matched.
- `fuzzy` — an approximate text match cleared the deterministic fuzzy gate (score + token-overlap +
  dominance over the runner-up). **Automatic, no LLM.**
- `reviewed` — an LLM/curator reviewed a case the deterministic tiers couldn't decide and the harness
  accepted it (its confidence cleared `REVIEW_ACCEPT_THRESHOLD`).

`fuzzy` vs `reviewed` never overlap — they are strictly ordered. `fuzzy` fires first and only on a
strong, unambiguous approximate match; anything it can't auto-accept falls to the review queue, and
only then can `reviewed` apply.

## Edge annotation (`edge.spoke`)

| field | type | meaning |
|---|---|---|
| `knowledge_graph` | string | which knowledge graph (`"spoke"`) |
| `support_status` | enum | what the KG says about this assertion: `supported` · `contradicted` · `related` · `different_relation` · `unsupported` · `not_evaluated` |
| `evaluated` | bool | was a verdict computed — both endpoints mapped **and** predicate checkable (`false` ⇒ `not_evaluated`) |
| `endpoints` | object | `{source_identifier, target_identifier}` — the target-KG ids of the two endpoints (`null`s when not evaluated) |
| `expected_relation_types` | list | the target-KG relation types that would confirm this predicate |
| `found_relation_types` | list | the relation types actually found between the two nodes (`[]` ⇒ none) |
| `evidence` | object \| null | the supporting relationship(s) (see below); `null` when nothing found |
| `notes` | string | mandatory; `""` when there's nothing to say |

> **Negation is not stored.** Whether a claim is negative is fully derivable from the edge's own
> `predicate` (the `not_` prefix), so a `negated` field would only duplicate it. `contradicted` already
> implies a negative claim; consumers who need the sign of any edge read `edge.predicate`.

### `support_status` values
- `supported` — a KG relation of the same family as the predicate exists (positive claim confirmed).
- `contradicted` — the claim is negative (`not_*`) yet the KG positively asserts it (`not_associated_with` vs `ASSOCIATES_DaG`).
- `related` — a *related but not identical* relation exists (`treats` backed only by a clinical-trials edge).
- `different_relation` — some relation exists between the nodes, but of a different kind than the predicate.
- `unsupported` — both endpoints mapped, but the KG has no relation between them.
- `not_evaluated` — an endpoint didn't map, or the predicate is provenance-only (nothing to check).

### `evidence` object (present when a relation was found)
```jsonc
{
  "matched_relation_types": ["DOWNREGULATES_OGdG","UPREGULATES_KGuG"], // found types matching the predicate
  "relation_count": 2,                                 // how many KG relationships support the verdict
  "attribute_keys": ["ncpm","sources",...],            // union of property keys on those relationships
  "sources": ["DISEASES"],                             // contributing source databases
  "relations": [                                       // per-relationship snapshot (list capped at 12)
    {"relation_type":"DOWNREGULATES_OGdG","sources":[...],"attribute_keys":[...]}
  ]
}
```
`direction` is intentionally omitted: a relation's orientation is fixed by its type + the endpoint
types, and the two endpoint ids are already recorded in `endpoints`.

## Nodes are resolved 1-to-1; edges are VERIFIED against a set
This is the key modelling asymmetry. A BioOKF **node is an entity** → resolve to exactly one target
node (the best single identity, even when the KG holds duplicates). A BioOKF **edge is an assertion**,
not an entity → there is no identity to bind to, so it is **not mapped 1-to-1 to one KG edge**.
Instead the fabric takes the two resolved endpoints and checks the assertion against **every**
relationship the KG holds between them (a BioOKF predicate can legitimately correspond to several KG
relation types — `regulates` ↔ UP/DOWNREGULATES). It then records **one verdict** (`support_status`)
plus **all** the relevant relationships as `evidence` (hence a list + `relation_count`). Example:
`BACH1 regulates HMOX1` is `supported` by relation_count = 2 (a DOWNREGULATES and an UPREGULATES edge
from different sources); `MYC affects_response_to ibrutinib` cites 5. Verdict semantics are
"any-of": a positive claim is `supported` if ≥1 relation matches its family; a negative claim is
`contradicted` if ≥1 positive relation of that family exists.

## Invariants (enforced by `validate` / `audit`)
1. Shape identity: output node/edge counts == input, order preserved.
2. Every node has `mapping_status`; every edge has `support_status`.
3. `mapping_status == mapped` ⇒ non-null `target_identifier`.
4. `support_status ∈ {supported, contradicted, related, different_relation, unsupported}` ⇒ both endpoints `mapped`.
5. `support_status == contradicted` ⇒ the BioOKF `predicate` is a `not_*` (negative) predicate.
