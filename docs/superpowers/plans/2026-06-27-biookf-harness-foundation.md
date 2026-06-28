# BioOKF Curation Harness: Foundation (SP0+SP1+SP2) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reconcile the predicate set to the authoritative 24, add git-backed version tracking (with a single step-committer that keeps `log.md` and git in lockstep), and add active-KB context, all as `bokf-core`/`bokf-cli`/`bokf-mcp` primitives.

**Architecture:** Pure logic stays in `bokf-core` (new modules `git.rs`, `log_sync.rs`, `registry.rs`, `active.rs`, plus a `model.rs` edit). The CLI (`bokf`) and MCP server (`bokf-mcp`, server name `biookf`) are thin front-ends. Git is driven by **shelling out to the system `git` binary** (`std::process::Command`), with no `git2`/libgit2 dependency.

**Tech Stack:** Rust 2021 workspace at `studio/`; crates `bokf-core`, `bokf-cli` (clap), `bokf-mcp` (rmcp); `serde`/`serde_yaml`/`serde_json`; new direct dep `uuid` (v4, already in lockfile), new dev-dep `tempfile`.

## Global Constraints

- Predicate set is **24** (`SCHEMA.md` authoritative): the 23 core **+ `used_to_study`**. One source of truth: `bokf_core::model::PREDICATES`.
- `log.md` headings stay **`## YYYY-MM-DD`** (newest-first); kind/summary/delta go in the body.
- Version tracking shells out to system **`git`**, with **no `git2`** dependency. Preflight errors clearly if `git` is absent.
- **`log_sync` is the sole step-committer** (log.md append + commit, atomic). The lower-level `bokf commit` is for non-logged lifecycle commits only. There is **no** per-tool auto-commit.
- `raw/**/original.*` is **gitignored** (immutable source bytes are not version-controlled).
- `restore` is **forward-only** (commit the old tree on top of HEAD; never rewind).
- All file writes that replace existing files use **temp-write + atomic rename**.
- Run all commands from `studio/`. Test runner: `cargo test`. Build: `cargo build`.

---

## File structure

| File | Responsibility | Action |
|---|---|---|
| `crates/bokf-core/src/model.rs` | controlled vocab incl. the 24th predicate | modify |
| `crates/bokf-core/src/lib.rs` | module decls, re-exports, vocab assert (24) | modify |
| `crates/bokf-core/src/lint.rs` | `used_to_study` domain/range + "24" message | modify |
| `crates/bokf-core/src/git.rs` | `GitRepo`, `ChangeKind`, `HistoryEntry`, date helper | create |
| `crates/bokf-core/src/log_sync.rs` | atomic log.md append + commit (sole step-committer) | create |
| `crates/bokf-core/src/registry.rs` | known-bundle registry + `validate_kb_id` | create |
| `crates/bokf-core/src/active.rs` | active-KB pointer | create |
| `crates/bokf-core/Cargo.toml` | add `uuid` dep + `tempfile` dev-dep | modify |
| `crates/bokf-cli/src/main.rs` | `predicates`/`commit`/`log`/`restore`/`log-sync`/`set-active`/`get-active`/`register` + scaffold integration | modify |
| `crates/bokf-mcp/src/main.rs` | MCP tools for the above | modify |
| `crates/bokf-mcp/src/ops.rs` | scaffold integration (git+register+activate) | modify |
| `crates/bokf-mcp/src/instructions.rs` | predicate line → 24 | modify |

---

## SP0: Predicate reconciliation

### Task 1: Add `used_to_study` to the model

**Files:** Modify `crates/bokf-core/src/model.rs`; Modify `crates/bokf-core/src/lib.rs:90`

**Interfaces, Produces:** `Predicate::UsedToStudy`; `PREDICATES: [&str; 24]` (adds `"used_to_study"` before `"reported_in"`).

- [ ] **Step 1: failing test.** In `model.rs`, append a `#[cfg(test)] mod tests` (or add to lib.rs tests). Add to `crates/bokf-core/src/lib.rs` test `node_type_palette_is_complete`, change the predicate assertion and add a round-trip:

```rust
        assert_eq!(model::PREDICATES.len(), 24);
        let p = model::Predicate::parse("used_to_study");
        assert!(!p.reversed);
        assert_eq!(p.predicate.as_str(), "used_to_study");
        assert!(p.predicate.is_valid());
```
(Replace the existing `assert_eq!(model::PREDICATES.len(), 23);` line.)

- [ ] **Step 2: run, expect fail.** `cargo test -p bokf-core node_type_palette_is_complete` → FAIL (len is 23 / `used_to_study` parses to Unknown).

- [ ] **Step 3: implement.** In `model.rs`:
  - Add variant `UsedToStudy,` to `enum Predicate` immediately before `ReportedIn,`.
  - Grow the const and insert the token before `"reported_in"`:
```rust
pub const PREDICATES: [&str; 24] = [
    "is_a", "part_of", "member_of", "derives_from", "located_in", "expressed_in", "encodes",
    "interacts_with", "binds", "regulates", "catalyzes", "converts_to", "participates_in", "causes",
    "predisposes_to", "treats", "prevents", "contraindicated_in", "affects_response_to",
    "has_phenotype", "measures", "associated_with", "used_to_study", "reported_in",
];
```
  - In `Predicate::parse`, add before the `reported_in` arm: `"used_to_study" => fwd(Predicate::UsedToStudy),`
  - In `Predicate::as_str`, add before the `ReportedIn` arm: `Predicate::UsedToStudy => "used_to_study",`

- [ ] **Step 4: run, expect pass.** `cargo test -p bokf-core node_type_palette_is_complete` → PASS.

- [ ] **Step 5: commit.** `git add -A && git commit -m "feat(bokf-core): add used_to_study predicate (24)"`

### Task 2: Domain/range for `used_to_study` + fix stale "23" messages

**Files:** Modify `crates/bokf-core/src/lint.rs:160` and `:212-244`

**Interfaces, Consumes:** `Predicate::UsedToStudy` (Task 1).

- [ ] **Step 1: failing test.** Add to `crates/bokf-core/src/lint.rs` a `#[cfg(test)] mod tests` block (create if absent):

