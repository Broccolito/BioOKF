//! `bokf`, the BioOKF command-line tool. Thin wrapper over `bokf-core`; this is
//! the primary terminal surface an AI agent (or human) drives.

use anyhow::{Context, Result};
use bokf_core::git::{today_iso, ChangeKind, GitRepo};
use bokf_core::lint::Severity;
use clap::{Parser, Subcommand};
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
    /// Summary statistics over the render-ready graph, with optional filters.
    Stats {
        path: PathBuf,
        /// Include only these node types (repeatable). Omit for all node types.
        #[arg(long = "node-type")]
        node_types: Vec<String>,
        /// Include only these edge predicates (repeatable). Omit for all predicates.
        #[arg(long = "edge-type")]
        edge_types: Vec<String>,
        /// Exclude synthesized provenance edges.
        #[arg(long)]
        no_synthesized: bool,
        /// Emit the report as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Control the live Studio graph filter over the GUI control socket.
    StudioFilter {
        /// Visible node types (repeatable). Omit to leave the current node filter unchanged.
        #[arg(long = "node-type")]
        node_types: Vec<String>,
        /// Visible edge predicates (repeatable). Omit to leave the current edge filter unchanged.
        #[arg(long = "edge-type")]
        edge_types: Vec<String>,
        /// Clear all graph filters in the running Studio.
        #[arg(long)]
        reset: bool,
        /// Emit the resulting Studio filter state as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Read post-filter statistics from the running Studio graph.
    StudioStats {
        /// Emit JSON. Human output is compact by default.
        #[arg(long)]
        json: bool,
    },
    /// Scaffold an empty BioOKF bundle (raw/, knowledge/, index.md, log.md, SCHEMA.md).
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
    /// Set the active KB id (defaults to the config dir).
    SetActive {
        #[arg(long)]
        root: Option<PathBuf>,
        kb_id: String,
    },
    /// Print the active KB id + resolved path (defaults to the config dir).
    GetActive {
        #[arg(long)]
        root: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
    /// Register/list/unregister a known bundle (defaults to the config dir).
    Register {
        #[arg(long)]
        root: Option<PathBuf>,
        kb_id: Option<String>,
        path: Option<PathBuf>,
        #[arg(long)]
        list: bool,
        #[arg(long)]
        unregister: Option<String>,
    },
    /// Deterministic accountability gate: lint + structure checks; exits 1 on any error.
    Verify {
        path: PathBuf,
        #[arg(long)]
        workflow: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Convert a file/folder/zip/URL (or --text) to raw Markdown under <bundle>'s raw/.
    Convert {
        path: Option<PathBuf>,
        #[arg(long)]
        text: Option<String>,
        #[arg(long)]
        title: Option<String>,
        /// Download and ingest a single URL (classifies its source provenance).
        #[arg(long)]
        url: Option<String>,
        /// Download and ingest a list of URLs, one per line (blank lines and `#` comments skipped).
        #[arg(long)]
        urls: Option<PathBuf>,
        /// Bundle to write raw/ into.
        #[arg(long)]
        into: PathBuf,
        /// Concatenate archive/folder members into one source.
        #[arg(long)]
        combined: bool,
        #[arg(long)]
        json: bool,
    },
    /// Install the PDFium library so PDF pages render to images for vision (one-time, automatic).
    InstallPdfium {
        /// Install directory (default: $BIOOKF_PDFIUM_DIR or ~/.biookf).
        #[arg(long)]
        dir: Option<PathBuf>,
        /// Only report whether PDF page rendering is available (exit 0) or not (exit 1); do not install.
        #[arg(long)]
        check: bool,
    },
    /// Rename a provisional figure to a content name and rewrite every reference.
    NameFigure {
        bundle: PathBuf,
        #[arg(long)]
        source: String,
        #[arg(long)]
        figure: String,
        #[arg(long = "as")]
        caption: String,
        #[arg(long)]
        json: bool,
    },
    /// Regenerate index.md (identifier registry + by-type catalog + subtypes-in-use), or --check it.
    Index {
        path: PathBuf,
        #[arg(long)]
        check: bool,
    },
    /// Relocate a Secondary KB's raw/ into the Main KB's raw/ (dedup by content, rename on collision).
    MergeRaw {
        mkb: PathBuf,
        skb: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// Snapshot the MKB identifier/path set before a merge (default), or --verify it stayed canonical after.
    MergeSnapshot {
        mkb: PathBuf,
        #[arg(long)]
        verify: bool,
    },
}

/// The config/bundle root for registry + active-pointer ops: an explicit
/// `--root` if given, else the canonical config dir (~/.config/biookf-studio).
fn resolve_root(root: Option<PathBuf>) -> PathBuf {
    root.unwrap_or_else(|| {
        bokf_core::config::ensure_config_dir().unwrap_or_else(|_| bokf_core::config::config_dir())
    })
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
        Cmd::Search {
            path,
            query,
            limit,
            json,
        } => cmd_search(path, query, limit, json),
        Cmd::Stats {
            path,
            node_types,
            edge_types,
            no_synthesized,
            json,
        } => cmd_stats(path, node_types, edge_types, !no_synthesized, json),
        Cmd::StudioFilter {
            node_types,
            edge_types,
            reset,
            json,
        } => cmd_studio_filter(node_types, edge_types, reset, json),
        Cmd::StudioStats { json } => cmd_studio_stats(json),
        Cmd::Scaffold { path, name } => cmd_scaffold(path, name),
        Cmd::Validate { file } => cmd_validate(file),
        Cmd::Get { path, identifier } => cmd_get(path, identifier),
        Cmd::Export { path, out, name } => cmd_export(path, out, name),
        Cmd::Predicates { json } => cmd_predicates(json),
        Cmd::LogSync {
            path,
            kind,
            summary,
            delta,
        } => cmd_log_sync(path, kind, summary, delta),
        Cmd::Commit {
            path,
            kind,
            summary,
            delta,
        } => cmd_commit(path, kind, summary, delta),
        Cmd::Log { path, limit, json } => cmd_log(path, limit, json),
        Cmd::Restore { path, sha, summary } => cmd_restore(path, sha, summary),
        Cmd::SetActive { root, kb_id } => cmd_set_active(resolve_root(root), kb_id),
        Cmd::GetActive { root, json } => cmd_get_active(resolve_root(root), json),
        Cmd::Register {
            root,
            kb_id,
            path,
            list,
            unregister,
        } => cmd_register(resolve_root(root), kb_id, path, list, unregister),
        Cmd::Verify {
            path,
            workflow,
            json,
        } => cmd_verify(path, workflow, json),
        Cmd::Convert {
            path,
            text,
            title,
            url,
            urls,
            into,
            combined,
            json,
        } => cmd_convert(path, text, title, url, urls, into, combined, json),
        Cmd::InstallPdfium { dir, check } => cmd_install_pdfium(dir, check),
        Cmd::NameFigure {
            bundle,
            source,
            figure,
            caption,
            json,
        } => cmd_name_figure(bundle, source, figure, caption, json),
        Cmd::Index { path, check } => cmd_index(path, check),
        Cmd::MergeRaw { mkb, skb, json } => cmd_merge_raw(mkb, skb, json),
        Cmd::MergeSnapshot { mkb, verify } => cmd_merge_snapshot(mkb, verify),
    }
}

fn cmd_index(path: PathBuf, check: bool) -> Result<()> {
    let bundle = bokf_core::open_bundle(&path)?;
    if check {
        let missing = bokf_core::index::missing_from_index(&bundle);
        if missing.is_empty() {
            println!("index.md is current ({} nodes)", bundle.nodes.len());
        } else {
            for m in &missing {
                println!("MISSING from index.md: {m}");
            }
            std::process::exit(1);
        }
    } else {
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Knowledge base".into());
        bokf_core::index::write_index(&bundle, &name)?;
        eprintln!("regenerated index.md ({} nodes)", bundle.nodes.len());
    }
    Ok(())
}

fn cmd_merge_raw(mkb: PathBuf, skb: PathBuf, json: bool) -> Result<()> {
    let res = bokf_core::merge::merge_raw(&mkb, &skb).map_err(anyhow::Error::msg)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&res)?);
    } else {
        eprintln!(
            "merge-raw: {} moved, {} renamed, {} dropped (duplicates)",
            res.moved.len(),
            res.renamed.len(),
            res.dropped_duplicates.len()
        );
        for (old, new) in &res.id_map {
            if old != new {
                println!("raw/{old} -> raw/{new}");
            }
        }
    }
    Ok(())
}

