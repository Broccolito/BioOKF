---
type: SequenceFeature
identifier: EPAS1 3' UTR
subtype: utr
xref: [SO:0000205, ENSEMBL:ENSG00000116016]
in_taxon: NCBITaxon:9606
description: >
  The 3' UTR of EPAS1 (HIF-2α) — a post-transcriptional cis-regulatory region carrying miRNA
  target sites and AU-rich elements that tune mRNA stability/translation.
edges:
  - predicate: part_of
    object: EPAS1 (gene)                       # HGNC:3374 — no page yet (tolerated broken link)
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: Ensembl
  - predicate: regulates                       # role = post-transcriptional regulation of its own gene
    object: EPAS1 (gene)
    direction: decreased
    aspect: abundance
    knowledge_level: knowledge_assertion
    agent_type: text_mining_agent
    primary_source: SemMedDB
    publications: [PMID:24759409]
---

# EPAS1 3' UTR

A `SequenceFeature` (`subtype: utr`) — **not** a `Gene` and **not** a `Variant`. A UTR is a
constitutive *region of the reference* (Sequence Ontology `three_prime_UTR`, `SO:0000205`), so it
files under `SequenceFeature`; a *deviation from the reference* would be a `Variant`. Its regulatory
*role* is an edge, not its type.

> **Class vs instance:** *this* EPAS1 3' UTR (with coordinates) is a `SequenceFeature` entity; the
> bare notion "a 3' UTR" (the SO class as a label) would be a `Concept`.

## Citations
- [Altitude adaptation in Tibetans (Denisovan EPAS1)](https://pubmed.ncbi.nlm.nih.gov/25043035/) (Huerta-Sánchez et al. 2014)
