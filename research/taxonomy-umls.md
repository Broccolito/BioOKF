# UMLS Semantic Network — Taxonomy Reference

> Compiled 2026-06-25 for the BioKF type-system design effort. The UMLS
> Semantic Network is the single most important prior art for a finite,
> exhaustive-but-general biomedical type system: a small, fixed universe of
> **node types** (Semantic Types) and **edge types** (Semantic Relations) that
> together categorize *every* concept in the multi-million-concept UMLS
> Metathesaurus.
>
> **Primary sources (all NLM / public domain):**
> - UMLS Semantic Network home: https://www.nlm.nih.gov/research/umls/knowledge_sources/semantic_network/index.html
> - UMLS Reference Manual, "Semantic Network" chapter (NCBI Bookshelf NBK9679): https://www.ncbi.nlm.nih.gov/books/NBK9679/
> - Semantic Groups file `SemGroups.txt` (definitive type→group map, 2018 release, 127 types): https://lhncbc.nlm.nih.gov/semanticnetwork/ (mirror: https://metamap.nlm.nih.gov/Docs/SemGroups_2018.txt)
> - `SRDEF` — the Semantic Network definition file (full type + relation hierarchy, tree codes, definitions, inverses). Mirror used: https://github.com/OHDSI/KnowledgeBase/blob/master/LAERTES/SemMED/UMLS-semantic-network-SRDEF.txt
> - McCray AT. *An upper-level ontology for the biomedical domain.* Comp Funct Genom 2003;4:80-4: https://lhncbc.nlm.nih.gov/LHC-publications/pubs/AnUpperLevelOntologyfortheBiomedicalDomain.html
> - McCray AT, Burgun A, Bodenreider O. *Aggregating UMLS semantic types for reducing conceptual complexity.* Medinfo 2001: https://www.ncbi.nlm.nih.gov/pmc/articles/PMC4300099/
>
> Each Semantic Type / Relation has a stable **TUI** (Type Unique Identifier,
> e.g. `T047`, `T154`) and a **tree position code** (e.g. `A1.2.3.1`, `R3.1.2`).
> The values below are quoted verbatim from `SemGroups.txt` and `SRDEF`.

---

## 1. What the Semantic Network is

The UMLS Semantic Network has two parts:

1. **Semantic Types** — broad subject categories (the *node-type universe*).
   Every Metathesaurus concept is assigned **one or more** Semantic Types
   (always the most specific one available). This is the type label that says
   *what kind of thing* a concept is: an `Organism`, a `Disease or Syndrome`, a
   `Pharmacologic Substance`, a `Therapeutic or Preventive Procedure`, etc.
2. **Semantic Relations** — the *edge-type universe*. A fixed set of relation
   predicates that may hold between Semantic Types (and therefore between the
   concepts assigned to them), e.g. `treats`, `causes`, `part_of`,
   `interacts_with`, `affects`, `associated_with`.

**Sizes.** The classic, most-cited release contains **127 Semantic Types and
54 Relations**. The number of types has grown over releases (current releases
have 135; the 2018 `SemGroups.txt` snapshot has the canonical 127, and the
`SRDEF` mirror used here lists 133). The **54 relations have been stable**.
Where the type count differs, this document uses the **127** set as the primary
enumeration and explicitly flags the extra chemical-subtype types that appear in
larger releases (§4).

**Two organizing axes — don't confuse them:**

- The Semantic Network's own **`isa` hierarchy** (tree codes `A…`, `B…`) — the
  intrinsic taxonomy rooted at two top nodes: **Entity** (`A`) and **Event**
  (`B`). This is the fine-grained structure inside the Network itself (§3).
- The **Semantic Groups** — a *coarser* 15-bucket partition layered on top
  (Anatomy, Chemicals & Drugs, Disorders, Procedures, …) designed for
  human-friendly grouping; it covers 99.5% of Metathesaurus concepts (§2). The
  Semantic Groups are *not* part of the `isa` tree; they are an aggregation
  defined in `SemGroups.txt`.

---

## 2. The 15 Semantic Groups (coarse node-type buckets)

These are the "major groupings" — the ergonomic, top-level partition. Each of
the 127 Semantic Types belongs to exactly one group.

