use super::*;

pub(super) fn run_record(
    timestamp: &str,
    run_id: &str,
    profile: &str,
    session: &str,
) -> serde_json::Value {
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
    assert_eq!(first.next_action.kind, "monitor");
    assert_eq!(first.events[1].detail, "Session sign-in");
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
fn session_title_renders_without_bidi_override_characters() {
    // The title becomes a native monitor window title, so a workspace
    // basename carrying a bidi override (U+202E) must never survive into it.
    let mut record = run_record("2026-06-30T02:00:00Z", "run-bidi", "codex", "backend");
    record["workspace"] = serde_json::json!("/sensitive/workspaces/pro\u{202E}txt.gpj");

    let data = ManagedSessionListData::from_active_runs_and_auth_status(
        [record],
        serde_json::json!({"profiles": []}),
    );

    let session = &data.sessions[0];
    assert_eq!(session.title, "Codex - protxt.gpj");
    assert!(!session.title.contains('\u{202E}'));
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
    assert_eq!(active.next_action.kind, "monitor");

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
fn managed_session_ended_run_is_neutral_terminal_not_a_failure() {
    // An interactive TTY run the user ended by closing it records status
    // "ended". It must render as a neutral terminal state: "Ended" label, no
    // action needed, and worktree cleanup allowed like other terminal states.
    let ended = serde_json::json!({
        "timestamp": "2026-07-01T05:00:00Z",
        "started_at": "2026-07-01T04:00:00Z",
        "finished_at": "2026-07-01T05:00:00Z",
        "run_id": "run-ended",
        "profile": "claude",
        "workspace": "/sensitive/workspaces/ended",
        "network": "provider",
        "status": "ended",
        "auth_scope": "agent",
        "session": "ended"
    });

    let data = ManagedSessionListData::from_active_and_recent_runs(
        [],
        [ended],
        serde_json::json!({"profiles": [{"name": "claude", "status": "unknown"}]}),
    );

    let session = data
        .sessions
        .iter()
        .find(|session| session.run_id == "run-ended")
        .expect("ended session");
    assert_eq!(session.status, "ended");
    assert_eq!(session.status_label, "Ended");
    assert_eq!(session.lifecycle, "ended");
    assert_eq!(session.next_action.kind, "none");
    assert_eq!(session.next_action.label, "No action needed");
}

#[test]
fn failed_session_reason_uses_only_sanitized_saved_evidence() {
    let failed_record = |run_id: &str, return_code: Option<i32>, provider_denied, auth_denied| {
        let mut record = serde_json::json!({
            "timestamp": "2026-07-01T05:00:00Z",
            "started_at": "2026-07-01T04:00:00Z",
            "finished_at": "2026-07-01T05:00:00Z",
            "run_id": run_id,
            "profile": "codex",
            "workspace": "/sensitive/workspaces/failed-project",
            "network": "provider",
            "status": "failed",
            "auth_scope": "project",
            "session": "failed-project",
            "provider_policy": {
                "denied": provider_denied,
                "reason": "secret provider detail from /Users/person/private"
            },
            "auth_broker": {
                "denied": auth_denied,
                "reason": "secret auth detail sk-test-should-not-leak"
            },
            "error": "raw agent output should-not-leak"
        });
        if let Some(code) = return_code {
            record["return_code"] = serde_json::json!(code);
        }
        record
    };
    let data = ManagedSessionListData::from_active_and_recent_runs(
        [],
        [
            failed_record("failed-auth", Some(1), 3, 1),
            failed_record("failed-network", Some(1), 2, 0),
            failed_record("failed-process", Some(17), 0, 0),
            failed_record("failed-unknown", None, 0, 0),
        ],
        serde_json::json!({"profiles": []}),
    );
    let detail = |run_id: &str| {
        data.sessions
            .iter()
            .find(|session| session.run_id == run_id)
            .expect("failed session")
            .next_action
            .detail
            .as_str()
    };

    assert_eq!(
        detail("failed-auth"),
        "An authentication request was blocked. Review this session's Sign-in details, then try again."
    );
    assert_eq!(
        detail("failed-network"),
        "A network request was blocked. Review this session's Network details, then try again."
    );
    assert_eq!(
        detail("failed-process"),
        "The assistant exited with code 17 before it finished. Review this session's details below, then start a new session."
    );
    assert_eq!(
        detail("failed-unknown"),
        "RunHaven couldn’t determine why this session ended. Review this session's details below, then start a new session."
    );

    let encoded = serde_json::to_string(&data).expect("serialize sessions");
    for forbidden in [
        "/sensitive/workspaces",
        "/Users/person/private",
        "sk-test-should-not-leak",
        "raw agent output",
    ] {
        assert!(!encoded.contains(forbidden), "leaked {forbidden:?}");
    }
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
    assert_eq!(auth.next_action.kind, "monitor");
    assert_eq!(auth.next_action.label, "Session running");
    assert_eq!(auth.events[1].detail, "Session sign-in");

    let ready = &data.sessions[1];
    assert_eq!(ready.lifecycle, "running");
    assert_eq!(ready.next_action.kind, "monitor");
    assert_eq!(ready.next_action.label, "Session running");
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
    assert_eq!(ready.events[1].label, "Sign-in");
    assert_eq!(ready.events[1].detail, "API key from this Mac");
    assert_eq!(ready.events[2].detail, "Assistant services only");

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
