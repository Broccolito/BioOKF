# 28-type stress test — 100 real nodes per BioOKF type (2,796 total)

Generated 100 real, diverse entities for every one of the 28 BioOKF node types (via a 28-agent
workflow) and ran them all through the combined engine, verifying every mapping against live SPOKE.
Goal: find inconsistencies and vulnerabilities across the *whole* type system, not just diseases.

## Safety result — the headline

Across all **2,796** nodes:
- **Broken ids: 0** — no deterministic tier ever pointed at a SPOKE node that doesn't exist.
- **Provenance leakage: 0** — all 1,197 not-mappable nodes (the 8 provenance types + Device,
  MaterialSample, MethodOrProcedure, SequenceFeature) stayed not_mappable; none falsely mapped into
  SPOKE.
- **Semantic flags: 26** (mapped with no lexical overlap) — nearly all correct synonym/format matches
  (HMG-CoA reductase inhibitors → Hydroxymethylglutaryl-CoA Reductase Inhibitors; NO2 → Nitrogen
  Dioxide; taxonomic aliases). ~4–6 are borderline fuzzy-tier over-specializations (see limitations).

**The engine does not fabricate mappings and does not leak provenance nodes** — the core safety
properties hold across every type.

## Coverage by type (deterministic tiers only; the LLM tier lifts the ambiguous set)

| tier of coverage | types |
|---|---|
| **high (well-covered)** | Gene 85%, BiologicalFunction 81%, Anatomy 79%, Molecule 71%, CellType 70%, Disease 69%, Phenotype 67%, GeographicLocation 65%, Organism 62% |
| **correctly not_mappable (no SPOKE analogue)** | MethodOrProcedure, Device, MaterialSample, SequenceFeature, Publication, Study, Dataset, Agent, Population, Concept, Other |
| **limited by SPOKE data shape** | Variant 7% (dbSNP-only), BiomedicalMeasure 1% (LOINC multiplicity), SocialFactor 5% (AHRQ/SNOMED noise), Exposure 21%, MolecularClass 29%, BiologicalPathway 38%, Food 38% |

The "limited" types are **not harness failures**: the correct SPOKE node usually *is* among the
candidates, but it sits amid many near-duplicates (HbA1c → dozens of LOINC codes; Household income →
AHRQ county indicators). Those land in the `ambiguous` review queue for the LLM tier rather than being
force-mapped — which is the safe behavior.

## Inconsistencies found — and fixed

1. **SequenceFeature → ProteinDomain was semantically wrong.** Enhancers/promoters/TFBS have no SPOKE
   node; routing them to ProteinDomain produced 100 nonsense ambiguities ("beta-globin locus control
   region" → "Globin" domain). Fixed: `SequenceFeature` is now `not_mappable` by default, with
   `subtype_routing` sending only true `protein_domain` subtypes to ProteinDomain. → 100 ambiguous
   became honest not_mappable.
2. **Structure → CellularComponent produced garbage for PDB entries.** Fixed with subtype routing:
   PDB methods (`xray`/`cryo_em`/`nmr`/`predicted`) → not_mappable; the benchmark's subcellular
   Structures (`general`/`cell junction`, e.g. caveolae, desmosome) still map to CellularComponent.
   → 91 ambiguous became honest not_mappable, **zero benchmark regression**.
3. **SNOMED/LOINC type-suffix mismatch.** SPOKE SDoH/ClinicalLab names carry qualifiers
   ("Household income **(observable entity)**") that blocked exact match. `norm_key` now strips a
   whitelist of trailing qualifiers (finding/disorder/observable entity/procedure/…), recovering
   e.g. Household income → SNOMED_224168007.

Net effect on the stress test: ambiguous 830 → **638** (−192 noise), not_mappable 998 → **1,197**
(honest), mapped unchanged, still **0 broken / 0 leakage**.

## Known limitations (not vulnerabilities)

- **Fuzzy tier over-specialization.** The lowest-confidence tier (conf 0.72) occasionally maps a
  general term to a more-specific SPOKE child (VEGF signaling → VEGF-activated neuropilin signaling;
  Dengue fever → dengue hemorrhagic fever) — ~0.7% of maps. A structural guard can't separate these
  from correct abbreviation-expansions (H2 antagonists → Histamine H2 Receptor Antagonists), so
  production should route all `fuzzy_accept` to LLM review.
- **BiomedicalMeasure / SocialFactor multiplicity.** SPOKE's ClinicalLab (LOINC) and SDoH (AHRQ)
  labels have many near-duplicate records per concept; a per-label preference (e.g. a canonical LOINC)
  or the LLM tier is needed to pick one. The candidates are correct; only the selection is ambiguous.

## Regression (after all fixes)

- **Disease benchmark: identical to baseline** — round8 == round7: 1,464 mapped, 209 agree, 97
  related, 14 contradict, 424 not_mappable, **0 nodes changed**, invariants PASS.
- **128-organism test: unchanged** — 118/128 (92%), 100% verified precision.

The stress test found real inconsistencies, they were fixed, and the original conversion still passes
exactly.
