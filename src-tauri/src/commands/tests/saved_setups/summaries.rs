use super::*;

#[test]
fn saved_setup_summaries_do_not_expose_raw_project_paths() {
    let (cache, _cache_home) = isolated_cache();
    let project = project_fixture(cache.path(), "private-project-name");
    let canonical = fs::canonicalize(&project).expect("canonical project");
    save_saved_setup_for_test("codex", &project).expect("save setup");

    let summaries = list_saved_setups_for_test().expect("list saved setups");
    assert_eq!(summaries.len(), 1, "saved setup appears in the API list");
    let summary = &summaries[0];
    assert_eq!(summary["assistant"], "codex");
    assert_eq!(summary["project"], "private-project-name");
    assert_eq!(summary["name"], "Codex · private-project-name");
    assert_eq!(summary["available"], true);
    assert_eq!(summary["pinned"], false);

    let encoded = serde_json::to_string(&summaries).expect("serialize summaries");
    assert!(
        !encoded.contains(&canonical.display().to_string()),
        "saved setup summaries must not expose the raw project path"
    );
}

#[test]
fn pin_and_unpin_are_persistent_and_reversible() {
    let (cache, _cache_home) = isolated_cache();
    let project = project_fixture(cache.path(), "pin-project");
    let id = save_saved_setup_for_test("claude", &project).expect("save setup");

    set_saved_setup_pinned_for_test(&id, true).expect("pin setup");
    let pinned = list_saved_setups_for_test().expect("list pinned setup");
    let pinned_summary = pinned
        .iter()
        .find(|summary| summary_id(summary) == id)
        .expect("pinned setup persists");
    assert_eq!(pinned_summary["pinned"], true);

    set_saved_setup_pinned_for_test(&id, false).expect("unpin setup");
    let unpinned = list_saved_setups_for_test().expect("list unpinned setup");
    let unpinned_summary = unpinned
        .iter()
        .find(|summary| summary_id(summary) == id)
        .expect("unpinned setup persists");
    assert_eq!(unpinned_summary["pinned"], false);
}
