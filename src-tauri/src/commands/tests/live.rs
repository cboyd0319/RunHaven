use super::live_support::{
    LiveTauriRunCleanup, assert_live_session_output_excludes, wait_for_live_input_acceptance,
    wait_for_live_session, wait_for_live_session_inactive, wait_for_live_session_output,
};
use super::*;

#[test]
#[ignore = "requires RUNHAVEN_LIVE_LOG_RUN_ID and a live RunHaven container"]
fn log_snapshot_reads_live_active_run() {
    let run_id = std::env::var("RUNHAVEN_LIVE_LOG_RUN_ID").expect("live run id");
    let expected = std::env::var("RUNHAVEN_LIVE_LOG_EXPECT").expect("expected log marker");
    let expect_truncated = std::env::var("RUNHAVEN_LIVE_LOG_EXPECT_TRUNCATED")
        .ok()
        .as_deref()
        == Some("1");

    let response = crate::commands::log_snapshot::get_log_snapshot(LogSnapshotRequest {
        run_id,
        lines: Some(20),
        confirm_sensitive_output: true,
    })
    .expect("live log snapshot");

    assert!(response.text.contains(&expected));
    if expect_truncated {
        assert!(response.truncated);
    }
}

#[test]
#[ignore = "requires RUNHAVEN_LIVE_MANAGED_SESSION_RUN_ID and a live RunHaven container"]
fn managed_session_live_smoke_reads_output_and_stops_run() {
    let run_id =
        std::env::var("RUNHAVEN_LIVE_MANAGED_SESSION_RUN_ID").expect("live managed session run id");
    let expected =
        std::env::var("RUNHAVEN_LIVE_MANAGED_SESSION_EXPECT").expect("expected log marker");
    let forbidden_workspace = std::env::var("RUNHAVEN_LIVE_MANAGED_SESSION_FORBIDDEN").ok();

    let sessions =
        crate::commands::sessions::get_managed_sessions_for_test(None).expect("managed sessions");
    let session = sessions
        .sessions
        .iter()
        .find(|session| session.run_id == run_id)
        .expect("live managed session")
        .clone();
    let session_id = session.id.clone();

    let encoded_session = serde_json::to_string(&session).expect("serialize managed session");
    if let Some(workspace) = forbidden_workspace.as_deref() {
        assert!(
            !encoded_session.contains(workspace),
            "managed session leaked workspace path"
        );
    }
    assert!(!encoded_session.contains("write_session_input"));
    assert!(!encoded_session.contains("attach_session"));

    let log = crate::commands::sessions::get_managed_session_log_snapshot_for_test(
        ManagedSessionLogSnapshotRequest {
            session_id: session_id.clone(),
            lines: Some(20),
            confirm_sensitive_output: true,
        },
    )
    .expect("managed session log snapshot");
    assert_eq!(log.session.id, session_id);
    assert_eq!(log.snapshot.run_id, run_id);
    assert!(log.snapshot.text.contains(&expected));

    let stopped = crate::commands::run_control::stop_managed_session_for_test(
        StopManagedSessionRequest {
            session_id,
            confirm_stop: true,
        },
        |_, _| Ok(true),
        crate::commands::run_control::stop_run_checked_for_test,
    )
    .expect("stop managed session");
    assert_eq!(stopped.result.run_id, run_id);
    assert_eq!(stopped.result.status, "stop-requested");
}

#[test]
#[ignore = "requires Apple container and a built shell image"]
fn managed_session_live_smoke_writes_input_and_reads_output() {
    let workspace = tempfile::tempdir().expect("workspace");
    std::fs::write(
        workspace.path().join("input.txt"),
        "runhaven live input smoke\n",
    )
    .expect("workspace fixture");
    let session_name = format!("gui-input-{}", std::process::id());
    let plan_request = shell_request(workspace.path().to_path_buf(), session_name);
    let plan = build_plan_response(plan_request.clone()).expect("live plan");
    let mut cleanup = LiveTauriRunCleanup {
        run_id: None,
        state_volume: Some(plan.state_volume.clone()),
        network_name: plan.network_name.clone(),
    };
    let registry = SessionInputRegistry::default();
    let output_registry = SessionOutputRegistry::default();

    let launched = launch_run_with_registries(
        LaunchRunRequest { plan: plan_request },
        registry.clone(),
        output_registry.clone(),
    )
    .expect("launch live shell through Tauri runner");
    cleanup.run_id = Some(launched.run_id.clone());

    let session = wait_for_live_session(&launched.run_id);
    let marker = format!("runhaven-live-input-{}", std::process::id());
    let input = format!("echo {marker}\n");
    let accepted = wait_for_live_input_acceptance(&session.id, &registry, &input);
    assert!(accepted.accepted);
    assert_eq!(accepted.accepted_bytes, input.len());
    assert_eq!(accepted.session.run_id, launched.run_id);

    wait_for_live_session_output(&session.id, &output_registry, &marker);

    let stopped = crate::commands::run_control::stop_managed_session_for_test(
        StopManagedSessionRequest {
            session_id: session.id,
            confirm_stop: true,
        },
        |_, _| Ok(true),
        crate::commands::run_control::stop_run_checked_for_test,
    )
    .expect("stop live managed session");
    assert_eq!(stopped.result.run_id, launched.run_id);
    assert_eq!(stopped.result.status, "stop-requested");
    let finished = wait_for_live_session_inactive(&launched.run_id);
    assert_eq!(finished.lifecycle, "stopped");
    assert!(!finished.input.attached);
}

