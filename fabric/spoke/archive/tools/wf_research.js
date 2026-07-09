export const meta = {
  name: 'spoke-biookf-research',
  description: 'Characterize every BioOKF + SPOKE node/edge type via real queries and real benchmark data',
  phases: [
    { title: 'SPOKE-nodes', detail: 'per-label deep dive with live queries' },
    { title: 'SPOKE-edges', detail: 'per-reltype endpoint + property + source dive' },
    { title: 'BioOKF-types', detail: 'per node type + predicate semantics from spec + benchmark' },
  ],
};

const WT = '/Users/wgu/Desktop/BioOKF-spoke-mapper';
const CLI = WT + '/fabric/spoke/spoke-cli';
const SKELETON = WT + '/fabric/spoke/schema/spoke_schema_skeleton.json';
const BENCH = WT + '/fabric/spoke/runs/benchmark-biored';
const SCHEMA_MD = WT + '/SCHEMA.md';
const SPEC_MD = WT + '/SPEC.md';

const COOKBOOK = `
HOW TO QUERY SPOKE (read-only Neo4j via the compiled CLI):
- Invoke: ${CLI} query "<CYPHER>" --stdout
- CRITICAL: the CLI only serializes SCALAR values. A bare node or map (RETURN n, RETURN properties(n))
  serializes to null. ALWAYS return individual scalar properties or scalar lists:
    MATCH (n:\\\`Gene\\\`) WITH n LIMIT 1 RETURN keys(n) AS keys
    MATCH (n:\\\`Gene\\\`) RETURN n.name AS name, n.identifier AS id, n.synonyms AS syn, n.source AS source, n.sources AS sources LIMIT 5
- Escape backticks around labels/reltypes with backticks in Cypher: use \\\`Label\\\`.
- Keep LIMIT tight (3-10). Protein has 39M nodes, Compound 2.4M, Variant 1M — NEVER scan unbounded.
- Edge shape: MATCH (a)-[r:\\\`RELTYPE\\\`]->(b) RETURN a.name AS a_name, a.identifier AS a_id, b.name AS b_name, b.identifier AS b_id, keys(r) AS rel_keys, r.sources AS rel_sources LIMIT 3
- Reltype endpoint signatures are ALREADY known: read ${SKELETON} (relationship_signatures, relationship_counts, label_properties). Use it; do not re-derive the whole meta-graph.
- Parse output with python3 (json.load) in a bash pipe.
- Writes (CREATE/MERGE/SET/DELETE) are blocked; do not attempt.
`;

const NODE_SCHEMA = {
  type: 'object',
  additionalProperties: false,
  required: ['labels'],
  properties: {
    labels: {
      type: 'array',
      items: {
        type: 'object',
        additionalProperties: false,
        required: ['label', 'spoke_count', 'identifier_scheme', 'name_property', 'synonyms', 'key_properties', 'data_sources', 'example_nodes', 'participating_reltypes', 'biookf_mapping', 'matching_notes'],
        properties: {
          label: { type: 'string' },
          spoke_count: { type: 'integer' },
          identifier_scheme: { type: 'string', description: 'what n.identifier holds: namespace/format + example, e.g. "NCBI Gene ID (bare integer, e.g. 351)"' },
          name_property: { type: 'string', description: 'what n.name holds and its convention' },
          synonyms: { type: 'string', description: 'which property holds synonyms/aliases and how populated' },
          key_properties: { type: 'array', items: { type: 'string' } },
          data_sources: { type: 'array', items: { type: 'string' }, description: 'source DBs/ontologies seen in source/sources/license props' },
          example_nodes: { type: 'array', items: { type: 'object', additionalProperties: true } },
          participating_reltypes: { type: 'array', items: { type: 'string' }, description: 'reltypes where this label is an endpoint' },
          biookf_mapping: { type: 'string', description: 'which BioOKF type(s) map to this label and why; note collisions/ambiguity' },
          matching_notes: { type: 'string', description: 'concrete strategy to match a BioOKF node NAME to this label: exact name, which synonym field, normalization (case/greek/hyphen), pitfalls' },
        },
      },
    },
  },
};

