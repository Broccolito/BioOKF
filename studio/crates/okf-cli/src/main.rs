//! `okf` — the BioOKF command-line tool. Thin wrapper over `okf-core`; this is
//! the primary terminal surface an AI agent (or human) drives.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use okf_core::lint::Severity;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "okf", version, about = "BioOKF knowledge-base toolkit")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Lint a bundle against the BioOKF v0.5 conformance rules.
    Lint {
        path: PathBuf,
        /// Emit the report as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Derive the render-ready graph (nodes + directional edges).
    Graph {
        path: PathBuf,
        /// Write JSON to this file instead of stdout.
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// BM25 full-text search over the bundle's concept documents.
    Search {
        path: PathBuf,
        query: String,
        #[arg(long, default_value_t = 10)]
        limit: usize,
        #[arg(long)]
        json: bool,
    },
    /// Summary statistics: node/edge counts by type/predicate.
    Stats { path: PathBuf },
    /// Scaffold an empty BioOKF bundle (raw/, knowledge/, index.md, log.md, schema.md).
    Scaffold {
        path: PathBuf,
        #[arg(long, default_value = "Untitled knowledge base")]
        name: String,
    },
    /// Validate a single concept-document file (validate-before-write).
    Validate { file: PathBuf },
    /// Look up a node by exact identifier (to reuse, never fork).
    Get { path: PathBuf, identifier: String },
    /// Export a self-contained bundle JSON (graph + per-node detail) for the GUI.
    Export {
        path: PathBuf,
        #[arg(long)]
        out: PathBuf,
        /// Display name for the bundle (defaults to the directory name).
        #[arg(long)]
        name: Option<String>,
    },
    /// Print the active controlled vocabulary (28 types, 24 predicates, enums).
    Predicates {
        #[arg(long)]
        json: bool,
    },
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e:#}");
        std::process::exit(2);
    }
}

fn run() -> Result<()> {
    match Cli::parse().cmd {
        Cmd::Lint { path, json } => cmd_lint(path, json),
        Cmd::Graph { path, out } => cmd_graph(path, out),
        Cmd::Search { path, query, limit, json } => cmd_search(path, query, limit, json),
        Cmd::Stats { path } => cmd_stats(path),
        Cmd::Scaffold { path, name } => cmd_scaffold(path, name),
        Cmd::Validate { file } => cmd_validate(file),
        Cmd::Get { path, identifier } => cmd_get(path, identifier),
        Cmd::Export { path, out, name } => cmd_export(path, out, name),
        Cmd::Predicates { json } => cmd_predicates(json),
    }
}

fn cmd_predicates(json: bool) -> Result<()> {
    use okf_core::model::{AGENT_TYPES, KNOWLEDGE_LEVELS, NODE_TYPES, PREDICATES};
    if json {
        let v = serde_json::json!({
            "node_types": NODE_TYPES,
            "predicates": PREDICATES,
            "knowledge_levels": KNOWLEDGE_LEVELS,
            "agent_types": AGENT_TYPES,
        });
        println!("{}", serde_json::to_string_pretty(&v)?);
    } else {
        println!("node types ({}):\n  {}", NODE_TYPES.len(), NODE_TYPES.join(", "));
        println!("predicates ({}):\n  {}", PREDICATES.len(), PREDICATES.join(", "));
        println!("knowledge_level: {}", KNOWLEDGE_LEVELS.join(", "));
        println!("agent_type: {}", AGENT_TYPES.join(", "));
    }
    Ok(())
}

fn cmd_validate(file: PathBuf) -> Result<()> {
    let content = std::fs::read_to_string(&file).with_context(|| format!("reading {}", file.display()))?;
    let v = okf_core::validate::validate_doc(&content);
    if v.valid {
        println!("VALID — type={} identifier={:?} {} edge(s)", v.node_type, v.identifier, v.edge_count);
    } else {
        println!("INVALID — type={} identifier={:?}", v.node_type, v.identifier);
    }
    for issue in &v.issues {
        println!("  - {issue}");
    }
    if !v.valid {
        std::process::exit(1);
    }
    Ok(())
}

fn cmd_get(path: PathBuf, identifier: String) -> Result<()> {
    let bundle = okf_core::open_bundle(&path)?;
    match bundle.get(&identifier) {
        Some(n) => {
            println!("{}", serde_json::to_string_pretty(n)?);
            Ok(())
        }
        None => {
            eprintln!("not found: `{identifier}` (no node with this identifier — safe to create a new one)");
            std::process::exit(1);
        }
    }
}

