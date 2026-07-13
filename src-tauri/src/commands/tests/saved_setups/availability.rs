use super::*;

#[test]
fn missing_saved_setup_project_is_reported_unavailable_without_exposing_its_path() {
    let (cache, _cache_home) = isolated_cache();
    let project = project_fixture(cache.path(), "removed-project");
    let canonical = fs::canonicalize(&project).expect("canonical project");
    let id = save_saved_setup_for_test("codex", &project).expect("save setup");
    fs::remove_dir(&project).expect("remove project");

    let summaries = list_saved_setups_for_test().expect("list saved setups");
    let summary = summaries
        .iter()
        .find(|summary| summary_id(summary) == id)
        .expect("missing project setup remains listed");
    assert_eq!(summary["available"], false);

    let error = select_saved_setup_for_test(&id).expect_err("missing project is unavailable");
    assert!(error.to_ascii_lowercase().contains("unavailable"));
    assert!(
        !error.contains(&canonical.display().to_string()),
        "unavailable error must not expose the raw project path"
    );
}
