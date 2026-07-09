//! spoke-fabric — native Rust BioOKF -> SPOKE mapping fabric.
//!
//! One tool, several subcommands. `query` / `test-connection` are the native SPOKE Bolt subtool
//! (replacing per-query spoke-cli subprocess spawning); `pipeline` / `link` / `audit` / `validate`
//! run the mapping harness. Parity + speed with the Python reference is the design goal.

mod annotate;
mod client;
mod config;
mod crosswalk;
mod normalize;
mod pipeline;
mod resolver;

use anyhow::Result;
use clap::{Parser, Subcommand};
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::client::Client;
use crate::crosswalk::{PredicateCrosswalk, TypeCrosswalk};
use crate::resolver::{BNode, EdgeChecker, NodeMatch, NodeResolver};

#[derive(Parser)]
#[command(name = "spoke-fabric", about = "BioOKF -> SPOKE mapping fabric (native Rust, Bolt subtool)")]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a raw Cypher query against SPOKE (native Bolt) and print JSON rows.
    Query {
        cypher: String,
        /// no-op flag kept for spoke-cli compatibility
        #[arg(long)]
        stdout: bool,
    },
    /// Check the SPOKE Bolt connection.
    TestConnection,
    /// Annotate a BioOKF export with SPOKE mappings (ungated).
    Link {
        export: String,
        #[arg(long)]
        out: String,
        #[arg(long)]
        report: String,
        #[arg(long)]
        adjudications: Option<String>,
        #[arg(long)]
        canonicalize: bool,
        #[arg(long)]
        validate: bool,
    },
    /// FAIL-CLOSED run: every step gated + a manifest written.
    Pipeline {
        export: String,
        #[arg(long)]
        out: String,
        #[arg(long)]
        report: String,
        #[arg(long)]
        curation: Option<String>,
    },
    /// Independent end-to-end audit of a pipeline output.
    Audit {
        #[arg(long)]
        out: String,
        #[arg(long)]
        manifest: String,
    },
    /// Check an annotated graph against the contract invariants.
    Validate { annotated: String },
    /// Print a human summary of a mapping report.
    Report { report: String },
    /// Resolve a single (type, name) against SPOKE.
    Lookup {
        name: String,
        #[arg(long = "type")]
        node_type: String,
        #[arg(long, default_value = "general")]
        subtype: String,
    },
    /// Link a tiny fixture and assert invariants.
    Selftest,
}

async fn connect() -> Result<Client> {
    let cfg = config::load()?;
    let graph = config::connect(&cfg).await?;
    Ok(Client::new(graph, false))
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("error: {:#}", e);
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Commands::Query { cypher, stdout: _ } => {
            let client = connect().await?;
            let rows = client.query(&cypher).await?;
            println!("{}", serde_json::to_string_pretty(&rows)?);
        }
        Commands::TestConnection => {
            let cfg = config::load()?;
            print!("Connecting to {} ... ", cfg.uri);
            let client = connect().await?;
            let rows = client.query("RETURN 1 AS ok").await?;
            let ok = rows.first().and_then(|r| r.get("ok")).and_then(|v| v.as_i64()) == Some(1);
            println!("{}", if ok { "OK" } else { "FAILED" });
            println!("  uri      : {}", cfg.uri);
            println!("  database : {}", cfg.db);
            println!("  user     : {}", cfg.user);
            if !ok {
                std::process::exit(2);
            }
        }
        Commands::Link { export, out, report, adjudications, canonicalize, validate } => {
            let client = connect().await?;
            let rep = run_link(&client, &export, &out, &report, adjudications.as_deref(), canonicalize).await?;
            print_report(&rep);
            if validate {
                let data: Value = serde_json::from_str(&std::fs::read_to_string(&out)?)?;
                let (ok, problems) = pipeline::validate_graph(&data);
                println!("\nVALIDATION: {}", if ok { "PASS" } else { "FAIL" });
                for p in &problems {
                    println!("  - {}", p);
                }
                if !ok {
                    std::process::exit(2);
                }
            }
        }
        Commands::Pipeline { export, out, report, curation } => {
            let client = connect().await?;
            let mut pl = pipeline::Pipeline::new(&client, true);
            let result = pl.run(&export, &out, &report, curation.as_deref()).await?;
            let stats = client.stats();
            eprintln!("spoke queries={} cache_hits={}", stats.queries, stats.cache_hits);
            if result != "PASS" {
                std::process::exit(2);
            }
        }
        Commands::Audit { out, manifest } => {
            let client = connect().await?;
            let ok = pipeline::audit(&out, &manifest, &client, 300).await?;
            if !ok {
                std::process::exit(2);
            }
        }
        Commands::Validate { annotated } => {
            let data: Value = serde_json::from_str(&std::fs::read_to_string(&annotated)?)?;
            let (ok, problems) = pipeline::validate_graph(&data);
            println!("VALIDATION: {}", if ok { "PASS" } else { "FAIL" });
            for p in &problems {
                println!("  - {}", p);
            }
            if !ok {
                std::process::exit(2);
            }
        }
        Commands::Report { report } => {
            let rep: Value = serde_json::from_str(&std::fs::read_to_string(&report)?)?;
            print_report(&rep);
        }
        Commands::Lookup { name, node_type, subtype } => {
            let client = connect().await?;
            let tx = TypeCrosswalk::load()?;
            let r = NodeResolver::new(&client, &tx);
            let node = BNode {
                identifier: name.clone(),
                node_type: Some(node_type),
                subtype: Some(subtype),
                synonyms: vec![],
                xref: vec![],
            };
            let matches = r.resolve_all(&[node]).await?;
            let m = matches.get(&name).unwrap();
            println!("{}", serde_json::to_string_pretty(&m.to_json("spoke"))?);
        }
        Commands::Selftest => {
            run_selftest().await?;
        }
    }
    Ok(())
}

