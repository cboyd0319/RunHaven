use super::*;

#[test]
fn managed_session_run_control_resolves_session_id_and_stays_confirmed() {
    let (_cache, _cache_home) = isolated_cache();
    write_active_run_payload(
        "managed-session-control",
        json!({
            "timestamp": "2026-06-30T04:00:00Z",
            "run_id": "managed-session-control",
            "profile": "codex",
            "workspace": "/sensitive/workspaces/project",
            "network": "provider",
            "status": "running",
            "container_name": "runhaven-codex-managed-session-control",
            "state_volume": "runhaven-codex-hidden-home",
            "session": "project",
            "token": "should-not-leak"
        }),
    )
    .expect("active run payload");

    let missing = crate::commands::run_control::stop_managed_session_for_test(
        StopManagedSessionRequest {
            session_id: "missing-session".to_string(),
            confirm_stop: true,
        },
        |_| unreachable!("unknown sessions must fail before stop"),
    )
    .expect_err("unknown session rejected");
    assert!(missing.contains("managed session not found"));

    let unconfirmed = crate::commands::run_control::stop_managed_session_for_test(
        StopManagedSessionRequest {
            session_id: "managed-session-control".to_string(),
            confirm_stop: false,
        },
        |_| unreachable!("unconfirmed stop must fail before mutation"),
    )
    .expect_err("confirmation required");
    assert!(unconfirmed.contains("Confirm the stop"));

    let stop = crate::commands::run_control::stop_managed_session_for_test(
        StopManagedSessionRequest {
            session_id: "managed-session-control".to_string(),
            confirm_stop: true,
        },
        |run_id| {
            assert_eq!(run_id, "managed-session-control");
            Ok(StopRunResponse {
                run_id: run_id.to_string(),
                container_name: "runhaven-codex-managed-session-control".to_string(),
                status: "stop-requested".to_string(),
            })
        },
    )
    .expect("stop managed session");
    assert_eq!(stop.session.id, "managed-session-control");
    assert_eq!(stop.result.status, "stop-requested");

    let kill = crate::commands::run_control::kill_managed_session_for_test(
        KillManagedSessionRequest {
            session_id: "managed-session-control".to_string(),
            confirm_kill: true,
        },
        |run_id| {
            assert_eq!(run_id, "managed-session-control");
            Ok(KillRunResponse {
                run_id: run_id.to_string(),
                container_name: "runhaven-codex-managed-session-control".to_string(),
                status: "kill-requested".to_string(),
            })
        },
    )
    .expect("kill managed session");
    assert_eq!(kill.session.id, "managed-session-control");
    assert_eq!(kill.result.status, "kill-requested");

    let repair = crate::commands::run_control::repair_managed_session_for_test(
        RepairManagedSessionRequest {
            session_id: "managed-session-control".to_string(),
            confirm_repair: true,
        },
        |run_id| {
            assert_eq!(run_id, "managed-session-control");
            Ok(RepairRunResponse {
                run_id: run_id.to_string(),
                container_name: "runhaven-codex-managed-session-control".to_string(),
                status: "removed".to_string(),
                marker_removed: true,
            })
        },
    )
    .expect("repair managed session");
    assert_eq!(repair.session.id, "managed-session-control");
    assert!(repair.result.marker_removed);

    let encoded = serde_json::to_string(&repair).expect("serialize repair response");
    for forbidden in [
        "/sensitive/workspaces",
        "runhaven-codex-hidden-home",
        "should-not-leak",
        "write_session_input",
        "attach_session",
    ] {
        assert!(
            !encoded.contains(forbidden),
            "managed session run-control leaked forbidden value {forbidden:?}"
        );
    }
}