```rust
#[cfg(test)]
mod tests {
    use crate::bundle::Bundle;
    use crate::parse::parse_node;

    fn lint_str(docs: &[(&str, &str)]) -> crate::lint::LintReport {
        let dir = tempfile::tempdir().unwrap();
        let k = dir.path().join("knowledge");
        for (rel, body) in docs {
            let p = k.join(rel);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(&p, body).unwrap();
        }
        let _ = parse_node; // ensure import used even if unused in some builds
        crate::lint::lint(&Bundle::open(dir.path()).unwrap())
    }

    #[test]
    fn used_to_study_range_violation_warns() {
        // organoid model used_to_study a Molecule (in-range) vs a Publication (out-of-range)
        let study = "---\ntype: Study\nidentifier: T2D GWAS\nsubtype: gwas\nraw_source: [raw/x]\nedges:\n  - predicate: used_to_study\n    object: Type 2 Diabetes\n    knowledge_level: knowledge_assertion\n    agent_type: data_analysis_pipeline\n    primary_source: T2D GWAS\n  - predicate: used_to_study\n    object: Some Paper\n    knowledge_level: knowledge_assertion\n    agent_type: data_analysis_pipeline\n    primary_source: T2D GWAS\n---\n# T2D GWAS\n";
        let disease = "---\ntype: Disease\nidentifier: Type 2 Diabetes\nsubtype: metabolic\n---\n# T2D\n";
        let paper = "---\ntype: Publication\nidentifier: Some Paper\nsubtype: article\nraw_source: [raw/p]\n---\n# p\n";
        let r = lint_str(&[("study/gwas.md", study), ("disease/t2d.md", disease), ("publication/paper.md", paper)]);
        // used_to_study -> Disease is fine; used_to_study -> Publication is out of range
        assert!(r.findings.iter().any(|f| f.rule == "edge.range" && f.message.contains("used_to_study")),
            "expected an edge.range warning for used_to_study targeting a Publication");
        // and it must NOT be flagged predicate.invalid anymore
        assert!(!r.findings.iter().any(|f| f.rule == "predicate.invalid"），);
    }
}
```
*(Note: replace the stray full-width characters if your editor inserts any; the assertion is `assert!(!r.findings.iter().any(|f| f.rule == "predicate.invalid"));`.)*

- [ ] **Step 2: run, expect fail.** `cargo test -p bokf-core used_to_study_range_violation_warns` → FAIL.

- [ ] **Step 3: implement.** In `lint.rs`:
  - Line ~160, change the message `"... not one of the 23 controlled predicates"` → `"... not one of the 24 controlled predicates"`.
  - In `lint_domain_range`, add an arm before the `_ => {}`:
```rust
        Predicate::UsedToStudy => {
            use NodeType::*;
            let in_range = matches!(obj.node_type,
                Disease | Phenotype | BiologicalPathway | BiologicalFunction | Gene | Variant | Molecule);
            if !in_range {
                warn(r, format!("`used_to_study` should target a studied entity (Disease/Phenotype/BiologicalPathway/BiologicalFunction/Gene/Variant/Molecule), but `{}` is a {}", e.object, obj.node_type.as_str()));
            }
        }
```

- [ ] **Step 4: run, expect pass.** `cargo test -p bokf-core used_to_study_range_violation_warns` → PASS. Also `cargo test -p bokf-core` (whole crate) → PASS.

- [ ] **Step 5: commit.** `git add -A && git commit -m "feat(bokf-core): used_to_study domain/range lint + 24-predicate messaging"`

### Task 3: `bokf predicates` CLI/MCP + sync instructions.md

**Files:** Modify `crates/bokf-cli/src/main.rs`; Modify `crates/bokf-mcp/src/main.rs`; Modify `crates/bokf-mcp/src/instructions.rs`; Test `crates/bokf-cli/tests/cli.rs`

**Interfaces, Produces:** CLI `bokf predicates [--json]`; MCP `bokf_predicates`.

- [ ] **Step 1: failing test.** Append to `crates/bokf-cli/tests/cli.rs`:
```rust
#[test]
fn predicates_lists_24() {
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_bokf"))
        .args(["predicates", "--json"]).output().unwrap();
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["predicates"].as_array().unwrap().len(), 24);
    assert!(v["predicates"].as_array().unwrap().iter().any(|p| p == "used_to_study"));
    assert_eq!(v["node_types"].as_array().unwrap().len(), 28);
}
```

- [ ] **Step 2: run, expect fail.** `cargo test -p bokf-cli predicates_lists_24` → FAIL (no `predicates` subcommand).

- [ ] **Step 3: implement (CLI).** In `main.rs`, add to `enum Cmd`:
```rust
    /// Print the active controlled vocabulary (28 types, 24 predicates, enums).
    Predicates {
        #[arg(long)]
        json: bool,
    },
```
Add to the `run()` match: `Cmd::Predicates { json } => cmd_predicates(json),`. Add the function:
```rust
fn cmd_predicates(json: bool) -> Result<()> {
    use bokf_core::model::{AGENT_TYPES, KNOWLEDGE_LEVELS, NODE_TYPES, PREDICATES};
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
```

- [ ] **Step 4: run, expect pass.** `cargo test -p bokf-cli predicates_lists_24` → PASS.

- [ ] **Step 5: implement (MCP + instructions).** In `bokf-mcp/src/main.rs`, add a tool method inside the `#[tool_router]` impl:
```rust
    #[tool(name = "bokf_predicates", description = "Print the active BioOKF vocabulary: 28 node types, 24 predicates, knowledge_level/agent_type enums.")]
    pub async fn predicates(&self) -> Result<CallToolResult, rmcp::model::ErrorData> {
        use bokf_core::model::{AGENT_TYPES, KNOWLEDGE_LEVELS, NODE_TYPES, PREDICATES};
        let v = serde_json::json!({"node_types": NODE_TYPES, "predicates": PREDICATES, "knowledge_levels": KNOWLEDGE_LEVELS, "agent_types": AGENT_TYPES});
        ok(serde_json::to_string_pretty(&v).unwrap_or_default())
    }
```
In `bokf-mcp/src/instructions.rs`, update any "23 predicates"/"23 forward-only" wording to **24** and mention `used_to_study`.

- [ ] **Step 6: verify + commit.** `cargo build && cargo test -p bokf-cli` → PASS. `git add -A && git commit -m "feat(bokf): bokf predicates tool + 24-predicate instructions"`

---

## SP1: Git-backed version tracking

### Task 4: Crate deps (uuid + tempfile)

**Files:** Modify `crates/bokf-core/Cargo.toml`

- [ ] **Step 1: edit.** Add to `[dependencies]`: `uuid = { version = "1", features = ["v4"] }`. Add a new section:
```toml
[dev-dependencies]
tempfile = "3"
```
- [ ] **Step 2: verify it resolves.** `cargo build -p bokf-core` → succeeds (fetches `tempfile`; `uuid` already in lockfile). If offline and the fetch fails, run once with network.
- [ ] **Step 3: commit.** `git add -A && git commit -m "build(bokf-core): add uuid dep + tempfile dev-dep"`

### Task 5: `git.rs` repo lifecycle + commit + a date helper

**Files:** Create `crates/bokf-core/src/git.rs`; Modify `crates/bokf-core/src/lib.rs`

**Interfaces, Produces:** `ChangeKind` (8 variants, `as_str`/`parse`); `GitRepo::{open,preflight,is_repo,ensure_repo,commit_all,head_sha}`; `today_iso()`.