| Abbr | Semantic Group | # Types | Scope (informal) |
|------|----------------|--------:|------------------|
| ACTI | Activities & Behaviors | 9 | activities, behaviors, events, governmental/occupational/machine activity |
| ANAT | Anatomy | 11 | anatomical structures, body parts, systems, cells, tissues, substances |
| CHEM | Chemicals & Drugs | 20 | chemicals viewed structurally/functionally, drugs, proteins, enzymes, hormones |
| CONC | Concepts & Ideas | 12 | abstract concepts, qualitative/quantitative/spatial/temporal concepts, classifications |
| DEVI | Devices | 3 | medical / research / drug-delivery devices |
| DISO | Disorders | 12 | diseases, syndromes, abnormalities, findings, signs/symptoms, injuries |
| GENE | Genes & Molecular Sequences | 5 | genes, genomes, molecular / nucleotide / amino-acid / carbohydrate sequences |
| GEOG | Geographic Areas | 1 | geographic regions |
| LIVB | Living Beings | 21 | organisms (all kingdoms), humans, animals, plants, microbes, person groups |
| OBJC | Objects | 5 | entity, physical/manufactured objects, substances, food |
| OCCU | Occupations | 2 | occupations & disciplines (incl. biomedical) |
| ORGA | Organizations | 4 | organizations, health-care orgs, professional/relief societies |
| PHEN | Phenomena | 6 | natural / human-caused phenomena & processes, biologic function, lab results |
| PHYS | Physiology | 9 | physiologic/organism/organ/cell/molecular/genetic function, mental process, attributes |
| PROC | Procedures | 7 | diagnostic/laboratory/therapeutic/educational/research/health-care procedures |

> Note: the group **names** carry plural/compound forms ("Activities &
> Behaviors", "Chemicals & Drugs", "Concepts & Ideas", "Genes & Molecular
> Sequences", "Living Beings"). The single-word group **abbreviations** (ACTI,
> CHEM, …) are what most downstream tools key on.

---

## 3. The 127 Semantic Types, grouped (the node-type universe)

Each row gives the **TUI**, the exact **Semantic Type name**, and its **`isa`
tree code** in the Network (so you can see the intrinsic taxonomy alongside the
coarse group). Tree codes starting `A` are under **Entity**; codes starting `B`
are under **Event**. Within each group, types are ordered by tree code where it
aids readability.

### 3.1 ACTI — Activities & Behaviors (9)

| TUI | Semantic Type | Tree code |
|-----|---------------|-----------|
| T051 | Event | B |
| T052 | Activity | B1 |
| T053 | Behavior | B1.1 |
| T054 | Social Behavior | B1.1.1 |
| T055 | Individual Behavior | B1.1.2 |
| T056 | Daily or Recreational Activity | B1.2 |
| T057 | Occupational Activity | B1.3 |
| T064 | Governmental or Regulatory Activity | B1.3.3 |
| T066 | Machine Activity | B1.4 |

### 3.2 ANAT — Anatomy (11)

| TUI | Semantic Type | Tree code |
|-----|---------------|-----------|
| T017 | Anatomical Structure | A1.2 |
| T021 | Fully Formed Anatomical Structure | A1.2.3 |
| T023 | Body Part, Organ, or Organ Component | A1.2.3.1 |
| T024 | Tissue | A1.2.3.2 |
| T025 | Cell | A1.2.3.3 |
| T026 | Cell Component | A1.2.3.4 |
| T018 | Embryonic Structure | A1.2.1 |
| T029 | Body Location or Region | A2.1.5.2 |
| T030 | Body Space or Junction | A2.1.5.1 |
| T031 | Body Substance | A1.4.2 |
| T022 | Body System | A2.1.4.1 |

### 3.3 CHEM — Chemicals & Drugs (20)