const EDGE_SCHEMA = {
  type: 'object',
  additionalProperties: false,
  required: ['reltypes'],
  properties: {
    reltypes: {
      type: 'array',
      items: {
        type: 'object',
        additionalProperties: false,
        required: ['reltype', 'endpoints', 'count', 'meaning', 'edge_properties', 'edge_sources', 'example', 'biookf_predicate_mapping', 'direction_notes'],
        properties: {
          reltype: { type: 'string' },
          endpoints: { type: 'array', items: { type: 'string' }, description: 'e.g. ["Compound->Disease"]' },
          count: { type: 'integer' },
          meaning: { type: 'string' },
          edge_properties: { type: 'array', items: { type: 'string' } },
          edge_sources: { type: 'array', items: { type: 'string' } },
          example: { type: 'object', additionalProperties: true },
          biookf_predicate_mapping: { type: 'string', description: 'which BioOKF predicate(s) map to this reltype family and confidence' },
          direction_notes: { type: 'string', description: 'directionality vs BioOKF (which side is subject); symmetry' },
        },
      },
    },
  },
};

const BIOOKF_SCHEMA = {
  type: 'object',
  additionalProperties: false,
  required: ['items'],
  properties: {
    items: {
      type: 'array',
      items: {
        type: 'object',
        additionalProperties: false,
        required: ['name', 'kind', 'definition', 'benchmark_count', 'subtypes_seen', 'identifier_pattern', 'xref_namespaces', 'spoke_targets', 'mapping_strategy', 'pitfalls'],
        properties: {
          name: { type: 'string' },
          kind: { type: 'string', enum: ['node_type', 'predicate'] },
          definition: { type: 'string', description: 'authoritative def distilled from SCHEMA.md/SPEC.md' },
          benchmark_count: { type: 'integer', description: 'occurrences in benchmark-biored (0 if absent)' },
          subtypes_seen: { type: 'array', items: { type: 'string' } },
          identifier_pattern: { type: 'string', description: 'how identifier/name looks in the benchmark' },
          xref_namespaces: { type: 'array', items: { type: 'string' }, description: 'canonical xref namespaces per SCHEMA + what actually appears in benchmark' },
          spoke_targets: { type: 'array', items: { type: 'string' }, description: 'candidate SPOKE label(s) or reltype(s) this maps to' },
          mapping_strategy: { type: 'string', description: 'waterfall strategy: when direct (type+name/synonym), when needs LLM/semantic' },
          pitfalls: { type: 'string', description: 'collisions, polarity/negation, direction, subtype-dependent target selection' },
        },
      },
    },
  },
};

// ---- SPOKE node label groups ----
const NODE_GROUPS = [
  { label: 'nodes:gene', items: ['Gene', 'PanGene', 'MiRNA'] },
  { label: 'nodes:protein', items: ['Protein', 'ProteinDomain', 'ProteinFamily', 'Complex'] },
  { label: 'nodes:chem', items: ['Compound', 'PharmacologicClass'] },
  { label: 'nodes:clinical', items: ['Disease', 'Symptom', 'SideEffect'] },
  { label: 'nodes:anatomy-cells', items: ['Anatomy', 'CellType', 'CellularComponent', 'AnatomyCellType', 'ProvCellType', 'CellLine'] },
  { label: 'nodes:variation', items: ['Variant', 'Haplotype', 'Chromosome', 'Cytoband'] },
  { label: 'nodes:function', items: ['Pathway', 'BiologicalProcess', 'MolecularFunction', 'Reaction', 'EC', 'PwGroup'] },
  { label: 'nodes:organism', items: ['Organism', 'SARSCov2'] },
  { label: 'nodes:nutrition', items: ['Food', 'Nutrient', 'DietarySupplement', 'Blend', 'ExtracellularParticle'] },
  { label: 'nodes:context', items: ['ClinicalLab', 'SDoH', 'Environment', 'Location', 'Version', 'DatabaseTimestamp', 'TestNode'] },
];

