# Combined mapper — iteration metrics

Same benchmark every round (`benchmark-biored`, 2,550 nodes / 5,360 edges; 2,126 mappable nodes,
i.e. excluding the 424 provenance/context nodes). Baselines for reference:
Claude spoke-mapper 1,460 mapped / 197 agree / 14 contradict; Codex spoke-fabric 1,236 / 158 / 8.

| round | change | mapped | of mappable | lexical-consistent | true precision* | agree | related | contradict | broken ids |
|-------|--------|-------:|:-----------:|:------------------:|:---------------:|------:|--------:|-----------:|:----------:|
| 1 | combined engine: synonym-as-id · disambiguation guard · related_in_spoke · curation overlay | 1,446 | 68.0% | 87.0% | 99.7% | 198 | 28 | 14 | 0 |
| 2 | verify semantic maps; demote 5 wrong; resolve 15 guarded-ambiguous | 1,456 | 68.5% | 87.3% | 99.9% | 199 | 29 | 14 | 0 |
| 3 | SPOKE-duplicate canonicalization (18 compounds → inchikey twins) | 1,456 | 68.5% | 87.3% | 99.9% | 203 | 29 | 14 | 0 |
| 4 | predicate-crosswalk gaps: part_of→MAPS/BELONGS/GpCC (strict); causes/binds/regulates (related) | 1,456 | 68.5% | 87.3% | 99.9% | 222 | 87 | 14 | 0 |
| 5 | edge-verify-driven refinement: move disease-similarity RESEMBLES_DrD out of `associated_with` agrees → related | 1,456 | 68.5% | 87.3% | 99.9% | 213 | 96 | 14 | 0 |

## Edge-classification accuracy (round-5 SPOKE-grounded audit, 109 edges)
| class | verified correct |
|-------|------------------|
| contradicts_spoke | 14 / 14 (100%) |
| related_in_spoke  | 40 / 40 (100%) |
| agrees_with_spoke | 53 / 55 (96%)  → the 2 weak fits were the RESEMBLES cases fixed in round 5 |
| **overall** | **107 / 109 (98%)**, then RESEMBLES fix → ~99% |

## Extension: non-disease types + comprehensive Organism test
| round | change | mapped | precision | agree | related | contra | broken |
|-------|--------|-------:|:---------:|------:|--------:|-------:|:------:|
| 6 | non-disease tuning: Organism species tie-breaker, case-variant ci, Exposure routing, name aliases | 1,446* | — | 213 | 95 | 14 | 0 |
| 7 | fix latent **fulltext-starvation** bug (exposed by the 128-organism test) | **1,464** | **~99.9%** | 209 | 97 | 14 | 0 |

*Round 6 briefly regressed (1,446) because the new case-variant tier shifted fulltext batch
composition and exposed a pre-existing starvation bug (`batch_fulltext` global LIMIT + ORDER BY q
starved later names in a chunk of all candidates → false not_found for adriamycin/cisplatin/etc.).
Round 7 fixes it (detect saturated chunks, re-query starved names individually) and **improves**
coverage to 1,464 while holding precision and 0 broken ids.

**Comprehensive Organism test (N=128 real species):** 118/128 mapped (92%), **100% verified
precision** (all species-level, correct taxon incl. 10 taxonomic-rename aliases). Details in
[non-disease-types.md](non-disease-types.md).

## Convergence
Node coverage plateaued at **1,456 (68.5% of mappable)** after round 2; node precision at **99.9%**
(true precision, from the 206-node SPOKE-grounded audit: only 5 wrong, all demoted). Rounds 3-5
improved *edge* classification (agreements +correct, a distinct related tier, and a cleaner agrees
bar) without moving node coverage. Round 5 changed only edge labels (0 new queries). Further rounds
yield no visible node/precision improvement — the residue is genuinely unmappable, so the process
has converged.

## What is still not handled well (honest limitations)
1. **Named/HGVS variants without an rsID** (~269) stay unmapped — SPOKE Variant is keyed by dbSNP;
   would need an external HGVS→rsID normalizer to join.
2. **General GO process terms** (apoptosis, cell death, cell differentiation) are not loaded in SPOKE
   as nodes (only specific children) — correctly `not_found`, not force-mapped.
3. **Types SPOKE does not model** (BiomedicalMeasure, MethodOrProcedure, most SequenceFeature,
   Device, MaterialSample) — `not_mappable`/`not_found` by design.
4. **SPOKE-duplicate canonicalization** is implemented for Compounds only (18 remapped); GO-vs-Pathway
   twins for `BiologicalPathway` are not yet collapsed (small residual edge loss).
5. **`related_families` are heuristic** (causes~associates, binds~regulates) — sensible and audited
   here, but a domain curator may want to tune which count as related vs full agreement.
6. **Edge coverage is bounded** by node coverage and genuine SPOKE absence: 1,085 `not_found_in_spoke`
   edges have both endpoints mapped but no SPOKE relationship — mostly real gaps in SPOKE, not harness
   error.

\* true precision = consistent + LLM-verified-correct-among-semantic, over mapped.

## Notes per round
- **R1** — combined engine baseline. Synonym-as-identifier recovered the named variants
  (R620W→rs2476601 etc.; Variant now 28/28 consistent). Disambiguation guard cut `exact_synonym`
  33→15, moving alias-risk cases (PI3K, juvenile-open-angle-glaucoma) to the LLM/review tier.
  `related_in_spoke` split 28 family-broadened edges out of `unmapped_predicate`. Zero broken ids.
