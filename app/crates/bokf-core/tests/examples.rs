//! Integration test against a real BioOKF bundle shipped under `examples/`,
//! exercising parse -> normalize -> graph -> lint -> search end to end.

use std::path::PathBuf;

/// Locate the `examples/` bundle, which may live in-repo (legacy) or anywhere on
/// disk via the registry (it can be moved out, e.g. onto the Desktop). Resolution
/// order: `OKF_EXAMPLES_DIR` override -> in-repo `examples/` -> registry id
/// `examples` under the repo root. Returns `None` if it is not present anywhere,
/// so the tests skip cleanly on a checkout that doesn't ship the bundle.
fn is_bundle(path: &std::path::Path) -> bool {
    path.is_dir() && path.join("knowledge").is_dir() && path.join("index.md").is_file()
}

fn examples_root() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("OKF_EXAMPLES_DIR") {
        let pb = PathBuf::from(p);
        if is_bundle(&pb) {
            return pb.canonicalize().ok();
        }
    }
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let in_repo = repo.join("examples");
    if is_bundle(&in_repo) {
        return in_repo.canonicalize().ok();
    }
    if let Ok(entries) = std::fs::read_dir(&in_repo) {
        let mut bundles = entries
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| is_bundle(path))
            .collect::<Vec<_>>();
        bundles.sort();
        if let Some(bundle) = bundles.into_iter().next() {
            return bundle.canonicalize().ok();
        }
    }
    if let Some(p) = bokf_core::registry::resolve(&repo, "examples") {
        let pb = PathBuf::from(p);
        if is_bundle(&pb) {
            return pb.canonicalize().ok();
        }
    }
    None
}

/// `let root = require_examples!();` — resolve or skip the test.
macro_rules! require_examples {
    () => {
        match examples_root() {
            Some(p) => p,
            None => {
                eprintln!("skip: examples bundle not present (set OKF_EXAMPLES_DIR or register id `examples`)");
                return;
            }
        }
    };
}

#[test]
fn opens_examples_bundle() {
    let bundle = bokf_core::open_bundle(require_examples!()).expect("open bundle");
    // The example must contain a nontrivial, parseable graph.
    assert!(
        bundle.nodes.len() >= 6,
        "expected >=6 nodes, got {}",
        bundle.nodes.len()
    );
    assert!(
        bundle.parse_errors.is_empty(),
        "parse errors: {:?}",
        bundle.parse_errors
    );
    // The metagraph example includes canonical controlled-type identifiers.
    assert!(
        bundle.contains("Gene") && bundle.contains("Publication"),
        "expected known identifiers"
    );
}

#[test]
fn derives_graph_from_examples() {
    let g = bokf_core::graph_of(require_examples!()).expect("graph");
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
    let bundle = bokf_core::open_bundle(require_examples!()).expect("open");
    let report = bokf_core::lint(&bundle);
    // Linting a repository example must complete without panicking. Examples may
    // intentionally be clean or demonstrate warnings depending on their purpose.
    println!(
        "lint: {} errors, {} warnings, {} infos",
        report.errors(),
        report.warnings(),
        report.infos()
    );
}

#[test]
fn searches_examples_bundle() {
    let bundle = bokf_core::open_bundle(require_examples!()).expect("open");
    let index = bokf_core::SearchIndex::build(&bundle);
    let hits = index.search("gene molecule", 5);
    assert!(!hits.is_empty(), "expected search hits for 'gene molecule'");
    assert!(hits[0].score > 0.0);
}