// ---- SPOKE reltype groups (by domain) ----
const EDGE_GROUPS = [
  { label: 'edges:assoc', items: ['ASSOCIATES_DaG','ASSOCIATES_GaS','ASSOCIATES_SaD','ASSOCIATES_VaP','AFFECTS_CamG','AFFECT_VaC','ADVRESPONSE_TO_mGarC','CORRELATES_AcA','RESEMBLES_DrD','SAME_CLnsCLn'] },
  { label: 'edges:binding', items: ['BINDS_CbP','BINDS_CbPD','INTERACTS_CiC','INTERACTS_CiF','INTERACTS_PDiPD','INTERACTS_PiC','INTERACTS_PiP','INTERACTS_as_LR','CLEAVESTO_PctP','TARGETS_MtG','TRANSPORTS_PtC'] },
  { label: 'edges:therapy', items: ['TREATS_CtD','CONTRAINDICATES_CcD','CAUSES_CcSE','CAUSES_OcD','IN_CLINICAL_TRIALS_FOR_CictD','RESPONDS_TO_OrC','RESISTANT_TO_mGrC','REDUCES_SEN_mGrsC','RESPONSE_TO_mGrC','CONTRAINDICATES_CcD'] },
  { label: 'edges:metabolic', items: ['ENCODES_GeM','ENCODES_GeP','ENCODES_OeP','ENCODES_PGeP','PARTICIPATES_CpR','PARTICIPATES_GpBP','PARTICIPATES_GpCC','PARTICIPATES_GpMF','PARTICIPATES_GpPW','PARTICIPATES_GpR','PARTICIPATES_PGpR','PARTICIPATES_PpR','CATALYZES_ECcR','CONSUMES_RcC','PRODUCES_RpC','MEMBEROF_PDmPF'] },
  { label: 'edges:regulation', items: ['DOWNREGULATES_AdG','DOWNREGULATES_CdG','DOWNREGULATES_GPdG','DOWNREGULATES_KGdG','DOWNREGULATES_OGdG','UPREGULATES_AuG','UPREGULATES_CuG','UPREGULATES_GPuG','UPREGULATES_KGuG','UPREGULATES_OGuG','REGULATES_PrG','MARKER_NEG_GmnD','MARKER_OF_GmoPCT','MARKER_POS_GmpD','ENRICHED_IN_GeiCT','ENRICHED_IN_GeiPCT','EXPRESSES_AeG'] },
  { label: 'edges:localization', items: ['EXPRESSEDIN_GeiCT','EXPRESSEDIN_GeiD','EXPRESSEDIN_PeCL','EXPRESSEDIN_PeCT','LOCALIZES_DlA','FOUNDIN_CfL','FOUNDIN_EfL','ISOLATEDIN_OiL','DECREASEDIN_PdD','INCREASEDIN_PiD','PRESENTS_DpS','IMPACTS_PiD'] },
  { label: 'edges:hierarchy', items: ['ISA_AiA','ISA_CLiCL','ISA_CLniCLn','ISA_CTiCT','ISA_CiC','ISA_DiD','ISA_ECiEC','ISA_EiE','ISA_FiF','ISA_OiO','ISA_PCTiCT','ISA_PCTiPCT','ISA_PWiPW','ISA_SiS','PARTOF_ApA','PARTOF_CTpA','PARTOF_CpC','PARTOF_CpPG','PARTOF_GpPG','PARTOF_LpL','PARTOF_PDpP','PARTOF_PGpPG','PARTOF_PpCP','PARTOF_PpPG','PARTOF_RpPW'] },
  { label: 'edges:membership', items: ['CONTAINS_BcC','CONTAINS_BcF','CONTAINS_CcC','CONTAINS_CcG','CONTAINS_DScB','CONTAINS_DScC','CONTAINS_FcC','INCLUDES_PCiC','BELONGS_HbG','BELONGS_VbG','BELONGS_VbH','HAS_CLnhV','HAS_PhEC','HASROLE_ChC','MAPS_VmG','DERIVES_FROM_CLndD'] },
  { label: 'edges:measurement', items: ['MEASURES_CLmA','MEASURES_CLmBP','MEASURES_CLmC','MEASURES_CLmCC','MEASURES_CLmCT','MEASURES_CLmMF','MEASURES_CLmO','MEASURES_CLmP','MEASURESIN_CLmA','MEASURESIN_CLmCT','PREVALENCE_DpL','PREVALENCEIN_SpL','MORTALITY_DmL'] },
];