#[test]
#[ignore = "requires Apple container and a built shell image"]
fn managed_session_live_smoke_isolates_two_sessions() {
    let workspace = tempfile::tempdir().expect("workspace");
    std::fs::write(
        workspace.path().join("input.txt"),
        "runhaven two-session input smoke\n",
    )
    .expect("workspace fixture");
    let session_prefix = format!("gui-two-{}", std::process::id());
    let alpha_request = shell_request(
        workspace.path().to_path_buf(),
        format!("{session_prefix}-alpha"),
    );
    let beta_request = shell_request(
        workspace.path().to_path_buf(),
        format!("{session_prefix}-beta"),
    );
    let alpha_plan = build_plan_response(alpha_request.clone()).expect("alpha live plan");
    let beta_plan = build_plan_response(beta_request.clone()).expect("beta live plan");
    assert_ne!(alpha_plan.state_volume, beta_plan.state_volume);
    assert_ne!(alpha_plan.container_name, beta_plan.container_name);

    let mut alpha_cleanup = LiveTauriRunCleanup {
        run_id: None,
        state_volume: Some(alpha_plan.state_volume.clone()),
        network_name: alpha_plan.network_name.clone(),
    };
    let mut beta_cleanup = LiveTauriRunCleanup {
        run_id: None,
        state_volume: Some(beta_plan.state_volume.clone()),
        network_name: if beta_plan.network_name == alpha_plan.network_name {
            None
        } else {
            beta_plan.network_name.clone()
        },
    };
    let registry = SessionInputRegistry::default();
    let output_registry = SessionOutputRegistry::default();

    let alpha = launch_run_with_registries(
        LaunchRunRequest {
            plan: alpha_request,
        },
        registry.clone(),
        output_registry.clone(),
    )
    .expect("launch alpha live shell through Tauri runner");
    alpha_cleanup.run_id = Some(alpha.run_id.clone());
    let beta = launch_run_with_registries(
        LaunchRunRequest { plan: beta_request },
        registry.clone(),
        output_registry.clone(),
    )
    .expect("launch beta live shell through Tauri runner");
    beta_cleanup.run_id = Some(beta.run_id.clone());

    let alpha_session = wait_for_live_session(&alpha.run_id);
    let beta_session = wait_for_live_session(&beta.run_id);
    assert_ne!(alpha_session.id, beta_session.id);
    assert_eq!(alpha_session.run_id, alpha.run_id);
    assert_eq!(beta_session.run_id, beta.run_id);

    let alpha_marker = format!("runhaven-live-alpha-{}", std::process::id());
    let beta_marker = format!("runhaven-live-beta-{}", std::process::id());
    let alpha_input = format!("echo {alpha_marker}\n");
    let beta_input = format!("echo {beta_marker}\n");

    let alpha_accepted = wait_for_live_input_acceptance(&alpha_session.id, &registry, &alpha_input);
    assert!(alpha_accepted.accepted);
    assert_eq!(alpha_accepted.accepted_bytes, alpha_input.len());
    assert_eq!(alpha_accepted.session.run_id, alpha.run_id);
    wait_for_live_session_output(&alpha_session.id, &output_registry, &alpha_marker);

    let beta_accepted = wait_for_live_input_acceptance(&beta_session.id, &registry, &beta_input);
    assert!(beta_accepted.accepted);
    assert_eq!(beta_accepted.accepted_bytes, beta_input.len());
    assert_eq!(beta_accepted.session.run_id, beta.run_id);
    wait_for_live_session_output(&beta_session.id, &output_registry, &beta_marker);

    assert_live_session_output_excludes(&alpha_session.id, &output_registry, &beta_marker);
    assert_live_session_output_excludes(&beta_session.id, &output_registry, &alpha_marker);

    let stopped_alpha = crate::commands::run_control::stop_managed_session_for_test(
        StopManagedSessionRequest {
            session_id: alpha_session.id,
            confirm_stop: true,
        },
        |_, _| Ok(true),
        crate::commands::run_control::stop_run_checked_for_test,
    )
    .expect("stop alpha live managed session");
    assert_eq!(stopped_alpha.result.run_id, alpha.run_id);
    assert_eq!(stopped_alpha.result.status, "stop-requested");

    let stopped_beta = crate::commands::run_control::stop_managed_session_for_test(
        StopManagedSessionRequest {
            session_id: beta_session.id,
            confirm_stop: true,
        },
        |_, _| Ok(true),
        crate::commands::run_control::stop_run_checked_for_test,
    )
    .expect("stop beta live managed session");
    assert_eq!(stopped_beta.result.run_id, beta.run_id);
    assert_eq!(stopped_beta.result.status, "stop-requested");

    let finished_alpha = wait_for_live_session_inactive(&alpha.run_id);
    let finished_beta = wait_for_live_session_inactive(&beta.run_id);
    assert_eq!(finished_alpha.lifecycle, "stopped");
    assert_eq!(finished_beta.lifecycle, "stopped");
}
