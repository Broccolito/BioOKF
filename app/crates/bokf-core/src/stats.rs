//! Graph filtering and summary statistics shared by the CLI, MCP server, and
//! Studio. Filters operate on the render-ready graph so external nodes and
//! synthesized provenance edges are handled the same way everywhere.

use crate::graph::{Graph, GraphEdge, GraphNode};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GraphFilter {
    #[serde(default)]
    pub node_types: Vec<String>,
    #[serde(default)]
    pub edge_predicates: Vec<String>,
    #[serde(default = "default_include_synthesized")]
    pub include_synthesized: bool,
}

fn default_include_synthesized() -> bool {
    true
}

impl GraphFilter {
    pub fn all() -> Self {
        Self {
            node_types: Vec::new(),
            edge_predicates: Vec::new(),
            include_synthesized: true,
        }
    }

    pub fn apply(&self, graph: &Graph) -> Graph {
        let node_set: HashSet<&str> = self.node_types.iter().map(String::as_str).collect();
        let edge_set: HashSet<&str> = self.edge_predicates.iter().map(String::as_str).collect();
        let nodes: Vec<GraphNode> = graph
            .nodes
            .iter()
            .filter(|n| node_set.is_empty() || node_set.contains(n.node_type.as_str()))
            .cloned()
            .collect();
        let visible_ids: HashSet<&str> = nodes.iter().map(|n| n.id.as_str()).collect();
        let edges: Vec<GraphEdge> = graph
            .edges
            .iter()
            .filter(|e| self.include_synthesized || !e.synthesized)
            .filter(|e| edge_set.is_empty() || edge_set.contains(e.predicate.as_str()))
            .filter(|e| {
                visible_ids.contains(e.source.as_str()) && visible_ids.contains(e.target.as_str())
            })
            .cloned()
            .collect();
        Graph { nodes, edges }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RankedNode {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub score: f64,
    pub degree: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComponentSummary {
    pub count: usize,
    pub largest: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct GraphStats {
    pub nodes: usize,
    pub edges: usize,
    pub by_node_type: BTreeMap<String, usize>,
    pub by_edge_predicate: BTreeMap<String, usize>,
    pub isolated_nodes: usize,
    pub density: f64,
    pub components: ComponentSummary,
    pub top_degree: Vec<RankedNode>,
    pub top_pagerank: Vec<RankedNode>,
    pub filter: GraphFilter,
}

impl GraphStats {
    pub fn from_graph(graph: &Graph, filter: &GraphFilter) -> Self {
        let filtered = filter.apply(graph);
        let degrees = degrees(&filtered);
        let pagerank = pagerank(&filtered, 30, 0.85);
        let mut by_node_type = BTreeMap::new();
        let mut by_edge_predicate = BTreeMap::new();

        for n in &filtered.nodes {
            *by_node_type.entry(n.node_type.clone()).or_default() += 1;
        }
        for e in &filtered.edges {
            *by_edge_predicate.entry(e.predicate.clone()).or_default() += 1;
        }

        let n = filtered.nodes.len();
        let e = filtered.edges.len();
        let density = if n > 1 {
            e as f64 / ((n * (n - 1)) as f64)
        } else {
            0.0
        };
        let components = components(&filtered);

        Self {
            nodes: n,
            edges: e,
            by_node_type,
            by_edge_predicate,
            isolated_nodes: filtered
                .nodes
                .iter()
                .filter(|node| degrees.get(node.id.as_str()).copied().unwrap_or(0) == 0)
                .count(),
            density,
            components,
            top_degree: ranked(&filtered, &degrees, 10),
            top_pagerank: ranked_float(&filtered, &pagerank, &degrees, 10),
            filter: filter.clone(),
        }
    }
}

fn degrees(graph: &Graph) -> HashMap<&str, usize> {
    let mut out: HashMap<&str, usize> = graph.nodes.iter().map(|n| (n.id.as_str(), 0)).collect();
    for e in &graph.edges {
        if let Some(v) = out.get_mut(e.source.as_str()) {
            *v += 1;
        }
        if let Some(v) = out.get_mut(e.target.as_str()) {
            *v += 1;
        }
    }
    out
}

fn pagerank(graph: &Graph, iterations: usize, damping: f64) -> HashMap<&str, f64> {
    let n = graph.nodes.len();
    if n == 0 {
        return HashMap::new();
    }
    let mut outgoing: HashMap<&str, Vec<&str>> = graph
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), Vec::new()))
        .collect();
    for e in &graph.edges {
        if let Some(list) = outgoing.get_mut(e.source.as_str()) {
            list.push(e.target.as_str());
        }
        if e.symmetric {
            if let Some(list) = outgoing.get_mut(e.target.as_str()) {
                list.push(e.source.as_str());
            }
        }
    }
    let base = (1.0 - damping) / n as f64;
    let mut rank: HashMap<&str, f64> = graph
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), 1.0 / n as f64))
        .collect();
    for _ in 0..iterations {
        let mut next: HashMap<&str, f64> = graph
            .nodes
            .iter()
            .map(|node| (node.id.as_str(), base))
            .collect();
        let sink_share: f64 = outgoing
            .iter()
            .filter(|(_, outs)| outs.is_empty())
            .map(|(id, _)| rank.get(id).copied().unwrap_or(0.0))
            .sum::<f64>()
            * damping
            / n as f64;
        for v in next.values_mut() {
            *v += sink_share;
        }
        for (id, outs) in &outgoing {
            if outs.is_empty() {
                continue;
            }
            let share = rank.get(id).copied().unwrap_or(0.0) * damping / outs.len() as f64;
            for target in outs {
                if let Some(v) = next.get_mut(target) {
                    *v += share;
                }
            }
        }
        rank = next;
    }
    rank
}