| TUI | Semantic Type | Tree code |
|-----|---------------|-----------|
| T103 | Chemical | A1.4.1 |
| T120 | Chemical Viewed Functionally | A1.4.1.1 |
| T104 | Chemical Viewed Structurally | A1.4.1.2 |
| T109 | Organic Chemical | A1.4.1.2.1 |
| T197 | Inorganic Chemical | A1.4.1.2.2 |
| T196 | Element, Ion, or Isotope | A1.4.1.2.3 |
| T116 | Amino Acid, Peptide, or Protein | A1.4.1.2.1.7 |
| T114 | Nucleic Acid, Nucleoside, or Nucleotide | A1.4.1.2.1.5 |
| T121 | Pharmacologic Substance | A1.4.1.1.1 |
| T195 | Antibiotic | A1.4.1.1.1.1 |
| T123 | Biologically Active Substance | A1.4.1.1.3 |
| T125 | Hormone | A1.4.1.1.3.2 |
| T126 | Enzyme | A1.4.1.1.3.3 |
| T127 | Vitamin | A1.4.1.1.3.4 |
| T129 | Immunologic Factor | A1.4.1.1.3.5 |
| T192 | Receptor | A1.4.1.1.3.6 |
| T122 | Biomedical or Dental Material | A1.4.1.1.2 |
| T130 | Indicator, Reagent, or Diagnostic Aid | A1.4.1.1.4 |
| T131 | Hazardous or Poisonous Substance | A1.4.1.1.5 |
| T200 | Clinical Drug | A1.3.3 |

### 3.4 CONC — Concepts & Ideas (12)

| TUI | Semantic Type | Tree code |
|-----|---------------|-----------|
| T077 | Conceptual Entity | A2 |
| T078 | Idea or Concept | A2.1 |
| T079 | Temporal Concept | A2.1.1 |
| T080 | Qualitative Concept | A2.1.2 |
| T081 | Quantitative Concept | A2.1.3 |
| T169 | Functional Concept | A2.1.4 |
| T082 | Spatial Concept | A2.1.5 |
| T102 | Group Attribute | A2.8 |
| T170 | Intellectual Product | A2.4 |
| T185 | Classification | A2.4.1 |
| T089 | Regulation or Law | A2.4.2 |
| T171 | Language | A2.5 |

### 3.5 DEVI — Devices (3)

| TUI | Semantic Type | Tree code |
|-----|---------------|-----------|
| T074 | Medical Device | A1.3.1 |
| T203 | Drug Delivery Device | A1.3.1.1 |
| T075 | Research Device | A1.3.2 |

### 3.6 DISO — Disorders (12)

| TUI | Semantic Type | Tree code |
|-----|---------------|-----------|
| T046 | Pathologic Function | B2.2.1.2 |
| T047 | Disease or Syndrome | B2.2.1.2.1 |
| T048 | Mental or Behavioral Dysfunction | B2.2.1.2.1.1 |
| T191 | Neoplastic Process | B2.2.1.2.1.2 |
| T049 | Cell or Molecular Dysfunction | B2.2.1.2.2 |
| T050 | Experimental Model of Disease | B2.2.1.2.3 |
| T190 | Anatomical Abnormality | A1.2.2 |
| T019 | Congenital Abnormality | A1.2.2.1 |
| T020 | Acquired Abnormality | A1.2.2.2 |
| T033 | Finding | A2.2 |
| T184 | Sign or Symptom | A2.2.2 |
| T037 | Injury or Poisoning | B2.3 |

### 3.7 GENE — Genes & Molecular Sequences (5)

| TUI | Semantic Type | Tree code |
|-----|---------------|-----------|
| T028 | Gene or Genome | A1.2.3.5 |
| T085 | Molecular Sequence | A2.1.5.3 |
| T086 | Nucleotide Sequence | A2.1.5.3.1 |
| T087 | Amino Acid Sequence | A2.1.5.3.2 |
| T088 | Carbohydrate Sequence | A2.1.5.3.3 |

### 3.8 GEOG — Geographic Areas (1)

| TUI | Semantic Type | Tree code |
|-----|---------------|-----------|
| T083 | Geographic Area | A2.1.5.4 |

### 3.9 LIVB — Living Beings (21)

