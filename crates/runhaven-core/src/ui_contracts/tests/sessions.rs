use super::*;

#[test]
fn managed_session_list_contract_merges_active_runs_with_auth_status() {
    let data = ManagedSessionListData::from_active_runs_and_auth_status(
        [
            serde_json::json!({
                "timestamp": "2026-06-30T01:00:00Z",
                "run_id": "run-older",
                "profile": "codex",
                "workspace": "/sensitive/workspaces/docs",
                "network": "provider",
                "status": "running",
                "container_name": "runhaven-codex-docs-run",
                "state_volume": "runhaven-codex-docs-home",
                "auth_scope": "project",
                "session": "docs"
            }),
            serde_json::json!({
                "timestamp": "2026-06-30T02:00:00Z",
                "run_id": "run-newer",
                "profile": "claude",
                "workspace": "/sensitive/workspaces/backend",
                "network": "provider",
                "status": "running",
                "container_name": "runhaven-claude-backend-run",
                "state_volume": "runhaven-claude-backend-home",
                "auth_scope": "agent",
                "session": "backend"
            }),
        ],
        serde_json::json!({
            "status": "api-key-broker",
            "runtime": "host-broker",
            "profiles": [
                {"name": "codex", "status": "api-key-broker"},
                {"name": "claude", "status": "api-key-broker"}
            ],
            "token": "should-not-leak"
        }),
    );

    assert_eq!(data.sessions.len(), 2);
    let first = &data.sessions[0];
    assert_eq!(first.id, "run-newer");
    assert_eq!(first.agent, "claude");
    assert_eq!(first.session, "backend");
    assert_eq!(first.project, "backend");
    assert_eq!(first.status, "running");
    assert_eq!(first.status_label, "Running");
    assert_eq!(first.auth_status, "api-key-broker");
    assert_eq!(first.auth_scope, "agent");
    assert_eq!(first.detail_command, "runhaven runs show run-newer");

    let encoded = serde_json::to_string(&data).expect("serialize managed sessions");
    for forbidden in [
        "/sensitive/workspaces",
        "should-not-leak",
        "state_volume",
        "runhaven-codex-docs-home",
    ] {
        assert!(
            !encoded.contains(forbidden),
            "managed session payload leaked forbidden value {forbidden:?}"
        );
    }
}

#[test]
fn managed_session_component_payload_round_trips_json() {
    let data = ManagedSessionListData::from_active_runs_and_auth_status(
        [serde_json::json!({
            "timestamp": "2026-06-30T02:00:00Z",
            "run_id": "run-123",
            "profile": "codex",
            "workspace": "/sensitive/workspaces/project",
            "network": "provider",
            "status": "running",
            "container_name": "runhaven-codex-project-run",
            "session": "project"
        })],
        serde_json::json!({
            "profiles": [{"name": "codex", "status": "api-key-broker"}]
        }),
    );
    let payload = RunHavenComponentPayload::ManagedSessionList(data);
    let encoded = serde_json::to_value(&payload).expect("serialize managed sessions");

    assert_eq!(
        encoded.get("type").and_then(serde_json::Value::as_str),
        Some("managedSessionList")
    );
    let data = encoded.get("data").expect("session data");
    assert!(data.get("sessions").is_some());
    assert!(data.to_string().contains("runhaven runs show run-123"));

    let decoded: RunHavenComponentPayload =
        serde_json::from_value(encoded).expect("deserialize managed sessions");
    assert_eq!(decoded, payload);
}

#[test]
fn managed_session_contract_derives_lifecycle_action_and_events() {
    let data = ManagedSessionListData::from_active_runs_and_auth_status(
        [
            serde_json::json!({
                "timestamp": "2026-06-30T03:00:00Z",
                "run_id": "run-auth",
                "profile": "claude",
                "workspace": "/sensitive/workspaces/auth",
                "network": "provider",
                "status": "running",
                "container_name": "runhaven-claude-auth-run",
                "state_volume": "runhaven-claude-auth-home",
                "session": "auth",
                "token": "should-not-leak"
            }),
            serde_json::json!({
                "timestamp": "2026-06-30T02:00:00Z",
                "run_id": "run-ready",
                "profile": "codex",
                "workspace": "/sensitive/workspaces/ready",
                "network": "provider",
                "status": "running",
                "container_name": "runhaven-codex-ready-run",
                "state_volume": "runhaven-codex-ready-home",
                "session": "ready"
            }),
        ],
        serde_json::json!({
            "profiles": [
                {"name": "codex", "status": "api-key-broker"},
                {"name": "claude", "status": "unknown"}
            ]
        }),
    );

    let auth = &data.sessions[0];
    assert_eq!(auth.lifecycle, "running");
    assert_eq!(auth.next_action.kind, "auth_attention");
    assert_eq!(auth.next_action.label, "Finish auth setup");
    assert_eq!(auth.next_action.detail, "Auth is not ready for Claude.");

    let ready = &data.sessions[1];
    assert_eq!(ready.lifecycle, "running");
    assert_eq!(ready.next_action.kind, "monitor");
    assert_eq!(ready.next_action.label, "Monitor session");
    assert_eq!(
        ready
            .events
            .iter()
            .map(|event| event.source.as_str())
            .collect::<Vec<_>>(),
        ["run", "auth", "network"]
    );
    assert_eq!(ready.events[0].label, "Run record");
    assert_eq!(ready.events[0].detail, "Running");
    assert_eq!(ready.events[1].detail, "API key broker");
    assert_eq!(ready.events[2].detail, "Provider only");

    let encoded = serde_json::to_string(&data).expect("serialize managed sessions");
    for forbidden in [
        "/sensitive/workspaces",
        "should-not-leak",
        "state_volume",
        "runhaven-claude-auth-home",
        "runhaven-codex-ready-home",
    ] {
        assert!(
            !encoded.contains(forbidden),
            "managed session payload leaked forbidden value {forbidden:?}"
        );
    }
}