- [ ] **Step 1: failing test.** Create `crates/bokf-core/tests/version.rs`:
```rust
use bokf_core::git::{ChangeKind, GitRepo};

#[test]
fn ensure_repo_then_commit_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("index.md"), "# Log\n").unwrap();
    let repo = GitRepo::open(dir.path());
    assert!(!repo.is_repo());
    repo.ensure_repo().unwrap();
    assert!(repo.is_repo());
    let sha = repo.commit_all(ChangeKind::Manual, "initial", None).unwrap();
    assert_eq!(sha.len(), 40);
    // .gitignore excludes raw originals
    assert!(std::fs::read_to_string(dir.path().join(".gitignore")).unwrap().contains("raw/**/original.*"));
}

#[test]
fn today_iso_is_well_formed() {
    let d = bokf_core::git::today_iso();
    assert_eq!(d.len(), 10);
    assert_eq!(&d[4..5], "-");
}
```
- [ ] **Step 2: run, expect fail.** `cargo test -p bokf-core --test version` → FAIL (no module).
- [ ] **Step 3: implement.** Create `crates/bokf-core/src/git.rs`:
```rust
//! Git-backed version tracking for a BioOKF bundle by shelling out to the system
//! `git` binary (no libgit2 dependency).
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeKind { Ingest, Convert, Link, Merge, Lint, Index, Restore, Manual }

impl ChangeKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChangeKind::Ingest => "ingest", ChangeKind::Convert => "convert",
            ChangeKind::Link => "link", ChangeKind::Merge => "merge",
            ChangeKind::Lint => "lint", ChangeKind::Index => "index",
            ChangeKind::Restore => "restore", ChangeKind::Manual => "manual",
        }
    }
    pub fn parse(s: &str) -> ChangeKind {
        match s {
            "ingest" => ChangeKind::Ingest, "convert" => ChangeKind::Convert,
            "link" => ChangeKind::Link, "merge" => ChangeKind::Merge,
            "lint" => ChangeKind::Lint, "index" => ChangeKind::Index,
            "restore" => ChangeKind::Restore, _ => ChangeKind::Manual,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub commit_sha: String,
    pub kind: ChangeKind,
    pub summary: String,
    pub delta: Option<String>,
    pub timestamp: String,
}

pub struct GitRepo { root: PathBuf }

impl GitRepo {
    pub fn open(root: impl AsRef<Path>) -> GitRepo { GitRepo { root: root.as_ref().to_path_buf() } }

    pub fn preflight() -> Result<(), String> {
        match Command::new("git").arg("--version").output() {
            Ok(o) if o.status.success() => Ok(()),
            _ => Err("`git` not found on PATH; install git to enable BioOKF version tracking".into()),
        }
    }
    pub fn is_repo(&self) -> bool { self.root.join(".git").exists() }

    fn run(&self, args: &[&str]) -> Result<String, String> {
        let out = Command::new("git").arg("-C").arg(&self.root).args(args).output()
            .map_err(|e| format!("git {args:?}: {e}"))?;
        if !out.status.success() {
            return Err(format!("git {args:?} failed: {}", String::from_utf8_lossy(&out.stderr).trim()));
        }
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    }

    pub fn ensure_repo(&self) -> Result<(), String> {
        GitRepo::preflight()?;
        if self.is_repo() { return Ok(()); }
        self.run(&["init", "-q"])?;
        self.run(&["config", "user.name", "BioOKF Curation"])?;
        self.run(&["config", "user.email", "curation@biookf.local"])?;
        self.run(&["config", "commit.gpgsign", "false"])?;
        let gi = self.root.join(".gitignore");
        if !gi.exists() { std::fs::write(&gi, "raw/**/original.*\n").map_err(|e| e.to_string())?; }
        Ok(())
    }

    fn has_head(&self) -> bool {
        Command::new("git").arg("-C").arg(&self.root).args(["rev-parse", "--verify", "HEAD"])
            .output().map(|o| o.status.success()).unwrap_or(false)
    }
    pub fn head_sha(&self) -> Result<String, String> { Ok(self.run(&["rev-parse", "HEAD"])?.trim().to_string()) }
    pub fn current_branch(&self) -> Result<String, String> { Ok(self.run(&["rev-parse", "--abbrev-ref", "HEAD"])?.trim().to_string()) }

    pub fn commit_all(&self, kind: ChangeKind, summary: &str, delta: Option<&str>) -> Result<String, String> {
        self.ensure_repo()?;
        self.run(&["add", "-A"])?;
        let dirty = !self.run(&["status", "--porcelain"])?.trim().is_empty();
        if !dirty && self.has_head() { return self.head_sha(); }
        let subject = format!("[{}] {}", kind.as_str(), summary);
        let body = delta.map(|d| format!("delta: {d}"));
        let mut args: Vec<String> = vec!["commit".into(), "-q".into(), "-m".into(), subject];
        if let Some(b) = &body { args.push("-m".into()); args.push(b.clone()); }
        if !self.has_head() && !dirty { args.push("--allow-empty".into()); }
        let argref: Vec<&str> = args.iter().map(String::as_str).collect();
        self.run(&argref)?;
        self.head_sha()
    }
}

/// Today's date as YYYY-MM-DD (UTC), dependency-free.
pub fn today_iso() -> String {
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let (y, m, d) = civil_from_days((secs / 86_400) as i64);
    format!("{y:04}-{m:02}-{d:02}")
}
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn civil_epoch() { assert_eq!(civil_from_days(0), (1970, 1, 1)); }
    #[test] fn changekind_roundtrip() { for k in [ChangeKind::Ingest, ChangeKind::Merge, ChangeKind::Manual] { assert_eq!(ChangeKind::parse(k.as_str()), k); } }
}
```
In `lib.rs` add `pub mod git;` and re-export `pub use git::{ChangeKind, GitRepo, HistoryEntry};`.

- [ ] **Step 4: run, expect pass.** `cargo test -p bokf-core --test version && cargo test -p bokf-core git::` → PASS.
- [ ] **Step 5: commit.** `git add -A && git commit -m "feat(bokf-core): git.rs repo lifecycle, commit_all, ChangeKind, today_iso"`

### Task 6: `git.rs` `log()` and forward-only `restore_to()`

**Files:** Modify `crates/bokf-core/src/git.rs`

**Interfaces, Produces:** `GitRepo::{log,restore_to}`.

