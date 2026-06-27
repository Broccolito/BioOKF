---
type: Molecule
identifier: Insulin
subtype: protein
xref: [CHEBI:145810, DRUGBANK:DB00030, UniProtKB:P01308]
synonyms: [INS, human insulin]
description: "Anabolic peptide hormone secreted by pancreatic beta-cells; stimulates glucose uptake and glycogen synthesis, suppresses lipolysis and hepatic gluconeogenesis. Its reduced action is insulin resistance."
edges:
  - predicate: regulates
    object: Hepatic gluconeogenesis
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: The integrative biology of type 2 diabetes (Roden & Shulman 2019)
    direction: decreased
  - predicate: regulates
    object: White adipose tissue lipolysis
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: The integrative biology of type 2 diabetes (Roden & Shulman 2019)
    direction: decreased
  - predicate: regulates
    object: De novo lipogenesis
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: The integrative biology of type 2 diabetes (Roden & Shulman 2019)
    direction: increased
  - predicate: treats
    object: Type 2 diabetes
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: The integrative biology of type 2 diabetes (Roden & Shulman 2019)
  - predicate: reported_in
    object: The integrative biology of type 2 diabetes (Roden & Shulman 2019)
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: The integrative biology of type 2 diabetes (Roden & Shulman 2019)
---

# Insulin

Under postprandial conditions, insulin rapidly stimulates lipid storage by inhibiting
[lipolysis](../biologicalpathway/wat-lipolysis.md), suppresses
[gluconeogenesis](../biologicalpathway/hepatic-gluconeogenesis.md) (indirectly via reduced WAT
NEFA/glycerol flux), and stimulates glycogen synthesis. Reduced insulin action defines
[insulin resistance](../phenotype/insulin-resistance.md).
