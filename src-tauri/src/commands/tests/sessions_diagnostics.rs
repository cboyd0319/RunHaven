use super::*;
use runhaven_core::ui_contracts::{
    ManagedSessionCleanupData, ManagedSessionEventData, ManagedSessionInputData,
    ManagedSessionNextActionData, ManagedSessionSummaryData,
};

#[test]
fn managed_session_diagnostics_are_scoped_to_selected_session() {
    use runhaven_core::support::paths::{
        auth_broker_log_path, egress_policy_log_path, ensure_private_parent,
    };
    use std::io::Write;

    let (_cache, _cache_home) = isolated_cache();
    write_active_run_payload(
        "managed-session-diagnostics",
        json!({
            "timestamp": "2026-07-01T03:00:00Z",
            "run_id": "managed-session-diagnostics",
            "profile": "codex",
            "workspace": "/sensitive/workspaces/diagnostics",
            "network": "provider",
            "status": "running",
            "container_name": "runhaven-codex-diagnostics",
            "state_volume": "runhaven-codex-diagnostics-home",
            "auth_scope": "project",
            "session": "diagnostics"
        }),
    )
    .expect("active run payload");

    ensure_private_parent(&auth_broker_log_path()).expect("auth log parent");
    let mut auth_log = std::fs::File::create(auth_broker_log_path()).expect("auth log");
    writeln!(
        auth_log,
        "{}",
        json!({
            "timestamp": "2026-07-01T03:00:01Z",
            "run_id": "managed-session-diagnostics",
            "profile": "codex",
            "workspace": "/sensitive/workspaces/diagnostics",
            "network": "provider",
            "broker": "codex-api-key",
            "method": "POST",
            "path": "/v1/responses?token=secret#fragment",
            "decision": "allowed",
            "reason": "ok",
            "upstream_status": 200,
            "count": 1
        })
    )
    .expect("write selected auth log");
    writeln!(
        auth_log,
        "{}",
        json!({
            "timestamp": "2026-07-01T03:00:02Z",
            "run_id": "other-run",
            "profile": "codex",
            "workspace": "/sensitive/workspaces/other",
            "broker": "codex-api-key",
            "method": "POST",
            "path": "/v1/other",
            "decision": "allowed",
            "reason": "other",
            "upstream_status": 200,
            "count": 1
        })
    )
    .expect("write other auth log");

    ensure_private_parent(&egress_policy_log_path()).expect("egress log parent");
    let mut egress_log = std::fs::File::create(egress_policy_log_path()).expect("egress log");
    writeln!(
        egress_log,
        "{}",
        json!({
            "timestamp": "2026-07-01T03:00:03Z",
            "run_id": "managed-session-diagnostics",
            "profile": "codex",
            "workspace": "/sensitive/workspaces/diagnostics",
            "network": "provider",
            "host": "api.openai.com",
            "port": 443,
            "decision": "allowed",
            "reason": "provider-host",
            "matched_rule": "api.openai.com",
            "count": 2
        })
    )
    .expect("write selected egress log");
    writeln!(
        egress_log,
        "{}",
        json!({
            "timestamp": "2026-07-01T03:00:04Z",
            "run_id": "other-run",
            "profile": "codex",
            "workspace": "/sensitive/workspaces/other",
            "network": "provider",
            "host": "other.example",
            "port": 443,
            "decision": "denied",
            "reason": "other",
            "matched_rule": "",
            "count": 1
        })
    )
    .expect("write other egress log");

    let missing = crate::commands::session_diagnostics::get_managed_session_diagnostics_for_test(
        ManagedSessionDiagnosticsRequest {
            session_id: "missing-session".to_string(),
        },
    )
    .expect_err("unknown session rejected");
    assert!(missing.contains("managed session not found"));

    let response = crate::commands::session_diagnostics::get_managed_session_diagnostics_for_test(
        ManagedSessionDiagnosticsRequest {
            session_id: "managed-session-diagnostics".to_string(),
        },
    )
    .expect("managed session diagnostics");

    assert_eq!(response.session.id, "managed-session-diagnostics");
    assert_eq!(response.auth_status.name, "codex");
    assert_eq!(response.auth_status.status, "login-state");
    assert_eq!(response.auth_status.status, response.session.auth_status);
    assert_eq!(response.auth_log_entries.len(), 1);
    assert_eq!(response.auth_log_entries[0].path, "/v1/responses");
    assert_eq!(response.egress_log_entries.len(), 1);
    assert_eq!(response.egress_log_entries[0].host, "api.openai.com");

    let encoded = serde_json::to_string(&response).expect("serialize diagnostics");
    for forbidden in [
        "/sensitive/workspaces",
        "runhaven-codex-diagnostics-home",
        "token=secret",
        "other.example",
        "other-run",
        "container inspect",
        "write_session_input",
        "attach_session",
        "resize_session",
    ] {
        assert!(
            !encoded.contains(forbidden),
            "managed session diagnostics leaked forbidden value {forbidden:?}"
        );
    }
}

#[test]
fn managed_session_diagnostics_preserve_session_auth_truth() {
    let session = managed_session_summary_with_auth_status("login-state");

    let auth_status = crate::commands::session_diagnostics::session_auth_status_for_test(&session);

    assert_eq!(auth_status.name, "codex");
    assert_eq!(auth_status.status, "login-state");
}

fn managed_session_summary_with_auth_status(auth_status: &str) -> ManagedSessionSummaryData {
    ManagedSessionSummaryData {
        id: "session-auth-truth".to_string(),
        run_id: "session-auth-truth".to_string(),
        agent: "codex".to_string(),
        session: "login".to_string(),
        title: "Codex - project".to_string(),
        project: "project".to_string(),
        status: "running".to_string(),
        status_label: "Running".to_string(),
        auth_status: auth_status.to_string(),
        auth_scope: "agent".to_string(),
        network: "provider".to_string(),
        started_at: "2026-07-01T03:30:00Z".to_string(),
        container_name: "runhaven-codex-session-auth-truth".to_string(),
        detail_command: "runhaven runs show session-auth-truth".to_string(),
        lifecycle: "running".to_string(),
        input: ManagedSessionInputData {
            attached: false,
            status: "detached".to_string(),
            detail: "Session input is not attached in this RunHaven window.".to_string(),
        },
        cleanup: ManagedSessionCleanupData {
            available: false,
            review_state: "none".to_string(),
            target: String::new(),
            label: String::new(),
            detail: String::new(),
        },
        next_action: ManagedSessionNextActionData {
            kind: "auth_attention".to_string(),
            label: "Finish sign-in".to_string(),
            detail: "Sign-in is not ready for Codex.".to_string(),
        },
        events: Vec::<ManagedSessionEventData>::new(),
    }
}
