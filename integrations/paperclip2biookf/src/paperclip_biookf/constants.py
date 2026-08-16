"""BioOKF v0.5 controlled vocabulary and Paperclip extraction contract."""

NODE_TYPES = [
    "Gene", "Molecule", "MolecularClass", "Variant", "SequenceFeature",
    "Structure", "Anatomy", "CellType", "Organism", "BiologicalPathway",
    "BiologicalFunction", "Disease", "Phenotype", "BiomedicalMeasure",
    "MethodOrProcedure", "Exposure", "SocialFactor", "Food", "Device",
    "MaterialSample", "Publication", "Study", "Dataset", "Agent",
    "Population", "GeographicLocation", "Concept", "Other",
]

POSITIVE_PREDICATES = [
    "is_a", "part_of", "member_of", "derives_from", "located_in",
    "expressed_in", "encodes", "interacts_with", "binds", "regulates",
    "catalyzes", "converts_to", "participates_in", "causes",
    "predisposes_to", "treats", "prevents", "contraindicated_in",
    "affects_response_to", "has_phenotype", "measures", "associated_with",
    "used_to_study", "reported_in",
]

NEGATABLE = [
    "binds", "interacts_with", "causes", "predisposes_to", "prevents",
    "treats", "affects_response_to", "associated_with", "expressed_in",
    "regulates", "has_phenotype",
]
PREDICATES = POSITIVE_PREDICATES + ["not_" + value for value in NEGATABLE]

KNOWLEDGE_LEVELS = [
    "knowledge_assertion", "statistical_association", "prediction",
    "observation", "not_provided",
]

EXTRACTION_SCHEMA = {
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "type": "object",
    "required": ["nodes", "edges"],
    "additionalProperties": False,
    "properties": {
        "nodes": {
            "type": "array",
            "maxItems": 60,
            "items": {
                "type": "object",
                "required": [
                    "identifier", "type", "subtype", "description",
                    "synonyms", "xref",
                ],
                "additionalProperties": False,
                "properties": {
                    "identifier": {"type": "string", "minLength": 1},
                    "type": {"enum": NODE_TYPES},
                    "subtype": {"type": "string", "minLength": 1},
                    "description": {"type": "string"},
                    "synonyms": {"type": "array", "items": {"type": "string"}},
                    "xref": {"type": "array", "items": {"type": "string"}},
                },
            },
        },
        "edges": {
            "type": "array",
            "maxItems": 120,
            "items": {
                "type": "object",
                "required": [
                    "subject", "predicate", "object", "knowledge_level",
                    "statement", "evidence_lines", "effect_metric",
                    "effect_size", "ci_lower", "ci_upper", "p_value",
                    "sample_size", "direction",
                ],
                "additionalProperties": False,
                "properties": {
                    "subject": {"type": "string", "minLength": 1},
                    "predicate": {"enum": PREDICATES},
                    "object": {"type": "string", "minLength": 1},
                    "knowledge_level": {"enum": KNOWLEDGE_LEVELS},
                    "statement": {"type": "string", "minLength": 1},
                    "evidence_lines": {
                        "type": "array", "minItems": 1,
                        "items": {"type": "string", "pattern": "^L[0-9]+(?:-L?[0-9]+)?$"},
                    },
                    "effect_metric": {"type": ["string", "null"]},
                    "effect_size": {"type": ["number", "null"]},
                    "ci_lower": {"type": ["number", "null"]},
                    "ci_upper": {"type": ["number", "null"]},
                    "p_value": {"type": ["number", "null"]},
                    "sample_size": {"type": ["integer", "null"]},
                    "direction": {"type": ["string", "null"]},
                },
            },
        },
    },
}

EXTRACTION_PROMPT = """\
Curate this source into BioOKF v0.5 candidate facts.

Extract durable, reusable biomedical entities and atomic relations explicitly supported by
the source. Classify every entity by identity, not by its role. Use only the supplied node types
and predicates. A number or one-off phrase is edge data, never a node. Negative findings use only
the allowed not_<predicate> values. Include both positive and negative findings when supported.

Every edge subject and object must exactly match an identifier in nodes. Use the most specific
Paperclip evidence line or line range supporting the relation. The statement must be a concise,
source-faithful claim, not a general summary. Put effect estimates, confidence intervals, p-values,
sample sizes, and direction in their structured fields. Use null when unavailable. Do not infer
xrefs from memory: include an xref only when stated in the source. Do not create a Publication node;
the harness creates the source node from Paperclip metadata. Do not create reported_in edges;
the harness adds them deterministically.

Respect predicate semantics and domain/range. In particular, treats and prevents target a Disease
or Phenotype; prevents means prevention of that clinical outcome, not inhibition of a molecular
process. Model inhibition of a pathway/function with regulates plus a decreased/inhibitory
direction. Use associated_with when the source does not establish direction or causality. Never
upgrade association to causes, treats, or prevents.
"""
