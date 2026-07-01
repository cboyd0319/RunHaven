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
            "session": "project",
            "token": "should-not-leak"
        }),
    )
    .expect("active run payload");

    let response =
        crate::commands::sessions::get_managed_sessions().expect("managed sessions response");

    assert_eq!(response.sessions.len(), 1);
    let session = &response.sessions[0];
    assert_eq!(session.run_id, "managed-session-run");
    assert_eq!(session.agent, "codex");
    assert_eq!(session.project, "project");
    assert_eq!(session.status, "running");
    assert_eq!(session.auth_status, "api-key-broker");
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