// ---- BioOKF type/predicate groups ----
const BIOOKF_GROUPS = [
  { label: 'biookf:molecular', items: 'Gene, Molecule, MolecularClass, Variant, SequenceFeature, Structure (node types)' },
  { label: 'biookf:biocontext', items: 'Anatomy, CellType, Organism, BiologicalPathway, BiologicalFunction (node types)' },
  { label: 'biookf:clinical', items: 'Disease, Phenotype, BiomedicalMeasure, MethodOrProcedure, Exposure, SocialFactor, Food, Device, MaterialSample (node types)' },
  { label: 'biookf:provenance', items: 'Publication, Study, Dataset, Agent, Population, GeographicLocation, Concept, Other (node types)' },
  { label: 'biookf:predicates-structural', items: 'PREDICATES: is_a, part_of, member_of, derives_from, located_in, expressed_in, encodes, reported_in, used_to_study, measures (+ their negations where applicable)' },
  { label: 'biookf:predicates-effect', items: 'PREDICATES: interacts_with, binds, regulates, catalyzes, converts_to, participates_in, causes, predisposes_to, treats, prevents, contraindicated_in, affects_response_to, has_phenotype, associated_with (+ all 11 not_* negatives)' },
];

function nodePrompt(items) {
  return `You are a biomedical knowledge-graph analyst. Characterize these SPOKE node labels by RUNNING REAL QUERIES: ${items.join(', ')}.
${COOKBOOK}
For EACH label, investigate empirically and fill the schema:
1. Confirm spoke_count from ${SKELETON}.
2. Identify the identifier scheme: query several real nodes, inspect n.identifier — what namespace/format is it (NCBIGene int, DOID CURIE, InChIKey, UniProt accession, UBERON, dbSNP rsID, NCBITaxon, etc.)? Give a concrete example.
3. name_property: what does n.name hold (symbol? full label? scientific name?).
4. synonyms: which property carries synonyms/aliases (synonyms, aliases, xrefs, mesh_list, omim_list, mesh_id) and is it populated?
5. key_properties: from keys(n) across a few samples.
6. data_sources: read source/sources/license props to list contributing DBs/ontologies.
7. example_nodes: 3 real nodes {identifier, name, synonyms sample}.
8. participating_reltypes: from ${SKELETON} relationship_signatures, which reltypes have this label as an endpoint.
9. biookf_mapping: which of the 28 BioOKF types map here (read ${SCHEMA_MD} typing cheatsheet). Note ambiguity (e.g. BioOKF Molecule -> Compound OR Protein by subtype).
10. matching_notes: concrete strategy to match a BioOKF node's NAME/synonyms to this label — exact name match, which synonym field to search, normalization (lowercasing, greek letters, hyphens/spaces, salt forms), and false-match pitfalls.
Return ONLY the structured object. Ground every claim in an actual query you ran.`;
}

function edgePrompt(items) {
  return `You are a biomedical knowledge-graph analyst. Characterize these SPOKE relationship types by RUNNING REAL QUERIES: ${items.join(', ')}.
${COOKBOOK}
SPOKE reltype names encode endpoints as VERB_XyZ (capitals = label abbreviations). The authoritative endpoint signature and count for each is in ${SKELETON}. For EACH reltype:
1. endpoints + count from the skeleton (verify with a sample query).
2. meaning: plain-English semantics of the relationship.
3. edge_properties: run MATCH (a)-[r:RELTYPE]->(b) RETURN keys(r) LIMIT 3 to list edge property keys.
4. edge_sources: inspect r.sources / r.source / provenance props for contributing DBs.
5. example: one real edge {a_name, b_name} (query real endpoints).
6. biookf_predicate_mapping: which BioOKF predicate(s) (is_a, part_of, treats, causes, regulates, binds, associated_with, expressed_in, located_in, encodes, participates_in, catalyzes, predisposes_to, prevents, contraindicated_in, affects_response_to, has_phenotype, measures, member_of, derives_from, converts_to, interacts_with, used_to_study, reported_in) map to this reltype, and confidence. Note that BioOKF collapses UP/DOWNREGULATES into 'regulates', and maps many association-like edges to 'associated_with'.
7. direction_notes: which endpoint is the BioOKF subject (BioOKF direction is always this-doc -> object); note symmetry.
Return ONLY the structured object. Ground every claim in an actual query.`;
}