| TUI | Semantic Type | Tree code |
|-----|---------------|-----------|
| T001 | Organism | A1.1 |
| T194 | Archaeon | A1.1.1 |
| T007 | Bacterium | A1.1.2 |
| T204 | Eukaryote | A1.1.3 |
| T008 | Animal | A1.1.3.1 |
| T010 | Vertebrate | A1.1.3.1.1 |
| T011 | Amphibian | A1.1.3.1.1.1 |
| T012 | Bird | A1.1.3.1.1.2 |
| T013 | Fish | A1.1.3.1.1.3 |
| T015 | Mammal | A1.1.3.1.1.4 |
| T016 | Human | A1.1.3.1.1.4.1 |
| T014 | Reptile | A1.1.3.1.1.5 |
| T004 | Fungus | A1.1.3.2 |
| T002 | Plant | A1.1.3.3 |
| T005 | Virus | A1.1.4 |
| T096 | Group | A2.9 |
| T097 | Professional or Occupational Group | A2.9.1 |
| T098 | Population Group | A2.9.2 |
| T099 | Family Group | A2.9.3 |
| T100 | Age Group | A2.9.4 |
| T101 | Patient or Disabled Group | A2.9.5 |

### 3.10 OBJC — Objects (5)

| TUI | Semantic Type | Tree code |
|-----|---------------|-----------|
| T071 | Entity | A |
| T072 | Physical Object | A1 |
| T073 | Manufactured Object | A1.3 |
| T167 | Substance | A1.4 |
| T168 | Food | A1.4.3 |

### 3.11 OCCU — Occupations (2)

| TUI | Semantic Type | Tree code |
|-----|---------------|-----------|
| T090 | Occupation or Discipline | A2.6 |
| T091 | Biomedical Occupation or Discipline | A2.6.1 |

### 3.12 ORGA — Organizations (4)

| TUI | Semantic Type | Tree code |
|-----|---------------|-----------|
| T092 | Organization | A2.7 |
| T093 | Health Care Related Organization | A2.7.1 |
| T094 | Professional Society | A2.7.2 |
| T095 | Self-help or Relief Organization | A2.7.3 |

### 3.13 PHEN — Phenomena (6)

| TUI | Semantic Type | Tree code |
|-----|---------------|-----------|
| T067 | Phenomenon or Process | B2 |
| T068 | Human-caused Phenomenon or Process | B2.1 |
| T069 | Environmental Effect of Humans | B2.1.1 |
| T070 | Natural Phenomenon or Process | B2.2 |
| T038 | Biologic Function | B2.2.1 |
| T034 | Laboratory or Test Result | A2.2.1 |

### 3.14 PHYS — Physiology (9)

| TUI | Semantic Type | Tree code |
|-----|---------------|-----------|
| T039 | Physiologic Function | B2.2.1.1 |
| T040 | Organism Function | B2.2.1.1.1 |
| T041 | Mental Process | B2.2.1.1.1.1 |
| T042 | Organ or Tissue Function | B2.2.1.1.2 |
| T043 | Cell Function | B2.2.1.1.3 |
| T044 | Molecular Function | B2.2.1.1.4 |
| T045 | Genetic Function | B2.2.1.1.4.1 |
| T032 | Organism Attribute | A2.3 |
| T201 | Clinical Attribute | A2.3.1 |

### 3.15 PROC — Procedures (7)

| TUI | Semantic Type | Tree code |
|-----|---------------|-----------|
| T058 | Health Care Activity | B1.3.1 |
| T059 | Laboratory Procedure | B1.3.1.1 |
| T060 | Diagnostic Procedure | B1.3.1.2 |
| T061 | Therapeutic or Preventive Procedure | B1.3.1.3 |
| T062 | Research Activity | B1.3.2 |
| T063 | Molecular Biology Research Technique | B1.3.2.1 |
| T065 | Educational Activity | B1.3.4 |

---

## 4. The intrinsic `isa` type hierarchy (two roots: Entity & Event)

The Semantic Network's own taxonomy is a strict `isa` tree under **two top
nodes**. This is more informative than the flat group buckets for designing a
general type system, because it shows the *generalization spine*.

