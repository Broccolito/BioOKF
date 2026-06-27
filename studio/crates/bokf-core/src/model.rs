//! BioOKF v0.5 data model: the 28 controlled node types, 23 forward-only edge
//! predicates, the provenance enums, and the in-memory `Node` / `Edge` structs.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// The 28 controlled node types (20 biomedical entities + 8 provenance/context),
/// plus an `Unknown` escape so an invalid `type:` can still be parsed and then
/// flagged by the linter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    // --- Biomedical entities (20) ---
    Gene,
    Molecule,
    MolecularClass,
    Variant,
    SequenceFeature,
    Structure,
    Anatomy,
    CellType,
    Organism,
    BiologicalPathway,
    BiologicalFunction,
    Disease,
    Phenotype,
    BiomedicalMeasure,
    MethodOrProcedure,
    Exposure,
    SocialFactor,
    Food,
    Device,
    MaterialSample,
    // --- Provenance & context (8) ---
    Publication,
    Study,
    Dataset,
    Agent,
    Population,
    GeographicLocation,
    Concept,
    Other,
    /// Anything that is not one of the 28 controlled values (lint error).
    Unknown(String),
}

/// The canonical 28 type names, in spec order.
pub const NODE_TYPES: [&str; 28] = [
    "Gene", "Molecule", "MolecularClass", "Variant", "SequenceFeature", "Structure", "Anatomy",
    "CellType", "Organism", "BiologicalPathway", "BiologicalFunction", "Disease", "Phenotype",
    "BiomedicalMeasure", "MethodOrProcedure", "Exposure", "SocialFactor", "Food", "Device",
    "MaterialSample", "Publication", "Study", "Dataset", "Agent", "Population",
    "GeographicLocation", "Concept", "Other",
];

/// Source-node types that may serve as a `primary_source` / `reported_in` target.
pub const PROVENANCE_TYPES: [&str; 4] = ["Publication", "Study", "Dataset", "Agent"];

