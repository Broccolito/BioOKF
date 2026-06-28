//! Git-backed version tracking for a BioOKF bundle, implemented by shelling out
//! to the system `git` binary (no libgit2 dependency).

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

/// The kind of curation change a commit records. Exactly 8 variants; the kind is
/// always passed explicitly by the caller (never inferred).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeKind {
    Ingest,
    Convert,
    Link,
    Merge,
    Lint,
    Index,
    Restore,
    Manual,
}

impl ChangeKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChangeKind::Ingest => "ingest",
            ChangeKind::Convert => "convert",
            ChangeKind::Link => "link",
            ChangeKind::Merge => "merge",
            ChangeKind::Lint => "lint",
            ChangeKind::Index => "index",
            ChangeKind::Restore => "restore",
            ChangeKind::Manual => "manual",
        }
    }
    pub fn parse(s: &str) -> ChangeKind {
        match s {
            "ingest" => ChangeKind::Ingest,
            "convert" => ChangeKind::Convert,
            "link" => ChangeKind::Link,
            "merge" => ChangeKind::Merge,
            "lint" => ChangeKind::Lint,
            "index" => ChangeKind::Index,
            "restore" => ChangeKind::Restore,
            _ => ChangeKind::Manual,
        }
    }
}

/// One reconstructed commit, parsed from the git log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub commit_sha: String,
    pub kind: ChangeKind,
    pub summary: String,
    pub delta: Option<String>,
    pub timestamp: String,
}

/// An open transaction: a temporary branch plus the base it was forked from.
pub struct Txn {
    pub branch: String,
    pub base: String,
}

pub struct GitRepo {
    root: PathBuf,
}

impl GitRepo {
    pub fn open(root: impl AsRef<Path>) -> GitRepo {
        GitRepo { root: root.as_ref().to_path_buf() }
    }

    pub fn preflight() -> Result<(), String> {
        match Command::new("git").arg("--version").output() {
            Ok(o) if o.status.success() => Ok(()),
            _ => Err("`git` not found on PATH; install git to enable BioOKF version tracking".into()),
        }
    }

    pub fn is_repo(&self) -> bool {
        self.root.join(".git").exists()
    }

    fn run(&self, args: &[&str]) -> Result<String, String> {
        let out = Command::new("git")
            .arg("-C")
            .arg(&self.root)
            .args(args)
            .output()
            .map_err(|e| format!("git {args:?}: {e}"))?;
        if !out.status.success() {
            return Err(format!("git {args:?} failed: {}", String::from_utf8_lossy(&out.stderr).trim()));
        }
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    }