fn components(graph: &Graph) -> ComponentSummary {
    let mut adj: HashMap<&str, Vec<&str>> = graph
        .nodes
        .iter()
        .map(|n| (n.id.as_str(), Vec::new()))
        .collect();
    for e in &graph.edges {
        if adj.contains_key(e.source.as_str()) && adj.contains_key(e.target.as_str()) {
            adj.get_mut(e.source.as_str())
                .unwrap()
                .push(e.target.as_str());
            adj.get_mut(e.target.as_str())
                .unwrap()
                .push(e.source.as_str());
        }
    }
    let mut seen = HashSet::new();
    let mut count = 0usize;
    let mut largest = 0usize;
    for node in &graph.nodes {
        if !seen.insert(node.id.as_str()) {
            continue;
        }
        count += 1;
        let mut size = 0usize;
        let mut q = VecDeque::from([node.id.as_str()]);
        while let Some(id) = q.pop_front() {
            size += 1;
            for nb in adj.get(id).into_iter().flatten() {
                if seen.insert(*nb) {
                    q.push_back(*nb);
                }
            }
        }
        largest = largest.max(size);
    }
    ComponentSummary { count, largest }
}

fn ranked(graph: &Graph, degrees: &HashMap<&str, usize>, limit: usize) -> Vec<RankedNode> {
    let mut out: Vec<RankedNode> = graph
        .nodes
        .iter()
        .map(|n| {
            let d = degrees.get(n.id.as_str()).copied().unwrap_or(0);
            RankedNode {
                id: n.id.clone(),
                node_type: n.node_type.clone(),
                score: d as f64,
                degree: d,
            }
        })
        .collect();
    out.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.id.cmp(&b.id))
    });
    out.truncate(limit);
    out
}

fn ranked_float(
    graph: &Graph,
    scores: &HashMap<&str, f64>,
    degrees: &HashMap<&str, usize>,
    limit: usize,
) -> Vec<RankedNode> {
    let mut out: Vec<RankedNode> = graph
        .nodes
        .iter()
        .map(|n| RankedNode {
            id: n.id.clone(),
            node_type: n.node_type.clone(),
            score: scores.get(n.id.as_str()).copied().unwrap_or(0.0),
            degree: degrees.get(n.id.as_str()).copied().unwrap_or(0),
        })
        .collect();
    out.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.id.cmp(&b.id))
    });
    out.truncate(limit);
    out
}
