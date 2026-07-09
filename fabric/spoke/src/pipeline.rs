//! Fail-closed pipeline orchestrator + independent audit + contract validation. Port of pipeline.py
//! and cli.validate_graph. Each step is wrapped in a GATE that bails on violation (never silently
//! skipped); a manifest proves each step ran; audit re-derives every check from the output file.

use anyhow::{bail, Result};
use futures::stream::{self, StreamExt};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};

use crate::annotate;
use crate::client::{cypher_value, Client, CONCURRENCY};
use crate::crosswalk::{PredicateCrosswalk, TypeCrosswalk};
use crate::resolver::{EdgeChecker, NodeMatch, NodeResolver};

/// Check every edge concurrently, returning annotations in original edge order.
pub async fn check_edges_concurrent(
    checker: &EdgeChecker<'_>,
    edges: &[Value],
    matches: &HashMap<String, NodeMatch>,
) -> Result<Vec<Value>> {
    let results: Vec<(usize, Result<Value>)> = stream::iter(edges.iter().enumerate())
        .map(|(i, e)| async move { (i, checker.check(e, matches, "spoke").await) })
        .buffer_unordered(CONCURRENCY)
        .collect()
        .await;
    let mut anns: Vec<Option<Value>> = (0..edges.len()).map(|_| None).collect();
    for (i, r) in results {
        anns[i] = Some(r?);
    }
    Ok(anns.into_iter().map(|a| a.unwrap()).collect())
}

pub const BIOOKF_TYPES: &[&str] = &[
    "Gene", "Molecule", "MolecularClass", "Variant", "SequenceFeature", "Structure", "Anatomy",
    "CellType", "Organism", "BiologicalPathway", "BiologicalFunction", "Disease", "Phenotype",
    "BiomedicalMeasure", "MethodOrProcedure", "Exposure", "SocialFactor", "Food", "Device",
    "MaterialSample", "Publication", "Study", "Dataset", "Agent", "Population", "GeographicLocation",
    "Concept", "Other",
];
pub const BASE_PREDICATES: &[&str] = &[
    "is_a", "part_of", "member_of", "derives_from", "located_in", "expressed_in", "encodes",
    "interacts_with", "binds", "regulates", "catalyzes", "converts_to", "participates_in", "causes",
    "predisposes_to", "treats", "prevents", "contraindicated_in", "affects_response_to",
    "has_phenotype", "measures", "associated_with", "used_to_study", "reported_in",
];
pub const REQUIRED_STEPS: &[&str] = &[
    "preflight", "load_export", "resolve_nodes", "apply_curation", "verify_node_ids",
    "canonicalize", "check_edges", "validate_invariants",
];

fn value_key(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Null => "".into(),
        other => other.to_string(),
    }
}

fn sha256_file(path: &str) -> String {
    use sha2::{Digest, Sha256};
    match std::fs::read(path) {
        Ok(bytes) => {
            let mut h = Sha256::new();
            h.update(&bytes);
            format!("{:x}", h.finalize())
        }
        Err(_) => String::new(),
    }
}

/// Subset of (label, str(id)) that actually EXIST in SPOKE.
pub async fn fetch_existing(client: &Client, keyset: &[(String, Value)]) -> Result<HashSet<(String, String)>> {
    let mut by_label: HashMap<String, Vec<Value>> = HashMap::new();
    for (lab, idv) in keyset {
        by_label.entry(lab.clone()).or_default().push(idv.clone());
    }
    // Build all (label, cypher) chunk queries up front, then run them concurrently.
    let mut tasks: Vec<(String, String)> = vec![];
    for (lab, ids) in &by_label {
        let mut seen = HashSet::new();
        let uniq: Vec<Value> = ids.iter().filter(|v| seen.insert(v.to_string())).cloned().collect();
        for chunk in uniq.chunks(200) {
            let lit = format!("[{}]", chunk.iter().map(cypher_value).collect::<Vec<_>>().join(", "));
            let cy = format!(
                "UNWIND {} AS q MATCH (n:`{}` {{identifier:q}}) RETURN DISTINCT n.identifier AS id",
                lit, lab
            );
            tasks.push((lab.clone(), cy));
        }
    }
    let results: Vec<Result<(String, Vec<Value>)>> = stream::iter(tasks)
        .map(|(lab, cy)| async move { client.query(&cy).await.map(|rows| (lab, rows)) })
        .buffer_unordered(CONCURRENCY)
        .collect()
        .await;
    let mut existing = HashSet::new();
    for res in results {
        let (lab, rows) = res?;
        for r in rows {
            if let Some(idv) = r.get("id") {
                existing.insert((lab.clone(), value_key(idv)));
            }
        }
    }
    Ok(existing)
}

