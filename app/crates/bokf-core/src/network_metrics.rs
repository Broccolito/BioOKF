//! Deterministic topology metrics over a simple undirected projection of a BioOKF graph.
//! Authored parallel edges are collapsed and self-loops/synthesized provenance edges are removed.

use crate::{Bundle, Graph, NodeType};
use leiden_rs::{GraphDataBuilder, Leiden, LeidenConfig};
use serde::Serialize;
use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap, HashMap, HashSet, VecDeque};
use std::path::{Component, Path};

#[derive(Debug, Clone, Copy, Default)]
pub struct NetworkOptions {
    pub exclude_provenance: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct GlobalMetrics {
    pub nodes: usize,
    pub edges: usize,
    pub density: f64,
    pub average_degree: f64,
    pub transitivity: f64,
    pub average_shortest_path_giant: Option<f64>,
    pub global_efficiency: f64,
    pub giant_component_fraction: f64,
    pub components: usize,
    pub degree_assortativity: Option<f64>,
    pub algebraic_connectivity: Option<f64>,
    pub modularity_leiden: Option<f64>,
    pub communities: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct NodeMetrics {
    pub id: String,
    pub node_type: String,
    pub degree: usize,
    pub betweenness: f64,
    pub clustering: f64,
    pub coreness: usize,
    pub community: Option<usize>,
    pub participation: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DegreeCcdfPoint {
    pub degree: usize,
    pub probability: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClusteringDegreePoint {
    pub degree: usize,
    pub mean_clustering: f64,
    pub nodes: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SourceYearPoint {
    pub year: u16,
    pub sources: usize,
    pub cumulative_sources: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct NetworkReport {
    pub projection: String,
    pub excluded_provenance: bool,
    pub global: GlobalMetrics,
    pub nodes: Vec<NodeMetrics>,
    pub degree_ccdf: Vec<DegreeCcdfPoint>,
    pub clustering_by_degree: Vec<ClusteringDegreePoint>,
    pub community_sizes: Vec<usize>,
    pub source_years: Vec<SourceYearPoint>,
    pub source_count: usize,
    pub sources_with_year: usize,
    pub sources_without_year: usize,
    pub notes: Vec<String>,
}

struct SimpleGraph {
    ids: Vec<String>,
    types: Vec<String>,
    adjacency: Vec<HashSet<usize>>,
    edges: Vec<(usize, usize)>,
}

fn is_provenance(node_type: &str) -> bool {
    matches!(node_type, "Publication" | "Study" | "Dataset" | "Agent")
}

fn source_node(node_type: &NodeType) -> bool {
    matches!(
        node_type,
        NodeType::Publication | NodeType::Study | NodeType::Dataset
    )
}

fn plausible_year(value: &serde_json::Value) -> Option<u16> {
    if let Some(number) = value.as_u64() {
        return (1500..=2100).contains(&number).then_some(number as u16);
    }
    if let Some(text) = value.as_str() {
        for token in text.split(|character: char| !character.is_ascii_digit()) {
            if token.len() == 4 {
                if let Ok(year) = token.parse::<u16>() {
                    if (1500..=2100).contains(&year) {
                        return Some(year);
                    }
                }
            }
        }
    }
    match value {
        serde_json::Value::Array(values) => values.iter().find_map(plausible_year),
        serde_json::Value::Object(values) => values.values().find_map(plausible_year),
        _ => None,
    }
}

fn explicit_year(value: &serde_json::Value) -> Option<u16> {
    const YEAR_KEYS: [&str; 13] = [
        "year",
        "publication_year",
        "published_year",
        "publication_date",
        "published",
        "published_at",
        "date_published",
        "issued",
        "date",
        "start_date",
        "study_start_date",
        "completion_date",
        "study_completion_date",
    ];
    let serde_json::Value::Object(values) = value else {
        return None;
    };
    for key in YEAR_KEYS {
        if let Some(year) = values.get(key).and_then(plausible_year) {
            return Some(year);
        }
    }
    values.values().find_map(explicit_year)
}

fn safe_raw_meta(root: &Path, raw_source: &str) -> Option<serde_json::Value> {
    let relative = Path::new(raw_source);
    let mut components = relative.components();
    if components.next() != Some(Component::Normal("raw".as_ref()))
        || components.any(|component| !matches!(component, Component::Normal(_)))
    {
        return None;
    }
    let raw_path = root.join(relative);
    let meta_path = if raw_path.file_name().and_then(|name| name.to_str()) == Some("meta.yaml") {
        raw_path
    } else if raw_path.is_dir() {
        raw_path.join("meta.yaml")
    } else {
        raw_path.parent()?.join("meta.yaml")
    };
    if std::fs::metadata(&meta_path).ok()?.len() > 1024 * 1024 {
        return None;
    }
    let content = std::fs::read_to_string(meta_path).ok()?;
    serde_yaml::from_str(&content).ok()
}

fn source_years(bundle: &Bundle) -> (Vec<SourceYearPoint>, usize, usize, usize) {
    let sources = bundle
        .nodes
        .iter()
        .filter(|node| source_node(&node.node_type))
        .collect::<Vec<_>>();
    let mut counts = BTreeMap::<u16, usize>::new();
    let mut unknown = 0usize;
    for node in &sources {
        let extra = serde_json::Value::Object(node.extra.clone().into_iter().collect());
        let year = explicit_year(&extra).or_else(|| {
            node.raw_source
                .iter()
                .filter_map(|path| safe_raw_meta(&bundle.root, path))
                .find_map(|metadata| explicit_year(&metadata))
        });
        if let Some(year) = year {
            *counts.entry(year).or_default() += 1;
        } else {
            unknown += 1;
        }
    }
    let mut cumulative = 0usize;
    let years = counts
        .into_iter()
        .map(|(year, count)| {
            cumulative += count;
            SourceYearPoint {
                year,
                sources: count,
                cumulative_sources: cumulative,
            }
        })
        .collect::<Vec<_>>();
    (years, sources.len(), sources.len() - unknown, unknown)
}

fn project(bundle: &Bundle, options: NetworkOptions) -> SimpleGraph {
    let graph = Graph::from_bundle(bundle);
    let kept: Vec<_> = graph
        .nodes
        .iter()
        .filter(|node| !node.external)
        .filter(|node| !options.exclude_provenance || !is_provenance(&node.node_type))
        .collect();
    let ids = kept.iter().map(|node| node.id.clone()).collect::<Vec<_>>();
    let types = kept
        .iter()
        .map(|node| node.node_type.clone())
        .collect::<Vec<_>>();
    let index: HashMap<&str, usize> = ids
        .iter()
        .enumerate()
        .map(|(i, id)| (id.as_str(), i))
        .collect();
    let mut unique = HashSet::new();
    for edge in graph.edges.iter().filter(|edge| !edge.synthesized) {
        let (Some(&a), Some(&b)) = (
            index.get(edge.source.as_str()),
            index.get(edge.target.as_str()),
        ) else {
            continue;
        };
        if a == b {
            continue;
        }
        unique.insert(if a < b { (a, b) } else { (b, a) });
    }
    let mut edges = unique.into_iter().collect::<Vec<_>>();
    edges.sort_unstable();
    let mut adjacency = vec![HashSet::new(); ids.len()];
    for &(a, b) in &edges {
        adjacency[a].insert(b);
        adjacency[b].insert(a);
    }
    SimpleGraph {
        ids,
        types,
        adjacency,
        edges,
    }
}

fn components(adjacency: &[HashSet<usize>]) -> Vec<Vec<usize>> {
    let mut seen = vec![false; adjacency.len()];
    let mut out = Vec::new();
    for start in 0..adjacency.len() {
        if seen[start] {
            continue;
        }
        let mut queue = VecDeque::from([start]);
        let mut component = Vec::new();
        seen[start] = true;
        while let Some(node) = queue.pop_front() {
            component.push(node);
            for &next in &adjacency[node] {
                if !seen[next] {
                    seen[next] = true;
                    queue.push_back(next);
                }
            }
        }
        out.push(component);
    }
    out.sort_by_key(|component| Reverse(component.len()));
    out
}

fn local_clustering(adjacency: &[HashSet<usize>]) -> Vec<f64> {
    adjacency
        .iter()
        .map(|neighbors| {
            let degree = neighbors.len();
            if degree < 2 {
                return 0.0;
            }
            let list = neighbors.iter().copied().collect::<Vec<_>>();
            let mut closed = 0usize;
            for i in 0..list.len() {
                for j in i + 1..list.len() {
                    if adjacency[list[i]].contains(&list[j]) {
                        closed += 1;
                    }
                }
            }
            closed as f64 / (degree * (degree - 1) / 2) as f64
        })
        .collect()
}

fn transitivity(adjacency: &[HashSet<usize>]) -> f64 {
    let triples: usize = adjacency
        .iter()
        .map(|neighbors| neighbors.len() * neighbors.len().saturating_sub(1) / 2)
        .sum();
    if triples == 0 {
        return 0.0;
    }
    let closed: usize = adjacency
        .iter()
        .map(|neighbors| {
            let list = neighbors.iter().copied().collect::<Vec<_>>();
            let mut count = 0;
            for i in 0..list.len() {
                for j in i + 1..list.len() {
                    count += usize::from(adjacency[list[i]].contains(&list[j]));
                }
            }
            count
        })
        .sum();
    closed as f64 / triples as f64
}

fn shortest_paths_and_betweenness(
    adjacency: &[HashSet<usize>],
    giant: &HashSet<usize>,
) -> (Option<f64>, f64, Vec<f64>) {
    let n = adjacency.len();
    let mut between = vec![0.0; n];
    let mut distance_sum_giant = 0.0;
    let mut distance_pairs_giant = 0usize;
    let mut inverse_distance_sum = 0.0;
    for source in 0..n {
        let mut stack = Vec::with_capacity(n);
        let mut predecessors = vec![Vec::new(); n];
        let mut sigma = vec![0.0; n];
        let mut distance = vec![-1i32; n];
        sigma[source] = 1.0;
        distance[source] = 0;
        let mut queue = VecDeque::from([source]);
        while let Some(v) = queue.pop_front() {
            stack.push(v);
            for &w in &adjacency[v] {
                if distance[w] < 0 {
                    distance[w] = distance[v] + 1;
                    queue.push_back(w);
                }
                if distance[w] == distance[v] + 1 {
                    sigma[w] += sigma[v];
                    predecessors[w].push(v);
                }
            }
        }
        for target in 0..n {
            if target == source || distance[target] <= 0 {
                continue;
            }
            inverse_distance_sum += 1.0 / distance[target] as f64;
            if giant.contains(&source) && giant.contains(&target) {
                distance_sum_giant += distance[target] as f64;
                distance_pairs_giant += 1;
            }
        }
        let mut dependency = vec![0.0; n];
        while let Some(w) = stack.pop() {
            for &v in &predecessors[w] {
                if sigma[w] > 0.0 {
                    dependency[v] += (sigma[v] / sigma[w]) * (1.0 + dependency[w]);
                }
            }
            if w != source {
                between[w] += dependency[w];
            }
        }
    }
    let normalization = if n > 2 {
        ((n - 1) * (n - 2)) as f64
    } else {
        1.0
    };
    for value in &mut between {
        *value /= normalization;
    }
    let average =
        (distance_pairs_giant > 0).then_some(distance_sum_giant / distance_pairs_giant as f64);
    let efficiency = if n > 1 {
        inverse_distance_sum / (n * (n - 1)) as f64
    } else {
        0.0
    };
    (average, efficiency, between)
}

fn assortativity(graph: &SimpleGraph) -> Option<f64> {
    if graph.edges.is_empty() {
        return None;
    }
    let m = graph.edges.len() as f64;
    let mut product = 0.0;
    let mut mean = 0.0;
    let mut squares = 0.0;
    for &(a, b) in &graph.edges {
        let j = graph.adjacency[a].len() as f64;
        let k = graph.adjacency[b].len() as f64;
        product += j * k;
        mean += 0.5 * (j + k);
        squares += 0.5 * (j * j + k * k);
    }
    let mean = mean / m;
    let denominator = squares / m - mean * mean;
    (denominator.abs() > 1e-12).then_some((product / m - mean * mean) / denominator)
}

fn coreness(adjacency: &[HashSet<usize>]) -> Vec<usize> {
    let n = adjacency.len();
    let mut degree = adjacency.iter().map(HashSet::len).collect::<Vec<_>>();
    let mut removed = vec![false; n];
    let mut core = vec![0; n];
    let mut heap = BinaryHeap::new();
    for (node, &value) in degree.iter().enumerate() {
        heap.push(Reverse((value, node)));
    }
    while let Some(Reverse((value, node))) = heap.pop() {
        if removed[node] || value != degree[node] {
            continue;
        }
        removed[node] = true;
        core[node] = value;
        for &next in &adjacency[node] {
            if !removed[next] && degree[next] > value {
                degree[next] -= 1;
                heap.push(Reverse((degree[next], next)));
            }
        }
    }
    core
}

fn algebraic_connectivity(adjacency: &[HashSet<usize>], component_count: usize) -> Option<f64> {
    let n = adjacency.len();
    if n < 2 {
        return None;
    }
    if component_count > 1 {
        return Some(0.0);
    }
    let max_degree = adjacency.iter().map(HashSet::len).max().unwrap_or(0);
    if max_degree == 0 {
        return Some(0.0);
    }
    let bound = (2 * max_degree) as f64;
    let mut x = (0..n)
        .map(|i| ((i * 37 + 11) % 101) as f64 - 50.0)
        .collect::<Vec<_>>();
    for _ in 0..300 {
        let mean = x.iter().sum::<f64>() / n as f64;
        for value in &mut x {
            *value -= mean;
        }
        let mut next = vec![0.0; n];
        for i in 0..n {
            let laplacian =
                adjacency[i].len() as f64 * x[i] - adjacency[i].iter().map(|&j| x[j]).sum::<f64>();
            next[i] = x[i] - laplacian / bound;
        }
        let norm = next.iter().map(|v| v * v).sum::<f64>().sqrt();
        if norm <= 1e-14 {
            break;
        }
        for value in &mut next {
            *value /= norm;
        }
        x = next;
    }
    let numerator: f64 = adjacency
        .iter()
        .enumerate()
        .map(|(i, neighbors)| {
            neighbors
                .iter()
                .filter(|&&j| i < j)
                .map(|&j| (x[i] - x[j]).powi(2))
                .sum::<f64>()
        })
        .sum();
    let denominator = x.iter().map(|value| value * value).sum::<f64>();
    Some((numerator / denominator).max(0.0))
}

fn leiden(graph: &SimpleGraph) -> (Option<f64>, Vec<Option<usize>>, Vec<usize>) {
    if graph.edges.is_empty() || graph.ids.is_empty() {
        return (None, vec![None; graph.ids.len()], Vec::new());
    }
    let mut builder = GraphDataBuilder::new(graph.ids.len());
    for &(a, b) in &graph.edges {
        if builder.add_edge(a, b, 1.0).is_err() {
            return (None, vec![None; graph.ids.len()], Vec::new());
        }
    }
    let Ok(data) = builder.build() else {
        return (None, vec![None; graph.ids.len()], Vec::new());
    };
    let mut config = LeidenConfig::default();
    config.seed = Some(42);
    config.skip_refinement = false;
    let Ok(output) = Leiden::new(config).run(&data) else {
        return (None, vec![None; graph.ids.len()], Vec::new());
    };
    let membership = output
        .partition
        .as_slice()
        .iter()
        .copied()
        .map(Some)
        .collect::<Vec<_>>();
    let mut sizes = output.partition.community_sizes();
    sizes.retain(|&size| size > 0);
    sizes.sort_by_key(|&size| Reverse(size));
    (Some(output.quality), membership, sizes)
}

pub fn analyze(bundle: &Bundle, options: NetworkOptions) -> Result<NetworkReport, String> {
    let graph = project(bundle, options);
    let n = graph.ids.len();
    let m = graph.edges.len();
    let component_list = components(&graph.adjacency);
    let giant: HashSet<usize> = component_list
        .first()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .collect();
    let clustering = local_clustering(&graph.adjacency);
    let (average_path, efficiency, betweenness) =
        shortest_paths_and_betweenness(&graph.adjacency, &giant);
    let cores = coreness(&graph.adjacency);
    let (modularity, communities, community_sizes) = leiden(&graph);
    let (source_years, source_count, sources_with_year, sources_without_year) =
        source_years(bundle);
    let participation = (0..n)
        .map(|node| {
            let degree = graph.adjacency[node].len();
            let own = communities.get(node).copied().flatten()?;
            if degree == 0 {
                return Some(0.0);
            }
            let mut counts = HashMap::<usize, usize>::new();
            for &neighbor in &graph.adjacency[node] {
                if let Some(community) = communities.get(neighbor).copied().flatten() {
                    *counts.entry(community).or_default() += 1;
                }
            }
            let _ = own;
            Some(
                1.0 - counts
                    .values()
                    .map(|&count| (count as f64 / degree as f64).powi(2))
                    .sum::<f64>(),
            )
        })
        .collect::<Vec<_>>();
    let mut nodes = (0..n)
        .map(|i| NodeMetrics {
            id: graph.ids[i].clone(),
            node_type: graph.types[i].clone(),
            degree: graph.adjacency[i].len(),
            betweenness: betweenness[i],
            clustering: clustering[i],
            coreness: cores[i],
            community: communities.get(i).copied().flatten(),
            participation: participation[i],
        })
        .collect::<Vec<_>>();
    nodes.sort_by(|a, b| {
        b.betweenness
            .total_cmp(&a.betweenness)
            .then_with(|| b.degree.cmp(&a.degree))
            .then_with(|| a.id.cmp(&b.id))
    });

    let mut degree_counts = BTreeMap::<usize, usize>::new();
    let mut clustering_degree = BTreeMap::<usize, (f64, usize)>::new();
    for i in 0..n {
        let degree = graph.adjacency[i].len();
        *degree_counts.entry(degree).or_default() += 1;
        let entry = clustering_degree.entry(degree).or_default();
        entry.0 += clustering[i];
        entry.1 += 1;
    }
    let degree_ccdf = degree_counts
        .keys()
        .map(|&degree| DegreeCcdfPoint {
            degree,
            probability: if n == 0 {
                0.0
            } else {
                degree_counts
                    .range(degree..)
                    .map(|(_, count)| count)
                    .sum::<usize>() as f64
                    / n as f64
            },
        })
        .collect();
    let clustering_by_degree = clustering_degree
        .into_iter()
        .map(|(degree, (sum, count))| ClusteringDegreePoint {
            degree,
            mean_clustering: sum / count as f64,
            nodes: count,
        })
        .collect();
    let possible_edges = n.saturating_mul(n.saturating_sub(1)) / 2;
    let global = GlobalMetrics {
        nodes: n,
        edges: m,
        density: if possible_edges > 0 {
            m as f64 / possible_edges as f64
        } else {
            0.0
        },
        average_degree: if n > 0 {
            2.0 * m as f64 / n as f64
        } else {
            0.0
        },
        transitivity: transitivity(&graph.adjacency),
        average_shortest_path_giant: average_path,
        global_efficiency: efficiency,
        giant_component_fraction: if n > 0 {
            giant.len() as f64 / n as f64
        } else {
            0.0
        },
        components: component_list.len(),
        degree_assortativity: assortativity(&graph),
        algebraic_connectivity: algebraic_connectivity(&graph.adjacency, component_list.len()),
        modularity_leiden: modularity,
        communities: community_sizes.len(),
    };
    Ok(NetworkReport {
        projection: "simple undirected; authored edges only; parallel edges collapsed; self-loops removed; external nodes excluded".into(),
        excluded_provenance: options.exclude_provenance,
        global,
        nodes,
        degree_ccdf,
        clustering_by_degree,
        community_sizes,
        source_years,
        source_count,
        sources_with_year,
        sources_without_year,
        notes: vec![
            "Average shortest path is computed on the giant component; global efficiency includes disconnected pairs as zero.".into(),
            "Leiden modularity uses resolution 1.0 and deterministic seed 42.".into(),
            "Degree is shown as a CCDF; no power-law fit is inferred.".into(),
            "Source years use explicit Publication, Study, or Dataset metadata; undated sources remain unknown rather than being inferred from free text.".into(),
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_metrics_are_sane() {
        let adjacency = vec![
            HashSet::from([1]),
            HashSet::from([0, 2]),
            HashSet::from([1]),
        ];
        let giant = HashSet::from([0, 1, 2]);
        let (path, efficiency, between) = shortest_paths_and_betweenness(&adjacency, &giant);
        assert!((path.unwrap() - 4.0 / 3.0).abs() < 1e-9);
        assert!((efficiency - 5.0 / 6.0).abs() < 1e-9);
        assert!((between[1] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn triangle_has_unit_transitivity() {
        let adjacency = vec![
            HashSet::from([1, 2]),
            HashSet::from([0, 2]),
            HashSet::from([0, 1]),
        ];
        assert!((transitivity(&adjacency) - 1.0).abs() < 1e-9);
        assert!(local_clustering(&adjacency)
            .iter()
            .all(|value| (*value - 1.0).abs() < 1e-9));
    }

    #[test]
    fn source_year_distribution_uses_explicit_metadata_and_tracks_unknowns() {
        let root = tempfile::tempdir().unwrap();
        let publications = root.path().join("knowledge/publication");
        let studies = root.path().join("knowledge/study");
        std::fs::create_dir_all(&publications).unwrap();
        std::fs::create_dir_all(&studies).unwrap();
        std::fs::write(
            publications.join("a.md"),
            "---\ntype: Publication\nidentifier: A\npublication_date: 2020-05-01\n---\n# A\n",
        )
        .unwrap();
        std::fs::create_dir_all(root.path().join("raw/b")).unwrap();
        std::fs::write(
            root.path().join("raw/b/meta.yaml"),
            "title: B\npublished_at: '2022-07-14'\n",
        )
        .unwrap();
        std::fs::write(
            studies.join("b.md"),
            "---\ntype: Study\nidentifier: B\nraw_source: [raw/b/source.md]\n---\n# B\n",
        )
        .unwrap();
        std::fs::write(
            publications.join("unknown.md"),
            "---\ntype: Publication\nidentifier: Undated 1999 title\n---\n# Undated 1999 title\n",
        )
        .unwrap();
        let bundle = Bundle::open(root.path()).unwrap();
        let (years, total, known, unknown) = source_years(&bundle);
        assert_eq!(
            years,
            vec![
                SourceYearPoint {
                    year: 2020,
                    sources: 1,
                    cumulative_sources: 1
                },
                SourceYearPoint {
                    year: 2022,
                    sources: 1,
                    cumulative_sources: 2
                }
            ]
        );
        assert_eq!((total, known, unknown), (3, 2, 1));
    }
}
