---
type: Molecule
identifier: Leptin
subtype: protein
xref: [CHEBI:81571, UniProtKB:P41159]
synonyms: [LEP, OB protein]
description: "Adipocyte-derived hormone reflecting white-adipose-tissue mass; acts as a fuel gauge signalling energy depletion to the brain and driving the leptin-HPA axis in the fed-to-fasting transition."
edges:
  - predicate: regulates
    object: Hepatic gluconeogenesis
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: The integrative biology of type 2 diabetes (Roden & Shulman 2019)
    direction: increased
  - predicate: derives_from
    object: White adipose tissue
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: The integrative biology of type 2 diabetes (Roden & Shulman 2019)
  - predicate: reported_in
    object: The integrative biology of type 2 diabetes (Roden & Shulman 2019)
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: The integrative biology of type 2 diabetes (Roden & Shulman 2019)
---

# Leptin

A fuel gauge for energy stored as TAG in WAT and glycogen in liver. The early postabsorptive
decline in hepatic glycogenolysis lowers plasma insulin and glucose, producing roughly a 50%
reduction in plasma leptin; a fall below 1 ng/ml stimulates the HPA axis, raising corticosterone
that drives WAT lipolysis and maintains
[gluconeogenesis](../biologicalpathway/hepatic-gluconeogenesis.md) during starvation.