/// Enforce docs/02-annotation-contract.md invariants. Returns (ok, problems[:50]).
pub fn validate_graph(data: &Value) -> (bool, Vec<String>) {
    let mut problems: Vec<String> = vec![];
    let graph = data.get("graph").unwrap_or(data);
    let empty = vec![];
    let nodes = graph.get("nodes").and_then(|v| v.as_array()).unwrap_or(&empty);
    let edges = graph.get("edges").and_then(|v| v.as_array()).unwrap_or(&empty);
    // (F5) The authoritative shape invariant is output==input (checked in validate_invariants against
    // the source file). We do NOT gate on the export's self-declared node_count/edge_count: `bokf
    // export` historically declared edge_count excluding synthesized edges, and a self-declared count
    // is redundant with the input↔output shape check — so relying on it only produced false failures.
    let mut by_id: HashMap<String, Value> = HashMap::new();
    for n in nodes {
        let id = n.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let sp = n.get("spoke");
        match sp.and_then(|s| s.get("mapping_status")) {
            None => {
                problems.push(format!("node {:?} missing spoke.mapping_status", id));
                continue;
            }
            Some(ms) => {
                by_id.insert(id.clone(), sp.cloned().unwrap());
                if ms.as_str() == Some("mapped") {
                    let ti = sp.and_then(|s| s.get("target_identifier"));
                    let null_id = matches!(ti, None | Some(Value::Null))
                        || matches!(ti, Some(Value::String(s)) if s.is_empty());
                    if null_id {
                        problems.push(format!("node {:?} mapped but no target_identifier", id));
                    }
                }
            }
        }
    }
    for e in edges {
        let src = e.get("source").and_then(|v| v.as_str()).unwrap_or("");
        let tgt = e.get("target").and_then(|v| v.as_str()).unwrap_or("");
        let sp = e.get("spoke");
        let st = match sp.and_then(|s| s.get("support_status")).and_then(|v| v.as_str()) {
            None => {
                problems.push(format!("edge {:?}->{:?} missing spoke.support_status", src, tgt));
                continue;
            }
            Some(s) => s,
        };
        if ["supported", "contradicted", "related", "different_relation", "unsupported"].contains(&st) {
            for endpt in [src, tgt] {
                let mapped = by_id
                    .get(endpt)
                    .and_then(|s| s.get("mapping_status"))
                    .and_then(|v| v.as_str())
                    == Some("mapped");
                if !mapped {
                    problems.push(format!(
                        "edge {:?}->{:?} support_status {} but endpoint {:?} not mapped",
                        src, tgt, st, endpt
                    ));
                    break;
                }
            }
        }
        if st == "contradicted" {
            let pred = e.get("predicate").and_then(|v| v.as_str()).unwrap_or("");
            if !pred.starts_with("not_") {
                problems.push(format!("edge {:?}->{:?} contradicted but predicate not negative", src, tgt));
            }
        }
    }
    let ok = problems.is_empty();
    problems.truncate(50);
    (ok, problems)
}

pub struct Pipeline<'a> {
    c: &'a Client,
    verbose: bool,
    steps: Vec<Value>,
    gates: serde_json::Map<String, Value>,
}

