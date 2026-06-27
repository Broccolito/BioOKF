# BioOKF Classification Tiebreakers & Guidance

Reference document for extracting concepts from sources. When facing an ambiguous classification, consult this document before deciding how to classify an entity by Type.

---

## Guiding Principles

**Disambiguate by use.** Many classification ties reduce to one rule: *when a term could map to more than one type, classify it by the role it plays in the specific passage, not by what it could abstractly denote.* TB-1 collects the recurring type-pairs where this applies. This is the deliberate opposite of TB-3 (identity over role): TB-1 disambiguates a *word whose referent changes* between passages; TB-3 fixes the type of an entity whose referent is stable but which plays a functional role (the role becomes an edge, not a type change). When both seem to apply, ask: is the term naming a *different kind of thing* here (→ TB-1, classify by use), or the *same thing in a functional role* (→ TB-3, classify by identity)?

**Mint the right node; don't bend to what exists.** Classify each concept and relationship into the node/predicate that is *correct*, then create whatever nodes that requires. Never bend a type, predicate, or edge object onto the nearest node that already exists just because the correct one is missing — the absence of a node is a reason to mint it, not a reason to misclassify. (Example: if `measures` should point at "MI genetic liability" (`BiomedicalMeasure`) but no such node exists, create it rather than pointing `measures` at the myocardial-infarction `Disease` node.)

## Tiebreaker Rules

### TB-1 — Disambiguate by use: one term, multiple candidate types

**The issue:** A single term often maps to different types depending on how a source uses it. Picking by what the term *could* mean, rather than how it is *used here*, is the most common source of classification ties.

**Rule:** Classify by the role the term plays in the specific passage. The recurring pairs follow.

**(a) Phenotype vs. BiomedicalMeasure — qualitative finding vs. quantified value.**

- Described qualitatively (a pattern, a sign, a finding) with no numeric value or unit → `Phenotype`

- A numeric value with units is reported, or the concept is explicitly framed as a measurement output → `BiomedicalMeasure`

- Examples: ground-glass opacities (CT finding, no value) → `Phenotype`; non-aerated lung volume (quantified from CT) → `BiomedicalMeasure`; lymphopenia (qualitative haematological sign) → `Phenotype`, lymphocyte count (the test) → `BiomedicalMeasure`; coagulopathy framed as "COVID-19 is characterized by coagulopathy" → `Phenotype`, the same concept as a primary diagnosis → `Disease`.

**(b) BiomedicalMeasure vs. Concept — anything quantitative is a measure.**

- Classify as `BiomedicalMeasure` if **either** trigger holds: (1) a numeric value or unit is reported, or (2) the term names a measurable quantity or a *type* of measurement/test, even when referenced broadly with no specific instance. Use `Concept` only when the term is not quantitative at all.

- Examples: penetrance, effect size, allele frequency → `BiomedicalMeasure` even when no value is given; heritability used quantitatively ("the heritability of height is \~80%," or even a bare reference to "the heritability of schizophrenia" as the estimable trait property) → `BiomedicalMeasure`; heritability used as an idea (a passage interrogating *what heritability means* — narrow- vs broad-sense, population-specificity, "heritability ≠ genetic determinism") → `Concept` (only split into both a `BiomedicalMeasure` and a `Concept` node per TB-2, instance `is_a` construct, when the source genuinely does both; the default for a mention is `BiomedicalMeasure`); polygenic risk score referenced generally → `BiomedicalMeasure`; pleiotropy, the omnigenic model → `Concept`.

**(c) Concept vs. entity types / MethodOrProcedure — within-type category vs. cross-cutting construct.**

- `Concept` is for abstract ideas with no specific instance identity — but being abstract or categorical is **not** by itself enough to make something a `Concept`.

- **Within-type categories stay in their type; cross-cutting constructs go to `Concept`.** A category that is a genuine subclass or clustering *within an entity type's own ontology hierarchy* takes that entity type, even when high-level or abstract — a type owns its whole class hierarchy. Reserve `Concept` for terms that **cut across** entity types, or are defined by a role/field/strategy rather than by an entity's own taxonomy.

- *Within-type* (→ entity type): "Mendelian disease" is a subclass within the `Disease` hierarchy — cystic fibrosis `is_a` Mendelian disease — so → `Disease`, not `Concept`. Likewise "SNP," "copy-number variant," "loss-of-function variant" are categories within `Variant` → `Variant`; "enhancer," "promoter," "transcription-factor binding site" are categories within `SequenceFeature` → `SequenceFeature`.

