use super::*;

#[test]
fn managed_session_status_resolves_session_id_and_sanitizes_container_status() {
    let (_cache, _cache_home) = isolated_cache();
    write_active_run_payload(
        "managed-session-status",
        json!({
            "timestamp": "2026-07-01T02:00:00Z",
            "run_id": "managed-session-status",
            "profile": "codex",
            "workspace": "/sensitive/workspaces/status",
            "network": "provider",
            "status": "running",
            "container_name": "runhaven-codex-status",
            "state_volume": "runhaven-codex-status-home",
            "auth_scope": "project",
            "api_key_broker_env": "OPENAI_API_KEY",
            "session": "status",
            "token": "should-not-leak"
        }),
    )
    .expect("active run payload");
    let registry = SessionInputRegistry::default();
    registry
        .register_writer("managed-session-status", "managed-session-status", |_| {
            Ok(0)
        })
        .expect("register input transport");

    let missing = crate::commands::sessions::managed_session_status_response_for_test(
        ManagedSessionStatusRequest {
            session_id: "missing-session".to_string(),
        },
        Some(&registry),
        None,
        false,
    )
    .expect_err("unknown session rejected");
    assert!(missing.contains("managed session not found"));

    let response = crate::commands::sessions::managed_session_status_response_for_test(
        ManagedSessionStatusRequest {
            session_id: "managed-session-status".to_string(),
        },
        Some(&registry),
        Some(json!({
            "active_run": {
                "timestamp": "2026-07-01T02:00:00Z",
                "run_id": "managed-session-status",
                "profile": "codex",
                "workspace": "/sensitive/workspaces/status",
                "network": "provider",
                "status": "running",
                "auth_scope": "project",
                "container_name": "runhaven-codex-status",
                "state_volume": "runhaven-codex-status-home",
                "session": "status"
            },
            "container": {
                "state": "running",
                "image": "runhaven/codex:0.1.0",
                "started_at": "2026-07-01T02:00:03Z",
                "resources": {
                    "cpus": 4,
                    "memory_in_bytes": 4294967296u64
                },
                "networks": [{
                    "network": "runhaven-managed-status",
                    "hostname": "runhaven-codex-status",
                    "ipv4_address": "192.168.64.21/24",
                    "ipv4_gateway": "192.168.64.1"
                }],
                "environment": ["OPENAI_API_KEY=fake-secret"],
                "mounts": [{"source": "/Users/example/private"}],
                "arguments": ["codex", "--secret-flag"]
            }
        })),
        false,
    )
    .expect("managed session status");

    assert_eq!(response.session.id, "managed-session-status");
    assert!(response.session.input.attached);
    assert_eq!(response.status_unavailable, None);
    let container = response.container.as_ref().expect("container status");
    assert_eq!(container.state, "running");
    assert_eq!(container.image.as_deref(), Some("runhaven/codex:0.1.0"));
    assert_eq!(container.resources.memory_bytes, Some(4_294_967_296));
    assert_eq!(
        container.networks[0].network.as_deref(),
        Some("runhaven-managed-status")
    );

    let encoded = serde_json::to_string(&response).expect("serialize status");
    for forbidden in [
        "/sensitive/workspaces",
        "runhaven-codex-status-home",
        "should-not-leak",
        "OPENAI_API_KEY",
        "fake-secret",
        "/Users/example",
        "secret-flag",
        "write_session_input",
        "attach_session",
        "resize_session",
    ] {
        assert!(
            !encoded.contains(forbidden),
            "managed session status leaked forbidden value {forbidden:?}"
        );
    }
}

