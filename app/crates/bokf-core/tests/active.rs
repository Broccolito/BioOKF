use bokf_core::registry;

#[test]
fn register_resolve_and_reject_dupes_and_bad_ids() {
    let dir = tempfile::tempdir().unwrap();
    // A registered base path must exist (AUDIT C6), so point at a real dir.
    let base = dir.path().join("ms-kb");
    std::fs::create_dir_all(&base).unwrap();
    let base_str = base.to_str().unwrap();
    registry::register(dir.path(), "ms-kb", base_str).unwrap();
    assert_eq!(registry::resolve(dir.path(), "ms-kb").as_deref(), Some(base_str));
    // Re-register the same (existing) path under the same id: rejected on the
    // dup-id check, not the existence check.
    assert!(registry::register(dir.path(), "ms-kb", base_str).is_err()); // dup id
    // A non-existent base path is rejected (AUDIT C6).
    assert!(registry::register(dir.path(), "other", "/abs/does-not-exist").is_err());
    assert!(registry::validate_kb_id("Bad_Id").is_err());
    assert!(registry::validate_kb_id("ok-1").is_ok());
    registry::unregister(dir.path(), "ms-kb").unwrap();
    assert!(registry::resolve(dir.path(), "ms-kb").is_none());
}

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
