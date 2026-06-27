---
type: Disease
identifier: Non-alcoholic fatty liver disease
subtype: metabolic_disorder
xref: [MONDO:0013209, DOID:0080208, MESH:D065626]
synonyms: [NAFLD, hepatic steatosis disorder, fatty liver disease]
description: "Liver disease defined by hepatic triacylglycerol accumulation, spanning simple steatosis to non-alcoholic steatohepatitis (NASH); tightly linked to hepatic insulin resistance in obesity and T2D."
edges:
  - predicate: located_in
    object: Liver
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: The integrative biology of type 2 diabetes (Roden & Shulman 2019)
  - predicate: has_phenotype
    object: Hepatic steatosis
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: The integrative biology of type 2 diabetes (Roden & Shulman 2019)
  - predicate: associated_with
    object: Insulin resistance
    knowledge_level: statistical_association
    agent_type: data_analysis_pipeline
    primary_source: The integrative biology of type 2 diabetes (Roden & Shulman 2019)
  - predicate: reported_in
    object: The integrative biology of type 2 diabetes (Roden & Shulman 2019)
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: The integrative biology of type 2 diabetes (Roden & Shulman 2019)
---

# Non-alcoholic fatty liver disease

Defined by hepatic [triacylglycerol](../molecule/diacylglycerol.md) accumulation. The review
argues NAFLD develops chiefly through increased NEFA flux from insulin-resistant white adipose
tissue rather than requiring selective hepatic insulin resistance. The
*sn*-1,2-DAG-PKCepsilon pathway tightly correlates with hepatic insulin resistance in obese
humans with NAFLD.
