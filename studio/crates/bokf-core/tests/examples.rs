//! Integration test against the real BioOKF `examples/` bundle (v0.4-era format),
//! exercising parse -> normalize -> graph -> lint -> search end to end.

use std::path::PathBuf;

fn examples_root() -> PathBuf {
    // studio/crates/bokf-core -> ../../.. -> BioOKF repo root
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples")
        .canonicalize()
        .expect("examples dir should exist")
}

#[test]
fn opens_examples_bundle() {
    let bundle = bokf_core::open_bundle(examples_root()).expect("open bundle");
    // 6 concept docs ship under examples/knowledge/
    assert!(bundle.nodes.len() >= 6, "expected >=6 nodes, got {}", bundle.nodes.len());
    assert!(bundle.parse_errors.is_empty(), "parse errors: {:?}", bundle.parse_errors);
    // identifiers resolved (title/id merge)
    assert!(bundle.contains("IL6") || bundle.contains("COVID-19"), "expected known identifiers");
}

#[test]
fn derives_graph_from_examples() {
    let g = bokf_core::graph_of(examples_root()).expect("graph");
    assert!(!g.nodes.is_empty());
    assert!(!g.edges.is_empty());
    // every edge endpoint exists as a node (real or external stub)
    let ids: std::collections::HashSet<_> = g.nodes.iter().map(|n| n.id.clone()).collect();
    for e in &g.edges {
        assert!(ids.contains(&e.source), "missing source {}", e.source);
        assert!(ids.contains(&e.target), "missing target {}", e.target);
    }
}

#[test]
fn lints_examples_bundle() {
    let bundle = bokf_core::open_bundle(examples_root()).expect("open");
    let report = bokf_core::lint(&bundle);
    // v0.4 examples use infores: primary_source + CURIE objects -> lint should
    // produce findings (warnings), and must not panic.
    assert!(!report.findings.is_empty(), "expected lint findings on legacy examples");
    println!(
        "lint: {} errors, {} warnings, {} infos",
        report.errors(),
        report.warnings(),
        report.infos()
    );
}

#[test]
fn searches_examples_bundle() {
    let bundle = bokf_core::open_bundle(examples_root()).expect("open");
    let index = bokf_core::SearchIndex::build(&bundle);
    let hits = index.search("interleukin cytokine", 5);
    assert!(!hits.is_empty(), "expected search hits for 'interleukin'");
    assert!(hits[0].score > 0.0);
}
