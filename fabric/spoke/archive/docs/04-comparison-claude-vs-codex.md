# BioOKF → SPOKE mapping: Claude Code vs Codex — comprehensive comparison

Both agents independently built a BioOKF→SPOKE linker and ran it on the same `benchmark-biored`
bundle (2,550 nodes / 5,360 edges). This compares the two **outputs** node-by-node and edge-by-edge,
adjudicates every disagreement against **live SPOKE** with neutral judge agents, and compares the two
**approaches**.

- Claude = `/Users/wgu/Desktop/BioOKF-spoke-mapper` (branch `codex/spoke-biookf-mapper`)
- Codex = `/Users/wgu/Desktop/BioOKF-spoke-fabric` (branch `codex/spoke-fabric-linker`)

## TL;DR

**On output quality, Claude's mapping is measurably better** — higher coverage *and* higher precision
on every contested set. **On engineering rigor, Codex is stronger** — better reproducibility,
auditing, edge-evidence capture, and safety conservatism. The best system merges Claude's
LLM-adjudication + canonical-name resolution with Codex's synonym-as-identifier matching, audit
tooling, and richer edge evidence.

## 1. Headline numbers

| metric | Claude | Codex |
|---|---|---|
| nodes mapped | **1,460** (57.3%) | 1,236 (48.5%) |
| — of *mappable* | **68.7%** | ~57% |
| match mechanism | deterministic + **398 LLM-adjudicated** | deterministic only (3 review, 0 overrides applied) |
| edges agree | **197** | 158 |
| edges contradict | **14** | 8 |
| not-mappable (by design) | 424 | 406 |

## 2. Overlap (per-node alignment, 2,550 nodes)

- **Both mapped: 1,200** → same SPOKE id **1,115**, *different* id **85** (direct disagreements)
- **Claude-only mapped: 260** (Molecule 80, Disease 66, Phenotype 44, BiologicalFunction 21, …)
- **Codex-only mapped: 36** (Gene 20, Variant 6, Phenotype 5, …)
- neither: 1,054

## 3. Adjudication scorecard (neutral judge agents, grounded in live SPOKE)

**The 85 disagreements** (both mapped, different SPOKE node):

| verdict | count |
|---|---|
| **Claude correct** | **39** |
| Codex correct | 2 |
| both valid (equivalent SPOKE records) | 43 |
| both wrong | 1 |

Where the disagreement is decisive, **Claude is right 39/41 (95%)**. The 43 "both valid" are SPOKE
**duplicate records** — the same molecule under a `CHEBI:` and an `inchikey:` node (amphetamine,
citalopram, bleomycin…), or a pathway as a GO process vs a Pathway node (glycolysis). Neither is wrong.

**The 36 Codex-only maps** (coverage Claude lacked):

| verdict | count |
|---|---|
| genuine win (correct) | 14 |
| **false positive (both wrong)** | **16** |
| defensible | 6 |

**44% of Codex's unique coverage is wrong.** The false positives are all one failure mode: a generic
or ambiguous name force-mapped to one specific gene via an alias — *cyclooxygenase*→COX8A (should be
PTGS), *PI3K*→PIK3CA (a family), *superoxide dismutase*→SOD1 (a family), *Tat*→human TAT gene
(BioOKF means HIV-1 Tat), *NS3*→KRAS, *p17*→POLE3.

**The 50 sampled Claude-only maps** (all LLM-adjudicated coverage):

| verdict | count |
|---|---|
| **correct** | **48** |
| false positive | **0** |
| defensible | 2 |

Claude's LLM tier added coverage at **96% precision with zero false positives** in the sample — it
recovered synonym/lexical-variant matches (adriamycin→Doxorubicin, GPi→medial globus pallidus) while
*refusing* to force ambiguous cases (apoptosis, endoplasmic reticulum → left unmapped).

## 4. The decisive difference: canonical vs alias gene resolution

26 of the 85 disagreements are genes, and Claude wins nearly all. Codex systematically resolved a
gene symbol to a **different gene that merely lists the symbol as an alias**, instead of the canonical
gene whose official symbol matches:

| BioOKF | Claude (canonical) | Codex (alias/paralog) |
|---|---|---|
| AR | 367 androgen receptor | 231 **AKR1B1** |
| NOS1/2/3 | 4842/4843/4846 | 340719/339345/342977 **NANOS1/2/3** |
| CDH1 | 999 cadherin-1 | 51343 **FZR1** |
| FAS | 355 death receptor | 2194 **FASN** |
| C2 | 717 complement C2 | 3183 **HNRNPC** |

Cause: Codex matches `name` and `synonyms` in one query and its tie-break doesn't strongly prefer an
exact **name** hit over a **synonym** hit, so alias collisions win. Claude matches exact name first
(index-backed) and flags multi-hits as ambiguous → it lands on the canonical gene. These mis-maps also
corrupt the affected genes' edges.

## 5. Edge differences

- Claude 197 agrees vs Codex 158. **Claude's 69 unique agreements** come mostly (52) from mapping
  *more nodes* (LLM-mapped endpoints make more edges checkable).
