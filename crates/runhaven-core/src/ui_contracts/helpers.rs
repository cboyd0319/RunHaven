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
        "stop-requested" | "kill-requested" => "Stopping".to_string(),
        "finished" | "succeeded" => "Finished".to_string(),
        "failed" => "Failed".to_string(),
        "stopped" => "Stopped".to_string(),
        "ended" => "Ended".to_string(),
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

/// True for the invisible Unicode format characters (general category Cf)
/// that can reorder, hide, or spoof neighboring display text: the bidi
/// controls (U+061C, U+200E/U+200F, U+202A-U+202E, U+2066-U+2069), the zero-width
/// characters (U+200B-U+200D, U+2060), the soft hyphen (U+00AD), and the BOM
/// (U+FEFF). `char::is_control` covers only category Cc, so these survive it
/// and could otherwise reach native window titles (a repo basename with a
/// bidi override would render a misleading monitor title) and in-app labels.
/// The standard library has no general-category test, so the set is pinned
/// explicitly here.
fn is_invisible_format_char(ch: char) -> bool {
    matches!(
        ch,
        '\u{061C}'
            | '\u{00AD}'
            | '\u{200B}'..='\u{200F}'
            | '\u{202A}'..='\u{202E}'
            | '\u{2060}'
            | '\u{2066}'..='\u{2069}'
            | '\u{FEFF}'
    )
}

/// Drop every control character (Cc) and invisible format character
/// ([`is_invisible_format_char`]) from single-line display text. This is the
/// shared sanitizer for titles and labels that reach the WebView and native
/// surfaces such as monitor window titles.
pub fn strip_terminal_controls(text: &str) -> String {
    text.chars()
        .filter(|ch| !ch.is_control() && !is_invisible_format_char(*ch))
        .collect()
}

/// [`strip_terminal_controls`] for multi-line display text: keeps `\n` and
/// `\t` for layout, drops every other control and invisible format character.
/// Dropping zero-width joiners can split joined emoji in displayed output;
/// that cosmetic cost is accepted so bidi overrides cannot reorder what the
/// user reads.
pub(super) fn strip_terminal_controls_preserving_lines(text: &str) -> String {
    text.chars()
        .filter(|ch| {
            *ch == '\n' || *ch == '\t' || (!ch.is_control() && !is_invisible_format_char(*ch))
        })
        .collect()
}
