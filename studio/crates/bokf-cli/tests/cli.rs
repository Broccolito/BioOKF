//! End-to-end CLI test: scaffold a bundle, author valid + invalid concept docs,
//! and verify `bokf lint` / `graph` / `search` behave correctly.

use std::path::PathBuf;
use std::process::Command;

fn bokf() -> &'static str {
    env!("CARGO_BIN_EXE_bokf")
}

fn tmp_bundle(name: &str) -> PathBuf {
    let mut d = std::env::temp_dir();
    d.push(format!("bokf-cli-test-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

fn write(path: &std::path::Path, content: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

#[test]
fn scaffold_lint_graph_search_roundtrip() {
    let dir = tmp_bundle("roundtrip");

    // scaffold
    let out = Command::new(bokf()).args(["scaffold", dir.to_str().unwrap(), "--name", "Test KB"]).output().unwrap();
    assert!(out.status.success(), "scaffold failed: {}", String::from_utf8_lossy(&out.stderr));
    assert!(dir.join("index.md").exists());
    assert!(dir.join("knowledge").is_dir());

    // a valid source node + two valid concept docs with a provenance-stamped edge
    write(&dir.join("knowledge/publication/src.md"), r#"---
type: Publication
identifier: Demo source
subtype: article
raw_source: [raw/demo.md]
---
# Demo source
"#);
    write(&dir.join("knowledge/gene/braf.md"), r#"---
type: Gene
identifier: BRAF
subtype: protein_coding
xref: [HGNC:1097]
edges:
  - predicate: associated_with
    object: Melanoma
    knowledge_level: statistical_association
    agent_type: text_mining_agent
    primary_source: Demo source
    effect_metric: odds_ratio
    effect_size: 5.1
---
# BRAF
A kinase gene.
"#);
    write(&dir.join("knowledge/disease/melanoma.md"), r#"---
type: Disease
identifier: Melanoma
subtype: neoplasm
edges:
  - predicate: reported_in
    object: Demo source
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: Demo source
---
# Melanoma
"#);

    // lint should be clean (0 errors)
    let out = Command::new(bokf()).args(["lint", dir.to_str().unwrap(), "--json"]).output().unwrap();
    let report: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let errors = report["findings"].as_array().unwrap().iter().filter(|f| f["severity"] == "error").count();
    assert_eq!(errors, 0, "expected 0 errors, report: {}", String::from_utf8_lossy(&out.stdout));
    assert!(out.status.success(), "clean bundle should exit 0");

    // graph: BRAF -> Melanoma edge present
    let out = Command::new(bokf()).args(["graph", dir.to_str().unwrap()]).output().unwrap();
    let g: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let edges = g["edges"].as_array().unwrap();
    assert!(edges.iter().any(|e| e["source"] == "BRAF" && e["target"] == "Melanoma" && e["predicate"] == "associated_with"));

    // search finds BRAF
    let out = Command::new(bokf()).args(["search", dir.to_str().unwrap(), "kinase", "--json"]).output().unwrap();
    let hits: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(hits.as_array().unwrap().iter().any(|h| h["identifier"] == "BRAF"));

    // now introduce an INVALID type + a bad predicate -> lint must flag errors and exit 1
    write(&dir.join("knowledge/other/bad.md"), r#"---
type: NotAType
identifier: Bad node
edges:
  - predicate: cures
    object: Melanoma
    knowledge_level: knowledge_assertion
    agent_type: manual_agent
    primary_source: Demo source
---
# Bad
"#);
    let out = Command::new(bokf()).args(["lint", dir.to_str().unwrap(), "--json"]).output().unwrap();
    let report: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let rules: Vec<String> = report["findings"].as_array().unwrap().iter().map(|f| f["rule"].as_str().unwrap_or("").to_string()).collect();
    assert!(rules.iter().any(|r| r == "type.invalid"), "should flag invalid type");
    assert!(rules.iter().any(|r| r == "predicate.invalid"), "should flag invalid predicate");
    assert!(!out.status.success(), "bundle with errors should exit nonzero");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn predicates_lists_24() {
    let out = Command::new(bokf()).args(["predicates", "--json"]).output().unwrap();
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["predicates"].as_array().unwrap().len(), 24);
    assert!(v["predicates"].as_array().unwrap().iter().any(|p| p == "used_to_study"));
    assert_eq!(v["node_types"].as_array().unwrap().len(), 28);
}

#[test]
fn cli_log_sync_then_log() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("log.md"), "# Change log\n").unwrap();
    let ok = Command::new(bokf())
        .args(["log-sync", dir.path().to_str().unwrap(), "--kind", "ingest", "--summary", "seed"])
        .output().unwrap();
    assert!(ok.status.success(), "{}", String::from_utf8_lossy(&ok.stderr));
    let log = Command::new(bokf())
        .args(["log", dir.path().to_str().unwrap(), "--json"]).output().unwrap();
    let v: serde_json::Value = serde_json::from_slice(&log.stdout).unwrap();
    assert_eq!(v[0]["kind"], "ingest");
}

#[test]
fn scaffold_registers_inits_and_activates() {
    let root = tempfile::tempdir().unwrap();
    let bundle = root.path().join("ms-kb");
    let s = Command::new(bokf())
        .args(["scaffold", bundle.to_str().unwrap(), "--name", "MS KB"]).output().unwrap();
    assert!(s.status.success(), "{}", String::from_utf8_lossy(&s.stderr));
    assert!(bundle.join(".git").exists());
    let ga = Command::new(bokf())
        .args(["get-active", root.path().to_str().unwrap(), "--json"]).output().unwrap();
    let v: serde_json::Value = serde_json::from_slice(&ga.stdout).unwrap();
    assert_eq!(v["id"], "ms-kb");
}

#[test]
fn end_to_end_scaffold_write_logsync_log_restore() {
    let root = tempfile::tempdir().unwrap();
    let bundle = root.path().join("demo-kb");
    let run = |args: &[&str]| Command::new(bokf()).args(args).output().unwrap();

    assert!(run(&["scaffold", bundle.to_str().unwrap(), "--name", "Demo"]).status.success());
    let ga = run(&["get-active", root.path().to_str().unwrap(), "--json"]);
    let v: serde_json::Value = serde_json::from_slice(&ga.stdout).unwrap();
    assert_eq!(v["id"], "demo-kb");

    // write a node + log-sync it
    let k = bundle.join("knowledge/gene");
    std::fs::create_dir_all(&k).unwrap();
    std::fs::write(k.join("il6.md"), "---\ntype: Gene\nidentifier: IL6\nsubtype: protein_coding\n---\n# IL6\n").unwrap();
    assert!(run(&["log-sync", bundle.to_str().unwrap(), "--kind", "ingest", "--summary", "add IL6", "--delta", "+1 node"]).status.success());

    // history has the scaffold commit + the ingest
    let log = run(&["log", bundle.to_str().unwrap(), "--json"]);
    let entries: serde_json::Value = serde_json::from_slice(&log.stdout).unwrap();
    assert_eq!(entries[0]["kind"], "ingest");
    let first_sha = entries.as_array().unwrap().last().unwrap()["commit_sha"].as_str().unwrap().to_string();

    // restore to the scaffold commit: IL6 is gone, history grew with a restore entry
    assert!(run(&["restore", bundle.to_str().unwrap(), &first_sha]).status.success());
    assert!(!k.join("il6.md").exists());
    let log2 = run(&["log", bundle.to_str().unwrap(), "--json"]);
    let e2: serde_json::Value = serde_json::from_slice(&log2.stdout).unwrap();
    assert_eq!(e2[0]["kind"], "restore");
}