fn cmd_merge_snapshot(mkb: PathBuf, verify: bool) -> Result<()> {
    let bundle = bokf_core::open_bundle(&mkb)?;
    if verify {
        let issues =
            bokf_core::merge::verify_snapshot(&mkb, &bundle).map_err(anyhow::Error::msg)?;
        if issues.is_empty() {
            println!("MKB unchanged since snapshot ✓");
        } else {
            for i in &issues {
                println!("CHANGED: {i}");
            }
            std::process::exit(1);
        }
    } else {
        bokf_core::merge::write_snapshot(&mkb, &bokf_core::merge::snapshot(&bundle))
            .map_err(anyhow::Error::msg)?;
        eprintln!(
            "pre-merge snapshot written ({} identifiers)",
            bundle.nodes.len()
        );
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn cmd_convert(
    path: Option<PathBuf>,
    text: Option<String>,
    title: Option<String>,
    url: Option<String>,
    urls: Option<PathBuf>,
    into: PathBuf,
    combined: bool,
    json: bool,
) -> Result<()> {
    use bokf_core::convert::{ingest, ingest_urls, SourceInput, SourceRecord};
    let results: Vec<std::result::Result<SourceRecord, String>> = if let Some(urls_file) = urls {
        let content = std::fs::read_to_string(&urls_file)
            .map_err(|e| anyhow::anyhow!("reading {}: {e}", urls_file.display()))?;
        let list: Vec<String> = content
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .collect();
        if list.is_empty() {
            anyhow::bail!("no URLs found in {}", urls_file.display());
        }
        ingest_urls(&into, list)
    } else {
        let input = if let Some(u) = url {
            SourceInput::Url(u)
        } else if let Some(t) = text {
            SourceInput::Text { text: t, title }
        } else if let Some(p) = path {
            SourceInput::Path(p)
        } else {
            anyhow::bail!("convert needs a <path>, --url, --urls <file>, or --text");
        };
        ingest(&into, input, combined)
            .map_err(anyhow::Error::msg)?
            .into_iter()
            .map(Ok)
            .collect()
    };
    if json {
        let ok: Vec<&SourceRecord> = results.iter().filter_map(|r| r.as_ref().ok()).collect();
        println!("{}", serde_json::to_string_pretty(&ok)?);
        for r in &results {
            if let Err(e) = r {
                eprintln!("FAILED: {e}");
            }
        }
    } else {
        for r in &results {
            match r {
                Ok(rec) => println!(
                    "{}  ({}{})  -> {}",
                    rec.source_id,
                    if rec.reused { "reused" } else { "new" },
                    if rec.needs_llm_fallback {
                        ", needs OCR/LLM"
                    } else {
                        ""
                    },
                    rec.source_md_path
                ),
                Err(e) => eprintln!("FAILED: {e}"),
            }
        }
    }
    // If a PDF was just ingested but page rasterization is unavailable, surface the one optional
    // step. PDFs still convert without it (the agent reads the PDF directly with vision).
    let ingested_pdf = results.iter().filter_map(|r| r.as_ref().ok()).any(|r| {
        into.join(format!("raw/{}/original.pdf", r.source_id))
            .exists()
    });
    if ingested_pdf && !bokf_core::pdf_raster::is_available() {
        eprintln!("Tip: run `bokf install-pdfium` once to render PDF pages as images for higher-fidelity vision reading. PDFs already convert without it.");
    }
    Ok(())
}

fn cmd_install_pdfium(dir: Option<PathBuf>, check: bool) -> Result<()> {
    if check {
        if bokf_core::pdf_raster::is_available() {
            println!("PDF page rendering is available.");
            return Ok(());
        }
        println!("PDF page rendering is not set up; run `bokf install-pdfium`.");
        std::process::exit(1);
    }
    if bokf_core::pdf_raster::is_available() {
        println!("PDFium is already available; PDF page rendering is enabled.");
        return Ok(());
    }
    eprintln!("Downloading the PDFium library (one-time, a few MB)...");
    let path = bokf_core::pdf_raster::install_pdfium(dir).map_err(anyhow::Error::msg)?;
    println!(
        "Installed PDFium to {}. PDF page rendering is now enabled.",
        path.display()
    );
    Ok(())
}

fn cmd_name_figure(
    bundle: PathBuf,
    source: String,
    figure: String,
    caption: String,
    json: bool,
) -> Result<()> {
    let new_rel = bokf_core::figures::name_figure(&bundle, &source, &figure, &caption)
        .map_err(anyhow::Error::msg)?;
    if json {
        println!(
            "{}",
            serde_json::json!({ "source": source, "figure": new_rel })
        );
    } else {
        println!("{source}  {figure} -> {new_rel}");
    }
    Ok(())
}

fn cmd_verify(path: PathBuf, workflow: Option<String>, json: bool) -> Result<()> {
    let bundle =
        bokf_core::open_bundle(&path).with_context(|| format!("opening {}", path.display()))?;
    let report = bokf_core::lint(&bundle);
    let ok = report.errors() == 0;
    let wf = workflow.unwrap_or_else(|| "any".to_string());
    if json {
        let v = serde_json::json!({
            "ok": ok,
            "workflow": wf,
            "errors": report.errors(),
            "warnings": report.warnings(),
            "infos": report.infos(),
            "has_index": bundle.has_index_md,
            "has_log": bundle.has_log_md,
            "findings": report.findings,
        });
        println!("{}", serde_json::to_string_pretty(&v)?);
    } else {
        println!(
            "verify [{wf}]: {} ({} errors, {} warnings, {} infos; index.md={}, log.md={})",
            if ok { "PASS" } else { "FAIL" },
            report.errors(),
            report.warnings(),
            report.infos(),
            bundle.has_index_md,
            bundle.has_log_md
        );
        for f in report
            .findings
            .iter()
            .filter(|f| f.severity != Severity::Info)
        {
            let tag = if f.severity == Severity::Error {
                "ERROR"
            } else {
                "WARN "
            };
            println!("  {tag} [{}] {}: {}", f.rule, f.subject, f.message);
        }
    }
    if !ok {
        std::process::exit(1);
    }
    Ok(())
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
                println!("(no active KB; run `bokf set-active`)");
            }
        }
    }
    Ok(())
}

