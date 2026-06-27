---
type: Molecule
identifier: Protein kinase C epsilon
subtype: protein
xref: [UniProtKB:Q02156]
synonyms: [PKCepsilon, PKCe, nPKCepsilon, novel protein kinase C isoform epsilon]
description: "Novel (calcium-independent) protein kinase C isoform; activated by plasma-membrane sn-1,2-DAG, it translocates and phosphorylates Thr1160 of the insulin receptor, inhibiting IR kinase activity (hepatic insulin resistance)."
edges:
  - predicate: regulates
    object: Insulin
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: The integrative biology of type 2 diabetes (Roden & Shulman 2019)
    direction: decreased
  - predicate: causes
    object: Insulin resistance
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: The integrative biology of type 2 diabetes (Roden & Shulman 2019)
  - predicate: located_in
    object: Liver
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: The integrative biology of type 2 diabetes (Roden & Shulman 2019)
  - predicate: reported_in
    object: The integrative biology of type 2 diabetes (Roden & Shulman 2019)
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: The integrative biology of type 2 diabetes (Roden & Shulman 2019)
---

# Protein kinase C epsilon

Translocated to the plasma membrane by [DAG](diacylglycerol.md), where it binds and
phosphorylates Thr1160 of the insulin receptor, inhibiting IR kinase activity. The high
evolutionary conservation of the Thr1160 residue suggests the DAG-nPKC mechanism served a
survival role during starvation while favouring
[insulin resistance](../phenotype/insulin-resistance.md) during overnutrition.