/// Annotate nodes + edges of an export in place (ungated path for `link` / `selftest`).
async fn annotate_graph(
    client: &Client,
    export: &mut Value,
    matches: &HashMap<String, NodeMatch>,
    px: &PredicateCrosswalk,
) -> Result<()> {
    // (F4) validate graph shape before the unwraps below, so a malformed export errors gracefully
    // instead of panicking (the gated pipeline checks this; the ungated `link` path did not).
    if export.pointer("/graph/nodes").and_then(|v| v.as_array()).is_none()
        || export.pointer("/graph/edges").and_then(|v| v.as_array()).is_none()
    {
        anyhow::bail!("export missing graph.nodes / graph.edges arrays");
    }
    {
        let nodes = export["graph"]["nodes"].as_array_mut().unwrap();
        for gn in nodes.iter_mut() {
            let id = gn.get("id").and_then(|v| v.as_str()).map(|s| s.to_string());
            let ann = match id.as_deref().and_then(|i| matches.get(i)) {
                Some(m) => m.to_json("spoke"),
                None => annotate::node_fallback("node id absent from bundle pages"),
            };
            gn["spoke"] = ann;
        }
    }
    let checker = EdgeChecker::new(client, px);
    let edges: Vec<Value> = export["graph"]["edges"].as_array().unwrap().clone();
    let anns = pipeline::check_edges_concurrent(&checker, &edges, matches).await?;
    let edges_mut = export["graph"]["edges"].as_array_mut().unwrap();
    for (e, ann) in edges_mut.iter_mut().zip(anns) {
        e["spoke"] = ann;
    }
    Ok(())
}

async fn run_link(
    client: &Client,
    export_path: &str,
    out_path: &str,
    report_path: &str,
    adjudications: Option<&str>,
    canonicalize: bool,
) -> Result<Value> {
    let tx = TypeCrosswalk::load()?;
    let px = PredicateCrosswalk::load()?;
    let (mut export, bnodes) = annotate::load_export(export_path)?;
    let mut matches = NodeResolver::new(client, &tx).resolve_all(&bnodes).await?;
    if let Some(adj) = adjudications {
        let parsed: Value = serde_json::from_str(&std::fs::read_to_string(adj)?)?;
        let decisions: Vec<Value> = match parsed {
            Value::Array(a) => a,
            Value::Object(ref o) => o.get("decisions").and_then(|v| v.as_array()).cloned().unwrap_or_default(),
            _ => vec![],
        };
        annotate::apply_adjudications(&mut matches, &decisions);
    }
    if canonicalize {
        annotate::canonicalize_duplicates(&mut matches, client).await?;
    }
    annotate_graph(client, &mut export, &matches, &px).await?;
    let report = annotate::build_report(
        export["graph"]["nodes"].as_array().unwrap(),
        export["graph"]["edges"].as_array().unwrap(),
        client.stats(),
    );
    export["spoke_report"] = report.clone();
    std::fs::write(out_path, serde_json::to_string_pretty(&export)?)?;
    std::fs::write(report_path, serde_json::to_string_pretty(&report)?)?;
    Ok(report)
}