fn cmd_register(
    root: PathBuf,
    kb_id: Option<String>,
    path: Option<PathBuf>,
    list: bool,
    unregister: Option<String>,
) -> Result<()> {
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
        (Some(id), Some(p)) => bokf_core::registry::register(&root, &id, &p.to_string_lossy())
            .map_err(anyhow::Error::msg)?,
        _ => anyhow::bail!("register needs <kb_id> <path>, or --list, or --unregister <id>"),
    }
    Ok(())
}

fn cmd_log_sync(path: PathBuf, kind: String, summary: String, delta: Option<String>) -> Result<()> {
    let sha = bokf_core::log_sync::log_sync(
        &path,
        ChangeKind::parse(&kind),
        &summary,
        delta.as_deref(),
        &today_iso(),
    )
    .map_err(anyhow::Error::msg)?;
    eprintln!("[{}] {}: {}", kind, summary, &sha[..8.min(sha.len())]);
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
    let entries = GitRepo::open(&path)
        .log(limit)
        .map_err(anyhow::Error::msg)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&entries)?);
    } else {
        for e in &entries {
            println!(
                "{}  [{}] {}  {}",
                &e.commit_sha[..8.min(e.commit_sha.len())],
                e.kind.as_str(),
                e.summary,
                e.delta.as_deref().unwrap_or("")
            );
        }
    }
    Ok(())
}

