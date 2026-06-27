//! `bokf` — the BioOKF command-line tool. Thin wrapper over `bokf-core`; this is
//! the primary terminal surface an AI agent (or human) drives.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use bokf_core::git::{today_iso, ChangeKind, GitRepo};
use bokf_core::lint::Severity;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "bokf", version, about = "BioOKF knowledge-base toolkit")]
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
    /// Append a dated log.md entry AND commit, atomically (the sole step-committer).
    LogSync {
        path: PathBuf,
        #[arg(long)]
        kind: String,
        #[arg(long)]
        summary: String,
        #[arg(long)]
        delta: Option<String>,
    },
    /// Lower-level: stage all + commit (non-logged lifecycle commit).
    Commit {
        path: PathBuf,
        #[arg(long)]
        kind: String,
        #[arg(long)]
        summary: String,
        #[arg(long)]
        delta: Option<String>,
    },
    /// Show commit history (newest-first).
    Log {
        path: PathBuf,
        #[arg(long, default_value_t = 20)]
        limit: usize,
        #[arg(long)]
        json: bool,
    },
    /// Forward-only restore to a prior commit.
    Restore {
        path: PathBuf,
        sha: String,
        #[arg(long)]
        summary: Option<String>,
    },
    /// Set the active KB id under <root>.
    SetActive { root: PathBuf, kb_id: String },
    /// Print the active KB id + resolved path under <root>.
    GetActive {
        root: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// Register/list/unregister a known bundle under <root>.
    Register {
        root: PathBuf,
        kb_id: Option<String>,
        path: Option<PathBuf>,
        #[arg(long)]
        list: bool,
        #[arg(long)]
        unregister: Option<String>,
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
        Cmd::LogSync { path, kind, summary, delta } => cmd_log_sync(path, kind, summary, delta),
        Cmd::Commit { path, kind, summary, delta } => cmd_commit(path, kind, summary, delta),
        Cmd::Log { path, limit, json } => cmd_log(path, limit, json),
        Cmd::Restore { path, sha, summary } => cmd_restore(path, sha, summary),
        Cmd::SetActive { root, kb_id } => cmd_set_active(root, kb_id),
        Cmd::GetActive { root, json } => cmd_get_active(root, json),
        Cmd::Register { root, kb_id, path, list, unregister } => cmd_register(root, kb_id, path, list, unregister),
    }
}

fn cmd_set_active(root: PathBuf, kb_id: String) -> Result<()> {
    bokf_core::active::set_active(&root, Some(&kb_id)).map_err(anyhow::Error::msg)?;
    eprintln!("active KB = {kb_id}");
    Ok(())
}

fn cmd_get_active(root: PathBuf, json: bool) -> Result<()> {
    match bokf_core::active::get_active(&root) {
        Some(id) => {
            let path = bokf_core::registry::resolve(&root, &id);
            if json {
                println!("{}", serde_json::json!({"id": id, "path": path}));
            } else {
                println!("{id}  {}", path.as_deref().unwrap_or("(unregistered path)"));
            }
        }
        None => {
            if json {
                println!("{}", serde_json::json!({ "id": null }));
            } else {
                println!("(no active KB — run `bokf set-active`)");
            }
        }
    }
    Ok(())
}

fn cmd_register(root: PathBuf, kb_id: Option<String>, path: Option<PathBuf>, list: bool, unregister: Option<String>) -> Result<()> {
    if list {
        for b in bokf_core::registry::list(&root) {
            println!("{}  {}", b.id, b.path);
        }
        return Ok(());
    }
    if let Some(id) = unregister {
        bokf_core::registry::unregister(&root, &id).map_err(anyhow::Error::msg)?;
        return Ok(());
    }
    match (kb_id, path) {
        (Some(id), Some(p)) => bokf_core::registry::register(&root, &id, &p.to_string_lossy()).map_err(anyhow::Error::msg)?,
        _ => anyhow::bail!("register needs <kb_id> <path>, or --list, or --unregister <id>"),
    }
    Ok(())
}

