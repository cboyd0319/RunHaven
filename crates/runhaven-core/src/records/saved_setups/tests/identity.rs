use std::fs;

use super::*;

#[test]
fn saved_setups_dedupe_by_assistant_and_canonical_project_path() {
    let (cache, _cache_home) = isolated_cache();
    let project = project_fixture(cache.path(), "canonical-project");
    let alias = project.join("..").join("canonical-project");

    save_saved_setup_for_test("codex", &project).expect("save canonical setup");
    save_saved_setup_for_test("codex", &alias).expect("save canonical alias");
    save_saved_setup_for_test("claude", &project).expect("save other assistant");

    let records = read_saved_setups_for_test().expect("read saved setups");
    assert_eq!(
        records.len(),
        2,
        "the same assistant and canonical project is one setup"
    );

    let canonical = fs::canonicalize(&project).expect("canonical project");
    assert!(
        records
            .iter()
            .all(|record| record.project_path == canonical)
    );
    assert!(records.iter().any(|record| record.assistant == "codex"));
    assert!(records.iter().any(|record| record.assistant == "claude"));
}
