//! A tiny std-only client for the running Studio GUI's control socket.
//!
//! When `biookf-studio` runs with the control channel on, it listens on a Unix
//! socket (`$HOME/.biookf/studio-mcp.sock` by default, override via
//! `BIOOKF_STUDIO_IPC`) speaking newline-delimited JSON:
//!   request  `{"command":..,"payload":{..},"id":..,"authToken":..}`
//!   response `{"success":bool,"data":any,"error":string|null,"id":..}`
//!
//! The socket lives in a per-user 0700 directory and the GUI requires an auth
//! token (AUDIT M1). The GUI writes that token to `<socket>.token` (0600); we
//! read it and attach it as `authToken` on every request.
//!
//! `call()` connects, writes one JSON line, reads one JSON line, and returns the
//! parsed `data` (or the `error`). A short connect timeout makes every call fail
//! fast when the GUI isn't running, so `bokf_studio_*` tools degrade gracefully.

use serde_json::Value;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::time::Duration;

/// Fail fast when the GUI isn't up: a short connect/read budget.
const CONNECT_TIMEOUT: Duration = Duration::from_millis(800);
const IO_TIMEOUT: Duration = Duration::from_secs(15);

/// Resolve the control-socket path (env `BIOOKF_STUDIO_IPC`, else
/// `$HOME/.biookf/studio-mcp.sock`). Must match the GUI's
/// `biookf_control_socket_path()` in `app/studio/src-tauri/src/main.rs`.
pub fn socket_path() -> String {
    if let Ok(p) = std::env::var("BIOOKF_STUDIO_IPC") {
        return p;
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| std::env::temp_dir().display().to_string());
    format!("{home}/.biookf/studio-mcp.sock")
}

/// Read the auth token the GUI wrote next to the socket (`<socket>.token`, 0600).
/// Returns `None` when absent (control channel off, or an older GUI) so calls
/// still work against a token-less server; a new token-requiring GUI will
/// reject a tokenless call with a clear "Authentication failed" error.
fn auth_token(sock: &str) -> Option<String> {
    let t = std::fs::read_to_string(format!("{sock}.token")).ok()?;
    let t = t.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

/// Send one `{command,payload}` request and return the response's `data`
/// (or the `error` string). Connects fresh each call (the GUI socket handles
/// one request per line and the calls are infrequent + idempotent).
pub fn call(command: &str, payload: Value) -> Result<Value, String> {
    let path = socket_path();
    // Connect with a short timeout so a missing GUI errors quickly. We must
    // resolve the path to a SocketAddr to use connect_timeout.
    let addr = std::os::unix::net::SocketAddr::from_pathname(&path)
        .map_err(|e| format!("bad socket path {path}: {e}"))?;
    let stream = UnixStream::connect_addr(&addr).map_err(|e| {
        format!(
            "Studio GUI not reachable on {path} ({e}); is it running with the control channel on?"
        )
    })?;
    stream.set_read_timeout(Some(IO_TIMEOUT)).ok();
    stream.set_write_timeout(Some(CONNECT_TIMEOUT)).ok();

    let mut writer = stream.try_clone().map_err(|e| e.to_string())?;
    let mut req = serde_json::json!({ "command": command, "payload": payload, "id": "bokf-mcp" });
    if let Some(tok) = auth_token(&path) {
        req["authToken"] = Value::String(tok);
    }
    let line = req.to_string();
    writer
        .write_all(line.as_bytes())
        .map_err(|e| format!("write failed: {e}"))?;
    writer
        .write_all(b"\n")
        .map_err(|e| format!("write failed: {e}"))?;
    writer.flush().map_err(|e| format!("flush failed: {e}"))?;

    let mut reader = BufReader::new(stream);
    let mut resp = String::new();
    reader
        .read_line(&mut resp)
        .map_err(|e| format!("read failed: {e}"))?;
    if resp.trim().is_empty() {
        return Err("empty response from Studio control socket".to_string());
    }
    let v: Value = serde_json::from_str(&resp).map_err(|e| format!("bad response JSON: {e}"))?;
    if v.get("success").and_then(Value::as_bool).unwrap_or(false) {
        Ok(v.get("data").cloned().unwrap_or(Value::Null))
    } else {
        Err(v
            .get("error")
            .and_then(Value::as_str)
            .unwrap_or("unknown error")
            .to_string())
    }
}

/// True iff the GUI answers `ping` (i.e. it's running with the control channel).
pub fn ping() -> bool {
    call("ping", serde_json::json!({})).is_ok()
}

/// Evaluate a JS EXPRESSION in the webview and return its value as a String.
/// `code` must be an expression (no top-level `return`), e.g.
/// `JSON.stringify(window.__bokf.getState())`. Returns `data.result`.
pub fn execute_js(code: &str) -> Result<String, String> {
    let data = call("execute_js", serde_json::json!({ "code": code }))?;
    data.get("result")
        .and_then(Value::as_str)
        .map(|s| s.to_string())
        .ok_or_else(|| "execute_js returned no result".to_string())
}

/// Take a screenshot of the given window; returns the base64 PNG/JPEG payload
/// (the data-URL prefix, if any, is stripped so it can be handed to an image
/// content block directly).
pub fn screenshot(window_label: &str) -> Result<String, String> {
    let data = call(
        "take_screenshot",
        serde_json::json!({ "windowLabel": window_label }),
    )?;
    let raw = first_base64(&data).ok_or_else(|| "screenshot returned no image data".to_string())?;
    Ok(strip_data_url(&raw))
}

/// `get_app_info` -> the full app/window/monitor info object.
pub fn app_info() -> Result<Value, String> {
    call("get_app_info", serde_json::json!({}))
}

/// `manage_window` with `{operation}` (close|focus|minimize|...). Targets "main".
pub fn manage_window(action: &str) -> Result<Value, String> {
    call(
        "manage_window",
        serde_json::json!({ "operation": action, "windowLabel": "main" }),
    )
}

/// Strip a `data:<mime>;base64,` prefix, leaving raw base64.
fn strip_data_url(s: &str) -> String {
    match s.find("base64,") {
        Some(i) => s[i + "base64,".len()..].to_string(),
        None => s.to_string(),
    }
}

/// Dig out the first long base64-ish string anywhere in the response value
/// (the screenshot data lives in `data` as a `data:image/...;base64,..` URL,
/// but we search defensively in case the shape shifts).
fn first_base64(v: &Value) -> Option<String> {
    match v {
        Value::String(s) if s.contains("base64,") || s.len() > 256 => Some(s.clone()),
        Value::Object(map) => map.values().find_map(first_base64),
        Value::Array(arr) => arr.iter().find_map(first_base64),
        _ => None,
    }
}
