use super::*;

fn run_record(timestamp: &str, run_id: &str, profile: &str, session: &str) -> serde_json::Value {
    serde_json::json!({
        "timestamp": timestamp,
        "run_id": run_id,
        "profile": profile,
        "workspace": format!("/sensitive/workspaces/{session}"),
        "network": "provider",
        "status": "running",
        "container_name": format!("runhaven-{profile}-{session}-run"),
        "state_volume": format!("runhaven-{profile}-{session}-home"),
        "auth_scope": "project",
        "session": session
    })
}

#[test]
fn managed_session_list_contract_uses_run_auth_without_leaking_static_payload() {
    let mut newer = run_record("2026-06-30T02:00:00Z", "run-newer", "claude", "backend");
    newer["auth_scope"] = serde_json::json!("agent");

    let data = ManagedSessionListData::from_active_runs_and_auth_status(
        [
            run_record("2026-06-30T01:00:00Z", "run-older", "codex", "docs"),
            newer,
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
    assert_eq!(first.auth_status, "login-state");
    assert_eq!(first.auth_scope, "agent");
    assert_eq!(first.next_action.kind, "auth_attention");
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
fn managed_session_list_includes_recent_finished_records_without_overriding_active_runs() {
    let data = ManagedSessionListData::from_active_and_recent_runs(
        [run_record(
            "2026-06-30T02:00:00Z",
            "run-still-active",
            "codex",
            "active",
        )],
        [
            serde_json::json!({
                "timestamp": "2026-06-30T04:00:00Z",
                "started_at": "2026-06-30T02:00:00Z",
                "finished_at": "2026-06-30T04:00:00Z",
                "run_id": "run-still-active",
                "profile": "codex",
                "workspace": "/sensitive/workspaces/active",
                "network": "provider",
                "status": "failed",
                "auth_scope": "project",
                "state_volume": "runhaven-codex-record-home",
                "session": "active"
            }),
            serde_json::json!({
                "timestamp": "2026-06-30T03:00:00Z",
                "started_at": "2026-06-30T01:00:00Z",
                "finished_at": "2026-06-30T03:00:00Z",
                "run_id": "run-finished",
                "profile": "claude",
                "workspace": "/sensitive/workspaces/finished",
                "network": "provider",
                "status": "succeeded",
                "auth_scope": "agent",
                "state_volume": "runhaven-claude-record-home",
                "session": "finished"
            }),
        ],
        serde_json::json!({
            "profiles": [
                {"name": "codex", "status": "api-key-broker"},
                {"name": "claude", "status": "unknown"}
            ]
        }),
    );

    assert_eq!(data.sessions.len(), 2);
    let active = data
        .sessions
        .iter()
        .find(|session| session.run_id == "run-still-active")
        .expect("active session");
    assert_eq!(active.status, "running");
    assert_eq!(active.container_name, "runhaven-codex-active-run");
    assert_eq!(active.auth_status, "login-state");
    assert_eq!(active.next_action.kind, "auth_attention");

    let finished = data
        .sessions
        .iter()
        .find(|session| session.run_id == "run-finished")
        .expect("finished session");
    assert_eq!(finished.status, "succeeded");
    assert_eq!(finished.lifecycle, "finished");
    assert_eq!(finished.started_at, "2026-06-30T01:00:00Z");
    assert_eq!(finished.container_name, "-");
    assert!(!finished.input.attached);
    assert_eq!(finished.next_action.kind, "none");

    let encoded = serde_json::to_string(&data).expect("serialize managed sessions");
    for forbidden in [
        "/sensitive/workspaces",
        "runhaven-codex-record-home",
        "runhaven-claude-record-home",
    ] {
        assert!(
            !encoded.contains(forbidden),
            "managed session payload leaked forbidden value {forbidden:?}"
        );
    }
}

#[test]
fn managed_session_auth_uses_run_owned_broker_proof_before_static_profile_status() {
    let mut brokered = run_record("2026-07-01T01:00:00Z", "run-brokered", "codex", "brokered");
    brokered["api_key_broker_env"] = serde_json::json!("OPENAI_API_KEY");

    let data = ManagedSessionListData::from_active_runs_and_auth_status(
        [
            run_record("2026-07-01T02:00:00Z", "run-login-state", "codex", "login"),
            brokered,
        ],
        serde_json::json!({
            "profiles": [{"name": "codex", "status": "api-key-broker"}],
            "token": "should-not-leak"
        }),
    );

    let login_state = data
        .sessions
        .iter()
        .find(|session| session.run_id == "run-login-state")
        .expect("login-state session");
    assert_eq!(login_state.auth_status, "login-state");
    assert_eq!(login_state.next_action.kind, "auth_attention");

    let brokered = data
        .sessions
        .iter()
        .find(|session| session.run_id == "run-brokered")
        .expect("brokered session");
    assert_eq!(brokered.auth_status, "api-key-broker");
    assert_eq!(brokered.next_action.kind, "monitor");

    let encoded = serde_json::to_string(&data).expect("serialize managed sessions");
    for forbidden in [
        "OPENAI_API_KEY",
        "/sensitive/workspaces",
        "runhaven-codex-login-home",
        "runhaven-codex-brokered-home",
        "should-not-leak",
    ] {
        assert!(
            !encoded.contains(forbidden),
            "managed session payload leaked forbidden value {forbidden:?}"
        );
    }
}

#[test]
fn managed_session_auth_marks_shell_no_auth_needed() {
    let mut record = run_record("2026-07-01T03:00:00Z", "run-shell", "shell", "shell");
    record["network"] = serde_json::json!("internet");

    let data = ManagedSessionListData::from_active_runs_and_auth_status(
        [record],
        serde_json::json!({"profiles": [{"name": "shell", "status": "n/a"}]}),
    );

    let session = data.sessions.first().expect("shell session");
    assert_eq!(session.auth_status, "no-auth-needed");
    assert_eq!(session.next_action.kind, "monitor");
    assert_eq!(session.events[1].detail, "No auth needed");
}

#[test]
fn managed_session_finished_run_proves_broker_from_history_auth_broker_channel() {
    // Finished runs drop out of the active-run store, so their broker proof
    // arrives through the history record's `auth_broker.broker` field instead of
    // `api_key_broker_env`. This locks that second channel against refactors of
    // `summarize_auth_broker` silently downgrading finished brokered sessions.
    let finished_record = |run_id: &str, session: &str| {
        serde_json::json!({
            "timestamp": "2026-07-01T04:00:00Z",
            "started_at": "2026-07-01T02:00:00Z",
            "finished_at": "2026-07-01T04:00:00Z",
            "run_id": run_id,
            "profile": "codex",
            "workspace": format!("/sensitive/workspaces/{session}"),
            "network": "provider",
            "status": "succeeded",
            "auth_scope": "project",
            "state_volume": format!("runhaven-codex-{session}-home"),
            "session": session
        })
    };

    let mut brokered = finished_record("run-history-brokered", "brokered");
    brokered["auth_broker"] = serde_json::json!({"broker": "codex-api-key", "entries": 0});
    let mut sign_in = finished_record("run-history-sign-in", "signin");
    sign_in["auth_broker"] = serde_json::json!({"broker": null, "entries": 0});

    let data = ManagedSessionListData::from_active_and_recent_runs(
        [],
        [brokered, sign_in],
        serde_json::json!({"profiles": [{"name": "codex", "status": "api-key-broker"}]}),
    );

    let brokered = data
        .sessions
        .iter()
        .find(|session| session.run_id == "run-history-brokered")
        .expect("brokered finished session");
    assert_eq!(brokered.auth_status, "api-key-broker");

    let sign_in = data
        .sessions
        .iter()
        .find(|session| session.run_id == "run-history-sign-in")
        .expect("sign-in finished session");
    assert_eq!(sign_in.auth_status, "login-state");
}

#[test]
fn managed_session_component_payload_round_trips_json() {
    let data = ManagedSessionListData::from_active_runs_and_auth_status(
        [run_record(
            "2026-06-30T02:00:00Z",
            "run-123",
            "codex",
            "project",
        )],
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
    let mut auth = run_record("2026-06-30T03:00:00Z", "run-auth", "claude", "auth");
    auth["token"] = serde_json::json!("should-not-leak");
    let mut ready = run_record("2026-06-30T02:00:00Z", "run-ready", "codex", "ready");
    ready["api_key_broker_env"] = serde_json::json!("OPENAI_API_KEY");

    let data = ManagedSessionListData::from_active_runs_and_auth_status(
        [auth, ready],
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
        "OPENAI_API_KEY",
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