fn cmd_restore(path: PathBuf, sha: String, summary: Option<String>) -> Result<()> {
    let new = GitRepo::open(&path)
        .restore_to(&sha, summary.as_deref())
        .map_err(anyhow::Error::msg)?;
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
        println!(
            "node types ({}):\n  {}",
            NODE_TYPES.len(),
            NODE_TYPES.join(", ")
        );
        println!(
            "predicates ({}):\n  {}",
            PREDICATES.len(),
            PREDICATES.join(", ")
        );
        println!("knowledge_level: {}", KNOWLEDGE_LEVELS.join(", "));
        println!("agent_type: {}", AGENT_TYPES.join(", "));
    }
    Ok(())
}

fn cmd_validate(file: PathBuf) -> Result<()> {
    let content =
        std::fs::read_to_string(&file).with_context(|| format!("reading {}", file.display()))?;
    let v = bokf_core::validate::validate_doc(&content);
    if v.valid {
        println!(
            "VALID: type={} identifier={:?} {} edge(s)",
            v.node_type, v.identifier, v.edge_count
        );
    } else {
        println!(
            "INVALID: type={} identifier={:?}",
            v.node_type, v.identifier
        );
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
            eprintln!("not found: `{identifier}` (no node with this identifier; safe to create a new one)");
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
    let bundle =
        bokf_core::open_bundle(&path).with_context(|| format!("opening {}", path.display()))?;
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
            eprintln!(
                "wrote {} nodes, {} edges to {}",
                graph.nodes.len(),
                graph.edges.len(),
                p.display()
            );
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
            println!(
                "{:.3}  [{}] {}\n        {}",
                h.score, h.node_type, h.identifier, h.snippet
            );
        }
        println!("\n{} hits", hits.len());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cli_parses_name_figure() {
        use clap::Parser;
        let c = Cli::try_parse_from([
            "bokf",
            "name-figure",
            "kb",
            "--source",
            "x-1",
            "--figure",
            "figures/fig-001.png",
            "--as",
            "A B",
        ])
        .unwrap();
        assert!(matches!(c.cmd, Cmd::NameFigure { .. }));
    }

    #[test]
    fn cli_parses_convert_url() {
        use clap::Parser;
        let c = Cli::try_parse_from([
            "bokf",
            "convert",
            "--url",
            "https://x.org/a",
            "--into",
            "kb",
        ])
        .unwrap();
        if let Cmd::Convert { url, .. } = c.cmd {
            assert_eq!(url.as_deref(), Some("https://x.org/a"));
        } else {
            panic!("expected Convert");
        }
    }
}

