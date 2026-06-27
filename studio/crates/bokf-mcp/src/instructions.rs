//! The MCP `instructions` block — the agent-facing operating brief. Mirrors how
//! BioRouter prepends `schema.md + --- + <PROCEDURE>`; here the essentials are
//! embedded so any client (Claude/Codex) knows the BioOKF v0.5 rules and the
//! ingest / query / lint loops without extra round-trips.

pub const INSTRUCTIONS: &str = r#"BioOKF MCP server — you curate biomedical knowledge bases in the Open Knowledge Format (BioOKF v0.5).

WHAT A BUNDLE IS
A bundle is a directory: `raw/` (immutable sources you never edit), `knowledge/<type>/<slug>.md` (the typed concept docs you author — the graph), plus `index.md` (catalog) and `log.md` (newest-first dated history). Every concept doc is YAML frontmatter + a Markdown body.

THE TWO HARD RULES
1) Every concept doc's `type` is EXACTLY ONE of these 28:
   Gene, Molecule, MolecularClass, Variant, SequenceFeature, Structure, Anatomy, CellType, Organism, BiologicalPathway, BiologicalFunction, Disease, Phenotype, BiomedicalMeasure, MethodOrProcedure, Exposure, SocialFactor, Food, Device, MaterialSample, Publication, Study, Dataset, Agent, Population, GeographicLocation, Concept, Other. If nothing fits, use Other with a `note:`. Never invent a type.
2) Every relationship is a frontmatter `edges:` entry whose `predicate` is one of these 24 (forward-only — no inverses):
   is_a, part_of, member_of, derives_from, located_in, expressed_in, encodes, interacts_with, binds, regulates, catalyzes, converts_to, participates_in, causes, predisposes_to, treats, prevents, contraindicated_in, affects_response_to, has_phenotype, measures, associated_with, used_to_study, reported_in. Direction is always this-document -> object.

MANDATORY FIELDS
- Node: `type` and `identifier` (human-readable AND unique across the bundle; NOT a CURIE — put CURIEs in `xref`). Always also coin a lowercase `subtype` (no controlled list).
- Edge: `predicate`, `object` (the target node's identifier), and the provenance triplet `knowledge_level` (knowledge_assertion|statistical_association|prediction|observation|not_provided), `agent_type` (manual_agent|automated_agent|text_mining_agent|data_analysis_pipeline|computational_model|not_provided), and `primary_source` (the identifier of a SOURCE NODE — a Publication/Study/Dataset/Agent — never a CURIE).
- Put numbers on edges, never in prose: p_value, effect_size + effect_metric, ci_lower/ci_upper, sample_size, sensitivity, specificity, frequency, direction (required for regulates/expressed_in), unit.

PROVENANCE IS NODE-BASED (v0.5)
Each claim's `primary_source` names a source node by identifier. A source node is either an ingested document (a Publication/Study/Dataset with `raw_source: [raw/...]`) or an external reference (e.g. HGNC, Gene Ontology, DrugBank — a node with its `infores:`/ontology CURIE in `xref`, no raw_source). Create each external-reference source node ONCE and reuse its identifier. Also add a `reported_in` edge to make provenance traversable.

=== INGEST A SOURCE ===
1. Save the source under `raw/` unchanged (use bokf_write outside knowledge/ is not allowed; ingested bytes are placed by the host — when given a raw path, treat it as already present).
2. Read/parse it fully.
3. Create a Publication/Study/Dataset node FOR THE SOURCE, with `raw_source` listing its raw/ path(s).
4. For each biomedical entity discussed, create or UPDATE its typed concept doc (reuse an existing identifier — never fork). Use bokf_validate_page to check a draft before writing, then bokf_write_page.
5. Add `xref` CURIEs where known (optional enrichment).
6. For each claim, add a typed `edges:` entry with the provenance triplet + any statistics, and a `reported_in` edge to the source node.
7. Update index.md (bokf_write_page) and append a dated entry to log.md (bokf_append_log).
A single source typically creates/updates 10-15 concept pages.

=== ANSWER A QUERY ===
Read index.md -> bokf_search to find relevant pages -> bokf_read_page to open them -> follow `edges:` (and use bokf_graph for neighborhood) -> synthesize a CITED answer (cite node identifiers + their sources). Filter by knowledge_level for clinical questions. Prefer graph-shaped reasoning ("what treats a Disease associated_with this Gene?"). Never invent facts not in the pages.

=== LINT ===
Run bokf_lint. It flags: invalid type/predicate; missing/duplicate/opaque identifiers; edge objects that don't resolve; missing or invalid provenance triplet; primary_source that isn't a source node; unanchored source nodes; domain/range violations; orphans; contradictions; duplicate edges (same predicate+object+source); near-duplicate subtypes (merge candidates); and type/directory mismatch (a node filed under the wrong `knowledge/<type>/` — a misclassification signal). Fix Errors first (rewrite the offending page with bokf_write_page), then Warnings. A missing `xref` is an enrichment opportunity, not an error; `subtype` is never linted.

TOOLS: bokf_list_bases, bokf_scaffold, bokf_list_pages, bokf_read_page, bokf_write_page, bokf_validate_page, bokf_append_log, bokf_lint, bokf_graph, bokf_search, bokf_stats, bokf_predicates. Always bokf_validate_page a concept doc before bokf_write_page, and bokf_lint after a batch of writes.
"#;
