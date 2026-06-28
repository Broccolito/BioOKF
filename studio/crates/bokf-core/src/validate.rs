//! Single-document validation (validate-before-write). Shared by the CLI
//! (`bokf validate`) and the MCP server (`bokf_validate_page`) so the agent can
//! check a draft before committing it.

use crate::model::*;
use crate::parse::parse_node;
use std::path::Path;

pub struct Validation {
    pub valid: bool,
    pub node_type: String,
    pub identifier: String,
    pub edge_count: usize,
    pub issues: Vec<String>,
}

/// Validate one concept-document's content. `valid` is false only on a hard
/// parse failure or an invalid controlled value (type/predicate/enum); softer
/// problems are returned as `issues`.
pub fn validate_doc(content: &str) -> Validation {
    match parse_node(content, Path::new("knowledge/_check.md")) {
        Err(e) => Validation {
            valid: false,
            node_type: String::new(),
            identifier: String::new(),
            edge_count: 0,
            issues: vec![format!("does not parse: {e}")],
        },
        Ok(n) => {
            let mut issues = Vec::new();
            let mut hard = false;
            if !n.node_type.is_valid() {
                issues.push(format!("type `{}` is not one of the 28 controlled types", n.raw_type));
                hard = true;
            }
            if n.identifier.contains(':') && !n.identifier.contains(' ') {
                issues.push("identifier looks like a bare CURIE; use a human-readable name and put the CURIE in xref".into());
            }
            if n.subtype.is_none() {
                issues.push("no subtype (expected, agent-coined)".into());
            }
            if n.node_type.is_provenance() && n.raw_source.is_empty() && n.xref.is_empty() {
                issues.push("source node has neither raw_source nor an external xref (unanchored)".into());
            }
            for e in &n.edges {
                let tag = format!("{} -> {}", e.predicate.as_str(), e.object);
                if !e.predicate.is_valid() {
                    issues.push(format!("edge {tag}: predicate `{}` is not one of the 23", e.raw_predicate));
                    hard = true;
                }
                match &e.knowledge_level {
                    None => { issues.push(format!("edge {tag}: missing knowledge_level")); hard = true; }
                    Some(v) if !KNOWLEDGE_LEVELS.contains(&v.as_str()) => { issues.push(format!("edge {tag}: invalid knowledge_level `{v}`")); hard = true; }
                    _ => {}
                }
                match &e.agent_type {
                    None => { issues.push(format!("edge {tag}: missing agent_type")); hard = true; }
                    Some(v) if !AGENT_TYPES.contains(&v.as_str()) => { issues.push(format!("edge {tag}: invalid agent_type `{v}`")); hard = true; }
                    _ => {}
                }
                if e.primary_source.is_none() {
                    issues.push(format!("edge {tag}: missing primary_source (must name a source node)"));
                    hard = true;
                }
                if matches!(e.predicate, Predicate::Regulates | Predicate::ExpressedIn) && e.direction.is_none() {
                    issues.push(format!("edge {tag}: `{}` should carry a direction", e.predicate.as_str()));
                }
            }
            Validation {
                valid: !hard,
                node_type: n.node_type.as_str().to_string(),
                identifier: n.identifier,
                edge_count: n.edges.len(),
                issues,
            }
        }
    }
}