fn cmd_export(path: PathBuf, out: PathBuf, name: Option<String>) -> Result<()> {
    let doc = okf_core::export::bundle_doc(&path, name)?;
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&out, serde_json::to_string(&doc)?)?;
    eprintln!(
        "exported {} ({} nodes) -> {}",
        doc.get("name").and_then(|v| v.as_str()).unwrap_or(""),
        doc.get("node_count").and_then(|v| v.as_u64()).unwrap_or(0),
        out.display()
    );
    Ok(())
}

fn cmd_lint(path: PathBuf, json: bool) -> Result<()> {
    let bundle = okf_core::open_bundle(&path).with_context(|| format!("opening {}", path.display()))?;
    let report = okf_core::lint(&bundle);
    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        for f in &report.findings {
            let tag = match f.severity {
                Severity::Error => "ERROR",
                Severity::Warn => "WARN ",
                Severity::Info => "INFO ",
            };
            let loc = f.path.as_deref().unwrap_or("");
            println!("{tag} [{}] {}: {}  {}", f.rule, f.subject, f.message, loc);
        }
        println!(
            "\n{} nodes · {} errors · {} warnings · {} infos",
            bundle.nodes.len(),
            report.errors(),
            report.warnings(),
            report.infos()
        );
    }
    if report.errors() > 0 {
        std::process::exit(1);
    }
    Ok(())
}

fn cmd_graph(path: PathBuf, out: Option<PathBuf>) -> Result<()> {
    let graph = okf_core::graph_of(&path)?;
    let json = serde_json::to_string_pretty(&graph.to_json())?;
    match out {
        Some(p) => {
            std::fs::write(&p, json)?;
            eprintln!("wrote {} nodes, {} edges to {}", graph.nodes.len(), graph.edges.len(), p.display());
        }
        None => println!("{json}"),
    }
    Ok(())
}

fn cmd_search(path: PathBuf, query: String, limit: usize, json: bool) -> Result<()> {
    let bundle = okf_core::open_bundle(&path)?;
    let index = okf_core::SearchIndex::build(&bundle);
    let hits = index.search(&query, limit);
    if json {
        println!("{}", serde_json::to_string_pretty(&hits)?);
    } else {
        for h in &hits {
            println!("{:.3}  [{}] {}\n        {}", h.score, h.node_type, h.identifier, h.snippet);
        }
        println!("\n{} hits", hits.len());
    }
    Ok(())
}

fn cmd_stats(path: PathBuf) -> Result<()> {
    let bundle = okf_core::open_bundle(&path)?;
    let mut by_type: BTreeMap<String, usize> = BTreeMap::new();
    let mut by_pred: BTreeMap<String, usize> = BTreeMap::new();
    let mut edge_count = 0;
    for n in &bundle.nodes {
        *by_type.entry(n.node_type.as_str().to_string()).or_default() += 1;
        for e in &n.edges {
            edge_count += 1;
            *by_pred.entry(e.predicate.as_str().to_string()).or_default() += 1;
        }
    }
    println!("Bundle: {}", path.display());
    println!("  {} nodes, {} edges", bundle.nodes.len(), edge_count);
    println!("  reserved: index.md={} log.md={} schema.md={}", bundle.has_index_md, bundle.has_log_md, bundle.has_schema_md);
    if !bundle.parse_errors.is_empty() {
        println!("  parse errors: {}", bundle.parse_errors.len());
    }
    println!("\nNodes by type:");
    for (t, c) in &by_type {
        println!("  {c:>4}  {t}");
    }
    println!("\nEdges by predicate:");
    for (p, c) in &by_pred {
        println!("  {c:>4}  {p}");
    }
    Ok(())
}

fn cmd_scaffold(path: PathBuf, name: String) -> Result<()> {
    std::fs::create_dir_all(path.join("raw"))?;
    std::fs::create_dir_all(path.join("knowledge"))?;
    let index = format!(
        "# {name}\n\n> BioOKF bundle index (catalog of concept pages).\n\nokf_version: 0.5\nbiookf_version: 0.5\n"
    );
    write_if_absent(&path.join("index.md"), &index)?;
    write_if_absent(&path.join("log.md"), &format!("# Change log — {name}\n"))?;
    write_if_absent(
        &path.join("schema.md"),
        "# BioOKF operating schema (v0.5)\n\nSee the canonical schema.md for the 28 node types and 23 predicates.\n",
    )?;
    eprintln!("scaffolded bundle at {}", path.display());
    Ok(())
}

fn write_if_absent(path: &std::path::Path, content: &str) -> Result<()> {
    if !path.exists() {
        std::fs::write(path, content)?;
    }
    Ok(())
}