- [ ] **Step 1: failing test.** Append to `crates/bokf-core/tests/version.rs`:
```rust
#[test]
fn log_parses_kind_and_restore_is_forward_only() {
    let dir = tempfile::tempdir().unwrap();
    let repo = GitRepo::open(dir.path());
    repo.ensure_repo().unwrap();
    std::fs::write(dir.path().join("a.md"), "v1").unwrap();
    let first = repo.commit_all(ChangeKind::Ingest, "add a", Some("+1 node")).unwrap();
    std::fs::write(dir.path().join("a.md"), "v2").unwrap();
    repo.commit_all(ChangeKind::Lint, "fix a", None).unwrap();
    let entries = repo.log(10).unwrap();
    assert_eq!(entries[0].kind, ChangeKind::Lint);
    assert_eq!(entries[1].kind, ChangeKind::Ingest);
    assert_eq!(entries[1].delta.as_deref(), Some("+1 node"));
    // restore to first: a.md becomes v1 again, via a NEW commit (history grows)
    let n_before = repo.log(50).unwrap().len();
    repo.restore_to(&first, Some("roll back a")).unwrap();
    assert_eq!(std::fs::read_to_string(dir.path().join("a.md")).unwrap(), "v1");
    assert_eq!(repo.log(50).unwrap().len(), n_before + 1);
    assert_eq!(repo.log(1).unwrap()[0].kind, ChangeKind::Restore);
}
```
- [ ] **Step 2: run, expect fail.** `cargo test -p bokf-core --test version log_parses_kind_and_restore_is_forward_only` → FAIL.
- [ ] **Step 3: implement.** Add to `impl GitRepo`:
```rust
    pub fn log(&self, limit: usize) -> Result<Vec<HistoryEntry>, String> {
        if !self.has_head() { return Ok(vec![]); }
        let raw = self.run(&["log", &format!("-n{limit}"), "--pretty=format:%H%x1f%s%x1f%b%x1f%cI%x1e"])?;
        let mut entries = Vec::new();
        for rec in raw.split('\x1e') {
            let rec = rec.trim_start_matches('\n');
            if rec.trim().is_empty() { continue; }
            let p: Vec<&str> = rec.split('\x1f').collect();
            if p.len() < 4 { continue; }
            let subject = p[1];
            let (kind, summary) = match (subject.find('['), subject.find(']')) {
                (Some(0), Some(close)) => (ChangeKind::parse(&subject[1..close]), subject[close + 1..].trim().to_string()),
                _ => (ChangeKind::Manual, subject.to_string()),
            };
            let delta = p[2].lines().find_map(|l| l.strip_prefix("delta: ").map(|s| s.to_string()));
            entries.push(HistoryEntry { commit_sha: p[0].to_string(), kind, summary, delta, timestamp: p[3].trim().to_string() });
        }
        Ok(entries)
    }

    pub fn restore_to(&self, sha: &str, summary: Option<&str>) -> Result<String, String> {
        self.ensure_repo()?;
        // bring the worktree + index to the old tree, then commit forward
        self.run(&["restore", "--source", sha, "--staged", "--worktree", "."])?;
        let s = summary.map(|s| s.to_string()).unwrap_or_else(|| format!("restore to {}", &sha[..sha.len().min(8)]));
        self.commit_all(ChangeKind::Restore, &s, Some(&format!("restored tree of {sha}")))
    }
```
- [ ] **Step 4: run, expect pass.** `cargo test -p bokf-core --test version` → PASS.
- [ ] **Step 5: commit.** `git add -A && git commit -m "feat(bokf-core): git log parsing + forward-only restore"`

### Task 7: `git.rs` transactions (branch + squash)

**Files:** Modify `crates/bokf-core/src/git.rs`; Modify `crates/bokf-core/src/lib.rs` (export `Txn`)

**Interfaces, Produces:** `Txn { branch, base }`; `GitRepo::{begin_txn,commit_txn,abort_txn}`.

- [ ] **Step 1: failing test.** Append to `crates/bokf-core/tests/version.rs`:
```rust
#[test]
fn txn_squashes_to_one_entry() {
    let dir = tempfile::tempdir().unwrap();
    let repo = GitRepo::open(dir.path());
    repo.ensure_repo().unwrap();
    std::fs::write(dir.path().join("seed.md"), "seed").unwrap();
    repo.commit_all(ChangeKind::Manual, "seed", None).unwrap();
    let base_n = repo.log(50).unwrap().len();
    let txn = repo.begin_txn("ingest paper").unwrap();
    for i in 0..3 { std::fs::write(dir.path().join(format!("n{i}.md")), "x").unwrap(); repo.commit_all(ChangeKind::Ingest, &format!("step {i}"), None).unwrap(); }
    let sha = repo.commit_txn(&txn, ChangeKind::Ingest, "ingest Paper X", Some("+3 nodes")).unwrap();
    assert_eq!(sha.len(), 40);
    // exactly one new entry on the base branch
    assert_eq!(repo.log(50).unwrap().len(), base_n + 1);
    assert_eq!(repo.log(1).unwrap()[0].summary, "ingest Paper X");
    assert!(std::fs::read_to_string(dir.path().join("n2.md")).is_ok());
}
```
- [ ] **Step 2: run, expect fail.** → FAIL.
- [ ] **Step 3: implement.** Add to `git.rs`:
```rust
pub struct Txn { pub branch: String, pub base: String }

impl GitRepo {
    pub fn begin_txn(&self, label: &str) -> Result<Txn, String> {
        self.ensure_repo()?;
        if !self.has_head() { self.commit_all(ChangeKind::Manual, "init", None)?; }
        let base = self.current_branch()?;
        let branch = format!("txn/{}-{}", slug(label), short_id());
        self.run(&["checkout", "-q", "-b", &branch])?;
        Ok(Txn { branch, base })
    }
    pub fn commit_txn(&self, txn: &Txn, kind: ChangeKind, summary: &str, delta: Option<&str>) -> Result<String, String> {
        self.run(&["checkout", "-q", &txn.base])?;
        self.run(&["merge", "--squash", &txn.branch])?;
        let sha = self.commit_all(kind, summary, delta)?;
        self.run(&["branch", "-D", &txn.branch])?;
        Ok(sha)
    }
    pub fn abort_txn(&self, txn: &Txn) -> Result<(), String> {
        self.run(&["checkout", "-q", "-f", &txn.base])?;
        self.run(&["branch", "-D", &txn.branch])?;
        Ok(())
    }
}

fn slug(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars().take(40) {
        if c.is_ascii_alphanumeric() { out.push(c.to_ascii_lowercase()); }
        else if !out.ends_with('-') { out.push('-'); }
    }
    out.trim_matches('-').to_string()
}
fn short_id() -> String { uuid::Uuid::new_v4().simple().to_string()[..8].to_string() }
```
In `lib.rs` extend the re-export: `pub use git::{ChangeKind, GitRepo, HistoryEntry, Txn};`.

- [ ] **Step 4: run, expect pass.** `cargo test -p bokf-core --test version` → PASS.
- [ ] **Step 5: commit.** `git add -A && git commit -m "feat(bokf-core): git transactions (branch + squash-merge)"`

### Task 8: `log_sync.rs` the sole step-committer

