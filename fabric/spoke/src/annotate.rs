//! Orchestrator: load a BioOKF export, resolve nodes + check edges, emit a shape-identical graph
//! with `spoke` annotations + a report. Port of annotate.py (apply_adjudications / canonicalize /
//! build_report).

use anyhow::{Context, Result};
use indexmap::IndexMap;
use serde_json::{json, Value};

use crate::client::{cypher_pairs, name_case_variants, Client};
use crate::resolver::{BNode, Candidate, NodeMatch, CONFIDENT_TIERS};

// The HARNESS owns the accept/reject gate for reviewed (LLM/curator) decisions — STRUCTURALLY, not
// numerically. The reviewer only CHOOSES a target among the fabric-surfaced candidates; the
// pipeline's `verify_node_ids` step then confirms that (label, identifier) actually exists in SPOKE.
// No self-reported confidence number is used or stored — confidence was deliberately dropped from
// the contract (arbitrary numbers are not part of the schema).

pub fn load_export(path: &str) -> Result<(Value, Vec<BNode>)> {
    let text = std::fs::read_to_string(path).with_context(|| format!("reading {}", path))?;
    let export: Value = serde_json::from_str(&text).with_context(|| format!("parsing {}", path))?;
    let mut bnodes = vec![];
    if let Some(pages) = export.get("pages").and_then(|v| v.as_object()) {
        for (ident, page) in pages {
            let node_type = page
                .get("node_type")
                .and_then(|v| v.as_str())
                .or_else(|| page.get("type").and_then(|v| v.as_str()))
                .map(|s| s.to_string());
            let subtype = page.get("subtype").and_then(|v| v.as_str()).map(|s| s.to_string());
            let synonyms = str_vec(page.get("synonyms"));
            let xref = str_vec(page.get("xref"));
            bnodes.push(BNode {
                identifier: ident.clone(),
                node_type,
                subtype,
                synonyms,
                xref,
            });
        }
    }
    Ok((export, bnodes))
}

fn str_vec(v: Option<&Value>) -> Vec<String> {
    v.and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default()
}

fn truthy(v: Option<&Value>) -> bool {
    match v {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(Value::Number(n)) => n.as_f64().map(|f| f != 0.0).unwrap_or(true),
        Some(Value::String(s)) => !s.is_empty(),
        Some(Value::Array(a)) => !a.is_empty(),
        Some(Value::Object(o)) => !o.is_empty(),
    }
}

#[derive(Default, Debug)]
pub struct Applied {
    pub accepted: usize,
    pub declined: usize,
    pub skipped: usize,
    pub kept_deterministic: usize,
}

/// Overlay LLM/curator review decisions. A confident deterministic match always wins; only
/// ambiguous/not_found/fuzzy nodes are overlaid. Acceptance is STRUCTURAL: a `map` decision naming a
/// target is accepted (as `reviewed`) and later re-verified to exist by `verify_node_ids`; anything
/// else leaves the node unmatched. No confidence number gates the decision.
pub fn apply_adjudications(
    matches: &mut std::collections::HashMap<String, NodeMatch>,
    decisions: &[Value],
) -> Applied {
    let mut applied = Applied::default();
    for d in decisions {
        let ident = match d.get("id").and_then(|v| v.as_str()) {
            Some(s) => s.to_string(),
            None => {
                applied.skipped += 1;
                continue;
            }
        };
        if !matches.contains_key(&ident) {
            applied.skipped += 1;
            continue;
        }
        let cur = matches.get(&ident).unwrap();
        if cur.status == "mapped"
            && cur.match_tier.as_deref().map(|t| CONFIDENT_TIERS.contains(&t)).unwrap_or(false)
        {
            applied.kept_deterministic += 1;
            continue;
        }
        let reason: String = d
            .get("reason")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .chars()
            .take(200)
            .collect();
        let decision = d.get("decision").and_then(|v| v.as_str()).unwrap_or("");
        let cand_labels = cur.candidate_labels.clone();
        // Structural gate — accept a `map` that names a target; verify_node_ids re-confirms it exists.
        if decision == "map" && truthy(d.get("spoke_identifier")) {
            let spid = normalize_spid(
                d.get("spoke_identifier").unwrap(),
                d.get("spoke_label").and_then(|v| v.as_str()),
            );
            let mut m = NodeMatch::new_mapped_reviewed(
                d.get("spoke_label").and_then(|v| v.as_str()).map(|s| s.to_string()),
                spid,
                d.get("spoke_name").and_then(|v| v.as_str()).map(|s| s.to_string()),
                cand_labels,
                format!("reviewer accepted: {}", reason),
            );
            std::mem::swap(matches.get_mut(&ident).unwrap(), &mut m);
            applied.accepted += 1;
        } else {
            let cur = matches.get_mut(&ident).unwrap();
            cur.status = "not_found".into();
            cur.candidates = vec![];
            cur.notes = format!("reviewer found no match: {}", reason);
            applied.declined += 1;
        }
    }
    applied
}

