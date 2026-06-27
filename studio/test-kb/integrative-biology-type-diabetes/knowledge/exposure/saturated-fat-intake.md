---
type: Exposure
identifier: Saturated fat intake
subtype: dietary
xref: [ECTO:0010032]
synonyms: [dietary saturated fat, saturated fatty acid intake, SFA intake]
description: "Dietary exposure to saturated fat; a single oral saturated-fat load acutely induces insulin resistance in liver, skeletal muscle and white adipose tissue in healthy humans."
edges:
  - predicate: causes
    object: Insulin resistance
    knowledge_level: statistical_association
    agent_type: data_analysis_pipeline
    primary_source: The integrative biology of type 2 diabetes (Roden & Shulman 2019)
  - predicate: predisposes_to
    object: Non-alcoholic fatty liver disease
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: The integrative biology of type 2 diabetes (Roden & Shulman 2019)
  - predicate: regulates
    object: Hepatic gluconeogenesis
    knowledge_level: statistical_association
    agent_type: data_analysis_pipeline
    primary_source: The integrative biology of type 2 diabetes (Roden & Shulman 2019)
    direction: increased
    effect_metric: percent_change
    effect_size: 70
    unit: percent
  - predicate: reported_in
    object: The integrative biology of type 2 diabetes (Roden & Shulman 2019)
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: The integrative biology of type 2 diabetes (Roden & Shulman 2019)
---

# Saturated fat intake

A single oral saturated-fat load simultaneously induced
[insulin resistance](../phenotype/insulin-resistance.md) in liver, skeletal muscle and WAT, and
was associated with 70% higher rates of hepatic
[gluconeogenesis](../biologicalpathway/hepatic-gluconeogenesis.md) and 20% lower rates of net
hepatic glycogenolysis. Chronic overfeeding also raised intestinal endotoxins promoting
TLR4-induced cytokine release by Kupffer cells.
