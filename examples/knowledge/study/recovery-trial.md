---
type: Study
identifier: RECOVERY trial
subtype: rct
xref: [clinicaltrials:NCT04381936, DOI:10.1016/S0140-6736(21)00676-0]
synonyms: [RECOVERY platform trial, Randomised Evaluation of COVID-19 Therapy]
description: Large UK platform randomized controlled trial of treatments for hospitalized COVID-19.
raw_source: [raw/pmid-33933206-recovery-tocilizumab.md]
edges:
  - predicate: associated_with                 # the trial evaluates this intervention→outcome
    object: Tocilizumab
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: RECOVERY trial             # a Study attests its own evaluation (terminating base case)
    qualifiers:
      population: hospitalized adults with COVID-19 and hypoxia + inflammation
    sample_size: 4116
---

# RECOVERY trial (tocilizumab arm)

An **ingested-document source node**: a `Study` (`subtype: rct`) distilled from the immutable file
in `raw_source`. It evaluated [tocilizumab](../molecule/tocilizumab.md) in hospitalized
[COVID-19](../disease/covid-19.md) patients, reducing 28-day mortality (RR 0.85, 95% CI 0.76–0.94;
n=4116) — the evidence behind the `treats` edge on the tocilizumab page, whose `primary_source`
names **this node**.

> A source node is provenance you can traverse to: claims point here via `primary_source` and
> `reported_in`, and `raw_source` anchors it to the exact bytes in `raw/`.
