use super::*;

#[test]
fn active_run_log_snapshot_contract_maps_core_payload_to_ui_shape() {
    let payload =
        crate::runtime::active::log_snapshot_payload_from_stdout("run-123", 20, b"one\ntwo\n", 64)
            .expect("active run log payload");

    let data = ActiveRunLogSnapshotData::from_active_log_snapshot_payload(payload)
        .expect("active run log snapshot data");

    assert_eq!(data.run_id, "run-123");
    assert!(!data.captured_at.is_empty());
    assert_eq!(data.requested_lines, 20);
    assert_eq!(data.text, "one\ntwo\n");
    assert_eq!(data.returned_lines, 2);
    assert!(!data.truncated);
    assert_eq!(data.source, "container-stdio");
    assert!(data.warnings.iter().any(|warning| {
        warning.contains("Raw container output") && warning.contains("workspace content")
    }));
}

#[test]
fn active_run_log_snapshot_contract_serializes_camel_case() {
    let payload =
        crate::runtime::active::log_snapshot_payload_from_stdout("run-123", 50, b"line\n", 64)
            .expect("active run log payload");
    let data = ActiveRunLogSnapshotData::from_active_log_snapshot_payload(payload)
        .expect("active run log snapshot data");

    let encoded = serde_json::to_value(&data).expect("serialize snapshot data");

    assert!(encoded.get("runId").is_some());
    assert!(encoded.get("capturedAt").is_some());
    assert!(encoded.get("requestedLines").is_some());
    assert!(encoded.get("returnedLines").is_some());
    assert!(encoded.get("run_id").is_none());
    assert!(encoded.get("captured_at").is_none());
    assert!(encoded.get("requested_lines").is_none());
    assert!(encoded.get("returned_lines").is_none());
}

#[test]
fn active_run_log_snapshot_component_payload_round_trips_json() {
    let payload =
        crate::runtime::active::log_snapshot_payload_from_stdout("run-123", 20, b"one\n", 64)
            .expect("active run log payload");
    let data = ActiveRunLogSnapshotData::from_active_log_snapshot_payload(payload)
        .expect("active run log snapshot data");
    let payload = RunHavenComponentPayload::ActiveRunLogSnapshot(Box::new(data));

    let encoded = serde_json::to_value(&payload).expect("serialize snapshot payload");

    assert_eq!(
        encoded.get("type").and_then(serde_json::Value::as_str),
        Some("activeRunLogSnapshot")
    );
    let data = encoded.get("data").expect("snapshot data");
    assert_eq!(
        data.get("runId").and_then(serde_json::Value::as_str),
        Some("run-123")
    );
    assert!(data.get("requestedLines").is_some());
    assert!(data.get("returnedLines").is_some());
    assert!(data.get("run_id").is_none());
    assert!(data.get("requested_lines").is_none());
    assert!(data.get("returned_lines").is_none());

    let decoded: RunHavenComponentPayload =
        serde_json::from_value(encoded).expect("deserialize snapshot payload");

    assert_eq!(decoded, payload);
}

#[test]
fn active_run_list_contract_omits_workspace_paths() {
    let data = ActiveRunListData::from_active_run_records([serde_json::json!({
        "timestamp": "2026-06-29T00:00:00Z",
        "run_id": "run-123",
        "profile": "codex",
        "workspace": "/Users/c/secret/project",
        "network": "provider",
        "status": "running",
        "auth_scope": "project",
        "container_name": "runhaven-codex-project-run",
        "state_volume": "runhaven-codex-shared-home",
        "session": "none"
    })]);

    assert_eq!(data.runs.len(), 1);
    assert_eq!(data.runs[0].run_id, "run-123");
    assert_eq!(data.runs[0].profile, "codex");
    assert_eq!(data.runs[0].auth_scope, "project");
    let serialized = serde_json::to_string(&data).expect("serialize");
    assert!(
        !serialized.contains("/Users/c/secret/project"),
        "TUI active-run summaries must not include workspace paths"
    );
}
