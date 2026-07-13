use super::*;

#[test]
fn reconcile_stale_sessions_flags_gone_containers_and_fails_safe() {
    use runhaven_core::ui_contracts::ManagedSessionListData;

    let record = |run_id: &str| {
        json!({
            "timestamp": "2026-07-01T05:00:00Z",
            "run_id": run_id,
            "profile": "codex",
            "workspace": format!("/sensitive/workspaces/{run_id}"),
            "network": "provider",
            "status": "running",
            "container_name": format!("runhaven-codex-{run_id}"),
            "state_volume": "runhaven-codex-hidden-home",
            "auth_scope": "project",
            "api_key_broker_env": "OPENAI_API_KEY",
            "session": run_id
        })
    };

    let mut sessions = ManagedSessionListData::from_active_runs_and_auth_status(
        [record("run-gone"), record("run-alive"), record("run-error")],
        json!({"profiles": [{"name": "codex", "status": "api-key-broker"}]}),
    );

    crate::commands::sessions::reconcile_stale_sessions(&mut sessions, &|run_id| match run_id {
        "run-gone" => Ok(true),
        "run-error" => Err("container service unavailable".to_string()),
        _ => Ok(false),
    });

    let action_kind = |id: &str| {
        sessions
            .sessions
            .iter()
            .find(|session| session.run_id == id)
            .expect("session")
            .next_action
            .kind
            .clone()
    };
    // Container proven gone -> recover. Container alive -> unchanged. Checker
    // error (service down / CLI missing) -> fail safe, session left as monitor.
    assert_eq!(action_kind("run-gone"), "recover");
    assert_eq!(action_kind("run-alive"), "monitor");
    assert_eq!(action_kind("run-error"), "monitor");
}

#[test]
fn managed_sessions_response_is_read_only_and_secret_free() {
    let (_cache, _cache_home) = isolated_cache();
    write_active_run_payload(
        "managed-session-run",
        json!({
            "timestamp": "2026-06-30T02:00:00Z",
            "run_id": "managed-session-run",
            "profile": "codex",
            "workspace": "/sensitive/workspaces/project",
            "network": "provider",
            "status": "running",
            "container_name": "runhaven-codex-managed-session",
            "state_volume": "runhaven-codex-secret-home",
            "auth_scope": "project",
            "session": "project",
            "token": "should-not-leak"
        }),
    )
    .expect("active run payload");

    let response = crate::commands::sessions::get_managed_sessions_for_test(None)
        .expect("managed sessions response");

    assert_eq!(response.sessions.len(), 1);
    let session = &response.sessions[0];
    assert_eq!(session.run_id, "managed-session-run");
    assert_eq!(session.agent, "codex");
    assert_eq!(session.project, "project");
    assert_eq!(session.status, "running");
    assert_eq!(session.auth_status, "login-state");
    assert_eq!(session.auth_scope, "project");
    assert_eq!(session.lifecycle, "running");
    assert_eq!(session.next_action.kind, "monitor");
    assert_eq!(
        session
            .events
            .iter()
            .map(|event| event.source.as_str())
            .collect::<Vec<_>>(),
        ["run", "auth", "network"]
    );

    let encoded = serde_json::to_string(&response).expect("serialize managed sessions");
    for forbidden in [
        "/sensitive/workspaces",
        "runhaven-codex-secret-home",
        "should-not-leak",
        "write_session_input",
        "attach_session",
    ] {
        assert!(
            !encoded.contains(forbidden),
            "managed sessions response leaked forbidden value {forbidden:?}"
        );
    }
}

