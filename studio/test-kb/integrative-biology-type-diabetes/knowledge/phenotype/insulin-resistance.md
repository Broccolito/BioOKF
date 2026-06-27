---
type: Phenotype
identifier: Insulin resistance
subtype: trait
xref: [HP:0000855, EFO:0002910]
synonyms: [impaired insulin sensitivity, reduced insulin sensitivity]
description: "Reduced ability of insulin to stimulate glucose disposal and suppress hepatic glucose production; the common underlying abnormality in obesity and T2D, present in liver, skeletal muscle and white adipose tissue."
edges:
  - predicate: associated_with
    object: Diacylglycerol
    knowledge_level: statistical_association
    agent_type: data_analysis_pipeline
    primary_source: The integrative biology of type 2 diabetes (Roden & Shulman 2019)
  - predicate: associated_with
    object: Protein kinase C epsilon
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: The integrative biology of type 2 diabetes (Roden & Shulman 2019)
  - predicate: predisposes_to
    object: Type 2 diabetes
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: The integrative biology of type 2 diabetes (Roden & Shulman 2019)
  - predicate: located_in
    object: Skeletal muscle
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: The integrative biology of type 2 diabetes (Roden & Shulman 2019)
  - predicate: reported_in
    object: The integrative biology of type 2 diabetes (Roden & Shulman 2019)
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: The integrative biology of type 2 diabetes (Roden & Shulman 2019)
---

# Insulin resistance

The common underlying abnormality in obesity and T2D. In the liver and muscle, plasma-membrane
*sn*-1,2-[diacylglycerol](../molecule/diacylglycerol.md) accumulation triggers translocation of
novel PKC isoforms ([PKCepsilon](../molecule/protein-kinase-c-epsilon.md) in liver; PKCtheta and
PKCepsilon in muscle), which phosphorylate and inhibit the insulin receptor / IRS-1. Skeletal
muscle insulin resistance, due to impaired insulin-stimulated glucose transport and glycogen
synthesis, is one of the earliest pathogenic events, declining decades before T2D onset.