impl<'a> Pipeline<'a> {
    pub fn new(c: &'a Client, verbose: bool) -> Self {
        Pipeline { c, verbose, steps: vec![], gates: serde_json::Map::new() }
    }

    fn record(&mut self, step: &str, status: &str, metrics: Value) {
        self.steps.push(json!({"step": step, "status": status, "metrics": metrics, "error": Value::Null}));
    }

    fn gate(cond: bool, msg: &str) -> Result<()> {
        if !cond {
            bail!("GATE: {}", msg);
        }
        Ok(())
    }

    async fn preflight(&mut self, tx: &TypeCrosswalk, px: &PredicateCrosswalk) -> Result<()> {
        let rows = self.c.query("RETURN 1 AS ok").await;
        match rows {
            Ok(r) if r.first().and_then(|x| x.get("ok")).and_then(|v| v.as_i64()) == Some(1) => {}
            _ => bail!("GATE: SPOKE connectivity check failed"),
        }
        let tkeys = tx.type_keys();
        let missing_t: Vec<&str> = BIOOKF_TYPES.iter().copied().filter(|t| !tkeys.contains(*t)).collect();
        Self::gate(missing_t.is_empty(), &format!("type_crosswalk missing types: {:?}", missing_t))?;
        let mut known = px.predicate_keys();
        known.extend(px.provenance_keys());
        let missing_p: Vec<&str> = BASE_PREDICATES.iter().copied().filter(|p| !known.contains(*p)).collect();
        Self::gate(missing_p.is_empty(), &format!("predicate_crosswalk missing predicates: {:?}", missing_p))?;
        let nidx = self.c.name_indexed_labels().await.len();
        self.record("preflight", "pass", json!({"types": tkeys.len(), "predicates": px.predicate_keys().len(), "name_indexed_labels": nidx}));
        Ok(())
    }

    fn load_export(&mut self, path: &str) -> Result<(Value, Vec<crate::resolver::BNode>)> {
        Self::gate(std::path::Path::new(path).exists(), &format!("export not found: {}", path))?;
        let (export, bnodes) = annotate::load_export(path)?;
        let has_graph = export.get("graph").and_then(|g| g.get("nodes")).is_some()
            && export.get("graph").and_then(|g| g.get("edges")).is_some();
        Self::gate(has_graph, "export missing graph.nodes/edges")?;
        Self::gate(!bnodes.is_empty(), "export has no page nodes")?;
        let gn = export["graph"]["nodes"].as_array().map(|a| a.len()).unwrap_or(0);
        let ge = export["graph"]["edges"].as_array().map(|a| a.len()).unwrap_or(0);
        self.record("load_export", "pass", json!({"input_sha256": sha256_file(path), "pages": bnodes.len(), "graph_nodes": gn, "graph_edges": ge}));
        Ok((export, bnodes))
    }

    async fn resolve_nodes(&mut self, tx: &TypeCrosswalk, bnodes: &[crate::resolver::BNode]) -> Result<HashMap<String, NodeMatch>> {
        let matches = NodeResolver::new(self.c, tx).resolve_all(bnodes).await?;
        Self::gate(matches.len() == bnodes.len(), &format!("resolver returned {} matches for {} nodes", matches.len(), bnodes.len()))?;
        let bad: Vec<&String> = matches.iter().filter(|(_, m)| m.status == "mapped" && matches!(&m.spoke_identifier, None | Some(Value::Null))).map(|(i, _)| i).collect();
        Self::gate(bad.is_empty(), &format!("{} mapped nodes have a null identifier", bad.len()))?;
        let count = |s: &str| matches.values().filter(|m| m.status == s).count();
        let mut tiers: HashMap<String, i64> = HashMap::new();
        for m in matches.values() {
            if m.status == "mapped" {
                if let Some(t) = &m.match_tier {
                    *tiers.entry(t.clone()).or_insert(0) += 1;
                }
            }
        }
        self.record("resolve_nodes", "pass", json!({"mapped": count("mapped"), "ambiguous": count("ambiguous"), "not_found": count("not_found"), "not_mappable": count("not_mappable"), "tiers": tiers}));
        Ok(matches)
    }