fn cmd_stats(
    path: PathBuf,
    node_types: Vec<String>,
    edge_types: Vec<String>,
    include_synthesized: bool,
    json: bool,
) -> Result<()> {
    let bundle = bokf_core::open_bundle(&path)?;
    let graph = bokf_core::Graph::from_bundle(&bundle);
    let filter = bokf_core::GraphFilter {
        node_types,
        edge_predicates: edge_types,
        include_synthesized,
    };
    let stats = bokf_core::GraphStats::from_graph(&graph, &filter);
    if json {
        println!("{}", serde_json::to_string_pretty(&stats)?);
        return Ok(());
    }
    println!("Bundle: {}", path.display());
    println!("  {} nodes, {} edges", stats.nodes, stats.edges);
    println!(
        "  density: {:.6} · components: {} (largest {}) · isolated: {}",
        stats.density, stats.components.count, stats.components.largest, stats.isolated_nodes
    );
    println!(
        "  reserved: index.md={} log.md={} SCHEMA.md={}",
        bundle.has_index_md, bundle.has_log_md, bundle.has_schema_md
    );
    if !bundle.parse_errors.is_empty() {
        println!("  parse errors: {}", bundle.parse_errors.len());
    }
    println!("\nNodes by type:");
    for (t, c) in count_desc(&stats.by_node_type) {
        println!("  {c:>4}  {t}");
    }
    println!("\nEdges by predicate:");
    for (p, c) in count_desc(&stats.by_edge_predicate) {
        println!("  {c:>4}  {p}");
    }
    println!("\nTop degree:");
    for n in &stats.top_degree {
        println!(
            "  {:>7.3}  {:>4}  [{}] {}",
            n.score, n.degree, n.node_type, n.id
        );
    }
    println!("\nTop PageRank:");
    for n in &stats.top_pagerank {
        println!(
            "  {:>7.5}  {:>4}  [{}] {}",
            n.score, n.degree, n.node_type, n.id
        );
    }
    Ok(())
}

