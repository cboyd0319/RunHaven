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
        |_, _| unreachable!("unknown sessions must fail before approval"),
        |_| unreachable!("unknown sessions must fail before stop"),
    )
    .expect_err("unknown session rejected");
    assert!(missing.contains("managed session not found"));

    let unconfirmed = crate::commands::run_control::stop_managed_session_for_test(
        StopManagedSessionRequest {
            session_id: "managed-session-control".to_string(),
            confirm_stop: false,
        },
        |_, _| unreachable!("unconfirmed stop must fail before approval"),
        |_| unreachable!("unconfirmed stop must fail before mutation"),
    )
    .expect_err("confirmation required");
    assert!(unconfirmed.contains("Confirm the stop"));

    let stop = crate::commands::run_control::stop_managed_session_for_test(
        StopManagedSessionRequest {
            session_id: "managed-session-control".to_string(),
            confirm_stop: true,
        },
        |session, run_id| {
            assert_eq!(session.id, "managed-session-control");
            assert_eq!(run_id, "managed-session-control");
            Ok(true)
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

    let mut stop_mutated = false;
    let denied_stop = crate::commands::run_control::stop_managed_session_for_test(
        StopManagedSessionRequest {
            session_id: "managed-session-control".to_string(),
            confirm_stop: true,
        },
        |session, run_id| {
            assert_eq!(session.id, "managed-session-control");
            assert_eq!(run_id, "managed-session-control");
            Ok(false)
        },
        |_| {
            stop_mutated = true;
            unreachable!("trusted denial must fail before stop")
        },
    )
    .expect_err("trusted stop denial rejected");
    assert!(denied_stop.contains("Stop canceled"));
    assert!(!stop_mutated);

    let kill = crate::commands::run_control::kill_managed_session_for_test(
        KillManagedSessionRequest {
            session_id: "managed-session-control".to_string(),
            confirm_kill: true,
        },
        |session, run_id| {
            assert_eq!(session.id, "managed-session-control");
            assert_eq!(run_id, "managed-session-control");
            Ok(true)
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
        |session, run_id| {
            assert_eq!(session.id, "managed-session-control");
            assert_eq!(run_id, "managed-session-control");
            Ok(true)
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

#[test]
fn managed_session_run_control_rejects_history_sessions_before_mutation() {
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
            "timestamp": "2026-07-01T05:00:00Z",
            "started_at": "2026-07-01T04:00:00Z",
            "finished_at": "2026-07-01T05:00:00Z",
            "run_id": "managed-session-history-control",
            "profile": "codex",
            "workspace": "/sensitive/workspaces/history",
            "network": "provider",
            "status": "succeeded",
            "auth_scope": "project",
            "session": "history"
        })
    )
    .expect("write run record");

    let stop = crate::commands::run_control::stop_managed_session_for_test(
        StopManagedSessionRequest {
            session_id: "managed-session-history-control".to_string(),
            confirm_stop: true,
        },
        |_, _| unreachable!("history session must fail before approval"),
        |_| unreachable!("history session must fail before stop"),
    )
    .expect_err("history session rejected");
    assert!(stop.contains("no longer active"));

    let kill = crate::commands::run_control::kill_managed_session_for_test(
        KillManagedSessionRequest {
            session_id: "managed-session-history-control".to_string(),
            confirm_kill: true,
        },
        |_, _| unreachable!("history session must fail before approval"),
        |_| unreachable!("history session must fail before kill"),
    )
    .expect_err("history session rejected");
    assert!(kill.contains("no longer active"));

    let repair = crate::commands::run_control::repair_managed_session_for_test(
        RepairManagedSessionRequest {
            session_id: "managed-session-history-control".to_string(),
            confirm_repair: true,
        },
        |_, _| unreachable!("history session must fail before approval"),
        |_| unreachable!("history session must fail before repair"),
    )
    .expect_err("history session rejected");
    assert!(repair.contains("no longer active"));
}