    fn apply_curation(&mut self, matches: &mut HashMap<String, NodeMatch>, curation_path: Option<&str>) -> Result<()> {
        match curation_path {
            None => {
                self.record("apply_curation", "skipped", json!({"reason": "no curation file"}));
            }
            Some(p) => {
                Self::gate(std::path::Path::new(p).exists(), &format!("curation file not found: {}", p))?;
                let text = std::fs::read_to_string(p)?;
                let parsed: Value = serde_json::from_str(&text)?;
                let decisions: Vec<Value> = match parsed {
                    Value::Array(a) => a,
                    Value::Object(ref o) => o.get("decisions").and_then(|v| v.as_array()).cloned().unwrap_or_default(),
                    _ => vec![],
                };
                annotate::apply_adjudications(matches, &decisions);
                self.record("apply_curation", "pass", json!({"curation_sha256": sha256_file(p), "decisions": decisions.len()}));
            }
        }
        Ok(())
    }

    async fn verify_node_ids(&mut self, matches: &HashMap<String, NodeMatch>) -> Result<()> {
        let mapped: Vec<(String, Value)> = matches
            .values()
            .filter(|m| m.status == "mapped")
            .map(|m| (m.spoke_label.clone().unwrap_or_default(), m.spoke_identifier.clone().unwrap_or(Value::Null)))
            .collect();
        let existing = fetch_existing(self.c, &mapped).await?;
        let broken: Vec<String> = matches
            .iter()
            .filter(|(_, m)| m.status == "mapped")
            .filter(|(_, m)| !existing.contains(&(m.spoke_label.clone().unwrap_or_default(), value_key(m.spoke_identifier.as_ref().unwrap_or(&Value::Null)))))
            .map(|(i, _)| i.clone())
            .collect();
        self.gates.insert("broken_ids".into(), json!(broken.len()));
        Self::gate(broken.is_empty(), &format!("{} mapped nodes point at a SPOKE id that does not exist (hallucinated/stale): {:?}", broken.len(), &broken[..broken.len().min(5)]))?;
        self.record("verify_node_ids", "pass", json!({"checked": mapped.len(), "broken": 0}));
        Ok(())
    }

    async fn canonicalize(&mut self, matches: &mut HashMap<String, NodeMatch>) -> Result<()> {
        let before: HashMap<String, Option<Value>> = matches.iter().map(|(i, m)| (i.clone(), m.spoke_identifier.clone())).collect();
        annotate::canonicalize_duplicates(matches, self.c).await?;
        let remapped: Vec<String> = matches.iter().filter(|(i, m)| before.get(*i).cloned().flatten() != m.spoke_identifier).map(|(i, _)| i.clone()).collect();
        let keys: Vec<(String, Value)> = remapped.iter().map(|i| (matches[i].spoke_label.clone().unwrap_or_default(), matches[i].spoke_identifier.clone().unwrap_or(Value::Null))).collect();
        let existing = fetch_existing(self.c, &keys).await?;
        let bad: Vec<&String> = remapped.iter().filter(|i| !existing.contains(&(matches[*i].spoke_label.clone().unwrap_or_default(), value_key(matches[*i].spoke_identifier.as_ref().unwrap_or(&Value::Null))))).collect();
        Self::gate(bad.is_empty(), &format!("canonicalization produced non-existent ids: {:?}", &bad[..bad.len().min(5)]))?;
        self.record("canonicalize", "pass", json!({"remapped": remapped.len()}));
        Ok(())
    }

