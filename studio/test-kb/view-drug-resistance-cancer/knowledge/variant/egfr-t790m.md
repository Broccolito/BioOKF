---
type: Variant
identifier: EGFR T790M
subtype: snv
synonyms: ["EGFR p.Thr790Met", "T790M gatekeeper mutation"]
consequence: missense_variant
xref: ["HGVS:NP_005219.2:p.Thr790Met"]
edges:
  - predicate: located_in
    object: EGFR
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: "Vasan 2019 — A view on drug resistance in cancer"
  - predicate: predisposes_to
    object: Cancer drug resistance
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: "Vasan 2019 — A view on drug resistance in cancer"
    mechanism: "gatekeeper mutation blocks access of ATP-competitive first-generation TKIs to the kinase domain"
  - predicate: affects_response_to
    object: Osimertinib
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: "Vasan 2019 — A view on drug resistance in cancer"
    direction: sensitizing
  - predicate: reported_in
    object: "Vasan 2019 — A view on drug resistance in cancer"
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: "Vasan 2019 — A view on drug resistance in cancer"
---

# EGFR T790M

The most common acquired-resistance mutation in EGFR-mutant NSCLC after first-generation EGFR
inhibitors. This gatekeeper threonine-to-methionine substitution sterically blocks ATP-competitive
TKIs. T790M is detectable in plasma ctDNA — now standard of care for patients progressing on
gefitinib or erlotinib — and confers sensitivity to the mutant-selective irreversible inhibitor
osimertinib.