impl NodeType {
    /// Parse a (possibly deprecated) `type:` token, normalizing v0.4/earlier
    /// aliases to their v0.5 canonical name. Unknown stays `Unknown`.
    pub fn parse(raw: &str) -> NodeType {
        let t = raw.trim();
        match t {
            "Gene" => NodeType::Gene,
            "Molecule" => NodeType::Molecule,
            "MolecularClass" => NodeType::MolecularClass,
            "Variant" => NodeType::Variant,
            "SequenceFeature" => NodeType::SequenceFeature,
            "Structure" => NodeType::Structure,
            "Anatomy" => NodeType::Anatomy,
            "CellType" => NodeType::CellType,
            "Organism" => NodeType::Organism,
            "BiologicalPathway" => NodeType::BiologicalPathway,
            "BiologicalFunction" => NodeType::BiologicalFunction,
            "Disease" => NodeType::Disease,
            "Phenotype" => NodeType::Phenotype,
            "BiomedicalMeasure" => NodeType::BiomedicalMeasure,
            "MethodOrProcedure" => NodeType::MethodOrProcedure,
            "Exposure" => NodeType::Exposure,
            "SocialFactor" => NodeType::SocialFactor,
            "Food" => NodeType::Food,
            "Device" => NodeType::Device,
            "MaterialSample" => NodeType::MaterialSample,
            "Publication" => NodeType::Publication,
            "Study" => NodeType::Study,
            "Dataset" => NodeType::Dataset,
            "Agent" => NodeType::Agent,
            "Population" => NodeType::Population,
            "GeographicLocation" => NodeType::GeographicLocation,
            "Concept" => NodeType::Concept,
            "Other" => NodeType::Other,
            // --- deprecated aliases (accept on read, normalize) ---
            "SDOH" | "SDoH" => NodeType::SocialFactor,
            "ClinicalMeasure" => NodeType::BiomedicalMeasure,
            "Procedure" | "Method" => NodeType::MethodOrProcedure,
            // Ambiguous aliases default to the most common resolution.
            "GenomicFeature" => NodeType::Variant,
            "Process" | "BiologicalProcess" => NodeType::BiologicalPathway,
            "ExposureOrFactor" => NodeType::Exposure,
            other => NodeType::Unknown(other.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            NodeType::Gene => "Gene",
            NodeType::Molecule => "Molecule",
            NodeType::MolecularClass => "MolecularClass",
            NodeType::Variant => "Variant",
            NodeType::SequenceFeature => "SequenceFeature",
            NodeType::Structure => "Structure",
            NodeType::Anatomy => "Anatomy",
            NodeType::CellType => "CellType",
            NodeType::Organism => "Organism",
            NodeType::BiologicalPathway => "BiologicalPathway",
            NodeType::BiologicalFunction => "BiologicalFunction",
            NodeType::Disease => "Disease",
            NodeType::Phenotype => "Phenotype",
            NodeType::BiomedicalMeasure => "BiomedicalMeasure",
            NodeType::MethodOrProcedure => "MethodOrProcedure",
            NodeType::Exposure => "Exposure",
            NodeType::SocialFactor => "SocialFactor",
            NodeType::Food => "Food",
            NodeType::Device => "Device",
            NodeType::MaterialSample => "MaterialSample",
            NodeType::Publication => "Publication",
            NodeType::Study => "Study",
            NodeType::Dataset => "Dataset",
            NodeType::Agent => "Agent",
            NodeType::Population => "Population",
            NodeType::GeographicLocation => "GeographicLocation",
            NodeType::Concept => "Concept",
            NodeType::Other => "Other",
            NodeType::Unknown(s) => s,
        }
    }

    pub fn is_valid(&self) -> bool {
        !matches!(self, NodeType::Unknown(_))
    }

    pub fn is_provenance(&self) -> bool {
        matches!(
            self,
            NodeType::Publication | NodeType::Study | NodeType::Dataset | NodeType::Agent
        )
    }

    /// Family used for hue grouping in the visualizer.
    pub fn category(&self) -> &'static str {
        match self {
            NodeType::Gene | NodeType::Variant | NodeType::SequenceFeature | NodeType::Structure => {
                "genomic"
            }
            NodeType::Molecule
            | NodeType::MolecularClass
            | NodeType::BiologicalPathway
            | NodeType::BiologicalFunction => "molecular",
            NodeType::Anatomy | NodeType::CellType | NodeType::Organism => "anatomy",
            NodeType::Disease
            | NodeType::Phenotype
            | NodeType::BiomedicalMeasure
            | NodeType::MethodOrProcedure => "clinical",
            NodeType::Exposure | NodeType::SocialFactor | NodeType::Food => "exposome",
            NodeType::Device | NodeType::MaterialSample => "physical",
            NodeType::Publication
            | NodeType::Study
            | NodeType::Dataset
            | NodeType::Agent
            | NodeType::Population
            | NodeType::GeographicLocation
            | NodeType::Concept
            | NodeType::Other => "provenance",
            NodeType::Unknown(_) => "unknown",
        }
    }

    /// Recommended hex color for the visualizer (the approved palette).
    pub fn color(&self) -> &'static str {
        match self {
            NodeType::Gene => "#6366A8",
            NodeType::Variant => "#8A86C4",
            NodeType::SequenceFeature => "#AAA6DA",
            NodeType::Structure => "#7C8FC9",
            NodeType::Molecule => "#2E8C84",
            NodeType::MolecularClass => "#5FB0A8",
            NodeType::BiologicalPathway => "#4FA38C",
            NodeType::BiologicalFunction => "#7CC3B0",
            NodeType::Anatomy => "#3F9E6E",
            NodeType::CellType => "#62B889",
            NodeType::Organism => "#8FCBA6",
            NodeType::Disease => "#C45B6B",
            NodeType::Phenotype => "#D98AA0",
            NodeType::BiomedicalMeasure => "#D98C5A",
            NodeType::MethodOrProcedure => "#C99750",
            NodeType::Exposure => "#B79A52",
            NodeType::SocialFactor => "#C2A86A",
            NodeType::Food => "#CBB87E",
            NodeType::Device => "#6E87A3",
            NodeType::MaterialSample => "#92A6BC",
            NodeType::Publication => "#6B7280",
            NodeType::Study => "#7A828E",
            NodeType::Dataset => "#88909C",
            NodeType::Agent => "#5E6672",
            NodeType::Population => "#7E8896",
            NodeType::GeographicLocation => "#94A0A0",
            NodeType::Concept => "#9AA0A8",
            NodeType::Other => "#AEB2B8",
            NodeType::Unknown(_) => "#D14B4B",
        }
    }
}

