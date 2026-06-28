//! The sole step-committer: append a dated `## YYYY-MM-DD` block to log.md AND
//! git-commit, atomically, so log.md and history never drift between commits.

use crate::git::{ChangeKind, GitRepo};
use std::path::Path;

/// Append a newest-first `## <date>` block (seeding `# Change log` if absent) and
/// commit it (with any other staged bundle changes) in one call. Returns the sha.
pub fn log_sync(bundle: &Path, kind: ChangeKind, summary: &str, delta: Option<&str>, date: &str) -> Result<String, String> {
    let log_path = bundle.join("log.md");
    let existing = std::fs::read_to_string(&log_path).unwrap_or_else(|_| "# Change log\n".to_string());
    let block = match delta {
        Some(d) => format!("\n## {date}\n\n{} | {summary}\n\n{d}\n", kind.as_str()),
        None => format!("\n## {date}\n\n{} | {summary}\n", kind.as_str()),
    };
    // newest-first: insert right after the first (title) line.
    let new = match existing.find('\n') {
        Some(i) => format!("{}{}{}", &existing[..=i], block, &existing[i + 1..]),
        None => format!("{existing}{block}"),
    };
    let tmp = bundle.join("log.md.tmp");
    std::fs::write(&tmp, &new).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, &log_path).map_err(|e| e.to_string())?;
    GitRepo::open(bundle).commit_all(kind, summary, delta)
}