    /// init-on-first-use: `git init` + bot identity + a `.gitignore` that excludes
    /// the immutable raw originals. Idempotent.
    pub fn ensure_repo(&self) -> Result<(), String> {
        GitRepo::preflight()?;
        if self.is_repo() {
            return Ok(());
        }
        self.run(&["init", "-q"])?;
        self.run(&["config", "user.name", "BioOKF Curation"])?;
        self.run(&["config", "user.email", "curation@biookf.local"])?;
        self.run(&["config", "commit.gpgsign", "false"])?;
        let gi = self.root.join(".gitignore");
        if !gi.exists() {
            // Immutable raw originals and the regenerable PDF page-image cache stay local; the
            // committed truth is the curated knowledge/ docs and each source's source.md.
            std::fs::write(&gi, "raw/**/original.*\nraw/**/pages/\n").map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    fn has_head(&self) -> bool {
        Command::new("git")
            .arg("-C")
            .arg(&self.root)
            .args(["rev-parse", "--verify", "HEAD"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    pub fn head_sha(&self) -> Result<String, String> {
        Ok(self.run(&["rev-parse", "HEAD"])?.trim().to_string())
    }

    pub fn current_branch(&self) -> Result<String, String> {
        Ok(self.run(&["rev-parse", "--abbrev-ref", "HEAD"])?.trim().to_string())
    }

    /// Stage everything and commit `[<kind>] <summary>` (+ optional `delta:` body).
    /// No-op (returns current HEAD) when there is nothing to commit.
    pub fn commit_all(&self, kind: ChangeKind, summary: &str, delta: Option<&str>) -> Result<String, String> {
        self.ensure_repo()?;
        self.run(&["add", "-A"])?;
        let dirty = !self.run(&["status", "--porcelain"])?.trim().is_empty();
        if !dirty && self.has_head() {
            return self.head_sha();
        }
        let subject = format!("[{}] {}", kind.as_str(), summary);
        let body = delta.map(|d| format!("delta: {d}"));
        let mut args: Vec<String> = vec!["commit".into(), "-q".into(), "-m".into(), subject];
        if let Some(b) = &body {
            args.push("-m".into());
            args.push(b.clone());
        }
        if !self.has_head() && !dirty {
            args.push("--allow-empty".into());
        }
        let argref: Vec<&str> = args.iter().map(String::as_str).collect();
        self.run(&argref)?;
        self.head_sha()
    }

    pub fn log(&self, limit: usize) -> Result<Vec<HistoryEntry>, String> {
        if !self.has_head() {
            return Ok(vec![]);
        }
        let raw = self.run(&["log", &format!("-n{limit}"), "--pretty=format:%H%x1f%s%x1f%b%x1f%cI%x1e"])?;
        let mut entries = Vec::new();
        for rec in raw.split('\x1e') {
            let rec = rec.trim_start_matches('\n');
            if rec.trim().is_empty() {
                continue;
            }
            let p: Vec<&str> = rec.split('\x1f').collect();
            if p.len() < 4 {
                continue;
            }
            let subject = p[1];
            let (kind, summary) = match (subject.find('['), subject.find(']')) {
                (Some(0), Some(close)) => (ChangeKind::parse(&subject[1..close]), subject[close + 1..].trim().to_string()),
                _ => (ChangeKind::Manual, subject.to_string()),
            };
            let delta = p[2].lines().find_map(|l| l.strip_prefix("delta: ").map(|s| s.to_string()));
            entries.push(HistoryEntry {
                commit_sha: p[0].to_string(),
                kind,
                summary,
                delta,
                timestamp: p[3].trim().to_string(),
            });
        }
        Ok(entries)
    }

    /// Forward-only restore: reproduce the tree of `sha` exactly (removing files
    /// added since) and record it as a NEW `[restore]` commit on top of HEAD.
    pub fn restore_to(&self, sha: &str, summary: Option<&str>) -> Result<String, String> {
        self.ensure_repo()?;
        // read-tree -u --reset makes index + worktree match `sha` exactly (incl.
        // deletions) without moving HEAD; commit_all then commits that tree forward.
        self.run(&["read-tree", "-u", "--reset", sha])?;
        let s = summary
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("restore to {}", &sha[..sha.len().min(8)]));
        self.commit_all(ChangeKind::Restore, &s, Some(&format!("restored tree of {sha}")))
    }

    pub fn begin_txn(&self, label: &str) -> Result<Txn, String> {
        self.ensure_repo()?;
        if !self.has_head() {
            self.commit_all(ChangeKind::Manual, "init", None)?;
        }
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

/// Today's date as YYYY-MM-DD (UTC), dependency-free.
pub fn today_iso() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let (y, m, d) = civil_from_days((secs / 86_400) as i64);
    format!("{y:04}-{m:02}-{d:02}")
}

/// Howard Hinnant's days-from-civil inverse (proleptic Gregorian).
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

fn slug(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars().take(40) {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches('-').to_string()
}

fn short_id() -> String {
    let s = uuid::Uuid::new_v4().simple().to_string();
    s[..8].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn civil_epoch() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
    }
    #[test]
    fn changekind_roundtrip() {
        for k in [ChangeKind::Ingest, ChangeKind::Merge, ChangeKind::Index, ChangeKind::Manual] {
            assert_eq!(ChangeKind::parse(k.as_str()), k);
        }
    }
}
