//! bokf-mcp: the BioOKF MCP server (stdio). Exposes thin, idempotent primitives
//! over `bokf-core` that an AI client (Claude/Codex) drives to ingest, query, and
//! lint BioOKF bundles. The Tauri GUI and CLI are alternate front-ends over the
//! same `bokf-core`; this server is the agentic backbone.

mod instructions;
mod ops;

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
    #[doc = "File/folder/zip path to convert (omit if using text)."] path: Option<String>,
    #[doc = "Inline text to ingest instead of a path."] text: Option<String>,
    #[doc = "Title for inline text."] title: Option<String>,
    #[doc = "Concatenate archive/folder members into one source."] combined: Option<bool>,
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
        match bokf_core::graph_of(&p.0.bundle) {
            Ok(g) => ok(serde_json::to_string_pretty(&g.to_json()).unwrap_or_default()),
            Err(e) => ok(format!("ERROR: {e}")),
        }
    }

    #[tool(name = "bokf_search", description = "BM25 full-text search over the bundle's concept documents.")]
    pub async fn search(&self, p: Parameters<SearchParam>) -> Result<CallToolResult, rmcp::model::ErrorData> {
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
        match bokf_core::active::set_active(Path::new(&p.0.root), Some(&p.0.kb_id)) {
            Ok(()) => ok(format!("active KB = {}", p.0.kb_id)),
            Err(e) => ok(format!("ERROR: {e}")),
        }
    }

    #[tool(name = "bokf_get_active", description = "Get the active KB id + resolved path under <root>.")]
    pub async fn get_active(&self, p: Parameters<RootParam>) -> Result<CallToolResult, rmcp::model::ErrorData> {
        let root = Path::new(&p.0.root);
        match bokf_core::active::get_active(root) {
            Some(id) => ok(serde_json::json!({"id": id, "path": bokf_core::registry::resolve(root, &id)}).to_string()),
            None => ok(serde_json::json!({ "id": null }).to_string()),
        }
    }

    #[tool(name = "bokf_verify", description = "Deterministic accountability gate: lint + structure checks; returns ok=true iff zero errors. Use at the end of an ingest/merge run.")]
    pub async fn verify(&self, p: Parameters<VerifyParam>) -> Result<CallToolResult, rmcp::model::ErrorData> {
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

    #[tool(name = "bokf_convert", description = "Convert a file/folder/zip (pdf/html/docx/pptx/csv/xlsx) or inline text into raw Markdown under the bundle's raw/, with a human-readable content-derived source id. Writes via bokf-core (not Edit), so raw/ guards don't block it.")]
    pub async fn convert(&self, p: Parameters<ConvertParam>) -> Result<CallToolResult, rmcp::model::ErrorData> {
        use bokf_core::convert::{ingest, SourceInput};
        let input = if let Some(t) = p.0.text {
            SourceInput::Text { text: t, title: p.0.title }
        } else if let Some(path) = p.0.path {
            SourceInput::Path(path.into())
        } else {
            return ok("ERROR: convert needs a `path` or `text`".to_string());
        };
        match ingest(Path::new(&p.0.bundle), input, p.0.combined.unwrap_or(false)) {
            Ok(recs) => ok(serde_json::to_string_pretty(&recs).unwrap_or_default()),
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
