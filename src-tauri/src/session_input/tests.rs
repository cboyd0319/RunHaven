use super::*;

#[test]
fn writer_is_bound_to_run_and_removed_on_unregister() {
    let registry = SessionInputRegistry::default();
    registry
        .register_writer("bound-session", "bound-run", |bytes| Ok(bytes.len()))
        .expect("register writer");

    assert!(
        registry
            .has_writer("bound-session", "bound-run")
            .expect("writer check")
    );
    assert!(
        !registry
            .has_writer("bound-session", "other-run")
            .expect("other run check")
    );

    registry.unregister("bound-session").expect("unregister");
    assert!(
        !registry
            .has_writer("bound-session", "bound-run")
            .expect("writer check after unregister")
    );
}
