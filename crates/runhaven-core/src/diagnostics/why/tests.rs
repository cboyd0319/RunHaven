use super::*;

fn str_field(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_string)
}

#[test]
fn network_payload_reports_each_mode() {
    for mode in SUPPORTED_NETWORK_MODES {
        let payload = why_network_payload(mode).expect("known mode");
        assert_eq!(str_field(&payload, "mode").as_deref(), Some(*mode));
        assert!(payload.get("behavior").is_some());
        assert!(payload.get("boundary").is_some());
    }
    let provider = why_network_payload("provider").expect("provider");
    assert!(provider.get("inspect").is_some());
}

#[test]
fn network_payload_errors_on_unknown_mode() {
    let error = why_network_payload("bogus").expect_err("unknown mode");
    assert!(error.to_string().contains("invalid network mode"));
}

#[test]
fn state_payload_reports_known_agent_patterns() {
    let payload = why_state_payload("claude").expect("known agent");
    assert_eq!(str_field(&payload, "profile").as_deref(), Some("claude"));
    assert_eq!(
        str_field(&payload, "state_volume_pattern").as_deref(),
        Some("runhaven-claude-<project-id>-home")
    );
    // The resolved workspace path is never embedded; only the placeholder is.
    let serialized = serde_json::to_string(&payload).expect("serialize");
    assert!(serialized.contains("<project-id>"));
    assert!(
        str_field(&payload, "project_id")
            .unwrap()
            .contains("not embedded")
    );
}

#[test]
fn state_payload_errors_on_unknown_agent() {
    assert!(why_state_payload("not-a-real-agent").is_err());
}

#[test]
fn host_payload_denies_ip_literal_and_single_label() {
    let ip = why_host_payload("1.1.1.1", 443, None).expect("ip literal");
    assert_eq!(str_field(&ip, "decision").as_deref(), Some("denied"));
    let single = why_host_payload("localhost", 443, None).expect("single label");
    assert_eq!(str_field(&single, "decision").as_deref(), Some("denied"));
}

#[test]
fn host_payload_allows_bundled_agent_host() {
    let payload = why_host_payload("api.openai.com", 443, Some("codex")).expect("codex host");
    assert_eq!(str_field(&payload, "decision").as_deref(), Some("allowed"));
    assert_eq!(str_field(&payload, "profile").as_deref(), Some("codex"));
    assert!(
        payload
            .get("matched_rules")
            .map(Value::is_array)
            .unwrap_or(false)
    );
}

#[test]
fn host_payload_reports_unmatched_host_with_records() {
    let payload = why_host_payload("api.example.com", 443, Some("codex")).expect("unmatched host");
    assert_eq!(
        str_field(&payload, "decision").as_deref(),
        Some("not-allowed")
    );
    assert!(
        payload
            .get("bundled_hosts")
            .map(Value::is_array)
            .unwrap_or(false)
    );
    assert!(
        payload
            .get("endpoint_records")
            .map(Value::is_array)
            .unwrap_or(false)
    );
}

#[test]
fn host_payload_rejects_zero_port_and_bad_host() {
    assert!(why_host_payload("api.openai.com", 0, None).is_err());
    assert!(why_host_payload("not a host", 443, None).is_err());
}

#[test]
fn host_payload_errors_on_unknown_agent() {
    assert!(why_host_payload("api.openai.com", 443, Some("nope")).is_err());
}

#[test]
fn workspace_payload_allows_existing_directory() {
    let dir = tempfile::tempdir().expect("dir");
    let payload = why_workspace_payload(dir.path(), "current", false).expect("existing dir");
    assert_eq!(str_field(&payload, "decision").as_deref(), Some("allowed"));
    assert_eq!(
        str_field(&payload, "verdict").as_deref(),
        Some("RunHaven can mount this folder safely.")
    );
}

#[test]
fn workspace_payload_denies_missing_path_with_pathfree_verdict() {
    let payload = why_workspace_payload(Path::new("/no/such/runhaven/path"), "current", false)
        .expect("missing path");
    assert_eq!(str_field(&payload, "decision").as_deref(), Some("denied"));
    // The path-free verdict never contains the probed path.
    assert!(!str_field(&payload, "verdict").unwrap().contains('/'));
}

#[test]
fn workspace_payload_errors_on_unknown_scope() {
    let dir = tempfile::tempdir().expect("dir");
    assert!(why_workspace_payload(dir.path(), "bogus-scope", false).is_err());
}