fn cmd_log_sync(path: PathBuf, kind: String, summary: String, delta: Option<String>) -> Result<()> {
    let sha = bokf_core::log_sync::log_sync(&path, ChangeKind::parse(&kind), &summary, delta.as_deref(), &today_iso())
        .map_err(anyhow::Error::msg)?;
    eprintln!("[{}] {} — {}", kind, summary, &sha[..8.min(sha.len())]);
    Ok(())
}

fn cmd_commit(path: PathBuf, kind: String, summary: String, delta: Option<String>) -> Result<()> {
    let sha = GitRepo::open(&path)
        .commit_all(ChangeKind::parse(&kind), &summary, delta.as_deref())
        .map_err(anyhow::Error::msg)?;
    eprintln!("{}", &sha[..8.min(sha.len())]);
    Ok(())
}

fn cmd_log(path: PathBuf, limit: usize, json: bool) -> Result<()> {
    let entries = GitRepo::open(&path).log(limit).map_err(anyhow::Error::msg)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&entries)?);
    } else {
        for e in &entries {
            println!("{}  [{}] {}  {}", &e.commit_sha[..8.min(e.commit_sha.len())], e.kind.as_str(), e.summary, e.delta.as_deref().unwrap_or(""));
        }
    }
    Ok(())
}

fn cmd_restore(path: PathBuf, sha: String, summary: Option<String>) -> Result<()> {
    let new = GitRepo::open(&path).restore_to(&sha, summary.as_deref()).map_err(anyhow::Error::msg)?;
    eprintln!("restored; new commit {}", &new[..8.min(new.len())]);
    Ok(())
}

fn cmd_predicates(json: bool) -> Result<()> {
    use bokf_core::model::{AGENT_TYPES, KNOWLEDGE_LEVELS, NODE_TYPES, PREDICATES};
    if json {
        let v = serde_json::json!({
            "node_types": NODE_TYPES.as_slice(),
            "predicates": PREDICATES.as_slice(),
            "knowledge_levels": KNOWLEDGE_LEVELS.as_slice(),
            "agent_types": AGENT_TYPES.as_slice(),
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
    let v = bokf_core::validate::validate_doc(&content);
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
    let bundle = bokf_core::open_bundle(&path)?;
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
    let doc = bokf_core::export::bundle_doc(&path, name)?;
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
    let bundle = bokf_core::open_bundle(&path).with_context(|| format!("opening {}", path.display()))?;
    let report = bokf_core::lint(&bundle);
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
    let graph = bokf_core::graph_of(&path)?;
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
    let bundle = bokf_core::open_bundle(&path)?;
    let index = bokf_core::SearchIndex::build(&bundle);
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
    let bundle = bokf_core::open_bundle(&path)?;
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
        "# BioOKF operating schema (v0.5)\n\nSee the canonical schema.md for the 28 node types and 24 predicates.\n",
    )?;

    // version-track + register + activate the new bundle (so the first post-scaffold
    // convert/ingest is not denied by the require-active guardrail).
    let repo = GitRepo::open(&path);
    if repo.ensure_repo().is_ok() {
        let _ = repo.commit_all(ChangeKind::Manual, &format!("create knowledge base {name}"), None);
    }
    let kb_id = path.file_name().map(|s| s.to_string_lossy().to_lowercase());
    if let (Some(id), Some(root)) = (kb_id, path.parent()) {
        if bokf_core::registry::validate_kb_id(&id).is_ok() {
            let abs = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
            let _ = bokf_core::registry::register(root, &id, &abs.to_string_lossy());
            let _ = bokf_core::active::set_active(root, Some(&id));
        }
    }
    eprintln!("scaffolded bundle at {}", path.display());
    Ok(())
}

fn write_if_absent(path: &std::path::Path, content: &str) -> Result<()> {
    if !path.exists() {
        std::fs::write(path, content)?;
    }
    Ok(())
}
