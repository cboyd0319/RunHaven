use super::*;

#[test]
fn managed_output_confirmation_omits_internal_session_id() {
    let message = crate::commands::managed_session_output::trusted_session_output_message(
        "Codex - project",
        "Failed",
        200,
    );

    assert!(!message.contains("Session ID"));
    assert!(message.contains("Session: Codex - project"));
    assert!(message.contains("Status: Failed"));
}

fn write_past_output_session(run_id: &str) {
    write_history_run_record(json!({
        "timestamp": "2026-07-01T05:00:00Z",
        "started_at": "2026-07-01T04:00:00Z",
        "finished_at": "2026-07-01T05:00:00Z",
        "run_id": run_id,
        "profile": "codex",
        "workspace": "/sensitive/workspaces/past-output",
        "network": "provider",
        "status": "succeeded",
        "auth_scope": "project",
        "session": "past-output"
    }));
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
    assert_eq!(response.snapshot.source, "container-stdio");
    assert_eq!(response.snapshot.text, "entry one\nentry two\n");
    assert_eq!(response.snapshot.returned_lines, 2);
    assert!(!response.snapshot.captured_at.is_empty());
    // The managed path adds the redaction notice to the shared warnings list.
    assert!(
        response
            .snapshot
            .warnings
            .iter()
            .any(|warning| warning.contains("removed automatically"))
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

// A running session whose output was streamed into the in-memory registry is
// served from the registry (instant), not the `container logs` CLI. The record
// still carries sensitive fields; the response must not leak them, proving the
// registry path redacts like the container path.
#[test]
fn managed_session_log_snapshot_serves_streamed_output_registry_first() {
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
    output_registry
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

// Locks the redaction-parity contract on the registry-first path: a token that
// an agent printed into its captured output must come back scrubbed through the
// in-memory registry, so no future non-redacting registry read can silently
// reintroduce the leak this path exists to prevent. End-to-end complement to the
// choke-point argument (both paths run redact_run_output).
#[test]
fn managed_session_log_snapshot_redacts_tokens_served_from_the_registry() {
    let (_cache, _cache_home) = isolated_cache();
    write_active_run_payload(
        "managed-session-registry-redact",
        json!({
            "timestamp": "2026-06-30T03:30:00Z",
            "run_id": "managed-session-registry-redact",
            "profile": "codex",
            "workspace": "/w",
            "network": "provider",
            "status": "running",
            "container_name": "runhaven-codex-redact",
            "state_volume": "runhaven-codex-hidden-home",
            "session": "redact"
        }),
    )
    .expect("active run payload");
    let output_registry = SessionOutputRegistry::default();
    output_registry
        .register("managed-session-registry-redact")
        .expect("register output");
    output_registry
        .append(
            "managed-session-registry-redact",
            b"agent printed sk-ant-oat01-SHOULDNOTLEAK0123456789abcdef then exited\n",
        )
        .expect("append output");

    let response =
        crate::commands::sessions::get_managed_session_log_snapshot_with_output_for_test(
            ManagedSessionLogSnapshotRequest {
                session_id: "managed-session-registry-redact".to_string(),
                lines: Some(20),
                confirm_sensitive_output: true,
            },
            &output_registry,
        )
        .expect("managed session output snapshot");

    assert!(
        !response
            .snapshot
            .text
            .contains("sk-ant-oat01-SHOULDNOTLEAK0123456789abcdef"),
        "registry-served snapshot leaked a token: {}",
        response.snapshot.text
    );
    // Still served (redacted), proving the registry path answered, not an error.
    assert!(response.snapshot.text.contains("agent printed"));
    assert!(response.snapshot.text.contains("[redacted]"));
}

#[test]
fn past_session_log_snapshot_still_serves_retained_registry_output() {
    let (_cache, _cache_home) = isolated_cache();
    write_past_output_session("managed-session-past-retained");
    let output_registry = SessionOutputRegistry::default();
    output_registry
        .register("managed-session-past-retained")
        .expect("register past output");
    output_registry
        .append("managed-session-past-retained", b"retained past output\n")
        .expect("append past output");

    let response =
        crate::commands::sessions::get_managed_session_log_snapshot_with_output_for_test(
            ManagedSessionLogSnapshotRequest {
                session_id: "managed-session-past-retained".to_string(),
                lines: Some(20),
                confirm_sensitive_output: true,
            },
            &output_registry,
        )
        .expect("retained past output");

    assert_eq!(response.session.lifecycle, "finished");
    assert_eq!(response.snapshot.text, "retained past output\n");
}

#[test]
fn past_session_without_retained_output_returns_a_fixed_id_free_error() {
    let (_cache, _cache_home) = isolated_cache();
    let run_id = "managed-session-past-missing-output";
    write_past_output_session(run_id);

    let error = crate::commands::sessions::get_managed_session_log_snapshot_for_test(
        ManagedSessionLogSnapshotRequest {
            session_id: run_id.to_string(),
            lines: Some(20),
            confirm_sensitive_output: true,
        },
    )
    .expect_err("past output was not retained");

    assert_eq!(error, "Output is no longer available for this past run.");
    assert!(!error.contains(run_id));
    assert!(!error.contains("active run"));
}

#[test]
fn active_session_without_retained_output_uses_the_active_fallback() {
    let (_cache, _cache_home) = isolated_cache();
    let run_id = "managed-session-active-fallback";
    write_active_run_payload(
        run_id,
        json!({
            "timestamp": "2026-07-01T05:00:00Z",
            "run_id": run_id,
            "profile": "codex",
            "workspace": "/sensitive/workspaces/active-output",
            "network": "provider",
            "status": "running",
            "container_name": "runhaven-codex-active-fallback",
            "state_volume": "runhaven-codex-hidden-home",
            "session": "active-output"
        }),
    )
    .expect("active run payload");
    let registry = SessionOutputRegistry::default();
    let mut approved = false;
    let mut fallback_called = false;

    let response = crate::commands::managed_session_output::get_managed_session_log_snapshot_with_fallback_for_test(
        ManagedSessionLogSnapshotRequest {
            session_id: run_id.to_string(),
            lines: Some(20),
            confirm_sensitive_output: true,
        },
        Some(&registry),
        |session, lines| {
            approved = true;
            assert_eq!(session.lifecycle, "running");
            assert_eq!(lines, 20);
            Ok(true)
        },
        |fallback_run_id, lines| {
            fallback_called = true;
            assert_eq!(fallback_run_id, run_id);
            crate::commands::log_snapshot::log_snapshot_response(
                fallback_run_id,
                lines,
                b"active fallback output\n",
                1024,
            )
        },
    )
    .expect("active fallback output");

    assert!(approved);
    assert!(fallback_called);
    assert_eq!(response.snapshot.text, "active fallback output\n");
}
