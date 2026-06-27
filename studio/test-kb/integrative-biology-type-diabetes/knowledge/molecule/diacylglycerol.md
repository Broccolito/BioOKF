---
type: Molecule
identifier: Diacylglycerol
subtype: metabolite
xref: [CHEBI:18035]
synonyms: [DAG, sn-1,2-diacylglycerol, sn-1,2-DAG]
description: "Lipid second messenger; plasma-membrane sn-1,2-DAG accumulation from lipid overload activates novel PKC isoforms, the proximal trigger of hepatic and muscle insulin resistance."
edges:
  - predicate: regulates
    object: Protein kinase C epsilon
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: The integrative biology of type 2 diabetes (Roden & Shulman 2019)
    direction: increased
  - predicate: causes
    object: Insulin resistance
    knowledge_level: statistical_association
    agent_type: data_analysis_pipeline
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

# Diacylglycerol

In obese humans with NAFLD, the plasma-membrane *sn*-1,2-DAG-PKCepsilon pathway tightly
correlates with hepatic [insulin resistance](../phenotype/insulin-resistance.md). DAG triggers
translocation of [PKCepsilon](../molecule/protein-kinase-c-epsilon.md) (liver) and PKCtheta
(muscle) to the plasma membrane, inducing inhibitory phosphorylation of the insulin receptor and
IRS-1.