**Files:** Create `crates/bokf-core/src/log_sync.rs`; Modify `crates/bokf-core/src/lib.rs`

**Interfaces, Produces:** `log_sync(bundle, kind, summary, delta, date) -> Result<String, String>`.

- [ ] **Step 1: failing test.** Append to `crates/bokf-core/tests/version.rs`:
```rust
#[test]
fn log_sync_appends_and_commits_atomically() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("log.md"), "# Change log\n").unwrap();
    std::fs::write(dir.path().join("seed.md"), "x").unwrap();
    let sha = bokf_core::log_sync::log_sync(dir.path(), ChangeKind::Ingest, "first source", Some("+1 source · 5 nodes"), "2026-06-27").unwrap();
    let log = std::fs::read_to_string(dir.path().join("log.md")).unwrap();
    assert!(log.contains("## 2026-06-27"));
    assert!(log.contains("ingest | first source"));
    assert!(log.contains("+1 source · 5 nodes"));
    // newest-first: the new block is above any later-added one
    let repo = GitRepo::open(dir.path());
    assert_eq!(repo.log(1).unwrap()[0].commit_sha, sha);
    assert_eq!(repo.log(1).unwrap()[0].kind, ChangeKind::Ingest);
}
```
- [ ] **Step 2: run, expect fail.** → FAIL.
- [ ] **Step 3: implement.** Create `crates/bokf-core/src/log_sync.rs`:
```rust
//! The sole step-committer: append a dated `## YYYY-MM-DD` block to log.md AND
//! git-commit, atomically, so log.md and history never drift.
use crate::git::{ChangeKind, GitRepo};
use std::path::Path;