- *Cross-cutting / role-defined* (→ `Concept`): "animal model" is not a kind of organism — it is *any* organism in the role of a disease proxy (mouse, hamster, monkey), defined by use and spanning taxa → `Concept`. A concrete animal model decomposes per TB-3: the strain (e.g., K18-hACE2 mice) = `Organism`, its use in an experiment = `MethodOrProcedure`. Fields/disciplines, abstract strategies, and statistical constructs likewise map to no single entity type → `Concept`.

- Executable vs. abstract: "principal component analysis" (the executable technique) → `MethodOrProcedure`, "a principal component" (a derived abstract variable) → `Concept`; "drug repurposing" (an abstract strategy) → `Concept`, a specific drug-repurposing pipeline with defined steps → `MethodOrProcedure`; "antibody-dependent enhancement" (a named mechanistic process with a biological instantiation) → `BiologicalPathway`.

- `Concept` is also the type for score definitions, classification systems, units of measurement, and ontology terms referenced by name rather than instantiated. It is distinct from `Other`: `Concept` fits the vocabulary — it is just abstract; `Other` is for things that genuinely belong nowhere.

- **`Concept` is a fallback, not a default.** Prefer a concrete entity type whenever a term plausibly fits one — especially a quantitative term, which is almost always `BiomedicalMeasure` (case b). Reach for `Concept` only after the entity types and `BiomedicalMeasure` have been ruled out. A high volume of `Concept` assignments in a run is a warning sign that entity types are being under-used.

**(d) Study vs. MethodOrProcedure — a study design in the abstract vs. a specific study instance.**

- The *approach or study design discussed in the abstract* ("GWAS identified hundreds of loci," "a case-control design," "Mendelian randomization showed…") → `MethodOrProcedure`

- A *specific, identifiable study instance* — a named cohort/trial/analysis with investigators, year, sample, or registry ID → `Study` (with the appropriate kind, e.g. `gwas`, `cohort`, `rct`)

- The same source can yield both: a `MethodOrProcedure` node for the design in the abstract and separate `Study` nodes for the specific studies it describes.

- **Spec reconciliation note:** the cheatsheet's flat study-design mappings (e.g., "GWAS → `Study`", BioOKF\_schema\_v0.3.md line 60) should be softened to "a *specific* study of this design → `Study`" during the next spec-editing pass, so they no longer contradict this usage-based rule.

---

### TB-2 — Multi-node classification: when to create more than one node for the same concept

**The issue:** Some concepts have more than one valid facet (e.g., obesity is both a `Disease` and a `Phenotype`). Agents may default to picking one when both are valid.

**Rule:** Create multiple nodes for the same real-world concept if and only if the relationship between the facets can be expressed accurately by one of the 23 allowed BioOKF edge predicates. The existence of a meaningful predicate linking them is both the justification for the split and its documentation. If no predicate from the allowed list accurately describes the relationship between the two facets, represent them as a single node — not two nodes with a fuzzy link.

**Examples:**

- Disease(obesity) `has_phenotype` Phenotype(obesity) — MONDO:0011122 / HP:0001513 → two nodes

- Disease(thrombosis) `has_phenotype` Phenotype(thrombosis) — two nodes when both facets are explicitly present

- Aerosol challenge (experimental) → `MethodOrProcedure` (the protocol) and `Exposure` (the event the subject experiences), linked by `causes`

- Large genomics consortia/projects used for multiple facets (GTEx, ENCODE, HapMap, 1000 Genomes, Human Genome Project) → the organization/consortium is an `Agent`; the data resource it produced is a `Dataset`, linked by `Dataset` `derives_from` `Agent`. Add the `Dataset` node only when the source actually invokes the resource facet (e.g., "the HapMap" the map vs "the HapMap Consortium"); if the source uses just one facet, create just that node. A `Study` node is added only when a specific study design is discussed.

**Anti-pattern:** A `same_as` or `equivalent_to` link between two nodes is always wrong. If two nodes are truly the same entity, collapse them into one with `xref`. If they are distinct facets, a meaningful directional predicate must exist between them — its absence is a signal to merge.

---

### TB-3 — Identity over role: classify by what the thing *is*, not what it *does*

**The issue:** Agents conflate a biological product (a tangible thing that exists) with the act of administering or using it (a procedure). These are always two distinct nodes. (Contrast with TB-1: there the *referent* of a word changes between passages; here the referent is stable and only its functional role varies — so identity, not use, fixes the type, and the role is expressed as an edge.)