#[test]
fn managed_session_status_reports_unavailable_without_leaking_backend_errors() {
    let (_cache, _cache_home) = isolated_cache();
    write_active_run_payload(
        "managed-session-status-unavailable",
        json!({
            "timestamp": "2026-07-01T02:30:00Z",
            "run_id": "managed-session-status-unavailable",
            "profile": "codex",
            "workspace": "/sensitive/workspaces/unavailable",
            "network": "provider",
            "status": "running",
            "container_name": "runhaven-codex-status-unavailable",
            "state_volume": "runhaven-codex-status-home",
            "session": "unavailable"
        }),
    )
    .expect("active run payload");

    let response = crate::commands::sessions::managed_session_status_response_for_test(
        ManagedSessionStatusRequest {
            session_id: "managed-session-status-unavailable".to_string(),
        },
        None,
        None,
        false,
    )
    .expect("managed session status fallback");

    assert!(response.container.is_none());
    assert_eq!(
        response.status_unavailable.as_deref(),
        Some("Container status is unavailable. The run may have stopped.")
    );
    let recovery = response.recovery.as_ref().expect("recovery guidance");
    assert_eq!(recovery.status, "attention");
    assert_eq!(recovery.title, "Runtime status unavailable");
    assert!(recovery.remedy.contains("Refresh sessions"));
    assert!(!recovery.remedy.contains("Repair marker"));
    assert!(!recovery.repair_available);
    let encoded = serde_json::to_string(&response).expect("serialize status");
    assert!(!encoded.contains("/sensitive/workspaces"));
    assert!(!encoded.contains("runhaven-codex-status-home"));
    assert!(!encoded.contains("container inspect"));
}

#[test]
fn managed_session_status_marks_repair_available_for_missing_container() {
    let (_cache, _cache_home) = isolated_cache();
    write_active_run_payload(
        "managed-session-status-repair",
        json!({
            "timestamp": "2026-07-01T02:45:00Z",
            "run_id": "managed-session-status-repair",
            "profile": "codex",
            "workspace": "/sensitive/workspaces/repair",
            "network": "provider",
            "status": "running",
            "container_name": "runhaven-codex-status-repair",
            "state_volume": "runhaven-codex-status-home",
            "session": "repair"
        }),
    )
    .expect("active run payload");

    let response = crate::commands::sessions::managed_session_status_response_for_test(
        ManagedSessionStatusRequest {
            session_id: "managed-session-status-repair".to_string(),
        },
        None,
        None,
        true,
    )
    .expect("managed session status repair");

    assert!(response.container.is_none());
    let recovery = response.recovery.as_ref().expect("recovery guidance");
    assert_eq!(recovery.status, "warning");
    assert_eq!(recovery.title, "Repair stale session marker");
    assert!(recovery.detail.contains("container no longer exists"));
    assert!(recovery.remedy.contains("repair the marker"));
    assert!(recovery.repair_available);
}

#[test]
fn managed_session_status_detaches_input_when_runtime_status_is_unavailable() {
    let (_cache, _cache_home) = isolated_cache();
    write_active_run_payload(
        "managed-session-status-detached",
        json!({
            "timestamp": "2026-07-01T02:50:00Z",
            "run_id": "managed-session-status-detached",
            "profile": "codex",
            "workspace": "/sensitive/workspaces/status",
            "network": "provider",
            "status": "running",
            "container_name": "runhaven-codex-status-detached",
            "state_volume": "runhaven-codex-status-home",
            "session": "status"
        }),
    )
    .expect("active run payload");
    let registry = SessionInputRegistry::default();
    registry
        .register_writer(
            "managed-session-status-detached",
            "managed-session-status-detached",
            |_| Ok(0),
        )
        .expect("register input transport");

    let response = crate::commands::sessions::managed_session_status_response_for_test(
        ManagedSessionStatusRequest {
            session_id: "managed-session-status-detached".to_string(),
        },
        Some(&registry),
        None,
        true,
    )
    .expect("managed session status");

    assert!(response.container.is_none());
    assert!(!response.session.input.attached);
    assert!(response.session.input.detail.contains("not attached"));
    assert_eq!(
        response.status_unavailable.as_deref(),
        Some("Container status is unavailable. The run may have stopped.")
    );
    assert!(
        response
            .recovery
            .as_ref()
            .expect("recovery guidance")
            .repair_available
    );
}