/// SPOKE labels whose `identifier` property is a Neo4j integer. A curator/LLM supplying such an id
/// as a digit-string must be coerced to int to match; ALL OTHER labels keep string ids verbatim —
/// notably Organism, whose NCBI-taxon ids are strings (e.g. "1770.41"), so coercing them to int
/// would silently break the match. Determined empirically from `n.identifier` types per label.
const INT_ID_LABELS: &[&str] = &["Gene", "DietarySupplement"];

fn normalize_spid(v: &Value, label: Option<&str>) -> Value {
    let int_label = label.map(|l| INT_ID_LABELS.contains(&l)).unwrap_or(false);
    if int_label {
        if let Value::String(s) = v {
            if !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()) {
                if let Ok(n) = s.parse::<i64>() {
                    return json!(n);
                }
            }
        }
    }
    v.clone()
}

/// SPOKE-duplicate canonicalization: remap a Compound match from a non-inchikey id to its exact-name
/// inchikey twin so edge-checking sees the real neighborhood. Only fires on exact name match.
pub async fn canonicalize_duplicates(
    matches: &mut std::collections::HashMap<String, NodeMatch>,
    client: &Client,
) -> Result<usize> {
    // targets: mapped Compound with a string id not starting with inchikey
    let mut names: IndexMap<String, Vec<String>> = IndexMap::new();
    for (ident, m) in matches.iter() {
        if m.status == "mapped"
            && m.spoke_label.as_deref() == Some("Compound")
            && matches!(&m.spoke_identifier, Some(Value::String(s)) if !s.starts_with("inchikey"))
        {
            if let Some(nm) = &m.spoke_name {
                names.entry(nm.clone()).or_default().push(ident.clone());
            }
        }
    }
    if names.is_empty() {
        return Ok(0);
    }
    let name_list: Vec<String> = names.keys().cloned().collect();
    let mut remapped = 0usize;
    for chunk in name_list.chunks(200) {
        // Index-backed case-insensitive match: flatten each name into (name, case-variant) rows and
        // seek `MATCH (n:Compound {name: variant})` per row — a NodeIndexSeek (vs `toLower(...)` or a
        // dynamic `IN` list, both of which force a full Compound-label scan of ~1M nodes).
        let mut pairs: Vec<(String, String)> = vec![];
        for nm in chunk {
            for v in name_case_variants(nm) {
                pairs.push((nm.clone(), v));
            }
        }
        let cy = format!(
            "UNWIND {} AS pair MATCH (n:`Compound` {{name: pair[1]}}) \
             WHERE n.identifier STARTS WITH 'inchikey' RETURN pair[0] AS q, n.identifier AS id, n.name AS name",
            cypher_pairs(&pairs)
        );
        // first inchikey twin per name
        let mut by_name: IndexMap<String, (Value, Option<String>)> = IndexMap::new();
        for r in client.query(&cy).await? {
            if let Some(q) = r.get("q").and_then(|v| v.as_str()) {
                by_name.entry(q.to_string()).or_insert_with(|| {
                    (
                        r.get("id").cloned().unwrap_or(Value::Null),
                        r.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    )
                });
            }
        }
        for (nm, (id, name)) in &by_name {
            if let Some(idents) = names.get(nm) {
                for ident in idents {
                    if let Some(m) = matches.get_mut(ident) {
                        m.spoke_identifier = Some(id.clone());
                        m.spoke_name = name.clone();
                        m.notes = if m.notes.is_empty() {
                            "canonicalized to inchikey twin".into()
                        } else {
                            format!("{} | canonicalized to inchikey twin", m.notes)
                        };
                        remapped += 1;
                    }
                }
            }
        }
    }
    Ok(remapped)
}

/// Fallback annotation for a graph node id absent from the bundle pages.
pub fn node_fallback(note: &str) -> Value {
    json!({
        "knowledge_graph": "spoke",
        "mapping_status": "not_found",
        "candidate_types": [],
        "target_type": Value::Null,
        "target_identifier": Value::Null,
        "target_name": Value::Null,
        "match_method": Value::Null,
        "candidates": [],
        "notes": note,
    })
}

// ---- report ----
fn counter_inc(map: &mut IndexMap<String, i64>, key: &str) {
    *map.entry(key.to_string()).or_insert(0) += 1;
}