function biookfPrompt(items) {
  return `You are a biomedical ontology analyst. Characterize these BioOKF ${items} .
Authoritative sources to READ:
- ${SCHEMA_MD} (operating schema v0.5: 28 node types, 24 positive + 11 not_* predicates, typing cheatsheet, domain/range).
- ${SPEC_MD} (normative spec; read the relevant sections).
- Benchmark manifestation: ${BENCH}/biookf-export.json (key 'pages' = full node docs keyed by identifier, each with node_type, subtype, xref, synonyms, in_taxon, edges; key 'graph' = nodes/edges). Use python3 to compute real distributions.
For EACH item fill the schema:
1. definition distilled from SCHEMA/SPEC.
2. benchmark_count: real count in benchmark-biored (nodes of that type, or edges of that predicate).
3. subtypes_seen: actual subtype tokens used in the benchmark for this type (or n/a for predicates).
4. identifier_pattern: how identifier/name looks in benchmark (human-readable label; note xrefs are essentially ABSENT for biomedical entities in this benchmark — only Publications carry PMIDs).
5. xref_namespaces: canonical namespaces per SCHEMA cheatsheet AND what actually appears in benchmark.
6. spoke_targets: candidate SPOKE label(s)/reltype(s) this maps to (cross-reference SPOKE labels: Gene, Protein, Compound, Disease, Symptom, SideEffect, Anatomy, CellType, Organism, Variant, Pathway, BiologicalProcess, MolecularFunction, Reaction, EC, PharmacologicClass, ProteinFamily, ProteinDomain, Food, Nutrient, etc.).
7. mapping_strategy: apply the WATERFALL philosophy — when can we auto-map (type match + exact name or strong synonym/semantic match) vs when must we defer to LLM/semantic adjudication.
8. pitfalls: type collisions (Disease vs Phenotype vs BiomedicalMeasure), subtype-dependent target (Molecule->Compound vs Protein), negation/polarity (not_* predicates must only contradict when a positive SPOKE edge exists), direction, provenance types (Publication/Study/Dataset/Agent) that have NO SPOKE equivalent.
Return ONLY the structured object.`;
}

phase('SPOKE-nodes');
const nodeThunks = NODE_GROUPS.map(g => () =>
  agent(nodePrompt(g.items), { label: g.label, phase: 'SPOKE-nodes', schema: NODE_SCHEMA }));
const edgeThunks = EDGE_GROUPS.map(g => () =>
  agent(edgePrompt(g.items), { label: g.label, phase: 'SPOKE-edges', schema: EDGE_SCHEMA }));
const biookfThunks = BIOOKF_GROUPS.map(g => () =>
  agent(biookfPrompt(g.items), { label: g.label, phase: 'BioOKF-types', schema: BIOOKF_SCHEMA }));

// Run all three tracks concurrently (independent). Barrier to collect the full corpus.
const [nodeRes, edgeRes, biookfRes] = await parallel([
  () => parallel(nodeThunks),
  () => parallel(edgeThunks),
  () => parallel(biookfThunks),
]);

const corpus = {
  spoke_nodes: (nodeRes || []).filter(Boolean).flatMap(r => r.labels || []),
  spoke_edges: (edgeRes || []).filter(Boolean).flatMap(r => r.reltypes || []),
  biookf: (biookfRes || []).filter(Boolean).flatMap(r => r.items || []),
};
log(`research corpus: ${corpus.spoke_nodes.length} labels, ${corpus.spoke_edges.length} reltypes, ${corpus.biookf.length} biookf items`);
return corpus;
