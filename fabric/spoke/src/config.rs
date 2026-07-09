//! Connection config. Reads the same `.env` the `spoke-cli` uses (KNOWLEDGE_GRAPH_*), so the native
//! Rust Bolt path and the legacy CLI share one source of truth. ROOT is baked at compile time (the
//! crate dir = .../fabric/spoke), mirroring the Python `HERE`-relative loading.

use anyhow::{Context, Result};
use neo4rs::{ConfigBuilder, Graph};
use std::path::{Path, PathBuf};

pub const ROOT: &str = env!("CARGO_MANIFEST_DIR");

pub fn root() -> PathBuf {
    PathBuf::from(ROOT)
}

pub fn data_path(name: &str) -> PathBuf {
    root().join("data").join(name)
}

#[derive(Clone, Debug)]
pub struct KgConfig {
    pub uri: String,
    pub user: String,
    pub pass: String,
    pub db: String,
}

fn var_any(keys: &[&str]) -> String {
    for k in keys {
        if let Ok(v) = std::env::var(k) {
            if !v.is_empty() {
                return v;
            }
        }
    }
    String::new()
}

/// Load the connection config from ROOT/.env (falling back to the ambient environment).
pub fn load() -> Result<KgConfig> {
    let envp: &Path = &root().join(".env");
    if envp.exists() {
        // Non-fatal: ambient env may already carry the values.
        let _ = dotenvy::from_path(envp);
    }
    let cfg = KgConfig {
        uri: var_any(&["KNOWLEDGE_GRAPH_URI", "KG_URI"]),
        user: var_any(&["KNOWLEDGE_GRAPH_USERNAME", "KG_USER"]),
        pass: var_any(&["KNOWLEDGE_GRAPH_PASSWORD", "KG_PASS"]),
        db: {
            let d = var_any(&["KNOWLEDGE_GRAPH_DATABASE", "KG_DB"]);
            if d.is_empty() { "neo4j".to_string() } else { d }
        },
    };
    if cfg.uri.is_empty() {
        anyhow::bail!("no KNOWLEDGE_GRAPH_URI in {}/.env or environment", ROOT);
    }
    Ok(cfg)
}

/// Open a pooled Bolt connection (reused across the whole run — the core speed win over per-query
/// subprocess spawning).
pub async fn connect(cfg: &KgConfig) -> Result<Graph> {
    let c = ConfigBuilder::default()
        .uri(&cfg.uri)
        .user(&cfg.user)
        .password(&cfg.pass)
        .db(cfg.db.as_str())
        .max_connections(48)
        .build()
        .context("building neo4rs config")?;
    Graph::connect(c).await.context("connecting to SPOKE over Bolt")
}