pub fn build_report(nodes: &[Value], edges: &[Value], stats: crate::client::Stats) -> Value {
    let mut node_status: IndexMap<String, i64> = IndexMap::new();
    let mut by_type: IndexMap<String, IndexMap<String, i64>> = IndexMap::new();
    let mut by_type_method: IndexMap<String, IndexMap<String, i64>> = IndexMap::new();
    for n in nodes {
        let sp = n.get("spoke").cloned().unwrap_or(Value::Null);
        let ms = sp.get("mapping_status").and_then(|v| v.as_str()).unwrap_or("");
        counter_inc(&mut node_status, ms);
        let typ = n.get("type").and_then(|v| v.as_str()).unwrap_or("null").to_string();
        counter_inc(by_type.entry(typ.clone()).or_default(), ms);
        if let Some(mm) = sp.get("match_method").and_then(|v| v.as_str()) {
            counter_inc(by_type_method.entry(typ).or_default(), mm);
        }
    }

    let mut edge_status: IndexMap<String, i64> = IndexMap::new();
    let mut edge_by_pred: IndexMap<String, IndexMap<String, i64>> = IndexMap::new();
    for e in edges {
        let ss = e
            .get("spoke")
            .and_then(|s| s.get("support_status"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        counter_inc(&mut edge_status, ss);
        let pred = e.get("predicate").and_then(|v| v.as_str()).unwrap_or("null").to_string();
        counter_inc(edge_by_pred.entry(pred).or_default(), ss);
    }

    let n_total = nodes.len() as i64;
    let n_mapped = *node_status.get("mapped").unwrap_or(&0);
    let mappable_total: i64 = node_status.iter().filter(|(k, _)| *k != "not_mappable").map(|(_, v)| *v).sum();

    let contradictions: Vec<Value> = edges
        .iter()
        .filter(|e| {
            e.get("spoke").and_then(|s| s.get("support_status")).and_then(|v| v.as_str()) == Some("contradicted")
        })
        .map(|e| {
            json!({
                "source": e.get("source"),
                "target": e.get("target"),
                "predicate": e.get("predicate"),
                "found_relation_types": e.get("spoke").and_then(|s| s.get("found_relation_types")),
            })
        })
        .collect();
    let supported = *edge_status.get("supported").unwrap_or(&0);

    let review_queue: Vec<Value> = nodes
        .iter()
        .filter(|n| {
            n.get("spoke").and_then(|s| s.get("mapping_status")).and_then(|v| v.as_str()) == Some("ambiguous")
        })
        .take(200)
        .map(|n| {
            json!({
                "identifier": n.get("id"),
                "type": n.get("type"),
                "candidates": n.get("spoke").and_then(|s| s.get("candidates")).cloned().unwrap_or(json!([])),
            })
        })
        .collect();
    let review_queue_size = nodes
        .iter()
        .filter(|n| {
            n.get("spoke").and_then(|s| s.get("mapping_status")).and_then(|v| v.as_str()) == Some("ambiguous")
        })
        .count();

    let round4 = |x: f64| (x * 10000.0).round() / 10000.0;
    json!({
        "totals": {
            "nodes": n_total,
            "edges": edges.len(),
            "nodes_mapped": n_mapped,
            "node_map_rate_overall": if n_total > 0 { round4(n_mapped as f64 / n_total as f64) } else { 0.0 },
            "node_map_rate_mappable": if mappable_total > 0 { round4(n_mapped as f64 / mappable_total as f64) } else { 0.0 },
            "edges_supported": supported,
            "edges_contradicted": contradictions.len(),
        },
        "mapping_status": imap_to_json(&node_status),
        "mapping_status_by_type": sorted_nested(&by_type),
        "match_method_by_type": sorted_nested(&by_type_method),
        "support_status": imap_to_json(&edge_status),
        "support_status_by_predicate": sorted_nested(&edge_by_pred),
        "contradictions": contradictions.into_iter().take(200).collect::<Vec<_>>(),
        "review_queue_size": review_queue_size,
        "review_queue_sample": review_queue,
        "knowledge_graph_query_stats": {"queries": stats.queries, "cache_hits": stats.cache_hits},
    })
}

fn imap_to_json(m: &IndexMap<String, i64>) -> Value {
    let mut o = serde_json::Map::new();
    for (k, v) in m {
        o.insert(k.clone(), json!(v));
    }
    Value::Object(o)
}

fn sorted_nested(m: &IndexMap<String, IndexMap<String, i64>>) -> Value {
    let mut keys: Vec<&String> = m.keys().collect();
    keys.sort();
    let mut o = serde_json::Map::new();
    for k in keys {
        o.insert(k.clone(), imap_to_json(&m[k]));
    }
    Value::Object(o)
}

// Helper constructors used by curation overlay.
impl NodeMatch {
    pub fn new_mapped_reviewed(
        spoke_label: Option<String>,
        spoke_identifier: Value,
        spoke_name: Option<String>,
        candidate_labels: Vec<String>,
        notes: String,
    ) -> Self {
        NodeMatch {
            status: "mapped".into(),
            spoke_label,
            spoke_identifier: Some(spoke_identifier),
            spoke_name,
            match_tier: Some("llm_accept".into()),
            candidate_labels,
            candidates: Vec::<Candidate>::new(),
            notes,
        }
    }
}