/// The forward-only edge predicates: 24 positive + 11 negative (`not_<X>`), plus
/// `Unknown` for an invalid token.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Predicate {
    IsA,
    PartOf,
    MemberOf,
    DerivesFrom,
    LocatedIn,
    ExpressedIn,
    Encodes,
    InteractsWith,
    Binds,
    Regulates,
    Catalyzes,
    ConvertsTo,
    ParticipatesIn,
    Causes,
    PredisposesTo,
    Treats,
    Prevents,
    ContraindicatedIn,
    AffectsResponseTo,
    HasPhenotype,
    Measures,
    AssociatedWith,
    UsedToStudy,
    ReportedIn,
    // --- negative (polarity) predicates: canonical `not_<X>` for the negatable set ---
    NotBinds,
    NotInteractsWith,
    NotCauses,
    NotPredisposesTo,
    NotPrevents,
    NotTreats,
    NotAffectsResponseTo,
    NotAssociatedWith,
    NotExpressedIn,
    NotRegulates,
    NotHasPhenotype,
    Unknown(String),
}

pub const PREDICATES: [&str; 35] = [
    "is_a", "part_of", "member_of", "derives_from", "located_in", "expressed_in", "encodes",
    "interacts_with", "binds", "regulates", "catalyzes", "converts_to", "participates_in", "causes",
    "predisposes_to", "treats", "prevents", "contraindicated_in", "affects_response_to",
    "has_phenotype", "measures", "associated_with", "used_to_study", "reported_in",
    // negative (polarity) predicates — canonical `not_<X>` for the 11 negatable predicates
    "not_binds", "not_interacts_with", "not_causes", "not_predisposes_to", "not_prevents",
    "not_treats", "not_affects_response_to", "not_associated_with", "not_expressed_in",
    "not_regulates", "not_has_phenotype",
];

/// The positive predicates that may carry a negation (curated; canonical form `not_<X>`).
/// Negating structural/definitional/provenance predicates is meaningless under
/// open-world semantics, so only these effect predicates are negatable.
pub const NEGATABLE: [&str; 11] = [
    "binds", "interacts_with", "causes", "predisposes_to", "prevents", "treats",
    "affects_response_to", "associated_with", "expressed_in", "regulates", "has_phenotype",
];

/// The symmetric predicates (rendered without a direction cue): positives + their negatives.
pub const SYMMETRIC: [&str; 4] =
    ["interacts_with", "associated_with", "not_interacts_with", "not_associated_with"];

/// Outcome of parsing a possibly-inverse predicate token.
pub struct PredicateParse {
    pub predicate: Predicate,
    /// True when the author used a deprecated inverse alias (e.g. `encoded_by`);
    /// the relationship's true direction is object -> this-document.
    pub reversed: bool,
}

