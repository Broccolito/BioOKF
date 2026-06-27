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
    assert!(std::fs::read_to_string(dir.path().join(".gitignore")).unwrap().contains("raw/**/original.*"));
}

#[test]
fn today_iso_is_well_formed() {
    let d = bokf_core::git::today_iso();
    assert_eq!(d.len(), 10);
    assert_eq!(&d[4..5], "-");
    assert_eq!(&d[7..8], "-");
}

#[test]
fn log_parses_kind_and_restore_is_forward_only() {
    let dir = tempfile::tempdir().unwrap();
    let repo = GitRepo::open(dir.path());
    repo.ensure_repo().unwrap();
    std::fs::write(dir.path().join("a.md"), "v1").unwrap();
    let first = repo.commit_all(ChangeKind::Ingest, "add a", Some("+1 node")).unwrap();
    std::fs::write(dir.path().join("a.md"), "v2").unwrap();
    std::fs::write(dir.path().join("b.md"), "new file").unwrap();
    repo.commit_all(ChangeKind::Lint, "fix a", None).unwrap();
    let entries = repo.log(10).unwrap();
    assert_eq!(entries[0].kind, ChangeKind::Lint);
    assert_eq!(entries[1].kind, ChangeKind::Ingest);
    assert_eq!(entries[1].delta.as_deref(), Some("+1 node"));

    // restore to first: a.md becomes v1, b.md (added later) is removed, history GROWS.
    let n_before = repo.log(50).unwrap().len();
    repo.restore_to(&first, Some("roll back a")).unwrap();
    assert_eq!(std::fs::read_to_string(dir.path().join("a.md")).unwrap(), "v1");
    assert!(!dir.path().join("b.md").exists(), "file added after the target must be removed");
    assert_eq!(repo.log(50).unwrap().len(), n_before + 1);
    assert_eq!(repo.log(1).unwrap()[0].kind, ChangeKind::Restore);
}

#[test]
fn txn_squashes_to_one_entry() {
    let dir = tempfile::tempdir().unwrap();
    let repo = GitRepo::open(dir.path());
    repo.ensure_repo().unwrap();
    std::fs::write(dir.path().join("seed.md"), "seed").unwrap();
    repo.commit_all(ChangeKind::Manual, "seed", None).unwrap();
    let base_n = repo.log(50).unwrap().len();
    let txn = repo.begin_txn("ingest paper").unwrap();
    for i in 0..3 {
        std::fs::write(dir.path().join(format!("n{i}.md")), "x").unwrap();
        repo.commit_all(ChangeKind::Ingest, &format!("step {i}"), None).unwrap();
    }
    let sha = repo.commit_txn(&txn, ChangeKind::Ingest, "ingest Paper X", Some("+3 nodes")).unwrap();
    assert_eq!(sha.len(), 40);
    assert_eq!(repo.log(50).unwrap().len(), base_n + 1);
    assert_eq!(repo.log(1).unwrap()[0].summary, "ingest Paper X");
    assert!(dir.path().join("n2.md").exists());
}

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
    let repo = GitRepo::open(dir.path());
    assert_eq!(repo.log(1).unwrap()[0].commit_sha, sha);
    assert_eq!(repo.log(1).unwrap()[0].kind, ChangeKind::Ingest);
}
