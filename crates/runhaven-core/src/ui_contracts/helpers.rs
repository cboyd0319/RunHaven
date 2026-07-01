use std::collections::BTreeMap;
use std::path::Path;

pub(super) fn string_field(record: &serde_json::Value, name: &str) -> String {
    strip_terminal_controls(
        record
            .get(name)
            .and_then(serde_json::Value::as_str)
            .unwrap_or("-"),
    )
}

pub(super) fn auth_profiles_by_name(payload: &serde_json::Value) -> BTreeMap<String, String> {
    payload
        .get("profiles")
        .and_then(serde_json::Value::as_array)
        .map(|items| {
            items
                .iter()
                .map(|item| (string_field(item, "name"), string_field(item, "status")))
                .collect()
        })
        .unwrap_or_default()
}

/// Reduce a workspace path to a display-safe basename. This is the single label
/// vocabulary shared by the session list and the desktop dashboard/run status so
/// the same record cannot render two different labels for edge-case paths.
pub fn project_label(workspace: &str) -> String {
    let trimmed = workspace.trim();
    if trimmed.is_empty() || trimmed == "-" {
        return "-".to_string();
    }
    Path::new(trimmed)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .map(strip_terminal_controls)
        .unwrap_or_else(|| "-".to_string())
}

pub(super) fn display_agent_name(agent: &str) -> String {
    let mut chars = agent.chars();
    let Some(first) = chars.next() else {
        return "-".to_string();
    };
    let mut label = first.to_uppercase().to_string();
    label.push_str(chars.as_str());
    label
}

pub(super) fn status_label(status: &str) -> String {
    match status {
        "running" => "Running".to_string(),
        "started" => "Starting".to_string(),
        "finished" | "succeeded" => "Finished".to_string(),
        "failed" => "Failed".to_string(),
        "stopped" => "Stopped".to_string(),
        other => display_agent_name(other),
    }
}

pub(super) fn u32_field(record: &serde_json::Value, name: &str) -> u32 {
    record
        .get(name)
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0) as u32
}

pub(super) fn i32_field(record: &serde_json::Value, name: &str) -> Option<i32> {
    record
        .get(name)
        .and_then(serde_json::Value::as_i64)
        .and_then(|value| i32::try_from(value).ok())
}

pub(super) fn u64_pointer_field(record: &serde_json::Value, pointer: &str) -> u64 {
    record
        .pointer(pointer)
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0)
}

pub(super) fn string_pointer_field(record: &serde_json::Value, pointer: &str) -> String {
    strip_terminal_controls(
        record
            .pointer(pointer)
            .and_then(serde_json::Value::as_str)
            .unwrap_or("-"),
    )
}

pub(super) fn count_field(record: &serde_json::Value) -> u64 {
    record
        .get("count")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(1)
}

pub(super) fn strip_terminal_controls(text: &str) -> String {
    text.chars().filter(|ch| !ch.is_control()).collect()
}

pub(super) fn strip_terminal_controls_preserving_lines(text: &str) -> String {
    text.chars()
        .filter(|ch| *ch == '\n' || *ch == '\t' || !ch.is_control())
        .collect()
}