pub fn log_sync(bundle: &Path, kind: ChangeKind, summary: &str, delta: Option<&str>, date: &str) -> Result<String, String> {
    let log_path = bundle.join("log.md");
    let existing = std::fs::read_to_string(&log_path).unwrap_or_else(|_| "# Change log\n".to_string());
    let block = match delta {
        Some(d) => format!("\n## {date}\n\n{} | {summary}\n\n{d}\n", kind.as_str()),
        None => format!("\n## {date}\n\n{} | {summary}\n", kind.as_str()),
    };
    // newest-first: insert right after the first (title) line
    let new = match existing.find('\n') {
        Some(i) => format!("{}{}{}", &existing[..=i], block, &existing[i + 1..]),
        None => format!("{existing}{block}"),
    };
    let tmp = bundle.join("log.md.tmp");
    std::fs::write(&tmp, &new).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, &log_path).map_err(|e| e.to_string())?;
    GitRepo::open(bundle).commit_all(kind, summary, delta)
}
```
In `lib.rs` add `pub mod log_sync;`.

- [ ] **Step 4: run, expect pass.** `cargo test -p bokf-core --test version` → PASS.
- [ ] **Step 5: commit.** `git add -A && git commit -m "feat(bokf-core): log_sync sole step-committer (atomic log.md + commit)"`

### Task 9: CLI wiring for commit/log/restore/log-sync

**Files:** Modify `crates/bokf-cli/src/main.rs`; Test `crates/bokf-cli/tests/cli.rs`

**Interfaces, Produces:** `bokf log-sync`, `bokf commit`, `bokf log`, `bokf restore`.

- [ ] **Step 1: failing test.** Append to `crates/bokf-cli/tests/cli.rs`:
```rust
#[test]
fn cli_log_sync_then_log() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("log.md"), "# Change log\n").unwrap();
    let ok = std::process::Command::new(env!("CARGO_BIN_EXE_bokf"))
        .args(["log-sync", dir.path().to_str().unwrap(), "--kind", "ingest", "--summary", "seed"])
        .output().unwrap();
    assert!(ok.status.success(), "{}", String::from_utf8_lossy(&ok.stderr));
    let log = std::process::Command::new(env!("CARGO_BIN_EXE_bokf"))
        .args(["log", dir.path().to_str().unwrap(), "--json"]).output().unwrap();
    let v: serde_json::Value = serde_json::from_slice(&log.stdout).unwrap();
    assert_eq!(v[0]["kind"], "ingest");
}
```
Add `tempfile = "3"` to `crates/bokf-cli/Cargo.toml` `[dev-dependencies]`.

- [ ] **Step 2: run, expect fail.** → FAIL.
- [ ] **Step 3: implement.** In `main.rs`, add variants to `enum Cmd`:
```rust
    /// Append a dated log.md entry AND commit, atomically (the sole step-committer).
    LogSync {
        path: PathBuf,
        #[arg(long)] kind: String,
        #[arg(long)] summary: String,
        #[arg(long)] delta: Option<String>,
    },
    /// Lower-level: stage all + commit (non-logged lifecycle commit).
    Commit {
        path: PathBuf,
        #[arg(long)] kind: String,
        #[arg(long)] summary: String,
        #[arg(long)] delta: Option<String>,
    },
    /// Show commit history (newest-first).
    Log { path: PathBuf, #[arg(long, default_value_t = 20)] limit: usize, #[arg(long)] json: bool },
    /// Forward-only restore to a prior commit.
    Restore { path: PathBuf, sha: String, #[arg(long)] summary: Option<String> },
```
Add to `run()`:
```rust
        Cmd::LogSync { path, kind, summary, delta } => cmd_log_sync(path, kind, summary, delta),
        Cmd::Commit { path, kind, summary, delta } => cmd_commit(path, kind, summary, delta),
        Cmd::Log { path, limit, json } => cmd_log(path, limit, json),
        Cmd::Restore { path, sha, summary } => cmd_restore(path, sha, summary),
```
Add functions:
```rust
use bokf_core::git::{today_iso, ChangeKind, GitRepo};

fn cmd_log_sync(path: PathBuf, kind: String, summary: String, delta: Option<String>) -> Result<()> {
    let sha = bokf_core::log_sync::log_sync(&path, ChangeKind::parse(&kind), &summary, delta.as_deref(), &today_iso())
        .map_err(anyhow::Error::msg)?;
    eprintln!("[{}] {}: {}", kind, summary, &sha[..8.min(sha.len())]);
    Ok(())
}
fn cmd_commit(path: PathBuf, kind: String, summary: String, delta: Option<String>) -> Result<()> {
    let sha = GitRepo::open(&path).commit_all(ChangeKind::parse(&kind), &summary, delta.as_deref()).map_err(anyhow::Error::msg)?;
    eprintln!("{}", &sha[..8.min(sha.len())]);
    Ok(())
}
fn cmd_log(path: PathBuf, limit: usize, json: bool) -> Result<()> {
    let entries = GitRepo::open(&path).log(limit).map_err(anyhow::Error::msg)?;
    if json { println!("{}", serde_json::to_string_pretty(&entries)?); }
    else { for e in &entries { println!("{}  [{}] {}  {}", &e.commit_sha[..8], e.kind.as_str(), e.summary, e.delta.as_deref().unwrap_or("")); } }
    Ok(())
}
fn cmd_restore(path: PathBuf, sha: String, summary: Option<String>) -> Result<()> {
    let new = GitRepo::open(&path).restore_to(&sha, summary.as_deref()).map_err(anyhow::Error::msg)?;
    eprintln!("restored; new commit {}", &new[..8.min(new.len())]);
    Ok(())
}
```
- [ ] **Step 4: run, expect pass.** `cargo test -p bokf-cli cli_log_sync_then_log` → PASS.
- [ ] **Step 5: commit.** `git add -A && git commit -m "feat(bokf-cli): log-sync/commit/log/restore subcommands"`

---

## SP2: Active-KB context

### Task 10: `registry.rs` known-bundle registry + `validate_kb_id`

**Files:** Create `crates/bokf-core/src/registry.rs`; Modify `crates/bokf-core/src/lib.rs`; Test `crates/bokf-core/tests/active.rs`

**Interfaces, Produces:** `registry::{Base, register, unregister, list, resolve, validate_kb_id}`.

- [ ] **Step 1: failing test.** Create `crates/bokf-core/tests/active.rs`:
```rust
use bokf_core::registry;

#[test]
fn register_resolve_and_reject_dupes_and_bad_ids() {
    let dir = tempfile::tempdir().unwrap();
    registry::register(dir.path(), "ms-kb", "/abs/ms-kb").unwrap();
    assert_eq!(registry::resolve(dir.path(), "ms-kb").as_deref(), Some("/abs/ms-kb"));
    assert!(registry::register(dir.path(), "ms-kb", "/other").is_err()); // dup id
    assert!(registry::validate_kb_id("Bad_Id").is_err());
    assert!(registry::validate_kb_id("ok-1").is_ok());
    registry::unregister(dir.path(), "ms-kb").unwrap();
    assert!(registry::resolve(dir.path(), "ms-kb").is_none());
}
```
- [ ] **Step 2: run, expect fail.** → FAIL.
- [ ] **Step 3: implement.** Create `crates/bokf-core/src/registry.rs`:
```rust
//! Known-bundle registry: <root>/registry.yaml = { bases: [ {id, path} ] }.
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Base { pub id: String, pub path: String }

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Registry { #[serde(default)] pub bases: Vec<Base> }

fn path_of(root: &Path) -> PathBuf { root.join("registry.yaml") }

pub fn load(root: &Path) -> Registry {
    std::fs::read_to_string(path_of(root)).ok()
        .and_then(|s| serde_yaml::from_str(&s).ok()).unwrap_or_default()
}
fn save(root: &Path, reg: &Registry) -> Result<(), String> {
    let p = path_of(root);
    let tmp = root.join("registry.yaml.tmp");
    std::fs::write(&tmp, serde_yaml::to_string(reg).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, &p).map_err(|e| e.to_string())
}
pub fn register(root: &Path, id: &str, path: &str) -> Result<(), String> {
    validate_kb_id(id)?;
    let mut reg = load(root);
    if reg.bases.iter().any(|b| b.id == id) { return Err(format!("kb-id `{id}` already registered")); }
    reg.bases.push(Base { id: id.to_string(), path: path.to_string() });
    save(root, &reg)
}
pub fn unregister(root: &Path, id: &str) -> Result<(), String> {
    let mut reg = load(root); reg.bases.retain(|b| b.id != id); save(root, &reg)
}
pub fn list(root: &Path) -> Vec<Base> { load(root).bases }
pub fn resolve(root: &Path, id: &str) -> Option<String> {
    load(root).bases.into_iter().find(|b| b.id == id).map(|b| b.path)
}
pub fn validate_kb_id(id: &str) -> Result<(), String> {
    if id.is_empty() || id.len() > 64 { return Err("kb-id must be 1..=64 chars".into()); }
    if !id.bytes().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-') {
        return Err("kb-id must be [a-z0-9-]".into());
    }
    if id.starts_with('-') || id.ends_with('-') || id.contains("--") {
        return Err("kb-id must not have leading/trailing/double '-'".into());
    }
    Ok(())
}
```
In `lib.rs` add `pub mod registry;`.

- [ ] **Step 4: run, expect pass.** `cargo test -p bokf-core --test active` → PASS.
- [ ] **Step 5: commit.** `git add -A && git commit -m "feat(bokf-core): registry.rs known-bundle registry + validate_kb_id"`

### Task 11: `active.rs` active-KB pointer

**Files:** Create `crates/bokf-core/src/active.rs`; Modify `crates/bokf-core/src/lib.rs`

**Interfaces, Produces:** `active::{get_active, set_active}`.

- [ ] **Step 1: failing test.** Append to `crates/bokf-core/tests/active.rs`:
```rust
#[test]
fn set_get_clear_active() {
    let dir = tempfile::tempdir().unwrap();
    assert!(bokf_core::active::get_active(dir.path()).is_none());
    bokf_core::active::set_active(dir.path(), Some("ms-kb")).unwrap();
    assert_eq!(bokf_core::active::get_active(dir.path()).as_deref(), Some("ms-kb"));
    assert!(bokf_core::active::set_active(dir.path(), Some("Bad_Id")).is_err());
    bokf_core::active::set_active(dir.path(), None).unwrap();
    assert!(bokf_core::active::get_active(dir.path()).is_none());
}
```
- [ ] **Step 2: run, expect fail.** → FAIL.
- [ ] **Step 3: implement.** Create `crates/bokf-core/src/active.rs`:
```rust
//! Active-KB pointer: <root>/.active-kb = plaintext kb-id of the active graph.
use std::path::{Path, PathBuf};
use std::sync::Mutex;

static LOCK: Mutex<()> = Mutex::new(());
fn path_of(root: &Path) -> PathBuf { root.join(".active-kb") }

pub fn get_active(root: &Path) -> Option<String> {
    let _g = LOCK.lock();
    std::fs::read_to_string(path_of(root)).ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}
pub fn set_active(root: &Path, id: Option<&str>) -> Result<(), String> {
    let _g = LOCK.lock();
    let p = path_of(root);
    match id {
        Some(id) => {
            crate::registry::validate_kb_id(id)?;
            let tmp = root.join(".active-kb.tmp");
            std::fs::write(&tmp, id).map_err(|e| e.to_string())?;
            std::fs::rename(&tmp, &p).map_err(|e| e.to_string())
        }
        None => { let _ = std::fs::remove_file(&p); Ok(()) }
    }
}
```
In `lib.rs` add `pub mod active;`.

- [ ] **Step 4: run, expect pass.** `cargo test -p bokf-core --test active` → PASS.
- [ ] **Step 5: commit.** `git add -A && git commit -m "feat(bokf-core): active.rs active-KB pointer"`

### Task 12: CLI wiring for set-active/get-active/register + scaffold integration

**Files:** Modify `crates/bokf-cli/src/main.rs`; Test `crates/bokf-cli/tests/cli.rs`

**Interfaces, Produces:** `bokf set-active`, `bokf get-active`, `bokf register`; `bokf scaffold` now git-inits + registers + set-actives.

- [ ] **Step 1: failing test.** Append to `crates/bokf-cli/tests/cli.rs`:
```rust
#[test]
fn scaffold_registers_inits_and_activates() {
    let root = tempfile::tempdir().unwrap();
    let bundle = root.path().join("ms-kb");
    let s = std::process::Command::new(env!("CARGO_BIN_EXE_bokf"))
        .args(["scaffold", bundle.to_str().unwrap(), "--name", "MS KB"]).output().unwrap();
    assert!(s.status.success(), "{}", String::from_utf8_lossy(&s.stderr));
    assert!(bundle.join(".git").exists());                       // git-inited
    let ga = std::process::Command::new(env!("CARGO_BIN_EXE_bokf"))
        .args(["get-active", root.path().to_str().unwrap(), "--json"]).output().unwrap();
    let v: serde_json::Value = serde_json::from_slice(&ga.stdout).unwrap();
    assert_eq!(v["id"], "ms-kb");                                 // active set to the new bundle
}
```
- [ ] **Step 2: run, expect fail.** → FAIL.
- [ ] **Step 3: implement.** Add to `enum Cmd`:
```rust
    /// Set the active KB id under <root>.
    SetActive { root: PathBuf, kb_id: String },
    /// Print the active KB id + resolved path under <root>.
    GetActive { root: PathBuf, #[arg(long)] json: bool },
    /// Register/list/unregister a known bundle under <root>.
    Register { root: PathBuf, kb_id: Option<String>, path: Option<PathBuf>, #[arg(long)] list: bool, #[arg(long)] unregister: Option<String> },
```
Add to `run()`:
```rust
        Cmd::SetActive { root, kb_id } => cmd_set_active(root, kb_id),
        Cmd::GetActive { root, json } => cmd_get_active(root, json),
        Cmd::Register { root, kb_id, path, list, unregister } => cmd_register(root, kb_id, path, list, unregister),
```
Add functions:
```rust
fn cmd_set_active(root: PathBuf, kb_id: String) -> Result<()> {
    bokf_core::active::set_active(&root, Some(&kb_id)).map_err(anyhow::Error::msg)?;
    eprintln!("active KB = {kb_id}");
    Ok(())
}
fn cmd_get_active(root: PathBuf, json: bool) -> Result<()> {
    match bokf_core::active::get_active(&root) {
        Some(id) => {
            let path = bokf_core::registry::resolve(&root, &id);
            if json { println!("{}", serde_json::json!({"id": id, "path": path})); }
            else { println!("{id}  {}", path.as_deref().unwrap_or("(unregistered path)")); }
        }
        None => { if json { println!("{}", serde_json::json!({"id": null})); } else { println!("(no active KB; run `bokf set-active`)"); } }
    }
    Ok(())
}
fn cmd_register(root: PathBuf, kb_id: Option<String>, path: Option<PathBuf>, list: bool, unregister: Option<String>) -> Result<()> {
    if list { for b in bokf_core::registry::list(&root) { println!("{}  {}", b.id, b.path); } return Ok(()); }
    if let Some(id) = unregister { bokf_core::registry::unregister(&root, &id).map_err(anyhow::Error::msg)?; return Ok(()); }
    match (kb_id, path) {
        (Some(id), Some(p)) => bokf_core::registry::register(&root, &id, &p.to_string_lossy()).map_err(anyhow::Error::msg)?,
        _ => anyhow::bail!("register needs <kb_id> <path>, or --list, or --unregister <id>"),
    }
    Ok(())
}
```
Update `cmd_scaffold`, after writing the files, before the final `eprintln!`:
```rust
    // version-track + register + activate
    let repo = bokf_core::git::GitRepo::open(&path);
    let kb_id = bokf_core::registry::validate_kb_id(
        &path.file_name().map(|s| s.to_string_lossy().to_lowercase()).unwrap_or_default()
    ).ok().map(|_| path.file_name().unwrap().to_string_lossy().to_lowercase());
    if repo.ensure_repo().is_ok() {
        let _ = repo.commit_all(bokf_core::git::ChangeKind::Manual, &format!("create knowledge base {name}"), None);
    }
    if let (Some(id), Some(root)) = (kb_id, path.parent()) {
        let abs = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
        let _ = bokf_core::registry::register(root, &id, &abs.to_string_lossy());
        let _ = bokf_core::active::set_active(root, Some(&id));
    }
```
- [ ] **Step 4: run, expect pass.** `cargo test -p bokf-cli scaffold_registers_inits_and_activates` → PASS.
- [ ] **Step 5: commit.** `git add -A && git commit -m "feat(bokf-cli): set-active/get-active/register + scaffold git-init+register+activate"`

### Task 13: MCP tools (log-sync/log/restore/set-active/get-active) + scaffold integration

**Files:** Modify `crates/bokf-mcp/src/main.rs`; Modify `crates/bokf-mcp/src/ops.rs`

**Interfaces, Produces:** `mcp__biookf__bokf_log_sync|bokf_log|bokf_restore|bokf_set_active|bokf_get_active`; `ops::scaffold` git-inits+registers+activates.

- [ ] **Step 1: implement (ops.rs scaffold).** At the end of `ops::scaffold`, before `Ok(...)`, mirror the CLI integration (git ensure_repo + initial commit + register + set-active) using `bokf_core::git`/`registry`/`active`. Reuse the same logic as Task 12 Step 3 (adapted to the `ops::scaffold(bundle, name)` signature; `root = bundle.parent()`).

- [ ] **Step 2: implement (main.rs tools).** Add `param!` structs and `#[tool]` methods:
```rust
param!(LogSyncParam { #[doc="Bundle dir."] bundle: String, #[doc="ingest|convert|link|merge|lint|index|restore|manual"] kind: String, #[doc="Summary."] summary: String, #[doc="Optional delta line."] delta: Option<String> });
param!(LogParam { #[doc="Bundle dir."] bundle: String, #[doc="Max entries (default 20)."] limit: Option<usize> });
param!(RestoreParam { #[doc="Bundle dir."] bundle: String, #[doc="Commit sha to restore to."] sha: String, #[doc="Optional summary."] summary: Option<String> });
param!(RootIdParam { #[doc="Root dir containing bundles."] root: String, #[doc="KB id."] kb_id: String });
param!(RootParam2 { #[doc="Root dir containing bundles."] root: String });
```
```rust
    #[tool(name="bokf_log_sync", description="Append a dated log.md entry AND commit atomically (the sole step-committer).")]
    pub async fn log_sync(&self, p: Parameters<LogSyncParam>) -> Result<CallToolResult, rmcp::model::ErrorData> {
        use bokf_core::git::{today_iso, ChangeKind};
        match bokf_core::log_sync::log_sync(Path::new(&p.0.bundle), ChangeKind::parse(&p.0.kind), &p.0.summary, p.0.delta.as_deref(), &today_iso()) {
            Ok(sha) => ok(format!("committed {} [{}] {}", &sha[..8.min(sha.len())], p.0.kind, p.0.summary)),
            Err(e) => ok(format!("ERROR: {e}")),
        }
    }
    #[tool(name="bokf_log", description="Show commit history (newest-first) as JSON.")]
    pub async fn log(&self, p: Parameters<LogParam>) -> Result<CallToolResult, rmcp::model::ErrorData> {
        match bokf_core::git::GitRepo::open(&p.0.bundle).log(p.0.limit.unwrap_or(20)) {
            Ok(es) => ok(serde_json::to_string_pretty(&es).unwrap_or_default()),
            Err(e) => ok(format!("ERROR: {e}")),
        }
    }
    #[tool(name="bokf_restore", description="Forward-only restore to a prior commit sha.")]
    pub async fn restore(&self, p: Parameters<RestoreParam>) -> Result<CallToolResult, rmcp::model::ErrorData> {
        match bokf_core::git::GitRepo::open(&p.0.bundle).restore_to(&p.0.sha, p.0.summary.as_deref()) {
            Ok(sha) => ok(format!("restored; new commit {}", &sha[..8.min(sha.len())])),
            Err(e) => ok(format!("ERROR: {e}")),
        }
    }
    #[tool(name="bokf_set_active", description="Set which KB is active under <root>.")]
    pub async fn set_active(&self, p: Parameters<RootIdParam>) -> Result<CallToolResult, rmcp::model::ErrorData> {
        match bokf_core::active::set_active(Path::new(&p.0.root), Some(&p.0.kb_id)) { Ok(()) => ok(format!("active KB = {}", p.0.kb_id)), Err(e) => ok(format!("ERROR: {e}")) }
    }
    #[tool(name="bokf_get_active", description="Get the active KB id + path under <root>.")]
    pub async fn get_active(&self, p: Parameters<RootParam2>) -> Result<CallToolResult, rmcp::model::ErrorData> {
        let root = Path::new(&p.0.root);
        match bokf_core::active::get_active(root) {
            Some(id) => ok(serde_json::json!({"id": id, "path": bokf_core::registry::resolve(root, &id)}).to_string()),
            None => ok(serde_json::json!({"id": null}).to_string()),
        }
    }
```
- [ ] **Step 3: verify build.** `cargo build` → succeeds (whole workspace, incl. bokf-mcp).
- [ ] **Step 4: commit.** `git add -A && git commit -m "feat(bokf-mcp): version-tracking + active-KB MCP tools; scaffold integration"`

---

## Integration

### Task 14: End-to-end foundation test

**Files:** Test `crates/bokf-cli/tests/cli.rs`

- [ ] **Step 1: write the test.**
```rust
#[test]
fn end_to_end_scaffold_write_logsync_log_restore() {
    let root = tempfile::tempdir().unwrap();
    let bundle = root.path().join("demo-kb");
    let bokf = env!("CARGO_BIN_EXE_bokf");
    let run = |args: &[&str]| std::process::Command::new(bokf).args(args).output().unwrap();

    assert!(run(&["scaffold", bundle.to_str().unwrap(), "--name", "Demo"]).status.success());
    // active is the new bundle
    let ga = run(&["get-active", root.path().to_str().unwrap(), "--json"]);
    let v: serde_json::Value = serde_json::from_slice(&ga.stdout).unwrap();
    assert_eq!(v["id"], "demo-kb");
    // write a node + log-sync it
    let k = bundle.join("knowledge/gene"); std::fs::create_dir_all(&k).unwrap();
    std::fs::write(k.join("il6.md"), "---\ntype: Gene\nidentifier: IL6\nsubtype: protein_coding\n---\n# IL6\n").unwrap();
    assert!(run(&["log-sync", bundle.to_str().unwrap(), "--kind", "ingest", "--summary", "add IL6", "--delta", "+1 node"]).status.success());
    // history has the seed commit + the ingest
    let log = run(&["log", bundle.to_str().unwrap(), "--json"]);
    let entries: serde_json::Value = serde_json::from_slice(&log.stdout).unwrap();
    assert_eq!(entries[0]["kind"], "ingest");
    let first_sha = entries.as_array().unwrap().last().unwrap()["commit_sha"].as_str().unwrap().to_string();
    // restore to the seed: IL6 file is gone, history grew
    assert!(run(&["restore", bundle.to_str().unwrap(), &first_sha]).status.success());
    assert!(!k.join("il6.md").exists());
    let log2 = run(&["log", bundle.to_str().unwrap(), "--json"]);
    let e2: serde_json::Value = serde_json::from_slice(&log2.stdout).unwrap();
    assert_eq!(e2[0]["kind"], "restore");
}
```
- [ ] **Step 2: run.** `cargo test -p bokf-cli end_to_end_scaffold_write_logsync_log_restore` → PASS.
- [ ] **Step 3: full suite + lint.** `cargo test` (all crates) → PASS; `cargo build` → clean; optionally `cargo clippy`.
- [ ] **Step 4: commit.** `git add -A && git commit -m "test(bokf): end-to-end foundation (scaffold→write→log-sync→log→restore)"`

---

## Self-Review

**Spec coverage:** SP0 → Tasks 1-3 (predicate add, domain/range, `bokf predicates`, instructions sync). SP1 → Tasks 4-9 (deps, git lifecycle/commit/date, log+restore, txn, log_sync, CLI). SP2 → Tasks 10-13 (registry, active, CLI, MCP+scaffold). Integration → Task 14. All foundation deliverables in the spec map to a task.

**Placeholder scan:** No TBD/TODO; every code step shows complete code. (Task 13 Step 1 says "mirror the CLI integration"; the exact code is given in Task 12 Step 3 and is reused verbatim with `root = bundle.parent()`.)

**Type consistency:** `ChangeKind`/`GitRepo`/`Txn`/`HistoryEntry` defined in Task 5/7 and consumed by Tasks 6/8/9/13 with matching signatures; `log_sync(bundle, kind, summary, delta, date)` consistent across Task 8/9/13; `validate_kb_id`/`register`/`resolve`/`get_active`/`set_active` consistent across Tasks 10-13.

**Known caveats for the implementer:** (1) `tempfile` and `uuid` must resolve, so run `cargo build` once with network (Task 4). (2) `git`'s default branch name (`main` vs `master`) doesn't matter here because transactions capture the base branch from `current_branch()`. (3) The Task 2 test contains a note to strip any full-width punctuation an editor may insert into the `assert!`.