impl Predicate {
    /// Parse a predicate token. Forward predicates map directly; the deprecated
    /// inverse aliases map to their forward predicate with `reversed = true`.
    pub fn parse(raw: &str) -> PredicateParse {
        let p = raw.trim();
        let fwd = |predicate| PredicateParse { predicate, reversed: false };
        let rev = |predicate| PredicateParse { predicate, reversed: true };
        match p {
            "is_a" => fwd(Predicate::IsA),
            "part_of" => fwd(Predicate::PartOf),
            "member_of" => fwd(Predicate::MemberOf),
            "derives_from" => fwd(Predicate::DerivesFrom),
            "located_in" => fwd(Predicate::LocatedIn),
            "expressed_in" => fwd(Predicate::ExpressedIn),
            "encodes" => fwd(Predicate::Encodes),
            "interacts_with" => fwd(Predicate::InteractsWith),
            "binds" => fwd(Predicate::Binds),
            "regulates" => fwd(Predicate::Regulates),
            "catalyzes" => fwd(Predicate::Catalyzes),
            "converts_to" => fwd(Predicate::ConvertsTo),
            "participates_in" => fwd(Predicate::ParticipatesIn),
            "causes" => fwd(Predicate::Causes),
            "predisposes_to" => fwd(Predicate::PredisposesTo),
            "treats" => fwd(Predicate::Treats),
            "prevents" => fwd(Predicate::Prevents),
            "contraindicated_in" => fwd(Predicate::ContraindicatedIn),
            "affects_response_to" => fwd(Predicate::AffectsResponseTo),
            "has_phenotype" => fwd(Predicate::HasPhenotype),
            "measures" => fwd(Predicate::Measures),
            "associated_with" => fwd(Predicate::AssociatedWith),
            "used_to_study" => fwd(Predicate::UsedToStudy),
            "reported_in" => fwd(Predicate::ReportedIn),
            // --- negative (polarity) predicates ---
            "not_binds" => fwd(Predicate::NotBinds),
            "not_interacts_with" => fwd(Predicate::NotInteractsWith),
            "not_causes" => fwd(Predicate::NotCauses),
            "not_predisposes_to" => fwd(Predicate::NotPredisposesTo),
            "not_prevents" => fwd(Predicate::NotPrevents),
            "not_treats" => fwd(Predicate::NotTreats),
            "not_affects_response_to" => fwd(Predicate::NotAffectsResponseTo),
            "not_associated_with" => fwd(Predicate::NotAssociatedWith),
            "not_expressed_in" => fwd(Predicate::NotExpressedIn),
            "not_regulates" => fwd(Predicate::NotRegulates),
            "not_has_phenotype" => fwd(Predicate::NotHasPhenotype),
            // --- deprecated inverse aliases: forward predicate, reversed direction ---
            "encoded_by" => rev(Predicate::Encodes),
            "caused_by" => rev(Predicate::Causes),
            "treated_by" => rev(Predicate::Treats),
            "produces" => rev(Predicate::Catalyzes),
            other => fwd(Predicate::Unknown(other.to_string())),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Predicate::IsA => "is_a",
            Predicate::PartOf => "part_of",
            Predicate::MemberOf => "member_of",
            Predicate::DerivesFrom => "derives_from",
            Predicate::LocatedIn => "located_in",
            Predicate::ExpressedIn => "expressed_in",
            Predicate::Encodes => "encodes",
            Predicate::InteractsWith => "interacts_with",
            Predicate::Binds => "binds",
            Predicate::Regulates => "regulates",
            Predicate::Catalyzes => "catalyzes",
            Predicate::ConvertsTo => "converts_to",
            Predicate::ParticipatesIn => "participates_in",
            Predicate::Causes => "causes",
            Predicate::PredisposesTo => "predisposes_to",
            Predicate::Treats => "treats",
            Predicate::Prevents => "prevents",
            Predicate::ContraindicatedIn => "contraindicated_in",
            Predicate::AffectsResponseTo => "affects_response_to",
            Predicate::HasPhenotype => "has_phenotype",
            Predicate::Measures => "measures",
            Predicate::AssociatedWith => "associated_with",
            Predicate::UsedToStudy => "used_to_study",
            Predicate::ReportedIn => "reported_in",
            Predicate::NotBinds => "not_binds",
            Predicate::NotInteractsWith => "not_interacts_with",
            Predicate::NotCauses => "not_causes",
            Predicate::NotPredisposesTo => "not_predisposes_to",
            Predicate::NotPrevents => "not_prevents",
            Predicate::NotTreats => "not_treats",
            Predicate::NotAffectsResponseTo => "not_affects_response_to",
            Predicate::NotAssociatedWith => "not_associated_with",
            Predicate::NotExpressedIn => "not_expressed_in",
            Predicate::NotRegulates => "not_regulates",
            Predicate::NotHasPhenotype => "not_has_phenotype",
            Predicate::Unknown(s) => s,
        }
    }

    pub fn is_valid(&self) -> bool {
        !matches!(self, Predicate::Unknown(_))
    }

    pub fn is_symmetric(&self) -> bool {
        matches!(
            self,
            Predicate::InteractsWith
                | Predicate::AssociatedWith
                | Predicate::NotInteractsWith
                | Predicate::NotAssociatedWith
        )
    }

    /// True for the `not_<X>` negative (polarity) predicates.
    pub fn is_negative(&self) -> bool {
        matches!(
            self,
            Predicate::NotBinds
                | Predicate::NotInteractsWith
                | Predicate::NotCauses
                | Predicate::NotPredisposesTo
                | Predicate::NotPrevents
                | Predicate::NotTreats
                | Predicate::NotAffectsResponseTo
                | Predicate::NotAssociatedWith
                | Predicate::NotExpressedIn
                | Predicate::NotRegulates
                | Predicate::NotHasPhenotype
        )
    }

