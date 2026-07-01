use super::*;

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
    assert_eq!(session.auth_status, "api-key-broker");
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

#[test]
fn managed_session_log_snapshot_resolves_session_id_and_stays_read_only() {
    let (_cache, _cache_home) = isolated_cache();
    write_active_run_payload(
        "managed-session-log",
        json!({
            "timestamp": "2026-06-30T03:00:00Z",
            "run_id": "managed-session-log",
            "profile": "codex",
            "workspace": "/sensitive/workspaces/project",
            "network": "provider",
            "status": "running",
            "container_name": "runhaven-codex-managed-session-log",
            "state_volume": "runhaven-codex-hidden-home",
            "session": "project",
            "token": "should-not-leak"
        }),
    )
    .expect("active run payload");

    let missing = crate::commands::sessions::managed_session_log_snapshot_response_for_test(
        ManagedSessionLogSnapshotRequest {
            session_id: "missing-session".to_string(),
            lines: Some(20),
            confirm_sensitive_output: true,
        },
        b"",
        1024,
    )
    .expect_err("unknown session rejected");

    assert!(missing.contains("managed session not found"));

    let response = crate::commands::sessions::managed_session_log_snapshot_response_for_test(
        ManagedSessionLogSnapshotRequest {
            session_id: "managed-session-log".to_string(),
            lines: Some(20),
            confirm_sensitive_output: true,
        },
        b"entry one\nentry two\n",
        1024,
    )
    .expect("managed session log snapshot");

    assert_eq!(response.session.id, "managed-session-log");
    assert_eq!(response.session.project, "project");
    assert_eq!(response.snapshot.run_id, "managed-session-log");
    assert_eq!(response.snapshot.requested_lines, 20);
    assert_eq!(response.snapshot.entries.len(), 1);
    assert_eq!(response.snapshot.entries[0].text, "entry one\nentry two\n");
    assert_eq!(response.cursor, response.snapshot.cursor);
    assert_eq!(response.entries.len(), 1);
    let session_entry = &response.entries[0];
    assert_eq!(session_entry.session_id, "managed-session-log");
    assert_eq!(session_entry.run_id, "managed-session-log");
    assert_eq!(session_entry.source, "container-stdio");
    assert_eq!(session_entry.cursor, response.snapshot.cursor);
    assert_eq!(session_entry.timestamp, response.snapshot.captured_at);
    assert_eq!(session_entry.text, "entry one\nentry two\n");
    assert_eq!(session_entry.line_count, 2);
    assert_eq!(session_entry.redaction.status, "notApplied");
    assert!(
        session_entry
            .redaction
            .message
            .contains("not automatically redacted")
    );

    let encoded = serde_json::to_string(&response).expect("serialize response");
    for forbidden in [
        "/sensitive/workspaces",
        "runhaven-codex-hidden-home",
        "should-not-leak",
        "write_session_input",
        "attach_session",
        "resize_session",
    ] {
        assert!(
            !encoded.contains(forbidden),
            "managed session log snapshot leaked forbidden value {forbidden:?}"
        );
    }
}

#[test]
fn managed_session_log_snapshot_requires_trusted_raw_output_approval() {
    let (_cache, _cache_home) = isolated_cache();
    write_active_run_payload(
        "managed-session-log-denied",
        json!({
            "timestamp": "2026-06-30T03:10:00Z",
            "run_id": "managed-session-log-denied",
            "profile": "codex",
            "workspace": "/sensitive/workspaces/project",
            "network": "provider",
            "status": "running",
            "container_name": "runhaven-codex-managed-session-log-denied",
            "state_volume": "runhaven-codex-hidden-home",
            "session": "project"
        }),
    )
    .expect("active run payload");

    let error = crate::commands::sessions::get_managed_session_log_snapshot_with_approval_for_test(
        ManagedSessionLogSnapshotRequest {
            session_id: "managed-session-log-denied".to_string(),
            lines: Some(20),
            confirm_sensitive_output: true,
        },
        None,
        |session, lines| {
            assert_eq!(session.id, "managed-session-log-denied");
            assert_eq!(lines, 20);
            Ok(false)
        },
    )
    .expect_err("trusted denial should stop managed output read");

    assert_eq!(error, "Raw output view canceled.");
}

#[test]
fn managed_session_log_snapshot_falls_back_to_desktop_output_registry() {
    let (_cache, _cache_home) = isolated_cache();
    write_active_run_payload(
        "managed-session-output-registry",
        json!({
            "timestamp": "2026-06-30T03:30:00Z",
            "run_id": "managed-session-output-registry",
            "profile": "codex",
            "workspace": "/sensitive/workspaces/output",
            "network": "provider",
            "status": "running",
            "container_name": "runhaven-codex-output-registry",
            "state_volume": "runhaven-codex-hidden-home",
            "session": "output",
            "token": "should-not-leak"
        }),
    )
    .expect("active run payload");
    let output_registry = SessionOutputRegistry::default();
    let _registration = output_registry
        .register("managed-session-output-registry")
        .expect("register output");
    output_registry
        .append(
            "managed-session-output-registry",
            b"desktop output marker\n",
        )
        .expect("append output");

    let response =
        crate::commands::sessions::get_managed_session_log_snapshot_with_output_for_test(
            ManagedSessionLogSnapshotRequest {
                session_id: "managed-session-output-registry".to_string(),
                lines: Some(20),
                confirm_sensitive_output: true,
            },
            &output_registry,
        )
        .expect("managed session output snapshot");

    assert_eq!(response.session.id, "managed-session-output-registry");
    assert_eq!(response.snapshot.run_id, "managed-session-output-registry");
    assert!(response.snapshot.text.contains("desktop output marker"));

    let encoded = serde_json::to_string(&response).expect("serialize output snapshot");
    for forbidden in [
        "/sensitive/workspaces",
        "runhaven-codex-hidden-home",
        "should-not-leak",
    ] {
        assert!(
            !encoded.contains(forbidden),
            "managed session output snapshot leaked forbidden value {forbidden:?}"
        );
    }
}
