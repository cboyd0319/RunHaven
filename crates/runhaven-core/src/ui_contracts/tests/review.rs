use super::*;

#[test]
fn run_diff_contract_bounds_output_and_preserves_layout() {
    let data = RunDiffData::from_run_diff_text(
        "run-\u{1b}123",
        "diff --git a/file b/file\n+\tsecret-shaped line\n-\u{1b}[31mold line\n",
        2,
    );

    assert_eq!(data.run_id, "run-123");
    assert_eq!(
        data.text,
        "diff --git a/file b/file\n+\tsecret-shaped line\n"
    );
    assert_eq!(data.returned_lines, 2);
    assert!(data.truncated);
    assert_eq!(data.source, "git diff");
    assert!(
        data.warnings
            .iter()
            .any(|warning| { warning.contains("workspace file contents") })
    );
    assert!(
        data.warnings
            .iter()
            .any(|warning| { warning.contains("truncated") })
    );
    assert!(!data.text.contains('\u{1b}'));
}

#[test]
fn run_diff_component_payload_round_trips_json() {
    let data = RunDiffData::from_run_diff_text("run-123", "+added\n", 200);
    let payload = RunHavenComponentPayload::RunDiff(Box::new(data));

    let encoded = serde_json::to_value(&payload).expect("serialize diff payload");

    assert_eq!(
        encoded.get("type").and_then(serde_json::Value::as_str),
        Some("runDiff")
    );
    let data = encoded.get("data").expect("diff data");
    assert_eq!(
        data.get("runId").and_then(serde_json::Value::as_str),
        Some("run-123")
    );
    assert!(data.get("returnedLines").is_some());
    assert!(data.get("run_id").is_none());
    assert!(data.get("returned_lines").is_none());

    let decoded: RunHavenComponentPayload =
        serde_json::from_value(encoded).expect("deserialize diff payload");

    assert_eq!(decoded, payload);
}

#[test]
fn run_control_result_contract_maps_core_payloads_to_ui_shape() {
    let stop = RunControlResultData::from_stop_payload(serde_json::json!({
        "run_id": "run-123",
        "container_name": "runhaven-codex-project-run",
        "return_code": 0
    }))
    .expect("stop result data");

    assert_eq!(stop.action, "stop");
    assert_eq!(stop.run_id, "run-123");
    assert_eq!(stop.container_name, "runhaven-codex-project-run");
    assert_eq!(stop.return_code, Some(0));
    assert_eq!(stop.status, "requested");
    assert_eq!(stop.marker_removed, None);
    assert!(stop.message.contains("Stop requested"));

    let force_stop = RunControlResultData::from_kill_payload(serde_json::json!({
        "run_id": "run-124",
        "container_name": "runhaven-codex-project-run",
        "return_code": 0
    }))
    .expect("force stop result data");
    assert!(force_stop.message.contains("Force stop requested"));
    assert!(!force_stop.message.contains("Hard stop"));

    let repair = RunControlResultData::from_repair_payload(serde_json::json!({
        "run_id": "run-456",
        "container_name": "runhaven-codex-stale-run",
        "inspect_return_code": 1,
        "marker_removed": true,
        "status": "removed"
    }))
    .expect("repair result data");

    assert_eq!(repair.action, "repair");
    assert_eq!(repair.run_id, "run-456");
    assert_eq!(repair.return_code, Some(1));
    assert_eq!(repair.status, "removed");
    assert_eq!(repair.marker_removed, Some(true));
    assert!(repair.message.contains("Removed stale active marker"));

    let payload = RunHavenComponentPayload::RunControlResult(Box::new(repair.clone()));
    let encoded = serde_json::to_value(&payload).expect("serialize run control payload");
    assert_eq!(
        encoded.get("type").and_then(serde_json::Value::as_str),
        Some("runControlResult")
    );
    let data = encoded.get("data").expect("run control data");
    assert!(data.get("runId").is_some());
    assert!(data.get("containerName").is_some());
    assert!(data.get("returnCode").is_some());
    assert!(data.get("markerRemoved").is_some());
    assert!(data.get("run_id").is_none());
    let decoded: RunHavenComponentPayload =
        serde_json::from_value(encoded).expect("deserialize run control payload");
    assert_eq!(decoded, payload);
}

#[test]
fn run_history_contract_omits_workspace_path_and_sanitizes_display_fields() {
    let data = RunHistoryListData::from_run_records([
        serde_json::json!({
            "timestamp": "2026-06-30T01:00:00Z",
            "started_at": "2026-06-30T00:00:00Z",
            "finished_at": "2026-06-30T01:00:00Z",
            "run_id": "run-123",
            "profile": "codex",
            "workspace": "/Users/c/secret/project",
            "workspace_scope": "current",
            "state_volume": "runhaven-codex-shared-home",
            "session": "none",
            "network": "provider",
            "status": "succeeded",
            "return_code": 0,
            "provider_policy": {"allowed": 3, "denied": 1},
            "auth_broker": {"allowed": 2, "denied": 0},
            "cleanup": {"provider_network": "removed"},
            "git": {"available": "false", "reason": "not-a-git-worktree"},
            "worktree": {"branch": "runhaven/codex/run-\u{1b}123"}
        }),
        serde_json::json!({
            "timestamp": "2026-06-30T02:00:00Z",
            "started_at": "2026-06-30T01:30:00Z",
            "finished_at": "2026-06-30T02:00:00Z",
            "run_id": "run-\u{1b}456",
            "profile": "codex",
            "workspace": "/Users/c/secret/project",
            "workspace_scope": "current",
            "state_volume": "runhaven-codex-shared-home",
            "session": "none",
            "network": "provider",
            "status": "failed",
            "return_code": 1,
            "provider_policy": {"allowed": 0, "denied": 0},
            "auth_broker": {"allowed": 0, "denied": 0},
            "cleanup": {"provider_network": "removed"},
            "git": {"available": "false", "reason": "not-a-git-worktree"}
        }),
    ]);

    assert_eq!(data.runs.len(), 2);
    let run = &data.runs[0];
    assert_eq!(run.run_id, "run-456");
    assert_eq!(run.review_command, "runhaven runs show run-456");
    assert_eq!(run.return_code, Some(1));

    let run = &data.runs[1];
    assert_eq!(run.run_id, "run-123");
    assert_eq!(
        run.worktree_branch.as_deref(),
        Some("runhaven/codex/run-123")
    );
    assert_eq!(run.provider_denied, 1);
    assert!(run.git_summary.contains("not-a-git-worktree"));

    let encoded = serde_json::to_string(&data).expect("serialize history data");
    assert!(!encoded.contains("/Users/c/secret/project"));
    assert!(!encoded.contains("\\u001b"));
}