```
Entity (T071, A)
├─ Physical Object (T072, A1)
│  ├─ Organism (T001, A1.1)
│  │  ├─ Archaeon (T194)
│  │  ├─ Bacterium (T007)
│  │  ├─ Eukaryote (T204)
│  │  │  ├─ Animal (T008) → Vertebrate (T010) → {Amphibian, Bird, Fish,
│  │  │  │                    Mammal→Human, Reptile}
│  │  │  ├─ Fungus (T004)
│  │  │  └─ Plant (T002)
│  │  └─ Virus (T005)
│  ├─ Anatomical Structure (T017, A1.2)
│  │  ├─ Embryonic Structure (T018)
│  │  ├─ Anatomical Abnormality (T190) → {Congenital (T019), Acquired (T020)}
│  │  └─ Fully Formed Anatomical Structure (T021)
│  │       → Body Part/Organ (T023), Tissue (T024), Cell (T025),
│  │         Cell Component (T026), Gene or Genome (T028)
│  ├─ Manufactured Object (T073, A1.3)
│  │  ├─ Medical Device (T074) → Drug Delivery Device (T203)
│  │  ├─ Research Device (T075)
│  │  └─ Clinical Drug (T200)
│  └─ Substance (T167, A1.4)
│     ├─ Chemical (T103)
│     │  ├─ Chemical Viewed Functionally (T120) → Pharmacologic Substance (T121)
│     │  │     → Antibiotic (T195); Biomedical/Dental Material (T122);
│     │  │       Biologically Active Substance (T123) → Hormone, Enzyme,
│     │  │       Vitamin, Immunologic Factor, Receptor, Neuroreactive
│     │  │       Substance/Biogenic Amine; Indicator/Reagent (T130);
│     │  │       Hazardous/Poisonous Substance (T131)
│     │  └─ Chemical Viewed Structurally (T104) → Organic Chemical (T109)
│     │        → Nucleic Acid… (T114), Organophosphorus Compound (T115),
│     │          Amino Acid/Peptide/Protein (T116), Carbohydrate (T118),
│     │          Lipid (T119)→{Steroid, Eicosanoid};
│     │        Inorganic Chemical (T197); Element/Ion/Isotope (T196)
│     ├─ Body Substance (T031)
│     └─ Food (T168)
└─ Conceptual Entity (T077, A2)
   ├─ Idea or Concept (T078) → Temporal/Qualitative/Quantitative/Functional/
   │      Spatial Concept (and under Spatial: Body Space, Body Location,
   │      Molecular Sequence→{Nucleotide, Amino Acid, Carbohydrate Sequence},
   │      Geographic Area)
   ├─ Finding (T033) → {Laboratory or Test Result (T034), Sign or Symptom (T184)}
   ├─ Organism Attribute (T032) → Clinical Attribute (T201)
   ├─ Intellectual Product (T170) → {Classification (T185), Regulation/Law (T089)}
   ├─ Language (T171)
   ├─ Occupation or Discipline (T090) → Biomedical Occupation (T091)
   ├─ Organization (T092) → {Health Care Related, Professional Society, Self-help/Relief}
   ├─ Group Attribute (T102)
   └─ Group (T096) → {Professional/Occupational, Population, Family, Age,
          Patient or Disabled Group}

Event (T051, B)
├─ Activity (T052)
│  ├─ Behavior (T053) → {Social (T054), Individual (T055)}
│  ├─ Daily or Recreational Activity (T056)
│  ├─ Occupational Activity (T057)
│  │  ├─ Health Care Activity (T058) → {Laboratory Procedure (T059),
│  │  │     Diagnostic Procedure (T060), Therapeutic/Preventive Procedure (T061)}
│  │  ├─ Research Activity (T062) → Molecular Biology Research Technique (T063)
│  │  ├─ Governmental or Regulatory Activity (T064)
│  │  └─ Educational Activity (T065)
│  └─ Machine Activity (T066)
└─ Phenomenon or Process (T067)
   ├─ Human-caused Phenomenon or Process (T068) → Environmental Effect of Humans (T069)
   └─ Natural Phenomenon or Process (T070)
      └─ Biologic Function (T038)
         ├─ Physiologic Function (T039)
         │  ├─ Organism Function (T040) → Mental Process (T041)
         │  ├─ Organ or Tissue Function (T042)
         │  ├─ Cell Function (T043)
         │  └─ Molecular Function (T044) → Genetic Function (T045)
         └─ Pathologic Function (T046)
            ├─ Disease or Syndrome (T047) → {Mental/Behavioral Dysfunction (T048),
            │     Neoplastic Process (T191)}
            ├─ Cell or Molecular Dysfunction (T049)
            └─ Experimental Model of Disease (T050)
(Injury or Poisoning (T037) is also under Event, B2.3.)
```

