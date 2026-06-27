//! okf-core — the BioOKF backend library.
//!
//! Parses a BioOKF v0.5 bundle (a Git-shippable tree of Markdown files with YAML
//! frontmatter), derives a render-ready graph, lints it against the v0.5
//! conformance rules, and provides BM25 search. This is the single source of
//! truth shared by the CLI (`okf`), the MCP server (`okf-mcp`), and the Tauri
//! visualizer — the GUI is only a front-end over this backend.

pub mod bundle;
pub mod export;
pub mod git;
pub mod graph;
pub mod lint;
pub mod model;
pub mod parse;
pub mod search;
pub mod validate;

pub use bundle::Bundle;
pub use git::{ChangeKind, GitRepo, HistoryEntry, Txn};
pub use graph::{Graph, GraphEdge, GraphNode};
pub use lint::{lint, Finding, LintReport, Severity};
pub use model::{Edge, Node, NodeType, Predicate};
pub use parse::parse_node;
pub use search::{SearchHit, SearchIndex};

/// Open a bundle, returning the parsed `Bundle`.
pub fn open_bundle(root: impl AsRef<std::path::Path>) -> std::io::Result<Bundle> {
    Bundle::open(root)
}

/// Convenience: open a bundle and derive its graph.
pub fn graph_of(root: impl AsRef<std::path::Path>) -> std::io::Result<Graph> {
    Ok(Graph::from_bundle(&Bundle::open(root)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_frontmatter_split() {
        let content = "---\ntype: Gene\nidentifier: IL6\n---\n# IL6\nbody text";
        let (fm, body) = parse::split_frontmatter(content).unwrap();
        assert!(fm.contains("type: Gene"));
        assert!(body.starts_with("# IL6"));
    }

    #[test]
    fn parses_a_node_with_edges_and_normalizes_legacy() {
        let content = r#"---
type: Gene
title: IL6
id: HGNC:6018
xref: [NCBIGene:3569]
synonyms: [IL-6, interleukin 6]
edges:
  - predicate: encodes
    object: interleukin-6 (protein)
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: HGNC
  - predicate: caused_by
    object: SARS-CoV-2
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: MONDO
---
# IL6
The gene.
"#;
        let node = parse_node(content, std::path::Path::new("knowledge/gene/il6.md")).unwrap();
        assert_eq!(node.node_type, NodeType::Gene);
        // title+id merged: identifier=title, legacy id demoted to xref
        assert_eq!(node.identifier, "IL6");
        assert!(node.xref.contains(&"HGNC:6018".to_string()));
        assert_eq!(node.edges.len(), 2);
        // inverse predicate normalized to forward + reversed flag
        let caused = node.edges.iter().find(|e| e.raw_predicate == "caused_by").unwrap();
        assert_eq!(caused.predicate, Predicate::Causes);
        assert!(caused.reversed);
    }

    #[test]
    fn node_type_palette_is_complete() {
        for t in model::NODE_TYPES {
            let nt = NodeType::parse(t);
            assert!(nt.is_valid(), "{t} should be valid");
            assert!(nt.color().starts_with('#'));
        }
        assert_eq!(model::NODE_TYPES.len(), 28);
        assert_eq!(model::PREDICATES.len(), 24);
        let p = model::Predicate::parse("used_to_study");
        assert!(!p.reversed);
        assert_eq!(p.predicate.as_str(), "used_to_study");
        assert!(p.predicate.is_valid());
    }
}
