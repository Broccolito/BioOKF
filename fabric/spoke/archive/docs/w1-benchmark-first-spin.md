# Benchmark BioRED First Spin

Input bundle: `/Users/wgu/Desktop/BioOKF/testing/benchmark-biored`

The BioRED benchmark bundle has 2,550 BioOKF nodes and 5,360 edges. It uses 22 of the 28 BioOKF node types and all 35 predicate forms. The run uses that bundle as read-only input; all generated artifacts are written under this worktree's `fabric/spoke/runs/benchmark-biored`.

## Output Files

- `biookf-export.json` - canonical `bokf export` input snapshot.
- `biookf-graph.json` - canonical `bokf graph` snapshot.
- `spoke-node-map.json` - per-node SPOKE mapping decisions.
- `spoke-annotated-graph.json` - full graph with SPOKE annotations.
- `spoke-mapped-subgraph.json` - graph filtered to mapped nodes.
- `spoke-mapping-report.json` and `spoke-mapping-review.md` - summary and review report.
- `spoke-audit.json` and `spoke-audit.md` - offline verifier output.
- `spoke-review-queue.json` and `spoke-review-queue.md` - LLM/curator queue with override templates.

## Iteration Notes

### Baseline

Initial exact-match mapping completed but was too conservative for Molecule nodes:

- Mapped nodes: 1,011 / 2,550
- Molecule nodes mapped: 0 / 337
- SPOKE-agreeing edges: 99
- Contradictions: 7

### Problems Found And Fixed

1. The SPOKE CLI write guard scans inside string literals. Terms like `set shifting impairment` and `drop in blood pressure` caused read-only Cypher queries to be rejected as if they contained `SET` or `DROP`. The linker now filters lookup terms that trip the current guard.
2. Broad exact-name scans over large labels such as `Protein`, `Compound`, `MiRNA`, and `PanGene` can stall. The linker now requires native identifier-like terms for those labels unless `--include-expensive-name-lookups` is set.
3. A first attempt at Compound synonym rescue hung because `any(s IN n.synonyms ...)` can still scan the large compound label. The default rescue is now exact `Compound.name` only; slower synonym rescue is opt-in via `--compound-rescue-synonyms`.
4. The separate worktree does not carry ignored Rust build artifacts or the `testing/` fixture. The harness resolves `bokf` and the benchmark bundle from the original checkout when needed while keeping generated artifacts in the worktree.
5. The first Protein identifier heuristic was too loose and allowed terms such as `CNSB002` and `RO4368554`, which triggered a slow Protein query. Expensive-label queries are now identifier/xref-only in the main pass, and Protein identifiers use a stricter UniProt accession pattern.
6. Exact Gene-to-Protein rescue initially surfaced a false-positive risk: `cAMP` can uppercase to the `CAMP` gene. The rescue now uses literal BioOKF terms only, blocks known ambiguous molecule acronyms, and keeps Gene-to-Protein mappings as `mapped_review`.
7. The deterministic pass needed a place for semantic/LLM review instead of unsafe fuzzy mapping. The harness now writes a 120-node/120-edge review queue with curation override templates, and `linker/curation_overrides.example.yaml` documents the reviewed override format.
8. Live verification of curated Protein overrides initially used `toString(n.identifier) IN ...`, which can time out on large labels. The override verifier now uses direct property equality for expensive labels such as Protein while keeping flexible identifier checks for smaller labels.

### Current Result

Command:

```bash
fabric/spoke/bin/spoke-map map-bundle \
  --bundle /Users/wgu/Desktop/BioOKF/testing/benchmark-biored \
  --out-dir fabric/spoke/runs/benchmark-biored \
  --refresh \
  --fail-on-query-warning
```

Current summary:

- Mapped nodes: 1,236 / 2,550
- Mapped automatically: 1,233
- Mapped for curator review: 3
- Not found: 908
- Not mappable provenance/context nodes: 406
- Molecule nodes mapped: 225 / 337, including 3 review-level Protein mappings
- SPOKE-agreeing edges: 158
- SPOKE contradictions: 8
- Direct edge not found in SPOKE: 876
- Edges not checked, mostly due to provenance or unmapped endpoints: 4,214
- Query warnings: 0
- Offline audit: pass
- Review queue: 120 node items and 120 edge items
- Curated overrides applied in this run: 0 nodes, 0 edges

Node type highlights:

- Gene: 586 / 619 mapped
- Disease: 177 / 334 mapped
- Molecule: 225 / 337 mapped after Compound name rescue and exact Gene-to-Protein rescue
- Anatomy: 42 / 52 mapped
- CellType: 36 / 48 mapped
- Phenotype: 112 / 240 mapped
- Variant: 30 / 321 mapped, mostly rsID-like terms

Review-level Molecule-to-Protein mappings:

- `CCK` -> `Protein:P06307` (`CCKN_HUMAN`) via `Gene.name = CCK`
- `EGF` -> `Protein:P01133` (`EGF_HUMAN`) via `Gene.name = EGF`
- `Noggin` -> `Protein:Q13253` (`NOGG_HUMAN`) via `Gene.name = NOG`

Contradictions currently include negative BioOKF assertions whose endpoints map to positive SPOKE evidence:

- `GCK not_associated_with type 2 diabetes mellitus`
- `GLP1R not_associated_with type 2 diabetes mellitus`
- `HFE not_associated_with hemochromatosis`
- `HNF4A not_associated_with type 2 diabetes mellitus`
- `RGS4 not_associated_with schizophrenia`
- `TCF1 not_associated_with type 2 diabetes mellitus`
- `xanthine oxidase not_associated_with hypertension`
- `allopurinol not_prevents hypertension`

## Next Improvements

- Add an index-aware or curated synonym path for compounds without reintroducing large scans.
- Add curated identifier enrichment before Disease, Phenotype, and Variant rescue, especially DOID/MONDO/MeSH/OMIM for diseases and rsID/HGVS/ClinVar enrichment for variants.
- Feed the review queue through an actual LLM/curator workflow and write accepted decisions into `linker/curation_overrides.yaml`.
- Expand edge-property extraction for relationship-specific fields beyond common `sources`, `phase`, and `purpose`.