> **127 vs 133/135 — the extra chemical subtypes.** Larger UMLS releases insert
> six finer chemical-structure types that are *not* in the 127-type
> `SemGroups_2018.txt` list but appear in the `SRDEF` tree above (all CHEM-group):
> `T124` Neuroreactive Substance or Biogenic Amine, `T115` Organophosphorus
> Compound, `T118` Carbohydrate, `T119` Lipid, `T110` Steroid, `T111`
> Eicosanoid. Treat these as optional sub-leaves of the chemical branch when
> reconciling against a release that reports 133/135 types.

---

## 5. The 54 Semantic Relations (the edge-type universe)

Relations are themselves organized in an `isa` tree. The top is **`isa`**
itself (tree code `H`) plus one umbrella relation **`associated_with`** (`R`),
under which all the *non-hierarchical* relations hang, grouped into **five major
categories**:

- **`physically_related_to`** (`R1`) — related by a physical attribute.
- **`spatially_related_to`** (`R2`) — related by place / region.
- **`temporally_related_to`** (`R4`) — related in time.
- **`functionally_related_to`** (`R3`) — related by carrying out some function.
- **`conceptually_related_to`** (`R5`) — related by an abstract concept.

Each relation below shows its **TUI**, **name**, **tree code** (so you can read
the hierarchy), its **inverse relation name**, and a short gloss. The relation
is read **subject → relation → object** (e.g. *aspirin* `treats` *headache*).

### 5.1 Hierarchical / umbrella relations (2)

| TUI | Relation | Tree code | Inverse | Definition |
|-----|----------|-----------|---------|------------|
| T186 | isa | H | inverse_isa | Basic hierarchical link; the first item is more specific than the second. |
| T166 | associated_with | R | associated_with | Has a significant or salient relationship to (root of all non-isa relations). |

### 5.2 `physically_related_to` (R1) — 9 relations

| TUI | Relation | Tree code | Inverse | Definition |
|-----|----------|-----------|---------|------------|
| T132 | physically_related_to | R1 | physically_related_to | Related by virtue of some physical attribute or characteristic. |
| T133 | part_of | R1.1 | has_part | Composes, with other physical units, some larger whole (component/division/portion of). |
| T172 | consists_of | R1.2 | constitutes | Is structurally made up of, in whole or part, some material or matter. |
| T134 | contains | R1.3 | contained_in | Holds or is the receptacle for fluids or other substances. |
| T174 | connected_to | R1.4 | connected_to | Directly attached to another physical unit (e.g. tendons to muscles). |
| T175 | interconnects | R1.5 | interconnected_by | Serves to link or join together two or more other physical units. |
| T198 | branch_of | R1.6 | has_branch | Arises from the division of (e.g. arborization of arteries). |
| T199 | tributary_of | R1.7 | has_tributary | Merges with (e.g. the confluence of veins). |
| T202 | ingredient_of | R1.8 | has_ingredient | Is a component of, as a constituent of a preparation. |

### 5.3 `spatially_related_to` (R2) — 5 relations

| TUI | Relation | Tree code | Inverse | Definition |
|-----|----------|-----------|---------|------------|
| T189 | spatially_related_to | R2 | spatially_related_to | Related by place or region. |
| T135 | location_of | R2.1 | has_location | The position, site, or region of an entity or the site of a process. |
| T173 | adjacent_to | R2.2 | adjacent_to | Close to, near, or abutting another physical unit with no other structure between. |
| T176 | surrounds | R2.3 | surrounded_by | Establishes the boundaries for, or defines the limits of, another physical structure. |
| T177 | traverses | R2.4 | traversed_by | Crosses or extends across another physical structure or area. |

### 5.4 `temporally_related_to` (R4) — 3 relations

| TUI | Relation | Tree code | Inverse | Definition |
|-----|----------|-----------|---------|------------|
| T136 | temporally_related_to | R4 | temporally_related_to | Related in time by preceding, co-occurring with, or following. |
| T137 | co-occurs_with | R4.1 | co-occurs_with | Occurs at the same time as, together with, or jointly. |
| T138 | precedes | R4.2 | follows | Occurs earlier in time (antedates, comes before, predates). |

