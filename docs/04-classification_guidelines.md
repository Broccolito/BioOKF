# BioOKF Classification Tiebreakers & Guidance

Reference document for extracting concepts from sources. When facing an ambiguous classification, consult this document before deciding how to classify an entity by Type.

---

## Guiding Principles

**Disambiguate by use.** Many classification ties reduce to one rule: *when a term could map to more than one type, classify it by the role it plays in the specific passage, not by what it could abstractly denote.* TB-1 collects the recurring type-pairs where this applies. This is the deliberate opposite of TB-2 (identity over role): TB-1 disambiguates a *word whose referent changes* between passages; TB-2 fixes the type of an entity whose referent is stable but which plays a functional role (the role becomes an edge, not a type change). When both seem to apply, ask: is the term naming a *different kind of thing* here (→ TB-1, classify by use), or the *same thing in a functional role* (→ TB-2, classify by identity)?

**Mint the right node; don't bend to what exists.** Classify each concept and relationship into the node/predicate that is *correct*, then create whatever nodes that requires. Never bend a type, predicate, or edge object onto the nearest node that already exists just because the correct one is missing — the absence of a node is a reason to mint it, not a reason to misclassify.

## Tiebreaker Rules

### TB-1 — Disambiguate by use: one term, multiple candidate types

**The issue:** A single term often maps to different types depending on how a source uses it. Picking by what the term *could* mean, rather than how it is *used here*, is the most common source of classification ties.

**Rule:** Classify by the role the term plays in the specific passage. The recurring pairs:

**(a) Phenotype vs. BiomedicalMeasure — qualitative finding vs. quantified value.** Qualitative (a pattern, sign, or finding, no value or unit) → `Phenotype`; a numeric value/unit, or explicitly framed as a measurement output → `BiomedicalMeasure`. *Ground-glass opacities* (CT finding) → `Phenotype`; *non-aerated lung volume* (quantified) → `BiomedicalMeasure`; *lymphopenia* (sign) → `Phenotype`, *lymphocyte count* (the test) → `BiomedicalMeasure`.

**(b) BiomedicalMeasure vs. Concept — anything quantitative is a measure.** Classify as `BiomedicalMeasure` if **either**: (1) a numeric value or unit is reported, or (2) the term names a measurable quantity or a *type* of measurement/test, even with no specific instance. *Penetrance, effect size, allele frequency, heritability, polygenic risk score* → `BiomedicalMeasure` even when no value is given. Use `Concept` only when the term is not quantitative at all (*pleiotropy, the omnigenic model*), or when a passage genuinely interrogates the idea itself (what heritability *means*) rather than a value — default a bare mention to `BiomedicalMeasure`.

**(c) Concept vs. entity types / MethodOrProcedure — within-type category vs. cross-cutting construct.** Being abstract or categorical is **not** by itself enough to make something a `Concept`. A category that is a genuine subclass *within an entity type's own hierarchy* stays in that type, however high-level — a type owns its whole class hierarchy: "Mendelian disease" → `Disease`; "SNP," "loss-of-function variant" → `Variant`; "enhancer," "promoter" → `SequenceFeature`. Reserve `Concept` for terms that **cut across** entity types or are defined by a role/field/strategy: "animal model" is *any* organism in a disease-proxy role → `Concept` (a concrete model decomposes per TB-2 — the strain, e.g. K18-hACE2 mice, = `Organism`; its experimental use = `MethodOrProcedure`); fields, abstract strategies, and statistical constructs likewise → `Concept`. Executable vs. abstract: "principal component analysis" (technique) → `MethodOrProcedure`, "a principal component" (derived variable) → `Concept`; "drug repurposing" (strategy) → `Concept`, a defined repurposing pipeline → `MethodOrProcedure`. `Concept` also covers score definitions, classification systems, units, and ontology terms referenced by name (distinct from `Other`, which is for things that fit the vocabulary nowhere). **`Concept` is a fallback, not a default** — prefer a concrete entity type whenever one plausibly fits, especially `BiomedicalMeasure` for anything quantitative; a high volume of `Concept` assignments signals entity types are being under-used.

**(d) Study vs. MethodOrProcedure — study design in the abstract vs. a specific study instance.** A study design discussed in the abstract ("GWAS identified hundreds of loci," "Mendelian randomization showed…") → `MethodOrProcedure`; a specific, identifiable study — a named cohort/trial/analysis with investigators, year, sample, or registry ID → `Study` (with the appropriate subtype, e.g. `gwas`, `cohort`, `rct`). The same source can yield both.

---

### TB-2 — Identity over role: classify by what the thing *is*, not what it *does*

**The issue:** Agents conflate a biological product (a tangible thing that exists) with the act of administering or using it (a procedure). These are always two distinct nodes. (Contrast with TB-1: there the *referent* of a word changes between passages; here the referent is stable and only its functional role varies — so identity, not use, fixes the type, and the role is expressed as an edge.)

**Rule:** Classify by the biological nature of the product itself, not by its function or how it is used.

- The tangible product → type determined by what it *is* biologically (`Molecule`, `Organism`, etc.)

- The act of administering or applying it → `MethodOrProcedure`

**Examples:**

- Ad26 vaccine (viral vector) → `Organism`, subtype: `vaccine`; vaccination act → `MethodOrProcedure`

- mRNA-1273 → `Molecule`, subtype: `vaccine`; vaccination act → `MethodOrProcedure`