    /// The positive predicate underlying a `not_<X>` (identity for positives/unknown).
    pub fn base(&self) -> Predicate {
        match self {
            Predicate::NotBinds => Predicate::Binds,
            Predicate::NotInteractsWith => Predicate::InteractsWith,
            Predicate::NotCauses => Predicate::Causes,
            Predicate::NotPredisposesTo => Predicate::PredisposesTo,
            Predicate::NotPrevents => Predicate::Prevents,
            Predicate::NotTreats => Predicate::Treats,
            Predicate::NotAffectsResponseTo => Predicate::AffectsResponseTo,
            Predicate::NotAssociatedWith => Predicate::AssociatedWith,
            Predicate::NotExpressedIn => Predicate::ExpressedIn,
            Predicate::NotRegulates => Predicate::Regulates,
            Predicate::NotHasPhenotype => Predicate::HasPhenotype,
            other => other.clone(),
        }
    }

    /// The negative form of a negatable positive predicate (`None` if not negatable).
    pub fn negated_form(&self) -> Option<Predicate> {
        Some(match self {
            Predicate::Binds => Predicate::NotBinds,
            Predicate::InteractsWith => Predicate::NotInteractsWith,
            Predicate::Causes => Predicate::NotCauses,
            Predicate::PredisposesTo => Predicate::NotPredisposesTo,
            Predicate::Prevents => Predicate::NotPrevents,
            Predicate::Treats => Predicate::NotTreats,
            Predicate::AffectsResponseTo => Predicate::NotAffectsResponseTo,
            Predicate::AssociatedWith => Predicate::NotAssociatedWith,
            Predicate::ExpressedIn => Predicate::NotExpressedIn,
            Predicate::Regulates => Predicate::NotRegulates,
            Predicate::HasPhenotype => Predicate::NotHasPhenotype,
            _ => return None,
        })
    }

    /// True for a positive predicate that may be negated.
    pub fn is_negatable(&self) -> bool {
        self.negated_form().is_some()
    }
}

/// Controlled provenance enums (Biolink).
pub const KNOWLEDGE_LEVELS: [&str; 5] = [
    "knowledge_assertion",
    "statistical_association",
    "prediction",
    "observation",
    "not_provided",
];
pub const AGENT_TYPES: [&str; 6] = [
    "manual_agent",
    "automated_agent",
    "text_mining_agent",
    "data_analysis_pipeline",
    "computational_model",
    "not_provided",
];

/// A typed relationship authored in a node's `edges:` list. Direction is always
/// host-document -> `object` (unless `reversed`, an accepted deprecated inverse).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub predicate: Predicate,
    pub raw_predicate: String,
    pub reversed: bool,
    pub object: String,
    pub knowledge_level: Option<String>,
    pub agent_type: Option<String>,
    pub primary_source: Option<String>,
    #[serde(default)]
    pub negated: bool,
    pub direction: Option<String>,
    #[serde(default)]
    pub publications: Vec<String>,
    /// Quantitative bundle (p_value, effect_size, effect_metric, ci_*, etc.).
    #[serde(default)]
    pub stats: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub qualifiers: BTreeMap<String, serde_json::Value>,
    pub note: Option<String>,
}

/// A typed concept document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub node_type: NodeType,
    pub raw_type: String,
    pub identifier: String,
    pub subtype: Option<String>,
    #[serde(default)]
    pub synonyms: Vec<String>,
    #[serde(default)]
    pub xref: Vec<String>,
    pub in_taxon: Option<String>,
    pub note: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub raw_source: Vec<String>,
    pub timestamp: Option<String>,
    #[serde(default)]
    pub edges: Vec<Edge>,
    #[serde(default)]
    pub body: String,
    /// Relative path of the source file within the bundle (display convenience).
    pub path: PathBuf,
    /// Preserved unknown frontmatter keys (so round-tripping never loses data).
    #[serde(default)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

impl Node {
    /// Text used for full-text (BM25) indexing.
    pub fn search_text(&self) -> String {
        let mut s = String::new();
        s.push_str(&self.identifier);
        s.push(' ');
        if let Some(st) = &self.subtype {
            s.push_str(st);
            s.push(' ');
        }
        for syn in &self.synonyms {
            s.push_str(syn);
            s.push(' ');
        }
        if let Some(d) = &self.description {
            s.push_str(d);
            s.push(' ');
        }
        s.push_str(&self.body);
        s
    }
}