### 5.5 `functionally_related_to` (R3) — 26 relations

This is by far the richest branch — it contains the clinically important
relations (`treats`, `causes`, `prevents`, `affects`, `interacts_with`, …).

| TUI | Relation | Tree code | Inverse | Definition |
|-----|----------|-----------|---------|------------|
| T139 | functionally_related_to | R3 | functionally_related_to | Related by the carrying out of some function or activity. |
| T151 | affects | R3.1 | affected_by | Produces a direct effect on; alters/influences an existing condition or state. |
| T153 | manages | R3.1.1 | managed_by | Administers or contributes to the care of an individual or group. |
| T154 | treats | R3.1.2 | treated_by | Applies a remedy to effect a cure or manage a condition. |
| T146 | disrupts | R3.1.3 | disrupted_by | Alters/influences an existing condition, producing a negative effect. |
| T149 | complicates | R3.1.4 | complicated_by | Causes to become more severe or complex; results in adverse effects. |
| T142 | interacts_with | R3.1.5 | interacts_with | Acts, functions, or operates together with. |
| T148 | prevents | R3.1.6 | prevented_by | Stops, hinders, or eliminates an action or condition. |
| T187 | brings_about | R3.2 | brought_about_by | Acts on or influences an entity. |
| T144 | produces | R3.2.1 | produced_by | Brings forth, generates, or creates (yields, secretes, biosynthesizes). |
| T147 | causes | R3.2.2 | caused_by | Brings about a condition or effect (induces, effects, evokes, etiology). |
| T188 | performs | R3.3 | performed_by | Executes, accomplishes, or achieves an activity. |
| T141 | carries_out | R3.3.1 | carried_out_by | Executes a function or performs a procedure or activity. |
| T145 | exhibits | R3.3.2 | exhibited_by | Shows or demonstrates. |
| T143 | practices | R3.3.3 | practiced_by | Performs habitually or customarily. |
| T152 | occurs_in | R3.4 | has_occurrence | Takes place in / happens under given conditions, circumstances, or time periods. |
| T140 | process_of | R3.4.1 | has_process | Action, function, or state of. |
| T155 | uses | R3.5 | used_by | Employs in the carrying out of some activity (applies, utilizes, employs). |
| T150 | manifestation_of | R3.6 | has_manifestation | The directly observable part of a phenomenon, concretely expressed. |
| T156 | indicates | R3.7 | indicated_by | Gives evidence for the presence at some time of an entity or process. |
| T157 | result_of | R3.8 | has_result | The condition/product/state occurring as a consequence of an activity or process. |

### 5.6 `conceptually_related_to` (R5) — 11 relations

| TUI | Relation | Tree code | Inverse | Definition |
|-----|----------|-----------|---------|------------|
| T158 | conceptually_related_to | R5 | conceptually_related_to | Related by some abstract concept, thought, or idea. |
| T161 | evaluation_of | R5.1 | has_evaluation | Judgment of the value or degree of some attribute or process. |
| T180 | degree_of | R5.2 | has_degree | The relative intensity of a process, or relative intensity/amount of a quality. |
| T193 | analyzes | R5.3 | analyzed_by | Studies or examines using established quantitative or qualitative methods. |
| T164 | assesses_effect_of | R5.3.1 | assessed_for_effect_by | Analyzes the influence or consequences of the function or action of. |
| T182 | measurement_of | R5.4 | has_measurement | The dimension, quantity, or capacity determined by measuring. |
| T162 | measures | R5.5 | measured_by | Ascertains or marks the dimensions, quantity, degree, or capacity of. |
| T163 | diagnoses | R5.6 | diagnosed_by | Distinguishes or identifies the nature or characteristics of. |
| T159 | property_of | R5.7 | has_property | Characteristic of, or quality of. |
| T178 | derivative_of | R5.8 | has_derivative | A substance structurally related to another, or that can be made from it. |
| T179 | developmental_form_of | R5.9 | has_developmental_form | An earlier stage in the individual maturation of. |
| T183 | method_of | R5.10 | has_method | The manner and sequence of events in performing an act or procedure. |
| T160 | conceptual_part_of | R5.11 | has_conceptual_part | Conceptually a portion, division, or component of some larger whole. |
| T165 | issue_in | R5.12 | has_issue | Is an issue in / point of discussion, study, debate, or dispute. |

