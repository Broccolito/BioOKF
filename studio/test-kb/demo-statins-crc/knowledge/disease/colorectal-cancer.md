---
type: Disease
identifier: Colorectal cancer
subtype: neoplasm
xref: [MONDO:0005575, DOID:9256, MESH:D015179, ICD10:C18, UMLS:C0009402]
synonyms: [colorectal carcinoma, bowel cancer, CRC]
description: Malignant neoplasm arising from the colon or rectum.
edges:
  - predicate: reported_in
    object: Statins and colorectal-cancer risk (cohort)
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: Statins and colorectal-cancer risk (cohort)
---

# Colorectal cancer

A common malignancy of the colon or rectum. This demo records two **negative** findings against it:
[atorvastatin](../molecule/atorvastatin.md) `not_prevents` it, and [HMGCR (gene)](../gene/hmgcr.md)
is `not_associated_with` it.