**Rule:** Classify by the biological nature of the product itself, not by its function or how it is used.

- The tangible product → type determined by what it *is* biologically (`Molecule`, `Organism`, etc.)

- The act of administering or applying it → `MethodOrProcedure`

**Examples:**

- Ad26 vaccine (viral vector) → `Organism`, kind: `vaccine`; vaccination act → `MethodOrProcedure`

- mRNA-1273 → `Molecule`, kind: `vaccine`; vaccination act → `MethodOrProcedure`

- Ad5-hACE2 adenoviral vector → `Organism`, kind: `vector`; respiratory tract transduction → `MethodOrProcedure`

- K18-hACE2 transgenic mice → `Organism` (a genetically defined strain); use as an animal model → `MethodOrProcedure`

- Gut microbiome → `Organism` (a biological entity); framing it as a "modifiable contributor to disease" is a *role*, expressed as an edge (`associated_with` / `predisposes_to`), not a reason to type it as `Exposure`

---

### TB-4 — Structural fit before type fit: fix the node/edge decomposition, don't bend the type

**The issue:** A classification feels forced — either no predicate fits the relationship, or no node type fits the term. The reflex is to conclude the type system has a gap and to bend the term onto the nearest existing node or predicate. The cause is a node/edge **structuring** error, not a missing type. This is the structural face of the top-of-document principle *"Mint the right node; don't bend to what exists."*

**Rule:** When a term or claim will not classify cleanly, correct the structure before suspecting a missing type. There are two failure modes, identified by which slot won't fill:

**(a) No predicate fits — a node is missing between subject and object.** An edge will not land on a sensible object because the real object is an intermediate readout that has not been created. Insert that node — a `BiomedicalMeasure` — and point the edge at it; reach the originally intended entity through a further edge. For `measures` specifically, ask what the subject is a *direct readout of*, and make that the object:

- If it directly quantifies a disease state → object is `Disease`.

- If it quantifies a functional outcome (efficacy, protection, response, variant function) → object is a `BiomedicalMeasure`, even when the entity ultimately being characterized is a disease, gene, or variant. Do not point `measures` at that entity.

*Examples:*

- Neutralizing antibody titres `measures` vaccine efficacy (`BiomedicalMeasure`); vaccine efficacy `associated_with` VAERD (`Disease`). Do not shortcut to: neutralizing antibody titres `measures` VAERD.

- Saturation genome editing `measures` BRCA1 variant function/pathogenicity (`BiomedicalMeasure`, kind `functional_assay`) — **not** `measures` BRCA1 (the `Gene`). The "functional assay" specificity lives in the measure's kind, not in a new predicate; there is no gap here.

