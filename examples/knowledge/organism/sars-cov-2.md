---
type: Organism
identifier: SARS-CoV-2
subtype: pathogen
xref: [NCBITaxon:2697049]
synonyms: [severe acute respiratory syndrome coronavirus 2, 2019-nCoV]
in_taxon: NCBITaxon:2697049
description: The betacoronavirus that causes COVID-19.
edges:
  - predicate: causes                          # forward etiology (replaces the inverse caused_by)
    object: COVID-19
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: MONDO
---

# SARS-CoV-2

The pathogen whose forward `causes` edge points to [COVID-19](../disease/covid-19.md). Modeling
etiology as `Organism causes Disease` (not `Disease caused_by Organism`) is the forward-only rule
in action.