- **Codex's 30 unique agreements** come mostly (23) from a **broader predicate crosswalk**: Codex maps
  `treats` → `{TREATS_, IN_CLINICAL_TRIALS_FOR_, MENTIONED_CLINICAL_TRIALS_FOR_}`, so "capecitabine
  *treats* head-and-neck cancer" counts as agreement off a clinical-trials edge. Claude's `treats` =
  `{TREATS_CtD}` only, and reports those as `spoke_edge_found_but_unmapped_predicate`. **Debatable**:
  "in trials for" ≠ "treats". Neither is strictly better; a distinct *partial/related* status would
  beat both.
- Codex captures **richer per-relationship edge evidence** (source_identifier, target_identifier,
  rel_keys, gwas_pvalue, sources) than Claude (keys + sources).

## 6. Approach / philosophy

| dimension | Codex | Claude |
|---|---|---|
| philosophy | **precision-first, human/LLM-in-the-loop** | **coverage+precision, autonomous LLM-in-the-loop** |
| hard cases | review queue → `curation_overrides.yaml` (verified before apply) | autonomous multi-agent LLM adjudication (ids verified) |
| fuzzy/semantic | none automatic (deferred to a curator) | `fuzzy_accept` tier + LLM |
| code | single 1,760-line `spoke_fabric.py` | modular (normalize/client/resolver/crosswalk/annotate/cli) |
| guard-word terms | **filters/drops** them (loses those nodes) | **splits** them (`'d'+'rop'`, keeps nodes) |
| expensive labels | identifier-only for Protein/Compound/Variant/MiRNA/PanGene | index-guarded name + fulltext |
| synonym→identifier | **yes** — extracts rsIDs from synonyms (wins Variants) | no — only name+xref as id candidates (**gap**) |
| gene resolution | name+synonym combined → alias collisions | exact-name-first → canonical |
| reproducibility/audit | **strong** — `audit-run`, `verify-run`, `--fail-on-query-warning`, topology & report-consistency checks, override verification | moderate — `selftest`, invariant `validate`, id verification |
| both hit the same 2 bugs (guard-word, expensive-label scans) and both fixed them | ✓ | ✓ |

## 7. What each does better — and what to borrow

**Borrow from Codex**
1. **Synonym-as-identifier matching.** Extract rsIDs / CURIEs from a node's *synonyms* and match as
   identifiers. This is exactly how Codex mapped named variants (R620W→rs2476601, Val158Met→rs4680)
   and official-synonym genes (ENT3→SLC29A3, VPS4→VPS4A) that Claude missed. *(Claude gap: id
   candidates were name+xref only, not synonyms.)*
2. **Richer edge-evidence capture** — keep per-relationship source_identifier, sources, and stat
   keys, not just the key list.
3. **Reproducibility & audit tooling** — `audit-run`/`verify-run`, `--fail-on-query-warning` so a
   skipped batch can't silently reduce coverage, topology-preservation checks, and **live
   verification of curated overrides**.
4. **A curation-override file** for human sign-off — the right home for genuinely low-confidence
   decisions that shouldn't be auto-applied.

**Borrow from Claude**
1. **Autonomous LLM adjudication as a tier** (with id verification) — added 260 nodes at 96%
   precision, 0 false positives; the single biggest coverage+correctness lever.
2. **Exact-name-first, canonical gene resolution** with ambiguity flagging — avoids the alias-collision
   errors (AR→AKR1B1) that cost Codex ~26 genes.
3. **Guard-word splitting** (keep the node) instead of dropping it.
4. **Disambiguation discipline** — refuse to map generic/family terms (cyclooxygenase, PI3K) to a
   single node; that discipline is exactly what Codex's deterministic synonym pass lacks.

## 8. Recommended best combined approach

A single waterfall that takes the best of both, in confidence order:

1. **Exact identifier** — from xref *and from synonyms* (borrow Codex: rsID/CURIE-in-synonyms).
2. **Exact canonical name** — name-index first, **name beats synonym**, multi-hit ⇒ ambiguous
   (Claude, fixes the gene bug).
3. **Exact synonym match with a disambiguation guard** — accept only when the synonym maps to a single
   node *and* the term isn't a family/generic (blocks cyclooxygenase→one gene; keeps ENT3→SLC29A3).
4. **LLM adjudication** of the ambiguous/not-found residue (Claude), **id-verified** before apply.
5. **Curation-override file** (Codex) for the low-confidence tail a human should sign off, plus
   auto-apply for high-confidence LLM decisions.
6. **SPOKE-duplicate canonicalization** — collapse `CHEBI:`/`inchikey:` (and GO-vs-Pathway) duplicate
   records so "both valid" disagreements disappear and edges resolve consistently.
7. **Edge layer**: Codex's rich evidence capture + a **new `related_in_spoke`/partial status** for
   family-broadened predicate matches (trials-for, causes-as-phenotype) instead of over-broadening
   `treats`; keep Claude's strict families as the "agrees" bar.
8. **Codex's audit/verify harness + `--fail-on-query-warning`** wrapping the whole thing.

Net: Codex's rigor and synonym/edge techniques as the *chassis*, Claude's LLM adjudication and
canonical-name resolution as the *engine*. Expected result: >70% of-mappable coverage at Claude's
precision, with Codex-grade reproducibility and auditability.