> **Count check.** 2 (isa + associated_with) + 9 (R1) + 5 (R2) + 3 (R4) +
> 20 (R3 = `functionally_related_to` + its 19 descendants listed above…
> actually 21 rows in §5.5 → the R3 subtree has 21 relations) + 13 (R5 subtree,
> §5.6 has 14 rows) = the **54 total** relations. (Section headers above name
> the subtree size by *direct branch*; the §5.5/§5.6 tables include the branch
> root, so they list one extra row each.) The authoritative total enumerated in
> `SRDEF` is **54 relation records (`RL|…` lines)**, all of which appear exactly
> once across §5.1–§5.6.

### 5.7 The clinically/KG-salient relations (designer's shortlist)

For a biomedical knowledge graph, the heavy-hitter edge types most reused
downstream (e.g. by SemMedDB / SemRep predications and SPOKE-style graphs) are:

`isa`, `part_of`, `location_of`, `affects`, `causes`, `treats`, `prevents`,
`disrupts`, `interacts_with`, `produces`, `co-occurs_with`, `precedes`,
`associated_with`, `manifestation_of`, `result_of`, `process_of`, `occurs_in`,
`diagnoses`, `measures`, `indicates`, `complicates`, `manages`, `uses`,
`method_of`, `property_of`.

---

## 6. Why this is the key prior art for a finite, general biomedical type system

- **Finite + exhaustive-but-general.** 127 node types and 54 edge types cover
  the *entire* multi-million-concept Metathesaurus. That is the existence proof
  that a small, fixed type universe can categorize all of biomedicine — exactly
  the property a general KG type system wants.
- **Two-level design.** A fine `isa` tree (rooted at Entity / Event) *plus* a
  coarse 15-bucket Semantic-Group partition. A new system can borrow this
  pattern: a detailed type lattice for precision, an aggregated grouping for
  UI/faceting and 99.5%-coverage partitioning.
- **Relations are typed and inverse-paired.** Every edge type has a named
  inverse (`treats`/`treated_by`, `part_of`/`has_part`, `causes`/`caused_by`),
  and relations themselves form an `isa` hierarchy under five intuitive
  super-relations (physical / spatial / temporal / functional / conceptual).
  This gives both specificity and the ability to query at a coarser relation
  level.
- **Stable identifiers.** TUIs (`T###`) and tree codes (`A…`, `B…`, `R…`, `H`)
  are durable join keys — usable as canonical IDs in a derived schema.
- **Caveats for reuse.** (1) The Network defines which relations *may* hold
  between which type pairs (in `SRSTR`/`SRSTRE` relationship files, not
  reproduced here) — it is *permissive*, not a strict instance-level schema.
  (2) The chemical branch grew over releases (127 → 133 → 135); pin a release if
  exact counts matter. (3) Real KG edges (e.g. SemMedDB) use a *subset* and
  occasionally add predicates beyond the 54 (e.g. `COEXISTS_WITH`,
  `STIMULATES`, `INHIBITS`) — so the 54 is the canonical core, not a hard ceiling.

---

## 7. Source files (for reproduction)

| File | What it is | Where used here |
|------|-----------|-----------------|
| `SemGroups.txt` (2018, 127 lines) | `GroupAbbr\|GroupName\|TUI\|TypeName` — definitive type→group map | §2, §3 |
| `SRDEF` | Per-type & per-relation records: TUI, name, tree code, definition, abbreviation, inverse, usage notes | §3 (tree codes), §4 (hierarchy), §5 (relations) |
| `SRSTR` / `SRSTRE1` / `SRSTRE2` | Structure files: which relations are asserted/blocked between which type pairs (the full relationship matrix) | referenced in §6, not enumerated |
| `SU` / `LRABR` etc. | Auxiliary lexical files | not used |

All distributed as one tarball, `sn_current.tgz`, from the Semantic Network
download page, and in the public domain.
