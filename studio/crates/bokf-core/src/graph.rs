//! Derive a render-ready graph from a bundle's `edges:`. Edge objects resolve to
//! either an in-bundle node (by `identifier`) or an external CURIE stub. Inverse
//! (deprecated) edges are flipped so every rendered edge points subject->object.

use crate::bundle::Bundle;
use crate::model::Predicate;
use serde::Serialize;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize)]
pub struct GraphNode {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub subtype: Option<String>,
    pub label: String,
    pub color: String,
    pub category: String,
    pub path: Option<String>,
    /// True for a CURIE referenced by an edge that has no concept document.
    pub external: bool,
    pub degree: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub predicate: String,
    pub symmetric: bool,
    /// True for implicit `reported_in` edges synthesized from `primary_source`
    /// so provenance is traversable/visible (rendered subtly by the UI).
    #[serde(default)]
    pub synthesized: bool,
    pub knowledge_level: Option<String>,
    pub agent_type: Option<String>,
    pub primary_source: Option<String>,
    #[serde(skip_serializing_if = "serde_json::Map::is_empty")]
    pub stats: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Graph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

fn ensure_external(
    nodes: &mut Vec<GraphNode>,
    index: &mut HashMap<String, usize>,
    external: &mut HashSet<String>,
    id: &str,
) {
    if index.contains_key(id) {
        return;
    }
    external.insert(id.to_string());
    index.insert(id.to_string(), nodes.len());
    nodes.push(GraphNode {
        id: id.to_string(),
        node_type: "External".to_string(),
        subtype: None,
        label: id.to_string(),
        color: "#B6BBC4".to_string(),
        category: "external".to_string(),
        path: None,
        external: true,
        degree: 0,
    });
}

impl Graph {
    pub fn from_bundle(bundle: &Bundle) -> Graph {
        let mut nodes: Vec<GraphNode> = Vec::new();
        let mut index: HashMap<String, usize> = HashMap::new();
        let mut external: HashSet<String> = HashSet::new();

        // Real nodes first.
        for n in &bundle.nodes {
            index.insert(n.identifier.clone(), nodes.len());
            nodes.push(GraphNode {
                id: n.identifier.clone(),
                node_type: n.node_type.as_str().to_string(),
                subtype: n.subtype.clone(),
                label: n.identifier.clone(),
                color: n.node_type.color().to_string(),
                category: n.node_type.category().to_string(),
                path: Some(n.path.to_string_lossy().to_string()),
                external: false,
                degree: 0,
            });
        }

        let mut edges: Vec<GraphEdge> = Vec::new();
        let mut explicit_reported: HashSet<(String, String)> = HashSet::new();
        for n in &bundle.nodes {
            for e in &n.edges {
                ensure_external(&mut nodes, &mut index, &mut external, &e.object);

                // Forward orientation: subject -> object, flipping deprecated inverses.
                let (source, target) = if e.reversed {
                    (e.object.clone(), n.identifier.clone())
                } else {
                    (n.identifier.clone(), e.object.clone())
                };
                if e.predicate.as_str() == "reported_in" {
                    explicit_reported.insert((source.clone(), target.clone()));
                }

                edges.push(GraphEdge {
                    source,
                    target,
                    predicate: e.predicate.as_str().to_string(),
                    symmetric: e.predicate.is_symmetric(),
                    synthesized: false,
                    knowledge_level: e.knowledge_level.clone(),
                    agent_type: e.agent_type.clone(),
                    primary_source: e.primary_source.clone(),
                    stats: e.stats.clone().into_iter().collect(),
                });
            }
        }

        // Synthesize implicit `reported_in` edges from `primary_source` so source
        // nodes connect to what cites them (one per distinct subject->source pair).
        let mut synth: HashSet<(String, String)> = HashSet::new();
        for n in &bundle.nodes {
            for e in &n.edges {
                let ps = match &e.primary_source {
                    Some(p) if p != "not_provided" && *p != n.identifier => p.clone(),
                    _ => continue,
                };
                let pair = (n.identifier.clone(), ps.clone());
                if explicit_reported.contains(&pair) || !synth.insert(pair.clone()) {
                    continue;
                }
                ensure_external(&mut nodes, &mut index, &mut external, &ps);
                edges.push(GraphEdge {
                    source: pair.0,
                    target: pair.1,
                    predicate: "reported_in".to_string(),
                    symmetric: false,
                    synthesized: true,
                    knowledge_level: None,
                    agent_type: None,
                    primary_source: None,
                    stats: serde_json::Map::new(),
                });
            }
        }

        // Degree counts.
        for e in &edges {
            if let Some(&i) = index.get(&e.source) {
                nodes[i].degree += 1;
            }
            if let Some(&i) = index.get(&e.target) {
                nodes[i].degree += 1;
            }
        }

        Graph { nodes, edges }
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }

    /// Identifiers of nodes with no incident edges (orphans).
    pub fn orphans(&self) -> Vec<String> {
        self.nodes
            .iter()
            .filter(|n| !n.external && n.degree == 0)
            .map(|n| n.id.clone())
            .collect()
    }
}

/// Build an adjacency map (undirected) for traversal/queries.
pub fn adjacency(bundle: &Bundle) -> HashMap<String, Vec<(String, Predicate)>> {
    let mut adj: HashMap<String, Vec<(String, Predicate)>> = HashMap::new();
    for n in &bundle.nodes {
        for e in &n.edges {
            let (a, b) = if e.reversed {
                (e.object.clone(), n.identifier.clone())
            } else {
                (n.identifier.clone(), e.object.clone())
            };
            adj.entry(a.clone()).or_default().push((b.clone(), e.predicate.clone()));
            adj.entry(b).or_default().push((a, e.predicate.clone()));
        }
    }
    adj
}