fn count_desc(map: &BTreeMap<String, usize>) -> Vec<(&str, usize)> {
    let mut rows: Vec<_> = map.iter().map(|(k, v)| (k.as_str(), *v)).collect();
    rows.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
    rows
}

fn cmd_studio_filter(
    node_types: Vec<String>,
    edge_types: Vec<String>,
    reset: bool,
    json: bool,
) -> Result<()> {
    let mut payload = serde_json::Map::new();
    if reset {
        payload.insert("reset".into(), serde_json::Value::Bool(true));
    }
    if !node_types.is_empty() {
        payload.insert("nodeTypes".into(), serde_json::to_value(node_types)?);
    }
    if !edge_types.is_empty() {
        payload.insert("edgeTypes".into(), serde_json::to_value(edge_types)?);
    }
    let arg = serde_json::Value::Object(payload);
    let code = format!("JSON.stringify(window.__bokf.setGraphFilter({arg}))");
    let value = studio_json(&code)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&value)?);
    } else {
        print_studio_filter(&value);
    }
    Ok(())
}

fn cmd_studio_stats(json: bool) -> Result<()> {
    let value = studio_json("JSON.stringify(window.__bokf.getGraphStats())")?;
    if json {
        println!("{}", serde_json::to_string_pretty(&value)?);
    } else {
        let nodes = value.get("nodes").and_then(|v| v.as_u64()).unwrap_or(0);
        let edges = value.get("edges").and_then(|v| v.as_u64()).unwrap_or(0);
        let density = value.get("density").and_then(|v| v.as_f64()).unwrap_or(0.0);
        println!("Studio graph: {nodes} nodes, {edges} edges · density {density:.6}");
        if let Some(obj) = value.get("by_node_type").and_then(|v| v.as_object()) {
            println!("\nNodes by type:");
            for (k, v) in json_count_desc(obj) {
                println!("  {v:>4}  {k}");
            }
        }
        if let Some(obj) = value.get("by_edge_predicate").and_then(|v| v.as_object()) {
            println!("\nEdges by predicate:");
            for (k, v) in json_count_desc(obj) {
                println!("  {v:>4}  {k}");
            }
        }
    }
    Ok(())
}