#[test]
fn managed_sessions_report_input_transport_attachment() {
    let (_cache, _cache_home) = isolated_cache();
    write_active_run_payload(
        "managed-session-input-attached",
        json!({
            "timestamp": "2026-06-30T02:15:00Z",
            "run_id": "managed-session-input-attached",
            "profile": "codex",
            "workspace": "/sensitive/workspaces/project",
            "network": "provider",
            "status": "running",
            "container_name": "runhaven-codex-managed-session-input-attached",
            "state_volume": "runhaven-codex-secret-home",
            "api_key_broker_env": "OPENAI_API_KEY",
            "session": "project"
        }),
    )
    .expect("active run payload");
    let registry = SessionInputRegistry::default();

    let detached = crate::commands::sessions::get_managed_sessions_for_test(Some(&registry))
        .expect("detached managed sessions response");
    assert!(!detached.sessions[0].input.attached);
    assert!(detached.sessions[0].input.detail.contains("not attached"));

    registry
        .register_writer(
            "managed-session-input-attached",
            "managed-session-input-attached",
            |_| Ok(0),
        )
        .expect("register input transport");

    let attached = crate::commands::sessions::get_managed_sessions_for_test(Some(&registry))
        .expect("attached managed sessions response");
    assert!(attached.sessions[0].input.attached);
    assert!(attached.sessions[0].input.detail.contains("ready"));
}

#[test]
fn managed_sessions_do_not_report_input_ready_for_non_monitor_sessions() {
    let (_cache, _cache_home) = isolated_cache();
    write_active_run_payload(
        "managed-session-stop-requested",
        json!({
            "timestamp": "2026-07-01T06:00:00Z",
            "run_id": "managed-session-stop-requested",
            "profile": "codex",
            "workspace": "/sensitive/workspaces/project",
            "network": "provider",
            "status": "stop-requested",
            "container_name": "runhaven-codex-managed-session-stop-requested",
            "state_volume": "runhaven-codex-secret-home",
            "session": "project"
        }),
    )
    .expect("active run payload");
    let registry = SessionInputRegistry::default();
    registry
        .register_writer(
            "managed-session-stop-requested",
            "managed-session-stop-requested",
            |_| Ok(0),
        )
        .expect("register input transport");

    let response = crate::commands::sessions::get_managed_sessions_for_test(Some(&registry))
        .expect("managed sessions response");
    let session = &response.sessions[0];
    assert_eq!(session.lifecycle, "stop-requested");
    assert_eq!(session.next_action.kind, "review");
    assert!(!session.input.attached);
    assert!(session.input.detail.contains("not attached"));
}

#[test]
fn managed_sessions_include_recent_finished_records() {
    let (_cache, _cache_home) = isolated_cache();
    runhaven_core::support::paths::ensure_private_parent(
        &runhaven_core::support::paths::runs_log_path(),
    )
    .expect("runs log parent");
    let mut file =
        std::fs::File::create(runhaven_core::support::paths::runs_log_path()).expect("runs log");
    use std::io::Write;
    writeln!(
        file,
        "{}",
        json!({
            "timestamp": "2026-06-30T09:00:00Z",
            "started_at": "2026-06-30T08:00:00Z",
            "finished_at": "2026-06-30T09:00:00Z",
            "run_id": "managed-session-finished-record",
            "profile": "codex",
            "workspace": "/sensitive/workspaces/finished",
            "network": "provider",
            "status": "succeeded",
            "auth_scope": "project",
            "state_volume": "runhaven-codex-finished-home",
            "session": "finished",
            "token": "should-not-leak"
        })
    )
    .expect("write run record");

    let response = crate::commands::sessions::get_managed_sessions_for_test(None)
        .expect("managed sessions response");

    assert_eq!(response.sessions.len(), 1);
    let session = &response.sessions[0];
    assert_eq!(session.id, "managed-session-finished-record");
    assert_eq!(session.lifecycle, "finished");
    assert_eq!(session.next_action.kind, "none");
    assert!(!session.input.attached);

    let session_value = serde_json::to_value(session).expect("serialize managed session");
    assert_eq!(
        session_value.get("outputAvailable"),
        Some(&json!(false)),
        "a summary must state whether this process still retains terminal output"
    );

    let encoded = serde_json::to_string(&response).expect("serialize managed sessions");
    for forbidden in [
        "/sensitive/workspaces",
        "runhaven-codex-finished-home",
        "should-not-leak",
    ] {
        assert!(
            !encoded.contains(forbidden),
            "managed session history leaked forbidden value {forbidden:?}"
        );
    }
}