async fn run_selftest() -> Result<()> {
    let client = connect().await?;
    let tx = TypeCrosswalk::load()?;
    let px = PredicateCrosswalk::load()?;
    let fixture = json!({
        "node_count": 3, "edge_count": 1,
        "pages": {
            "PTPN22": {"node_type": "Gene", "subtype": "protein_coding", "synonyms": ["LYP"], "xref": []},
            "type 1 diabetes mellitus": {"node_type": "Disease", "subtype": "general", "synonyms": [], "xref": []},
            "Nonexistent Xyzzy Entity": {"node_type": "Gene", "subtype": "general", "synonyms": [], "xref": []}
        },
        "graph": {
            "nodes": [
                {"id": "PTPN22", "type": "Gene"},
                {"id": "type 1 diabetes mellitus", "type": "Disease"},
                {"id": "Nonexistent Xyzzy Entity", "type": "Gene"}
            ],
            "edges": [
                {"source": "PTPN22", "target": "type 1 diabetes mellitus", "predicate": "associated_with"}
            ]
        }
    });
    let tmp = config::root().join("_selftest_fixture.json");
    std::fs::write(&tmp, serde_json::to_string(&fixture)?)?;
    let (mut export, bnodes) = annotate::load_export(tmp.to_str().unwrap())?;
    let matches = NodeResolver::new(&client, &tx).resolve_all(&bnodes).await?;
    annotate_graph(&client, &mut export, &matches, &px).await?;
    let _ = std::fs::remove_file(&tmp);
    let (ok, problems) = pipeline::validate_graph(&export);
    let nodes: HashMap<String, Value> = export["graph"]["nodes"].as_array().unwrap().iter().map(|n| (n["id"].as_str().unwrap().to_string(), n["spoke"].clone())).collect();
    let edge = &export["graph"]["edges"][0]["spoke"];
    let g = |id: &str, k: &str| nodes[id].get(k).cloned().unwrap_or(Value::Null);
    println!("PTPN22: {} {}", g("PTPN22", "mapping_status"), g("PTPN22", "target_identifier"));
    println!("T1DM  : {} {}", g("type 1 diabetes mellitus", "mapping_status"), g("type 1 diabetes mellitus", "target_identifier"));
    println!("bogus : {}", g("Nonexistent Xyzzy Entity", "mapping_status"));
    println!("edge  : {} {}", edge["support_status"], edge["found_relation_types"]);
    println!("invariants: {}", if ok { "PASS".to_string() } else { format!("FAIL {:?}", problems) });
    assert_eq!(nodes["PTPN22"]["mapping_status"].as_str(), Some("mapped"), "PTPN22 should map");
    let bogus = nodes["Nonexistent Xyzzy Entity"]["mapping_status"].as_str();
    assert!(bogus == Some("not_found") || bogus == Some("ambiguous"));
    assert!(ok, "invariants must hold");
    println!("SELFTEST PASS");
    Ok(())
}

fn print_report(rep: &Value) {
    let t = &rep["totals"];
    let f = |k: &str| t.get(k).and_then(|v| v.as_f64()).unwrap_or(0.0);
    let i = |k: &str| t.get(k).and_then(|v| v.as_i64()).unwrap_or(0);
    println!("{}", "=".repeat(64));
    println!(
        "NODES {}  mapped {} ({:.1}% overall, {:.1}% of mappable)",
        i("nodes"),
        i("nodes_mapped"),
        f("node_map_rate_overall") * 100.0,
        f("node_map_rate_mappable") * 100.0
    );
    println!("EDGES {}  supported {}  contradicted {}", i("edges"), i("edges_supported"), i("edges_contradicted"));
    println!("{}", "-".repeat(64));
    println!("mapping_status: {}", rep["mapping_status"]);
    println!("support_status: {}", rep["support_status"]);
    println!("{}", "-".repeat(64));
    println!("node map-rate by type:");
    if let Some(bt) = rep["mapping_status_by_type"].as_object() {
        for (typ, counts) in bt {
            let tot: i64 = counts.as_object().map(|o| o.values().filter_map(|v| v.as_i64()).sum()).unwrap_or(0);
            let mp = counts.get("mapped").and_then(|v| v.as_i64()).unwrap_or(0);
            let pct = if tot > 0 { mp as f64 / tot as f64 * 100.0 } else { 0.0 };
            println!("  {:20} {:>4}/{:<4} ({:.0}%)  {}", typ, mp, tot, pct, counts);
        }
    }
}
