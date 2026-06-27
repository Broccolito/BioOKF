use okf_core::registry;

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

#[test]
fn set_get_clear_active() {
    let dir = tempfile::tempdir().unwrap();
    assert!(okf_core::active::get_active(dir.path()).is_none());
    okf_core::active::set_active(dir.path(), Some("ms-kb")).unwrap();
    assert_eq!(okf_core::active::get_active(dir.path()).as_deref(), Some("ms-kb"));
    assert!(okf_core::active::set_active(dir.path(), Some("Bad_Id")).is_err());
    okf_core::active::set_active(dir.path(), None).unwrap();
    assert!(okf_core::active::get_active(dir.path()).is_none());
}