    async fn check_edges(&mut self, export: &mut Value, matches: &HashMap<String, NodeMatch>, px: &PredicateCrosswalk) -> Result<()> {
        // annotate nodes
        {
            let nodes = export["graph"]["nodes"].as_array_mut().unwrap();
            for gn in nodes.iter_mut() {
                let id = gn.get("id").and_then(|v| v.as_str()).map(|s| s.to_string());
                let ann = match id.as_deref().and_then(|i| matches.get(i)) {
                    Some(m) => m.to_json("spoke"),
                    None => annotate::node_fallback("id absent from bundle pages"),
                };
                gn["spoke"] = ann;
            }
        }
        // annotate edges — independent per edge, so run them concurrently (huge wall-clock win)
        let checker = EdgeChecker::new(self.c, px);
        let edges: Vec<Value> = export["graph"]["edges"].as_array().unwrap().clone();
        let anns = check_edges_concurrent(&checker, &edges, matches).await?;
        {
            let edges_mut = export["graph"]["edges"].as_array_mut().unwrap();
            for (e, ann) in edges_mut.iter_mut().zip(anns) {
                e["spoke"] = ann;
            }
        }
        let nodes_ok = export["graph"]["nodes"].as_array().unwrap().iter().all(|n| n.get("spoke").and_then(|s| s.get("mapping_status")).is_some());
        let edges_ok = export["graph"]["edges"].as_array().unwrap().iter().all(|e| e.get("spoke").and_then(|s| s.get("support_status")).is_some());
        Self::gate(nodes_ok, "some node missing spoke.mapping_status")?;
        Self::gate(edges_ok, "some edge missing spoke.support_status")?;
        let mut est: HashMap<String, i64> = HashMap::new();
        for e in export["graph"]["edges"].as_array().unwrap() {
            let ss = e.get("spoke").and_then(|s| s.get("support_status")).and_then(|v| v.as_str()).unwrap_or("");
            *est.entry(ss.to_string()).or_insert(0) += 1;
        }
        self.record("check_edges", "pass", json!({"supported": est.get("supported").copied().unwrap_or(0), "related": est.get("related").copied().unwrap_or(0), "contradicted": est.get("contradicted").copied().unwrap_or(0), "support_status": est}));
        Ok(())
    }

    fn validate_invariants(&mut self, export: &Value, input_path: &str) -> Result<()> {
        let (ok, problems) = validate_graph(export);
        Self::gate(ok, &format!("contract invariants failed: {:?}", &problems[..problems.len().min(5)]))?;
        let src_text = std::fs::read_to_string(input_path)?;
        let src: Value = serde_json::from_str(&src_text)?;
        let sn = src["graph"]["nodes"].as_array().map(|a| a.len()).unwrap_or(0);
        let se = src["graph"]["edges"].as_array().map(|a| a.len()).unwrap_or(0);
        let on = export["graph"]["nodes"].as_array().map(|a| a.len()).unwrap_or(0);
        let oe = export["graph"]["edges"].as_array().map(|a| a.len()).unwrap_or(0);
        Self::gate(sn == on && se == oe, "shape mismatch: node/edge counts differ from input")?;
        let order_ok = src["graph"]["nodes"].as_array().unwrap().iter().zip(export["graph"]["nodes"].as_array().unwrap().iter()).all(|(a, b)| a.get("id") == b.get("id"));
        Self::gate(order_ok, "node order not preserved")?;
        self.gates.insert("invariants".into(), json!("pass"));
        self.record("validate_invariants", "pass", json!({"nodes": on, "edges": oe, "shape_identical": true}));
        Ok(())
    }