- Polygenic risk score `measures` MI genetic liability/risk (`BiomedicalMeasure`); that liability `predisposes_to` / `associated_with` myocardial infarction (`Disease`) — **not** PRS `measures` myocardial infarction. (If the "MI genetic liability" node doesn't exist yet, create it — see the general principle on minting nodes at the top.)

**(b) No node type fits — the term names a relationship, not a thing.** Some named terms denote a *statistical association between two entities* rather than a standalone entity — e.g., an eQTL (expression quantitative trait locus) names the correlation between a variant and a gene's expression level. Forcing such a term into a node type (`SequenceFeature`, `Variant`, `BiomedicalMeasure`, `Concept`) fails because the term is a relationship. Represent it as an **edge between those entities, not as a node**. This mirrors the existing principle that a raw numeric value is edge data, never a node.

*Examples:*

- An eQTL → an `associated_with` edge between the `Variant` (or, if a locus with coordinates is named, the `SequenceFeature`) and the expression readout (`Gene` / expression `BiomedicalMeasure`). Do **not** create an "eQTL" node, and do **not** use `regulates` — an eQTL is a correlation, not a directed regulator.

- If the source names a specific locus *with coordinates*, that locus is a legitimate `SequenceFeature` node; the eQTL relationship is still the edge connecting it to the expression trait.

---

### TB-5 — When no exact label exists, add specificity with a kind

**The issue:** The node types and edge predicates are a closed, fixed vocabulary. A concept or relationship will sometimes map cleanly to the *right general* type or predicate while that choice drops a specific distinction the source draws — there is no exact label for the finer sense, and there is no option to add one. The closed vocabulary fixes the *category*; the specifics are carried in a `kind`. Nodes and edges both take a `kind`.

**Rule:** When the vocabulary gives you the right general category but not an exact match, record that category in `type`/`predicate` and put the specific sense in a `kind` — on the node when it is the entity that is under-specified, on the edge when it is the relationship. This is the normal way to express specificity the closed set does not name — not a shortfall to work around.

**Worked examples:**

- **Fomite — node kind.** A fomite (contaminated cage surface, bedding, food bowl) is an `Exposure` — it is the physical vehicle/medium of an exposure. Record `Exposure` + `kind: fomite`. The vehicle-vs-event distinction is exactly what the kind carries (sibling kinds: `aerosol`, `droplet`), with the two facets linkable per TB-2 (`Exposure[fomite]` `causes` `Exposure[event]`) when the source draws both.

- **Bats as natural reservoir of SARS-CoV-2 — edge kind.** The relationship is `associated_with` — a symmetric ecological association between an organism and a pathogen. Keep `associated_with` and carry the ecological role in the edge's `kind` (e.g., `kind: natural_reservoir`, with sibling values `vector`, `intermediate_host`, `amplifying_host`, `carrier`). The directionality and ecological sense the bare predicate does not name are recoverable from the kind.

**The boundary — when a kind is the wrong tool:** Use a kind only when the bare type/predicate is *correct on its own*. If the kind is doing the *classificatory work* — the assignment only makes sense once you read the kind — then the type or predicate is the wrong choice, and the fix is to re-examine the closed vocabulary and pick the type/predicate that genuinely fits, not to lean on the kind. **Test:** strip the kind and read the bare claim. Is it still true and not misleading? A fomite is still an `Exposure`; a bat is still `associated_with` the virus — so the kind is right. If the bare claim is false or category-wrong without the kind, the type/predicate choice itself needs revisiting. (This is the same caution as "What kinds are NOT," below — a kind adds specificity *within* a correct type or predicate; it never compensates for the wrong one.)

The kinds material below — what kinds are for, what they are not, and example values — is the supporting detail for this rule.

---

## Kinds Guidance

`kind` attributes (e.g., `molecule_kind`, `organism_kind` on nodes; a `kind` on an edge) are open-ended subtypes within a node type or predicate. Unlike the 28 node types and 23 predicates, kinds are not a closed vocabulary — the agent coins the value. They are the primary tool TB-5 invokes for adding specificity within a correctly-chosen type or predicate. A node's `kind` refines what the entity *is*; an edge's `kind` refines what the relationship *is*. This section collects guidance on how to use them well; it applies equally to node kinds and edge kinds.

### What kinds are for

Kinds add specificity within a type or predicate when the distinction is meaningful for querying — i.e., a user or agent would actually filter by it. Use the coarsest granularity that is still a useful filter. Finer distinctions (e.g., delivery platform, mechanism) belong in other node attributes or edge metadata, not in kind.

### What kinds are NOT

- A replacement for choosing the right type or predicate. If a concept belongs in `Organism`, adding `kind: molecule` does not make it a `Molecule`.

- A way to compensate for the wrong type or predicate. If the kind is doing most of the classificatory work — the assignment only reads correctly once you see the kind — the type or predicate itself is the wrong pick. Re-examine the closed vocabulary and choose the type/predicate that genuinely fits (TB-5), rather than leaning on the kind.

- A fine-grained subclassification system. Use `vaccine` not `vaccine-mRNA` — platform details go elsewhere.

### Example kinds

| Type / predicate       | kind                | Example                                                                   |
| ---------------------- | ------------------- | ------------------------------------------------------------------------- |
| `Molecule`             | `vaccine`           | mRNA-1273, inactivated SARS-CoV-2 vaccine, DNA vaccine                    |
| `Organism`             | `vaccine`           | YF17D-vectored SARS-CoV-2 vaccine, Ad26, ChAdOx1 nCoV-19                  |
| `Organism`             | `vector`            | Ad5-hACE2 adenoviral vector, AAV-hACE2 vector                             |
| `Organism`             | `community`         | gut microbiome (a multi-taxon microbial community, not a single organism) |
| `Exposure`             | `fomite`            | contaminated cage surface / bedding / food bowl (vehicle of exposure; siblings `aerosol`, `droplet`) |
| `BiomedicalMeasure`    | `functional_assay`  | saturation genome editing readout of variant function/pathogenicity       |
| `associated_with` (edge) | `natural_reservoir` | bats `associated_with` SARS-CoV-2 (ecological role; siblings `vector`, `intermediate_host`, `carrier`) |
