// helpers.rs holds only pub(super) functions, so this suite imports them
// directly rather than through the `use super::*;` re-export the sibling
// suites rely on for public data types.
use crate::ui_contracts::helpers::{
    auth_profiles_by_name, count_field, display_agent_name, i32_field, project_label, status_label,
    string_field, string_pointer_field, strip_terminal_controls,
    strip_terminal_controls_preserving_lines, u32_field, u64_pointer_field,
};

#[test]
fn string_field_defaults_to_dash_and_strips_terminal_controls() {
    let record = serde_json::json!({
        "clean": "ok",
        "noisy": "run-\u{1b}[31m123\u{7}",
        "number": 7,
    });

    assert_eq!(string_field(&record, "clean"), "ok");
    // Missing key falls back to the "-" placeholder.
    assert_eq!(string_field(&record, "absent"), "-");
    // A present-but-non-string value is treated as missing, not stringified.
    assert_eq!(string_field(&record, "number"), "-");
    // Escape and bell control bytes are removed from display text.
    assert_eq!(string_field(&record, "noisy"), "run-[31m123");
}

#[test]
fn auth_profiles_by_name_parses_sorted_map_and_strips_controls() {
    let payload = serde_json::json!({
        "profiles": [
            {"name": "codex", "status": "brokered"},
            {"name": "cla\u{1b}ude", "status": "iso\u{7}lated"},
            {"status": "missing-name"},
        ]
    });

    let map = auth_profiles_by_name(&payload);

    // BTreeMap keeps names sorted and strips control bytes from names/status.
    let entries: Vec<(&str, &str)> = map
        .iter()
        .map(|(name, status)| (name.as_str(), status.as_str()))
        .collect();
    assert_eq!(
        entries,
        vec![
            // Missing name collapses to the "-" placeholder key, which sorts
            // ahead of the letters in the BTreeMap ordering.
            ("-", "missing-name"),
            ("claude", "isolated"),
            ("codex", "brokered"),
        ]
    );
}

#[test]
fn auth_profiles_by_name_missing_profiles_is_empty() {
    assert!(auth_profiles_by_name(&serde_json::json!({})).is_empty());
    // A non-array "profiles" value is ignored rather than panicking.
    assert!(auth_profiles_by_name(&serde_json::json!({"profiles": "nope"})).is_empty());
}

#[test]
fn project_label_reduces_sensitive_path_to_basename() {
    // Only the final path component is shown; the sensitive directory chain
    // never appears in the label.
    assert_eq!(project_label("/Users/c/secret/project"), "project");
    assert_eq!(project_label("/Users/c/secret/project/"), "project");
    // Control bytes in the basename are stripped.
    assert_eq!(project_label("/Users/c/secret/pro\u{1b}ject"), "project");
}

#[test]
fn project_label_empty_dash_and_rootless_inputs_return_dash() {
    assert_eq!(project_label(""), "-");
    assert_eq!(project_label("   "), "-");
    assert_eq!(project_label("-"), "-");
    // A path with no final component (root only) has no basename to show.
    assert_eq!(project_label("/"), "-");
}

#[test]
fn display_agent_name_capitalizes_first_char_and_guards_empty() {
    assert_eq!(display_agent_name("codex"), "Codex");
    assert_eq!(display_agent_name(""), "-");
    // Multi-byte first char capitalization does not truncate the tail.
    assert_eq!(display_agent_name("émigré"), "Émigré");
}

#[test]
fn status_label_maps_known_states_and_titlecases_unknown() {
    assert_eq!(status_label("running"), "Running");
    assert_eq!(status_label("started"), "Starting");
    assert_eq!(status_label("finished"), "Finished");
    assert_eq!(status_label("succeeded"), "Finished");
    assert_eq!(status_label("failed"), "Failed");
    assert_eq!(status_label("stopped"), "Stopped");
    // Unknown states fall back to title-casing.
    assert_eq!(status_label("queued"), "Queued");
    assert_eq!(status_label(""), "-");
}

#[test]
fn numeric_fields_apply_documented_defaults_and_bounds() {
    let record = serde_json::json!({
        "count": 4,
        "small": 5,
        "negative": -3,
        "too_big": i64::from(i32::MAX) + 1,
        "nested": {"deep": 9},
    });

    assert_eq!(u32_field(&record, "small"), 5);
    // Missing numeric fields default to zero.
    assert_eq!(u32_field(&record, "absent"), 0);

    assert_eq!(i32_field(&record, "negative"), Some(-3));
    // Values outside i32 range are rejected rather than truncated.
    assert_eq!(i32_field(&record, "too_big"), None);
    assert_eq!(i32_field(&record, "absent"), None);

    // count_field defaults to 1 (a single event), not 0.
    assert_eq!(count_field(&record), 4);
    assert_eq!(count_field(&serde_json::json!({})), 1);

    assert_eq!(u64_pointer_field(&record, "/nested/deep"), 9);
    assert_eq!(u64_pointer_field(&record, "/nested/missing"), 0);
}

#[test]
fn string_pointer_field_resolves_pointer_with_dash_default() {
    let record = serde_json::json!({"worktree": {"branch": "runhaven/codex/run-\u{1b}123"}});

    assert_eq!(
        string_pointer_field(&record, "/worktree/branch"),
        "runhaven/codex/run-123"
    );
    assert_eq!(string_pointer_field(&record, "/worktree/absent"), "-");
}

#[test]
fn strip_terminal_controls_variants_differ_on_whitespace_controls() {
    let input = "a\u{1b}b\tc\nd\u{7}e";

    // The strict form removes every control byte, including tab and newline.
    assert_eq!(strip_terminal_controls(input), "abcde");
    // The line-preserving form keeps tab and newline for multi-line layout.
    assert_eq!(strip_terminal_controls_preserving_lines(input), "ab\tc\nde");
}
