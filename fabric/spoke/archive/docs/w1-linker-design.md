# BioOKF to SPOKE Linker Design

This first-spin linker is deliberately a waterfall. It favors deterministic evidence first, then leaves hard cases in a review queue for an LLM or curator.

## Inputs

- BioOKF bundle or a `bokf export` JSON file.
- Live SPOKE access through `fabric/spoke/spoke-cli`.
- Declarative mapping files:
  - `linker/type_crosswalk.yaml`
  - `linker/predicate_crosswalk.yaml`

## Node Mapping Waterfall

1. Select candidate SPOKE labels from the BioOKF `type`, `subtype`, and xref namespaces.
2. Query SPOKE for exact normalized matches against:
   - BioOKF `identifier`
   - BioOKF `xref` values and CURIE suffixes
   - BioOKF `synonyms`
3. Rank candidates by:
   - exact identifier/xref match
   - exact name match
   - exact synonym match
   - label priority from the crosswalk
4. Assign one best match if it exceeds the confidence threshold.
5. Otherwise mark the node as `not_found` or `not_mappable`.

Expensive SPOKE labels such as `Protein`, `Compound`, `Variant`, `MiRNA`, and
`PanGene` do not use broad name/synonym scans in the main pass. The default
path only uses exact identifiers/xrefs for those labels. Curated rescue passes
handle safe high-value cases:

- `Compound.name` exact rescue for BioOKF `Molecule` nodes. Compound synonym
  array rescue remains opt-in via `--compound-rescue-synonyms`.
- Exact `Gene.name -> ENCODES_GeP -> Protein` rescue for BioOKF `Molecule`
  nodes. These are marked `mapped_review` unless the BioOKF subtype is already
  protein-like. Gene synonym matching is opt-in because short biomedical
  synonyms can be ambiguous.

Semantic and nuanced matches enter through an explicit curation hook rather than
automatic fuzzy matching. Reviewers can copy
`linker/curation_overrides.example.yaml` to `linker/curation_overrides.yaml` and
add:

- `nodes` overrides with exact SPOKE `label` and `identifier`. The harness
  verifies those references against live SPOKE before applying them by default.
- `edges` overrides for reviewed contradictions, predicate-family mismatches, or
  directionality/polarity corrections. Edge overrides retain the same status
  vocabulary as deterministic edge annotations, with added curation provenance.

The output graph preserves every original BioOKF node and adds a `spoke` annotation object. Downstream users can filter to mapped nodes without losing the original graph.

## Edge Mapping Waterfall

1. Only check edges whose source and target nodes are both mapped to SPOKE.
2. Convert the BioOKF predicate to candidate SPOKE relationship prefixes.
3. Query SPOKE for direct or undirected relationships between the mapped nodes.
4. Annotate:
   - `agrees_with_spoke` when a SPOKE relationship matches the predicate family.
   - `spoke_edge_found_but_unmapped_predicate` when an edge exists but does not match the family.
   - `contradicts_spoke` for BioOKF `not_*` predicates when SPOKE has a positive base relationship.
   - `not_found_in_spoke` when no SPOKE relationship is found.
   - `not_checked` when one or both nodes are unmapped or the predicate is provenance-only.

## Guardrails

- `not_*` predicates are not treated as simple absence. They only contradict SPOKE when a positive base relationship exists.
- Provenance/context BioOKF nodes are not forced into biomedical SPOKE nodes.
- Broad BioOKF types such as `Molecule` and `Phenotype` rely on subtype/xref narrowing.
- Every low-confidence or many-candidate match is retained in the review report.
- Query warnings are recorded in the JSON report. Use `--fail-on-query-warning`
  in reproducibility runs so skipped SPOKE batches cannot quietly reduce
  coverage.
- `spoke-map audit-run` and `bin/verify-run` enforce graph topology
  preservation, per-node/per-edge annotations, mapped-subgraph derivation,
  report consistency, review queue derivation, and `.env` safety.
- `spoke-map build-review-queue` creates the LLM/curator worklist from the
  annotated graph. The queue includes override templates but does not apply them
  until a reviewer writes a curation override file.

## Known First-Spin Gaps

- The harness currently uses exact matching, narrow deterministic rescues, and
  explicit curated overrides. Fuzzy/semantic suggestions should feed the review
  queue and override file, not an unguarded automatic mapper.
- SPOKE relationship properties vary by relationship type; the first pass captures common properties and the list of relationship keys.
- Some BioOKF predicates intentionally collapse multiple SPOKE relation families. Those cases are marked with lower confidence in `predicate_crosswalk.yaml`.
- Disease, Phenotype, and Variant coverage need curated identifier enrichment
  before automatic mapping. The current harness intentionally leaves acronym,
  vocabulary-boundary, and gene-context variant cases unresolved.