    pub async fn run(&mut self, export_path: &str, out_path: &str, report_path: &str, curation_path: Option<&str>) -> Result<String> {
        let result = self.run_inner(export_path, out_path, report_path, curation_path).await;
        let final_result = match &result {
            Ok(_) => {
                self.gates.insert("all_passed".into(), json!(true));
                "PASS".to_string()
            }
            Err(e) => {
                self.gates.insert("all_passed".into(), json!(false));
                self.steps.push(json!({"step": "EXCEPTION", "status": "fail", "metrics": {}, "error": format!("{}", e)}));
                "FAIL".to_string()
            }
        };
        let manifest = json!({
            "pipeline_version": 1,
            "steps": self.steps,
            "gates": self.gates,
            "result": final_result,
            "output": if final_result == "PASS" { json!({"path": out_path, "sha256": sha256_file(out_path)}) } else { Value::Null },
        });
        let manifest_path = format!("{}.manifest.json", out_path.trim_end_matches(".json"));
        std::fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)?;
        self.print_checklist(&final_result);
        if self.verbose {
            eprintln!("\nmanifest: {}", manifest_path);
        }
        Ok(final_result)
    }

    async fn run_inner(&mut self, export_path: &str, out_path: &str, report_path: &str, curation_path: Option<&str>) -> Result<()> {
        let timing = std::env::var("SPOKE_FABRIC_TIMING").is_ok();
        macro_rules! phase {
            ($name:expr, $e:expr) => {{
                let t = std::time::Instant::now();
                let r = $e;
                if timing { eprintln!("[phase] {:14} {:.1}s", $name, t.elapsed().as_secs_f64()); }
                r
            }};
        }
        let tx = TypeCrosswalk::load()?;
        let px = PredicateCrosswalk::load()?;
        phase!("preflight", self.preflight(&tx, &px).await?);
        let (mut export, bnodes) = self.load_export(export_path)?;
        let mut matches = phase!("resolve_nodes", self.resolve_nodes(&tx, &bnodes).await?);
        self.apply_curation(&mut matches, curation_path)?;
        phase!("verify_node_ids", self.verify_node_ids(&matches).await?);
        phase!("canonicalize", self.canonicalize(&mut matches).await?);
        phase!("check_edges", self.check_edges(&mut export, &matches, &px).await?);
        self.validate_invariants(&export, export_path)?;
        let report = annotate::build_report(
            export["graph"]["nodes"].as_array().unwrap(),
            export["graph"]["edges"].as_array().unwrap(),
            self.c.stats(),
        );
        export["spoke_report"] = report.clone();
        std::fs::write(out_path, serde_json::to_string_pretty(&export)?)?;
        std::fs::write(report_path, serde_json::to_string_pretty(&report)?)?;
        Ok(())
    }

    fn print_checklist(&self, result: &str) {
        println!("\n{}", "=".repeat(60));
        println!("PIPELINE CHECKLIST  ->  {}", result);
        println!("{}", "=".repeat(60));
        let done: HashMap<&str, &Value> = self.steps.iter().map(|s| (s["step"].as_str().unwrap_or(""), s)).collect();
        for step in REQUIRED_STEPS {
            let s = done.get(step);
            let status = s.and_then(|s| s["status"].as_str()).unwrap_or("NOT RUN");
            let mark = match status {
                "pass" => "x",
                "skipped" => "-",
                _ => "!",
            };
            let mut info = String::new();
            if let Some(s) = s {
                if let Some(m) = s["metrics"].as_object() {
                    let parts: Vec<String> = m.iter().take(3).filter(|(_, v)| !v.is_object() && !v.is_array()).map(|(k, v)| format!("{}={}", k, v)).collect();
                    info = parts.join(" ");
                }
            }
            println!("  [{}] {:20} {:8} {}", mark, step, status, info);
        }
        if let Some(last) = self.steps.last() {
            if last["step"].as_str() == Some("EXCEPTION") {
                println!("  ERROR: {}", last["error"].as_str().unwrap_or(""));
            }
        }
    }
}