- Ad5-hACE2 adenoviral vector → `Organism`, subtype: `vector`; respiratory tract transduction → `MethodOrProcedure`

- K18-hACE2 transgenic mice → `Organism` (a genetically defined strain); use as an animal model → `MethodOrProcedure`

- Gut microbiome → `Organism` (a biological entity); framing it as a "modifiable contributor to disease" is a *role*, expressed as an edge (`associated_with` / `predisposes_to`), not a reason to type it as `Exposure`

---

### TB-3 — When no exact label exists, add specificity with a subtype

**The issue:** The node types and edge predicates are a closed, fixed vocabulary. A concept or relationship will sometimes map cleanly to the *right general* type or predicate while that choice drops a specific distinction the source draws — there is no exact label for the finer sense, and there is no option to add one. The closed vocabulary fixes the *category*; the specifics are carried in a `subtype`. Nodes and edges both take a `subtype`.

**Rule:** When the vocabulary gives you the right general category but not an exact match, record that category in `type`/`predicate` and put the specific sense in a `subtype` — on the node when it is the entity that is under-specified, on the edge when it is the relationship. This is the normal way to express specificity the closed set does not name — not a shortfall to work around.

**Worked examples:**

- **Fomite — node subtype.** A fomite (contaminated cage surface, bedding, food bowl) is an `Exposure` — it is the physical vehicle/medium of an exposure. Record `Exposure` \+ `subtype: fomite`. The vehicle-vs-event distinction is exactly what the subtype carries (sibling subtypes: `aerosol`, `droplet`).

- **Bats as natural reservoir of SARS-CoV-2 — edge subtype.** The relationship is `associated_with` — a symmetric ecological association between an organism and a pathogen. Keep `associated_with` and carry the ecological role in the edge's `subtype` (e.g., `subtype: natural_reservoir`, with sibling values `vector`, `intermediate_host`, `amplifying_host`, `carrier`). The directionality and ecological sense the bare predicate does not name are recoverable from the subtype.

**The boundary — when a subtype is the wrong tool:** Use a subtype only when the bare type/predicate is *correct on its own*. If the subtype is doing the *classificatory work* — the assignment only makes sense once you read the subtype — then the type or predicate is the wrong choice, and the fix is to re-examine the closed vocabulary and pick the type/predicate that genuinely fits, not to lean on the subtype. **Test:** strip the subtype and read the bare claim. Is it still true and not misleading? A fomite is still an `Exposure`; a bat is still `associated_with` the virus — so the subtype is right. If the bare claim is false or category-wrong without the subtype, the type/predicate choice itself needs revisiting. (This is the same caution as "What subtypes are NOT," below — a subtype adds specificity *within* a correct type or predicate; it never compensates for the wrong one.)

The subtypes material below — what subtypes are for, what they are not, and example values — is the supporting detail for this rule.

---

## Subtypes: Further Guidance

`subtype` is the single agent-coined field that adds specificity within a node type or predicate (e.g., `subtype: protein` on a node; a `subtype` on an edge). Unlike the 28 node types and 23 predicates, subtypes are not a closed vocabulary — the agent coins the value. They are the primary tool TB-3 invokes for adding specificity within a correctly-chosen type or predicate. A node's `subtype` refines what the entity *is*; an edge's `subtype` refines what the relationship *is*. This section collects guidance on how to use them well; it applies equally to node subtypes and edge subtypes.

### What subtypes are for

Subtypes add specificity within a type or predicate when the distinction is meaningful for querying — i.e., a user or agent would actually filter by it. Use the coarsest granularity that is still a useful filter. Finer distinctions (e.g., delivery platform, mechanism) belong in other node attributes or edge metadata, not in the subtype.

### What subtypes are NOT

- A replacement for choosing the right type or predicate. If a concept belongs in `Organism`, adding `subtype: molecule` does not make it a `Molecule`.

- A way to compensate for the wrong type or predicate. If the subtype is doing most of the classificatory work — the assignment only reads correctly once you see the subtype — the type or predicate itself is the wrong pick. Re-examine the closed vocabulary and choose the type/predicate that genuinely fits (TB-3), rather than leaning on the subtype.

- A fine-grained subclassification system. Use `vaccine` not `vaccine-mRNA` — platform details go elsewhere.

### Example subtypes

| Type / predicate         | subtype             | Example                                                                                                |
| ------------------------ | ------------------- | ------------------------------------------------------------------------------------------------------ |
| `Molecule`               | `vaccine`           | mRNA-1273, inactivated SARS-CoV-2 vaccine, DNA vaccine                                                 |
| `Organism`               | `vaccine`           | YF17D-vectored SARS-CoV-2 vaccine, Ad26, ChAdOx1 nCoV-19                                               |
| `Organism`               | `vector`            | Ad5-hACE2 adenoviral vector, AAV-hACE2 vector                                                          |
| `Organism`               | `community`         | gut microbiome (a multi-taxon microbial community, not a single organism)                              |
| `Exposure`               | `fomite`            | contaminated cage surface / bedding / food bowl (vehicle of exposure; siblings `aerosol`, `droplet`)   |
| `BiomedicalMeasure`      | `functional_assay`  | saturation genome editing readout of variant function/pathogenicity                                    |
| `associated_with` (edge) | `natural_reservoir` | bats `associated_with` SARS-CoV-2 (ecological role; siblings `vector`, `intermediate_host`, `carrier`) |
