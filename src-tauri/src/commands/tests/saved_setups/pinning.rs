use super::*;

#[test]
fn explicit_pinning_saves_dedupes_and_returns_only_an_opaque_id() {
    let (cache, _cache_home) = isolated_cache();
    let project = project_fixture(cache.path(), "pinned-draft-project");
    let alias = project.join("..").join("pinned-draft-project");

    let id = pin_saved_setup_for_test("codex", &project).expect("pin draft setup");
    let duplicate_id = pin_saved_setup_for_test("codex", &alias).expect("pin canonical duplicate");

    assert_eq!(duplicate_id, id);
    assert!(id.starts_with("setup-"));
    assert!(!id.contains(&project.display().to_string()));

    let summaries = list_saved_setups_for_test().expect("list pinned setup");
    assert_eq!(summaries.len(), 1, "canonical duplicate stays one setup");
    assert_eq!(summaries[0]["id"], id);
    assert_eq!(summaries[0]["pinned"], true);
}

#[test]
fn launches_do_not_save_project_paths_without_the_save_setup_action() {
    let launch_source = include_str!("../../launch/mod.rs");

    assert!(!launch_source.contains("save_confirmed_setup"));
    assert!(!launch_source.contains("saved_setups::save_setup"));
}
