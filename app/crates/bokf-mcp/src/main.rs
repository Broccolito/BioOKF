//! bokf-mcp: the BioOKF MCP server (stdio). Exposes thin, idempotent primitives
//! over `bokf-core` that an AI client (Claude/Codex) drives to ingest, query, and
//! lint BioOKF bundles. The Tauri GUI and CLI are alternate front-ends over the
//! same `bokf-core`; this server is the agentic backbone.

mod instructions;
mod ops;
mod studio_client;

use anyhow::Result;
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, Content, Implementation, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
    transport::stdio,
    ServerHandler, ServiceExt,
};
use serde::{Deserialize, Serialize};
use std::path::Path;

macro_rules! param {
    ($name:ident { $($(#[doc = $doc:expr])? $field:ident : $ty:ty),* $(,)? }) => {
        #[derive(Debug, Serialize, Deserialize, rmcp::schemars::JsonSchema)]
        pub struct $name { $( $(#[doc = $doc])? pub $field: $ty ),* }
    };
}

param!(BundleParam { #[doc = "Path to the BioOKF bundle directory."] bundle: String });
param!(RootParam { #[doc = "Path to a directory that contains one or more bundles."] root: String });
param!(ScaffoldParam {
    #[doc = "Path where the new bundle directory should be created."] bundle: String,
    #[doc = "Human-readable name for the knowledge base."] name: Option<String>,
});
param!(ReadParam {
    #[doc = "Path to the bundle directory."] bundle: String,
    #[doc = "Logical page path, e.g. knowledge/gene/il6.md, index.md, or raw/<id>."] page: String,
});
param!(WriteParam {
    #[doc = "Path to the bundle directory."] bundle: String,
    #[doc = "Page path under knowledge/ (or index.md/log.md/SCHEMA.md). raw/ is read-only."] page: String,
    #[doc = "Full file content (YAML frontmatter + Markdown body for concept docs)."] content: String,
});
param!(ValidateParam { #[doc = "Full concept-document content to validate (not written)."] content: String });
param!(AppendLogParam {
    #[doc = "Path to the bundle directory."] bundle: String,
    #[doc = "ISO date, e.g. 2026-06-27."] date: String,
    #[doc = "Markdown summary of what changed."] entry: String,
});
param!(SearchParam {
    #[doc = "Path to the bundle directory."] bundle: String,
    #[doc = "Full-text query."] query: String,
    #[doc = "Max hits (default 10)."] limit: Option<usize>,
});
param!(LogSyncParam {
    #[doc = "Path to the bundle directory."] bundle: String,
    #[doc = "ingest|convert|link|merge|lint|index|restore|manual"] kind: String,
    #[doc = "Summary of what changed."] summary: String,
    #[doc = "Optional delta line (e.g. '+3 nodes')."] delta: Option<String>,
});
param!(LogParam {
    #[doc = "Path to the bundle directory."] bundle: String,
    #[doc = "Max entries (default 20)."] limit: Option<usize>,
});
param!(RestoreParam {
    #[doc = "Path to the bundle directory."] bundle: String,
    #[doc = "Commit sha to restore to."] sha: String,
    #[doc = "Optional summary."] summary: Option<String>,
});
param!(RootKbParam {
    #[doc = "Root directory that contains bundles."] root: String,
    #[doc = "KB id to make active."] kb_id: String,
});
param!(VerifyParam {
    #[doc = "Path to the bundle directory."] bundle: String,
    #[doc = "Workflow context: ingest|merge (optional)."] workflow: Option<String>,
});
param!(ConvertParam {
    #[doc = "Bundle directory to write raw/ into."] bundle: String,
    #[doc = "File/folder/zip path to convert (omit if using text/url)."] path: Option<String>,
    #[doc = "Inline text to ingest instead of a path."] text: Option<String>,
    #[doc = "Title for inline text."] title: Option<String>,
    #[doc = "Download and ingest a single URL (classifies its source provenance)."] url: Option<String>,
    #[doc = "Download and ingest a list of URLs (classifies each)."] urls: Option<Vec<String>>,
    #[doc = "Concatenate archive/folder members into one source."] combined: Option<bool>,
});
param!(NameFigureParam {
    #[doc = "Bundle directory."] bundle: String,
    #[doc = "Source id (the raw/<id> folder name)."] source: String,
    #[doc = "Current figure path relative to raw/<id>, e.g. figures/fig-001.png."] figure: String,
    #[doc = "Content caption to name the figure by."] caption: String,
});
param!(IndexParam {
    #[doc = "Bundle directory."] bundle: String,
    #[doc = "Only check currency (don't rewrite index.md)."] check: Option<bool>,
});
param!(MergeRawParam {
    #[doc = "Main KB (canonical) bundle dir."] mkb: String,
    #[doc = "Secondary KB bundle dir to relocate raw/ from."] skb: String,
});
param!(MergeSnapshotParam {
    #[doc = "Main KB bundle dir."] mkb: String,
    #[doc = "Verify against an existing snapshot instead of writing one."] verify: Option<bool>,
});

// --- Studio GUI control params -------------------------------------------------
param!(StudioOpenParam {
    #[doc = "Optional bundles root (OKF_ROOT) for the launched Studio; defaults to the GUI's own default."] root: Option<String>,
});
param!(StudioSelectParam {
    #[doc = "Base/KB id to make active in the GUI (calls window.__bokf.selectBase)."] base: Option<String>,
    #[doc = "Node identifier to select + focus (calls window.__bokf.selectNode)."] node: Option<String>,
});
param!(StudioSearchParam {
    #[doc = "Query string to push into the GUI's search box (window.__bokf.search)."] query: String,
});
param!(StudioNarrateParam {
    #[doc = "Short status message to show in the GUI's live 'AI agent' banner (e.g. 'merging diabetes KB')."] action: String,
});

#[derive(Clone)]
pub struct BokfServer {
    tool_router: ToolRouter<Self>,
}

impl Default for BokfServer {
    fn default() -> Self {
        Self::new()
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for BokfServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            server_info: Implementation {
                name: "biookf".to_string(),
                version: env!("CARGO_PKG_VERSION").to_owned(),
                title: Some("BioOKF Studio".to_string()),
                icons: None,
                website_url: None,
            },
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            instructions: Some(instructions::INSTRUCTIONS.to_string()),
            ..Default::default()
        }
    }
}

fn ok(text: String) -> Result<CallToolResult, rmcp::model::ErrorData> {
    Ok(CallToolResult::success(vec![Content::text(text)]))
}
fn from_result(r: Result<String, String>) -> Result<CallToolResult, rmcp::model::ErrorData> {
    match r {
        Ok(t) => ok(t),
        // Surface tool-level failures as content (so the agent can read+react),
        // matching BioRouter's habit of returning errors as text where useful.
        Err(e) => ok(format!("ERROR: {e}")),
    }
}

#[tool_router(router = tool_router)]
impl BokfServer {
    pub fn new() -> Self {
        Self { tool_router: Self::tool_router() }
    }

    #[tool(name = "bokf_list_bases", description = "List BioOKF bundles found under a root directory.")]
    pub async fn list_bases(&self, p: Parameters<RootParam>) -> Result<CallToolResult, rmcp::model::ErrorData> {
        from_result(ops::list_bases(Path::new(&p.0.root)).map(|v| v.join("\n")))
    }

    #[tool(name = "bokf_scaffold", description = "Create an empty BioOKF bundle (raw/, knowledge/, index.md, log.md, SCHEMA.md).")]
    pub async fn scaffold(&self, p: Parameters<ScaffoldParam>) -> Result<CallToolResult, rmcp::model::ErrorData> {
        let name = p.0.name.unwrap_or_else(|| "Untitled knowledge base".to_string());
        from_result(ops::scaffold(Path::new(&p.0.bundle), &name))
    }

    #[tool(name = "bokf_list_pages", description = "List the concept-document pages under knowledge/.")]
    pub async fn list_pages(&self, p: Parameters<BundleParam>) -> Result<CallToolResult, rmcp::model::ErrorData> {
        from_result(ops::list_pages(Path::new(&p.0.bundle)).map(|v| v.join("\n")))
    }

    #[tool(name = "bokf_read_page", description = "Read one page (concept doc, raw source, or index/log/schema).")]
    pub async fn read_page(&self, p: Parameters<ReadParam>) -> Result<CallToolResult, rmcp::model::ErrorData> {
        from_result(ops::read_page(Path::new(&p.0.bundle), &p.0.page))
    }

    #[tool(name = "bokf_write_page", description = "Create/overwrite a concept doc (or index/log/schema); validates concept docs on write.")]
    pub async fn write_page(&self, p: Parameters<WriteParam>) -> Result<CallToolResult, rmcp::model::ErrorData> {
        from_result(ops::write_page(Path::new(&p.0.bundle), &p.0.page, &p.0.content))
    }

    #[tool(name = "bokf_validate_page", description = "Validate a concept-document draft (type/identifier/predicates/provenance) WITHOUT writing it.")]
    pub async fn validate_page(&self, p: Parameters<ValidateParam>) -> Result<CallToolResult, rmcp::model::ErrorData> {
        from_result(ops::validate_page(&p.0.content))
    }

    #[tool(name = "bokf_append_log", description = "Append a dated entry to the bundle's log.md (newest-first).")]
    pub async fn append_log(&self, p: Parameters<AppendLogParam>) -> Result<CallToolResult, rmcp::model::ErrorData> {
        from_result(ops::append_log(Path::new(&p.0.bundle), &p.0.date, &p.0.entry))
    }

    #[tool(name = "bokf_lint", description = "Lint a bundle against BioOKF v0.5 conformance rules; returns a JSON report.")]
    pub async fn lint(&self, p: Parameters<BundleParam>) -> Result<CallToolResult, rmcp::model::ErrorData> {
        narrate_to_studio(&format!("linting · {}", kb_name(&p.0.bundle)));
        match bokf_core::open_bundle(&p.0.bundle) {
            Ok(b) => {
                let r = bokf_core::lint(&b);
                let summary = serde_json::json!({
                    "errors": r.errors(), "warnings": r.warnings(), "infos": r.infos(),
                    "findings": r.findings,
                });
                ok(serde_json::to_string_pretty(&summary).unwrap_or_default())
            }
            Err(e) => ok(format!("ERROR opening bundle: {e}")),
        }
    }

    #[tool(name = "bokf_graph", description = "Return the render-ready graph (nodes + directional edges) as JSON.")]
    pub async fn graph(&self, p: Parameters<BundleParam>) -> Result<CallToolResult, rmcp::model::ErrorData> {
        narrate_to_studio(&format!("building graph · {}", kb_name(&p.0.bundle)));
        match bokf_core::graph_of(&p.0.bundle) {
            Ok(g) => ok(serde_json::to_string_pretty(&g.to_json()).unwrap_or_default()),
            Err(e) => ok(format!("ERROR: {e}")),
        }
    }

    #[tool(name = "bokf_search", description = "BM25 full-text search over the bundle's concept documents.")]
    pub async fn search(&self, p: Parameters<SearchParam>) -> Result<CallToolResult, rmcp::model::ErrorData> {
        narrate_to_studio(&format!("searching · \"{}\"", p.0.query));
        match bokf_core::open_bundle(&p.0.bundle) {
            Ok(b) => {
                let idx = bokf_core::SearchIndex::build(&b);
                let hits = idx.search(&p.0.query, p.0.limit.unwrap_or(10));
                ok(serde_json::to_string_pretty(&hits).unwrap_or_default())
            }
            Err(e) => ok(format!("ERROR: {e}")),
        }
    }

    #[tool(name = "bokf_stats", description = "Summary statistics: node/edge counts by type and predicate.")]
    pub async fn stats(&self, p: Parameters<BundleParam>) -> Result<CallToolResult, rmcp::model::ErrorData> {
        narrate_to_studio(&format!("computing stats · {}", kb_name(&p.0.bundle)));
        match bokf_core::open_bundle(&p.0.bundle) {
            Ok(b) => {
                let mut by_type = std::collections::BTreeMap::<String, usize>::new();
                let mut edges = 0usize;
                for n in &b.nodes {
                    *by_type.entry(n.node_type.as_str().to_string()).or_default() += 1;
                    edges += n.edges.len();
                }
                let v = serde_json::json!({ "nodes": b.nodes.len(), "edges": edges, "by_type": by_type, "parse_errors": b.parse_errors.len() });
                ok(serde_json::to_string_pretty(&v).unwrap_or_default())
            }
            Err(e) => ok(format!("ERROR: {e}")),
        }
    }

    #[tool(name = "bokf_predicates", description = "Print the active BioOKF vocabulary: 28 node types, 35 predicates (24 positive + 11 negative not_<X>), knowledge_level/agent_type enums.")]
    pub async fn predicates(&self) -> Result<CallToolResult, rmcp::model::ErrorData> {
        use bokf_core::model::{AGENT_TYPES, KNOWLEDGE_LEVELS, NODE_TYPES, PREDICATES};
        let v = serde_json::json!({"node_types": NODE_TYPES.as_slice(), "predicates": PREDICATES.as_slice(), "knowledge_levels": KNOWLEDGE_LEVELS.as_slice(), "agent_types": AGENT_TYPES.as_slice()});
        ok(serde_json::to_string_pretty(&v).unwrap_or_default())
    }

    #[tool(name = "bokf_log_sync", description = "Append a dated log.md entry AND commit atomically (the sole step-committer). kind = ingest|convert|link|merge|lint|index|restore|manual.")]
    pub async fn log_sync(&self, p: Parameters<LogSyncParam>) -> Result<CallToolResult, rmcp::model::ErrorData> {
        use bokf_core::git::{today_iso, ChangeKind};
        match bokf_core::log_sync::log_sync(Path::new(&p.0.bundle), ChangeKind::parse(&p.0.kind), &p.0.summary, p.0.delta.as_deref(), &today_iso()) {
            Ok(sha) => ok(format!("committed {} [{}] {}", &sha[..8.min(sha.len())], p.0.kind, p.0.summary)),
            Err(e) => ok(format!("ERROR: {e}")),
        }
    }

    #[tool(name = "bokf_log", description = "Show commit history (newest-first) as JSON.")]
    pub async fn log(&self, p: Parameters<LogParam>) -> Result<CallToolResult, rmcp::model::ErrorData> {
        match bokf_core::git::GitRepo::open(&p.0.bundle).log(p.0.limit.unwrap_or(20)) {
            Ok(es) => ok(serde_json::to_string_pretty(&es).unwrap_or_default()),
            Err(e) => ok(format!("ERROR: {e}")),
        }
    }

    #[tool(name = "bokf_restore", description = "Forward-only restore the bundle to a prior commit sha.")]
    pub async fn restore(&self, p: Parameters<RestoreParam>) -> Result<CallToolResult, rmcp::model::ErrorData> {
        match bokf_core::git::GitRepo::open(&p.0.bundle).restore_to(&p.0.sha, p.0.summary.as_deref()) {
            Ok(sha) => ok(format!("restored; new commit {}", &sha[..8.min(sha.len())])),
            Err(e) => ok(format!("ERROR: {e}")),
        }
    }

    #[tool(name = "bokf_set_active", description = "Set which KB is active under <root>.")]
    pub async fn set_active(&self, p: Parameters<RootKbParam>) -> Result<CallToolResult, rmcp::model::ErrorData> {
        narrate_to_studio(&format!("switching active KB · {}", p.0.kb_id));
        match bokf_core::active::set_active(&mcp_root(&p.0.root), Some(&p.0.kb_id)) {
            Ok(()) => ok(format!("active KB = {}", p.0.kb_id)),
            Err(e) => ok(format!("ERROR: {e}")),
        }
    }

    #[tool(name = "bokf_get_active", description = "Get the active KB id + resolved path under <root>.")]
    pub async fn get_active(&self, p: Parameters<RootParam>) -> Result<CallToolResult, rmcp::model::ErrorData> {
        let root = mcp_root(&p.0.root);
        let root = root.as_path();
        match bokf_core::active::get_active(root) {
            Some(id) => ok(serde_json::json!({"id": id, "path": bokf_core::registry::resolve(root, &id)}).to_string()),
            None => ok(serde_json::json!({ "id": null }).to_string()),
        }
    }

    #[tool(name = "bokf_verify", description = "Deterministic accountability gate: lint + structure checks; returns ok=true iff zero errors. Use at the end of an ingest/merge run.")]
    pub async fn verify(&self, p: Parameters<VerifyParam>) -> Result<CallToolResult, rmcp::model::ErrorData> {
        narrate_to_studio(&format!("verifying · {}", kb_name(&p.0.bundle)));
        match bokf_core::open_bundle(&p.0.bundle) {
            Ok(b) => {
                let r = bokf_core::lint(&b);
                let v = serde_json::json!({
                    "ok": r.errors() == 0,
                    "workflow": p.0.workflow.unwrap_or_else(|| "any".into()),
                    "errors": r.errors(), "warnings": r.warnings(), "infos": r.infos(),
                    "has_index": b.has_index_md, "has_log": b.has_log_md,
                    "findings": r.findings,
                });
                ok(serde_json::to_string_pretty(&v).unwrap_or_default())
            }
            Err(e) => ok(format!("ERROR opening bundle: {e}")),
        }
    }

    #[tool(name = "bokf_convert", description = "Convert a file/folder/zip (pdf/html/docx/pptx/csv/xlsx), inline text, or a URL/list of URLs into raw Markdown under the bundle's raw/, with a human-readable content-derived source id. URL ingestion classifies source provenance (peer-reviewed/preprint/web). Writes via bokf-core (not Edit), so raw/ guards don't block it.")]
    pub async fn convert(&self, p: Parameters<ConvertParam>) -> Result<CallToolResult, rmcp::model::ErrorData> {
        use bokf_core::convert::{ingest, ingest_urls, SourceInput};
        if let Some(urls) = p.0.urls {
            let results = ingest_urls(Path::new(&p.0.bundle), urls);
            let sources: Vec<_> = results.iter().filter_map(|r| r.as_ref().ok()).collect();
            let errors: Vec<String> = results.iter().filter_map(|r| r.as_ref().err().cloned()).collect();
            let out = serde_json::json!({ "sources": sources, "errors": errors });
            return ok(serde_json::to_string_pretty(&out).unwrap_or_default());
        }
        let input = if let Some(u) = p.0.url {
            SourceInput::Url(u)
        } else if let Some(t) = p.0.text {
            SourceInput::Text { text: t, title: p.0.title }
        } else if let Some(path) = p.0.path {
            SourceInput::Path(path.into())
        } else {
            return ok("ERROR: convert needs a `path`, `url`, `urls`, or `text`".to_string());
        };
        match ingest(Path::new(&p.0.bundle), input, p.0.combined.unwrap_or(false)) {
            Ok(recs) => ok(serde_json::to_string_pretty(&recs).unwrap_or_default()),
            Err(e) => ok(format!("ERROR: {e}")),
        }
    }

    #[tool(name = "bokf_name_figure", description = "Rename a provisional figure (raw/<id>/figures/fig-NNN.ext) to a content name, rewriting its source.md reference and meta.yaml. Writes via bokf-core, so raw/ guards don't block it.")]
    pub async fn name_figure(&self, p: Parameters<NameFigureParam>) -> Result<CallToolResult, rmcp::model::ErrorData> {
        match bokf_core::figures::name_figure(Path::new(&p.0.bundle), &p.0.source, &p.0.figure, &p.0.caption) {
            Ok(new_rel) => ok(serde_json::json!({"source": p.0.source, "figure": new_rel}).to_string()),
            Err(e) => ok(format!("ERROR: {e}")),
        }
    }

    #[tool(name = "bokf_index", description = "Regenerate index.md (identifier registry + by-type catalog + subtypes-in-use), or check=true to list identifiers missing from it.")]
    pub async fn index(&self, p: Parameters<IndexParam>) -> Result<CallToolResult, rmcp::model::ErrorData> {
        match bokf_core::open_bundle(&p.0.bundle) {
            Ok(b) => {
                if p.0.check.unwrap_or(false) {
                    let missing = bokf_core::index::missing_from_index(&b);
                    ok(serde_json::json!({"current": missing.is_empty(), "missing": missing}).to_string())
                } else {
                    let name = Path::new(&p.0.bundle).file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "Knowledge base".into());
                    match bokf_core::index::write_index(&b, &name) {
                        Ok(()) => ok(format!("regenerated index.md ({} nodes)", b.nodes.len())),
                        Err(e) => ok(format!("ERROR: {e}")),
                    }
                }
            }
            Err(e) => ok(format!("ERROR: {e}")),
        }
    }

    #[tool(name = "bokf_merge_raw", description = "Relocate the SKB's raw/ into the MKB's raw/ (dedup by content, rename on collision); returns the source-id remapping for rewriting raw_source refs.")]
    pub async fn merge_raw(&self, p: Parameters<MergeRawParam>) -> Result<CallToolResult, rmcp::model::ErrorData> {
        narrate_to_studio(&format!("merging · {} ← {}", kb_name(&p.0.mkb), kb_name(&p.0.skb)));
        match bokf_core::merge::merge_raw(Path::new(&p.0.mkb), Path::new(&p.0.skb)) {
            Ok(res) => ok(serde_json::to_string_pretty(&res).unwrap_or_default()),
            Err(e) => ok(format!("ERROR: {e}")),
        }
    }

    #[tool(name = "bokf_merge_snapshot", description = "Snapshot the MKB identifier/path set before a merge, or verify=true to confirm the MKB stayed canonical afterward.")]
    pub async fn merge_snapshot(&self, p: Parameters<MergeSnapshotParam>) -> Result<CallToolResult, rmcp::model::ErrorData> {
        let root = Path::new(&p.0.mkb);
        match bokf_core::open_bundle(&p.0.mkb) {
            Ok(b) => {
                if p.0.verify.unwrap_or(false) {
                    match bokf_core::merge::verify_snapshot(root, &b) {
                        Ok(issues) => ok(serde_json::json!({"unchanged": issues.is_empty(), "issues": issues}).to_string()),
                        Err(e) => ok(format!("ERROR: {e}")),
                    }
                } else {
                    match bokf_core::merge::write_snapshot(root, &bokf_core::merge::snapshot(&b)) {
                        Ok(()) => ok(format!("snapshot written ({} identifiers)", b.nodes.len())),
                        Err(e) => ok(format!("ERROR: {e}")),
                    }
                }
            }
            Err(e) => ok(format!("ERROR: {e}")),
        }
    }

    // === Studio GUI control (bokf_studio_*) =================================
    // Open/close and drive/observe the running BioOKF Studio GUI over its
    // newline-delimited-JSON control socket (see studio_client). When the GUI
    // isn't running, calls fail fast and surface as readable ERROR text.

    #[tool(name = "bokf_studio_open", description = "Launch the BioOKF Studio GUI (with its agent control channel on), or report it's already running.")]
    pub async fn studio_open(&self, p: Parameters<StudioOpenParam>) -> Result<CallToolResult, rmcp::model::ErrorData> {
        if studio_client::ping() {
            return ok(serde_json::json!({"running": true, "note": "already running"}).to_string());
        }
        match spawn_studio(p.0.root.as_deref()) {
            Ok(bin) => {
                // Poll up to ~20s for BOTH the control socket AND the webview/frontend
                // (window.__bokf) to be ready; the socket answers ping early in setup,
                // before the window exists, so callers that immediately run execute_js
                // would otherwise hit "Webview not found: main".
                let mut running = false;
                let mut ready = false;
                for _ in 0..80 {
                    std::thread::sleep(std::time::Duration::from_millis(250));
                    if !running {
                        running = studio_client::ping();
                    }
                    if running {
                        if let Ok(r) =
                            studio_client::execute_js("(window.__bokf && window.__BOKF_READY) ? '1' : '0'")
                        {
                            if r.trim().trim_matches('"') == "1" {
                                ready = true;
                                break;
                            }
                        }
                    }
                }
                ok(serde_json::json!({
                    "running": running,
                    "ready": ready,
                    "binary": bin,
                    "socket": studio_client::socket_path(),
                    "note": if ready { "launched (webview ready)" }
                            else if running { "socket up but webview not ready within ~20s" }
                            else { "spawned but control socket did not answer within ~20s" },
                }).to_string())
            }
            Err(e) => ok(format!("ERROR: {e}")),
        }
    }

    #[tool(name = "bokf_studio_close", description = "Close the running Studio GUI window (via its control socket).")]
    pub async fn studio_close(&self) -> Result<CallToolResult, rmcp::model::ErrorData> {
        match studio_client::manage_window("close") {
            Ok(v) => ok(serde_json::json!({"closed": true, "result": v}).to_string()),
            Err(e) => ok(format!("ERROR: {e}")),
        }
    }

    #[tool(name = "bokf_studio_status", description = "Whether the Studio GUI is running, plus its app/window info if reachable.")]
    pub async fn studio_status(&self) -> Result<CallToolResult, rmcp::model::ErrorData> {
        let running = studio_client::ping();
        let info = if running { studio_client::app_info().ok() } else { None };
        ok(serde_json::json!({"running": running, "socket": studio_client::socket_path(), "info": info}).to_string())
    }

    #[tool(name = "bokf_studio_state", description = "The complete GUI status as structured JSON, the way to know what the app is doing WITHOUT a screenshot. Returns {base, baseName, basePath, loading, counts, query, selectedNode, selectedEdge, panelOpen, sidebarCollapsed, terminalOpen, lint, lastAgentAction, bases[]}.")]
    pub async fn studio_state(&self) -> Result<CallToolResult, rmcp::model::ErrorData> {
        studio_json("JSON.stringify(window.__bokf.getState())")
    }

    #[tool(name = "bokf_studio_graph", description = "Read the graph the GUI is currently rendering: {nodes, edges}.")]
    pub async fn studio_graph(&self) -> Result<CallToolResult, rmcp::model::ErrorData> {
        studio_json("JSON.stringify(window.__bokf.getGraph())")
    }

    #[tool(name = "bokf_studio_select", description = "Drive the GUI: select a base and/or a node, then return the resulting state.")]
    pub async fn studio_select(&self, p: Parameters<StudioSelectParam>) -> Result<CallToolResult, rmcp::model::ErrorData> {
        let mut calls = String::new();
        if let Some(base) = &p.0.base {
            calls.push_str(&format!("window.__bokf.selectBase({});", json_str(base)));
        }
        if let Some(node) = &p.0.node {
            calls.push_str(&format!("window.__bokf.selectNode({});", json_str(node)));
        }
        if calls.is_empty() {
            return ok("ERROR: bokf_studio_select needs `base` and/or `node`".to_string());
        }
        // Fire the side-effecting calls, wait for any bundle load to settle, then
        // read back state, so switching base returns the NEW base's data, not the
        // previous bundle's (selectBase loads the bundle asynchronously).
        if let Err(e) = studio_client::execute_js(&format!("(function(){{{calls}return true;}})()")) {
            return ok(format!("ERROR: {e}"));
        }
        studio_wait_settled();
        studio_json("JSON.stringify(window.__bokf.getState())")
    }

    #[tool(name = "bokf_studio_reload", description = "Reload the GUI's data from disk, then return the settled state.")]
    pub async fn studio_reload(&self) -> Result<CallToolResult, rmcp::model::ErrorData> {
        if let Err(e) = studio_client::execute_js("(window.__bokf.reload(), true)") {
            return ok(format!("ERROR: {e}"));
        }
        studio_wait_settled();
        studio_json("JSON.stringify(window.__bokf.getState())")
    }

    #[tool(name = "bokf_studio_search", description = "Drive the GUI's search box with a query, then return the resulting state.")]
    pub async fn studio_search(&self, p: Parameters<StudioSearchParam>) -> Result<CallToolResult, rmcp::model::ErrorData> {
        let code = format!(
            "(function(){{window.__bokf.search({});return JSON.stringify(window.__bokf.getState());}})()",
            json_str(&p.0.query)
        );
        studio_json(&code)
    }

    #[tool(name = "bokf_studio_screenshot", description = "Capture a screenshot of the Studio GUI window (returned as an image). For visual inspection only; use bokf_studio_state to READ status.")]
    pub async fn studio_screenshot(&self) -> Result<CallToolResult, rmcp::model::ErrorData> {
        match studio_client::screenshot("main") {
            Ok(b64) => Ok(CallToolResult::success(vec![Content::image(b64, "image/jpeg")])),
            Err(e) => ok(format!("ERROR: {e}")),
        }
    }

    #[tool(name = "bokf_studio_narrate", description = "Show a short status line in the Studio GUI's live 'AI agent' banner so a human watching sees what you're doing (e.g. before a long merge/lint/convert). No-op if the GUI isn't open.")]
    pub async fn studio_narrate(&self, p: Parameters<StudioNarrateParam>) -> Result<CallToolResult, rmcp::model::ErrorData> {
        narrate_to_studio(&p.0.action);
        ok(serde_json::json!({"narrated": p.0.action}).to_string())
    }
}

/// Short display name for a bundle path (its final path component).
fn kb_name(path: &str) -> &str {
    std::path::Path::new(path).file_name().and_then(|s| s.to_str()).unwrap_or(path)
}

/// Best-effort: tell the running Studio GUI what the agent is doing, so its live
/// "AI agent" banner narrates the action to a watching human. Fire-and-forget in a
/// detached thread; never blocks the tool; silently no-ops if the GUI is closed.
fn narrate_to_studio(action: &str) {
    let a = action.to_string();
    std::thread::spawn(move || {
        let code = format!(
            "(window.__bokf&&window.__bokf.narrate)?window.__bokf.narrate({}):0",
            serde_json::to_string(&a).unwrap_or_else(|_| "\"\"".into())
        );
        let _ = studio_client::execute_js(&code);
    });
}

/// Wait (up to ~3s) for any in-flight bundle load to settle
/// (`window.__bokfLoading` false), so select/reload read the new base's data
/// rather than the previous bundle's. Returns immediately if nothing is loading
/// or the webview is unreachable.
fn studio_wait_settled() {
    for _ in 0..40 {
        match studio_client::execute_js("window.__bokfLoading ? '1' : '0'") {
            Ok(r) if r.trim().trim_matches('"') == "0" => return,
            Ok(_) => {}
            Err(_) => return,
        }
        std::thread::sleep(std::time::Duration::from_millis(75));
    }
}

/// Run an execute_js expression that returns a JSON string, and surface the
/// parsed JSON (pretty) as the tool result. On failure, surface ERROR text.
fn studio_json(code: &str) -> Result<CallToolResult, rmcp::model::ErrorData> {
    match studio_client::execute_js(code) {
        Ok(s) => match serde_json::from_str::<serde_json::Value>(&s) {
            Ok(v) => ok(serde_json::to_string_pretty(&v).unwrap_or(s)),
            // Not JSON (shouldn't happen for these expressions); pass through.
            Err(_) => ok(s),
        },
        Err(e) => ok(format!("ERROR: {e}")),
    }
}

/// JSON-encode a string for safe interpolation into a JS expression.
fn json_str(s: &str) -> String {
    serde_json::Value::String(s.to_string()).to_string()
}

/// Resolve the registry/active-pointer root: the caller-provided path if
/// non-empty, else the canonical config dir (~/.config/biookf-studio). This is
/// what keeps the MCP server, CLI, and Studio reading the same pointer.
fn mcp_root(provided: &str) -> std::path::PathBuf {
    if provided.trim().is_empty() {
        bokf_core::config::ensure_config_dir().unwrap_or_else(|_| bokf_core::config::config_dir())
    } else {
        std::path::PathBuf::from(provided)
    }
}

/// Locate the `biookf-studio` binary and spawn it detached with the control
/// channel on. Order: env `BIOOKF_STUDIO_BIN`, else next to the current exe
/// (the bokf-mcp and biookf-studio binaries sit together in target/{debug,release}).
fn spawn_studio(root: Option<&str>) -> Result<String, String> {
    let bin = locate_studio_bin()?;
    let mut cmd = std::process::Command::new(&bin);
    cmd.env("BIOOKF_STUDIO_CONTROL", "1")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    // Point the spawned Studio at the same root: an explicit one if given, else
    // the canonical config dir (the Studio defaults to it too; this is explicit).
    let r = root
        .filter(|s| !s.trim().is_empty())
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| bokf_core::config::ensure_config_dir().unwrap_or_else(|_| bokf_core::config::config_dir()));
    cmd.env("OKF_ROOT", r);
    cmd.spawn().map_err(|e| format!("failed to spawn {}: {e}", bin.display()))?;
    Ok(bin.display().to_string())
}

/// Find the `biookf-studio` executable.
fn locate_studio_bin() -> Result<std::path::PathBuf, String> {
    if let Ok(p) = std::env::var("BIOOKF_STUDIO_BIN") {
        let p = std::path::PathBuf::from(p);
        if p.exists() {
            return Ok(p);
        }
        return Err(format!("BIOOKF_STUDIO_BIN points at a missing file: {}", p.display()));
    }
    let exe = std::env::current_exe().map_err(|e| format!("cannot find current exe: {e}"))?;
    let dir = exe.parent().ok_or("current exe has no parent dir")?;
    let name = if cfg!(windows) { "biookf-studio.exe" } else { "biookf-studio" };
    let candidate = dir.join(name);
    if candidate.exists() {
        return Ok(candidate);
    }
    Err(format!(
        "biookf-studio not found next to {} (looked for {}); set BIOOKF_STUDIO_BIN",
        dir.display(),
        candidate.display()
    ))
}

#[tokio::main]
async fn main() -> Result<()> {
    // stdout is the JSON-RPC stream; all logging must go to stderr.
    let service = BokfServer::new().serve(stdio()).await.inspect_err(|e| {
        eprintln!("bokf-mcp serve error: {e:?}");
    })?;
    service.waiting().await?;
    Ok(())
}
