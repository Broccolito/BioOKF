# Coverage beyond diseases — organisms, bacteria, SDoH, food, exposures

The engine is **type-generic**: every label is driven by [type_crosswalk.yaml](../linker/type_crosswalk.yaml),
not code. To support a type you edit YAML; the waterfall, SPOKE query layer, and fulltext indexes
already work for any SPOKE label. This note records what the deterministic tiers achieve for the
non-disease types and the tuning added for them.

## Tuning added

1. **Per-label tie-breakers** (`label_preferences` in the crosswalk). When an exact-name match returns
   several SPOKE records that share a name, retry with a preferred property. SPOKE stores one
   *species* Organism node plus many *strain* nodes per name, all named identically — so
   `Organism → prefer level=species` resolves each to the correct NCBI taxon (E. coli→562,
   M. tuberculosis→1773, H. sapiens→9606) instead of going to review. Mechanism is generic; add a
   rule for any label with duplicate records.
2. **Case-variant exact match** (`batch_exact_by_name_ci`). Fulltext crowds out common short names —
   SPOKE *has* "Ethanol", but hundreds of "…ethanol" compounds bury it below the candidate cutoff.
   Trying case variants (`ethanol`/`Ethanol`/`ETHANOL`/`Ethanol`) against the case-sensitive name
   index catches them directly. Fixed ethanol, caffeine, folic acid.
3. **Exposure routing** — `behavioral` now routes to `[Compound, Environment]` so chemical exposures
   (alcohol, tobacco, caffeine) resolve to the Compound.

## Validation (30 curated names, DETERMINISTIC tiers only — no LLM)

| type | deterministic coverage | notes |
|------|:----------------------:|-------|
| **Organism — bacteria/pathogens** | **10/10 (100%)** | all resolved to the correct NCBI taxon via the species tie-breaker |
| **Organism — model organisms** | 1/6 (human only) | mouse/rat/fly/zebrafish/yeast are **absent from SPOKE's Organism table** — the engine correctly refuses to force-map (fulltext for "Mus musculus" returns only *Xanthomonas* sp. MUS-060, and it declines it) |
| **SocialFactor → SDoH** | 4/5 (80%) | food/transportation insecurity, impaired mobility, personal care disability all mapped |
| **Molecule** | 4/4 (100%) | incl. folic acid, caffeine, calcium (case-variant fix) |
| **Exposure** | 2/2 (100%) | ethanol, arsenic → Compound |
| **Food** | 1/4 (25%) | granularity mismatch: FOODON is hyper-specific ("green tea leaf (dry)", "green tea (powdered)") with no coarse "green tea" node → an LLM-tier / "prefer most general" case, not a tie-breaker case |

These are the **deterministic floor**. The residue (ambiguous model organisms, "green tea",
"housing insecurity") is exactly what the LLM adjudication tier resolves on the real benchmark at 96%
precision — this validation deliberately ran *without* it to isolate deterministic behaviour.

## Honest limits (SPOKE data, not harness)

- **SPOKE `Organism` = human + microbes/pathogens.** The classic model-organism taxa (10090 mouse,
  10116 rat, 7227 fly, 7955 zebrafish, 559292 yeast) are simply not present. No harness change maps
  what isn't there — and forcing a match would be a false positive, which the guards correctly prevent.
- **SPOKE `Organism` identifiers are `taxon.strain` floats**, not bare taxon ids — resolve organisms
  by name + species-level, never by feeding in a raw NCBI taxon id (the id-verification guardrail
  demotes guessed taxon ids).
- **FOODON granularity** — SPOKE Food is preparation-level; coarse food concepts need semantic
  (LLM) resolution or a "most-general-match" preference.
- A few BioOKF types (`BiomedicalMeasure`, `MethodOrProcedure`, `Device`) have **no SPOKE label at
  all** and remain `not_mappable` by design, regardless of tuning.

## Comprehensive Organism test (N=128 real microbial species)

Ran 128 bacteria/microbes as written in biomedical text through the deterministic engine, then
verified every mapping against live SPOKE (species-level? correct organism?).

| metric | result |
|--------|--------|
| mapped | **118 / 128 (92%)** |
| verified precision | **118 / 118 (100%)** — every mapped organism is species-level (tie-breaker worked) and the correct taxon |
| unmapped | 10 — eukaryotic parasites (Plasmodium vivax, Giardia, Entamoeba, Trypanosoma, Histoplasma, Pneumocystis) genuinely absent from SPOKE + a couple needing more aliases |

Two fixes were needed and made along the way, both surfaced by this larger test:

1. **Name-alias map (taxonomic reclassification).** SPOKE stores **zero** organism synonyms, so old
   scientific names can't be bridged from within SPOKE — yet SPOKE *has* the organisms under their
   **new** names (Lactobacillus rhamnosus → Lacticaseibacillus rhamnosus = taxon 47715; Mycoplasma →
   Mycoplasmoides; Candida glabrata → Nakaseomyces glabratus). Added a `name_aliases` crosswalk
   section (old→canonical name); this is a small stand-in for NCBI Taxonomy's `names.dmp` synonym
   table, which production should load in full. Recovered 10 organisms (84% → 92%).
2. **Fulltext starvation bug (found by the 128-node test, latent all along).** `batch_fulltext`
   used one global `LIMIT` with `ORDER BY q`, so when early-alphabetical names in a chunk flood the
   limit, later names got **zero** candidates → false `not_found` (it silently dropped adriamycin,
   cisplatin, coenzyme Q10 on the disease benchmark depending on chunk composition). Fixed: detect
   saturated chunks and re-query their starved names individually. This is a general correctness fix,
   not organism-specific.

## Takeaway

The approach is not disease-specific. Bacteria/pathogens and SDoH work well now; the tie-breaker +
case-variant fixes closed the deterministic gaps that the disease-centric benchmark never exercised;
and the remaining misses are either genuine SPOKE data gaps (model organisms) or LLM-tier granularity
cases — not architectural limits.
