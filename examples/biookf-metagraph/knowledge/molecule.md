---
type: Molecule
identifier: Molecule
subtype: metatype
edges:
  - {predicate: member_of, object: MolecularClass, knowledge_level: knowledge_assertion, agent_type: manual_agent, primary_source: Publication}
  - {predicate: interacts_with, object: Gene, knowledge_level: knowledge_assertion, agent_type: manual_agent, primary_source: Publication}
  - {predicate: binds, object: SequenceFeature, knowledge_level: knowledge_assertion, agent_type: manual_agent, primary_source: Publication}
  - {predicate: catalyzes, object: BiologicalFunction, knowledge_level: knowledge_assertion, agent_type: manual_agent, primary_source: Publication}
  - {predicate: converts_to, object: Food, knowledge_level: knowledge_assertion, agent_type: manual_agent, primary_source: Publication}
  - {predicate: participates_in, object: BiologicalPathway, knowledge_level: knowledge_assertion, agent_type: manual_agent, primary_source: Publication}
  - {predicate: treats, object: Disease, knowledge_level: knowledge_assertion, agent_type: manual_agent, primary_source: Publication}
  - {predicate: not_binds, object: Gene, knowledge_level: knowledge_assertion, agent_type: manual_agent, primary_source: Publication}
  - {predicate: not_interacts_with, object: Disease, knowledge_level: knowledge_assertion, agent_type: manual_agent, primary_source: Publication}
  - {predicate: not_treats, object: Phenotype, knowledge_level: knowledge_assertion, agent_type: manual_agent, primary_source: Publication}
---

# Molecule

Archetype for proteins, drugs, metabolites, complexes, ions, and RNA species.