fn print_studio_filter(value: &serde_json::Value) {
    let node_hidden = value
        .pointer("/filter/hiddenNodeTypes")
        .and_then(|v| v.as_array())
        .map(|v| v.len())
        .unwrap_or(0);
    let edge_hidden = value
        .pointer("/filter/hiddenEdgeTypes")
        .and_then(|v| v.as_array())
        .map(|v| v.len())
        .unwrap_or(0);
    let nodes = value
        .pointer("/stats/nodes")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let edges = value
        .pointer("/stats/edges")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    println!("Studio filter: {node_hidden} hidden node type(s), {edge_hidden} hidden edge type(s)");
    println!("Visible graph: {nodes} nodes, {edges} edges");
}

fn json_count_desc(map: &serde_json::Map<String, serde_json::Value>) -> Vec<(&str, u64)> {
    let mut rows: Vec<_> = map
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_u64().unwrap_or(0)))
        .collect();
    rows.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
    rows
}

#[cfg(unix)]
fn studio_json(code: &str) -> Result<serde_json::Value> {
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::net::{SocketAddr, UnixStream};

    let path = std::env::var("BIOOKF_STUDIO_IPC")
        .unwrap_or_else(|_| "/tmp/biookf-tauri-mcp.sock".to_string());
    let addr = SocketAddr::from_pathname(&path)
        .map_err(|e| anyhow::anyhow!("bad Studio socket path {path}: {e}"))?;
    let stream = UnixStream::connect_addr(&addr)
        .map_err(|e| anyhow::anyhow!("Studio GUI not reachable on {path} ({e}); open Studio with the control channel enabled"))?;
    let mut writer = stream.try_clone()?;
    let line = serde_json::json!({ "command": "execute_js", "payload": { "code": code }, "id": "bokf-cli" }).to_string();
    writer.write_all(line.as_bytes())?;
    writer.write_all(b"\n")?;
    writer.flush()?;

    let mut reader = BufReader::new(stream);
    let mut resp = String::new();
    reader.read_line(&mut resp)?;
    let value: serde_json::Value = serde_json::from_str(&resp)?;
    if !value
        .get("success")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        anyhow::bail!(
            "{}",
            value
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("Studio command failed")
        );
    }
    let result = value
        .pointer("/data/result")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Studio returned no JavaScript result"))?;
    Ok(serde_json::from_str(result)?)
}

#[cfg(not(unix))]
fn studio_json(_code: &str) -> Result<serde_json::Value> {
    anyhow::bail!("Studio live control is only available on Unix-like platforms in this build")
}

fn cmd_scaffold(path: PathBuf, name: String) -> Result<()> {
    std::fs::create_dir_all(path.join("raw"))?;
    std::fs::create_dir_all(path.join("knowledge"))?;
    let index = format!(
        "# {name}\n\n> BioOKF bundle index (catalog of concept pages).\n\nokf_version: 0.5\nbiookf_version: 0.5\n"
    );
    write_if_absent(&path.join("index.md"), &index)?;
    write_if_absent(&path.join("log.md"), &format!("# Change log: {name}\n"))?;
    write_if_absent(
        &path.join("SCHEMA.md"),
        "# BioOKF operating schema (v0.5)\n\nSee the canonical SCHEMA.md for the 28 node types and 35 predicates (24 positive + 11 negative).\n",
    )?;

    // version-track + register + activate the new bundle (so the first post-scaffold
    // convert/ingest is not denied by the require-active guardrail).
    let repo = GitRepo::open(&path);
    if repo.ensure_repo().is_ok() {
        let _ = repo.commit_all(
            ChangeKind::Manual,
            &format!("create knowledge base {name}"),
            None,
        );
    }
    let kb_id = path.file_name().map(|s| s.to_string_lossy().to_lowercase());
    if let (Some(id), Ok(root)) = (kb_id, bokf_core::config::ensure_config_dir()) {
        if bokf_core::registry::validate_kb_id(&id).is_ok() {
            let abs = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
            let _ = bokf_core::registry::register(&root, &id, &abs.to_string_lossy());
            let _ = bokf_core::active::set_active(&root, Some(&id));
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