/// Independent end-to-end audit — does NOT trust the manifest; re-derives every check from the output.
pub async fn audit(out_path: &str, manifest_path: &str, client: &Client, sample: usize) -> Result<bool> {
    let mut checks: Vec<(String, bool, String)> = vec![];
    let mut chk = |name: &str, ok: bool, detail: &str| checks.push((name.to_string(), ok, detail.to_string()));

    let export: Value = serde_json::from_str(&std::fs::read_to_string(out_path)?)?;
    let man: Value = if std::path::Path::new(manifest_path).exists() {
        serde_json::from_str(&std::fs::read_to_string(manifest_path)?)?
    } else {
        json!({})
    };
    let graph = export.get("graph").unwrap_or(&export);
    let nodes = graph["nodes"].as_array().cloned().unwrap_or_default();
    let edges = graph["edges"].as_array().cloned().unwrap_or_default();

    let steps: HashMap<String, String> = man.get("steps").and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|s| Some((s.get("step")?.as_str()?.to_string(), s.get("status")?.as_str()?.to_string()))).collect()).unwrap_or_default();
    let missing: Vec<&str> = REQUIRED_STEPS.iter().copied().filter(|s| !steps.contains_key(*s)).collect();
    chk("manifest: result PASS", man.get("result").and_then(|v| v.as_str()) == Some("PASS"), man.get("result").and_then(|v| v.as_str()).unwrap_or(""));
    chk("manifest: all required steps ran", missing.is_empty(), &format!("missing={:?}", missing));
    chk("manifest: no failed step", steps.values().all(|v| v == "pass" || v == "skipped"), "");

    let (ok, problems) = validate_graph(&export);
    chk("output: contract invariants", ok, &if problems.is_empty() { String::new() } else { format!("{} problems", problems.len()) });

    chk("output: every node annotated", nodes.iter().all(|n| n.get("spoke").and_then(|s| s.get("mapping_status")).is_some()), "");
    chk("output: every edge annotated", edges.iter().all(|e| e.get("spoke").and_then(|s| s.get("support_status")).is_some()), "");

    let nullid: Vec<&Value> = nodes.iter().filter(|n| n.get("spoke").and_then(|s| s.get("mapping_status")).and_then(|v| v.as_str()) == Some("mapped") && matches!(n.get("spoke").and_then(|s| s.get("target_identifier")), None | Some(Value::Null))).collect();
    chk("output: mapped nodes have identifiers", nullid.is_empty(), &format!("{} null", nullid.len()));

    // independent broken-id re-check (sampled)
    let mapped: Vec<(String, Value)> = nodes.iter().filter(|n| n.get("spoke").and_then(|s| s.get("mapping_status")).and_then(|v| v.as_str()) == Some("mapped")).filter_map(|n| {
        let sp = n.get("spoke")?;
        Some((sp.get("target_type")?.as_str()?.to_string(), sp.get("target_identifier")?.clone()))
    }).collect();
    let step = (mapped.len() / sample.max(1)).max(1);
    let probe: Vec<(String, Value)> = mapped.iter().step_by(step).take(sample).cloned().collect();
    let existing = fetch_existing(client, &probe).await?;
    let broken: Vec<&(String, Value)> = probe.iter().filter(|p| !existing.contains(&(p.0.clone(), value_key(&p.1)))).collect();
    chk(&format!("output: sampled ids exist in SPOKE ({})", probe.len()), broken.is_empty(), &format!("{} broken", broken.len()));

    let bad_contra: Vec<&Value> = edges.iter().filter(|e| e.get("spoke").and_then(|s| s.get("support_status")).and_then(|v| v.as_str()) == Some("contradicted") && !e.get("predicate").and_then(|v| v.as_str()).unwrap_or("").starts_with("not_")).collect();
    chk("output: contradictions are negated claims", bad_contra.is_empty(), &format!("{} bad", bad_contra.len()));

    let allok = checks.iter().all(|(_, ok, _)| *ok);
    println!("{}", "=".repeat(64));
    println!("END-TO-END AUDIT  ->  {}", if allok { "PASS" } else { "FAIL" });
    println!("{}", "=".repeat(64));
    for (name, ok, detail) in &checks {
        println!("  [{}] {:42} {}", if *ok { "x" } else { "!" }, name, detail);
    }
    Ok(allok)
}
