//! BioOKF Studio — the Tauri desktop app. A thin front-end: every command
//! delegates to `okf-core`, so the GUI is a pure visualizer/dashboard over the
//! same backend the CLI and MCP server use.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;

/// Root that contains the bundles (env `OKF_ROOT`, else the BioOKF repo root).
fn repo_root() -> PathBuf {
    std::env::var("OKF_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.."))
}

/// Candidate bundle directories: the canonical `examples/` plus every dir under
/// `studio/test-kb/`.
fn candidate_bundles() -> Vec<PathBuf> {
    let root = repo_root();
    let mut v = vec![root.join("examples")];
    if let Ok(rd) = std::fs::read_dir(root.join("studio/test-kb")) {
        for e in rd.flatten() {
            if e.path().is_dir() {
                v.push(e.path());
            }
        }
    }
    v.into_iter()
        .filter(|p| p.join("knowledge").is_dir() || p.join("index.md").is_file())
        .collect()
}

fn resolve(id: &str) -> Option<PathBuf> {
    candidate_bundles()
        .into_iter()
        .find(|p| p.file_name().map(|n| n.to_string_lossy() == id).unwrap_or(false))
}

#[tauri::command]
fn list_bases() -> Result<serde_json::Value, String> {
    let mut out = Vec::new();
    for p in candidate_bundles() {
        if let Ok(info) = okf_core::export::base_info(&p) {
            out.push(info);
        }
    }
    Ok(serde_json::Value::Array(out))
}

#[tauri::command]
fn get_bundle(id: String) -> Result<serde_json::Value, String> {
    let path = resolve(&id).ok_or_else(|| format!("unknown bundle: {id}"))?;
    okf_core::export::bundle_doc(&path, None).map_err(|e| e.to_string())
}

#[tauri::command]
fn lint_bundle(id: String) -> Result<serde_json::Value, String> {
    let path = resolve(&id).ok_or_else(|| format!("unknown bundle: {id}"))?;
    let bundle = okf_core::open_bundle(&path).map_err(|e| e.to_string())?;
    let report = okf_core::lint(&bundle);
    serde_json::to_value(&report).map_err(|e| e.to_string())
}

#[tauri::command]
fn search_bundle(id: String, query: String, limit: Option<usize>) -> Result<serde_json::Value, String> {
    let path = resolve(&id).ok_or_else(|| format!("unknown bundle: {id}"))?;
    let bundle = okf_core::open_bundle(&path).map_err(|e| e.to_string())?;
    let index = okf_core::SearchIndex::build(&bundle);
    let hits = index.search(&query, limit.unwrap_or(10));
    serde_json::to_value(&hits).map_err(|e| e.to_string())
}

fn main() {
    let builder = tauri::Builder::default()
        .setup(|_app| {
            // Native macOS vibrancy: the whole window becomes translucent frosted
            // glass (preserving the rounded window corners), so the canvas shows the
            // blurred desktop and the app's own surfaces layer on top.
            #[cfg(target_os = "macos")]
            {
                use tauri::Manager;
                use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial, NSVisualEffectState};
                if let Some(win) = _app.get_webview_window("main") {
                    let _ = apply_vibrancy(
                        &win,
                        NSVisualEffectMaterial::Sidebar,
                        Some(NSVisualEffectState::Active),
                        None,
                    );
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_bases,
            get_bundle,
            lint_bundle,
            search_bundle
        ]);

    // Debug-only: expose the webview to AI agents over MCP (drive/inspect/screenshot).
    #[cfg(feature = "debug-mcp")]
    let builder = builder.plugin(tauri_plugin_mcp::init_with_config(
        tauri_plugin_mcp::PluginConfig::new("BioOKF Studio".to_string())
            .start_socket_server(true)
            .socket_path(std::path::PathBuf::from("/tmp/biookf-tauri-mcp.sock")),
    ));

    builder
        .run(tauri::generate_context!())
        .expect("error while running BioOKF Studio");
}
