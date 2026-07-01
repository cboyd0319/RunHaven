use std::process::Command;
use std::thread::sleep;
use std::time::{Duration, Instant};

use runhaven_core::runtime::active::{repair_active_run, stop_active_run};
use runhaven_core::runtime::network::is_runhaven_managed_network;
use runhaven_core::runtime::state::delete_volume;
use runhaven_core::ui_contracts::ManagedSessionSummaryData;

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

    let sessions = crate::commands::sessions::get_managed_sessions().expect("managed sessions");
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

    let stopped = crate::commands::run_control::stop_managed_session(StopManagedSessionRequest {
        session_id,
        confirm_stop: true,
    })
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
        LaunchRunRequest {
            plan: plan_request,
            confirm_launch: true,
            confirmed_warnings: Vec::new(),
        },
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

    let stopped = crate::commands::run_control::stop_managed_session(StopManagedSessionRequest {
        session_id: session.id,
        confirm_stop: true,
    })
    .expect("stop live managed session");
    assert_eq!(stopped.result.run_id, launched.run_id);
    assert_eq!(stopped.result.status, "stop-requested");
    wait_for_live_session_removed(&launched.run_id);
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
            confirm_launch: true,
            confirmed_warnings: Vec::new(),
        },
        registry.clone(),
        output_registry.clone(),
    )
    .expect("launch alpha live shell through Tauri runner");
    alpha_cleanup.run_id = Some(alpha.run_id.clone());
    let beta = launch_run_with_registries(
        LaunchRunRequest {
            plan: beta_request,
            confirm_launch: true,
            confirmed_warnings: Vec::new(),
        },
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

    let stopped_alpha =
        crate::commands::run_control::stop_managed_session(StopManagedSessionRequest {
            session_id: alpha_session.id,
            confirm_stop: true,
        })
        .expect("stop alpha live managed session");
    assert_eq!(stopped_alpha.result.run_id, alpha.run_id);
    assert_eq!(stopped_alpha.result.status, "stop-requested");

    let stopped_beta =
        crate::commands::run_control::stop_managed_session(StopManagedSessionRequest {
            session_id: beta_session.id,
            confirm_stop: true,
        })
        .expect("stop beta live managed session");
    assert_eq!(stopped_beta.result.run_id, beta.run_id);
    assert_eq!(stopped_beta.result.status, "stop-requested");

    wait_for_live_session_removed(&alpha.run_id);
    wait_for_live_session_removed(&beta.run_id);
}

struct LiveTauriRunCleanup {
    run_id: Option<String>,
    state_volume: Option<String>,
    network_name: Option<String>,
}

impl Drop for LiveTauriRunCleanup {
    fn drop(&mut self) {
        if let Some(run_id) = self.run_id.as_deref() {
            let _ = stop_active_run(run_id);
            sleep(Duration::from_secs(1));
            let _ = repair_active_run(run_id);
        }
        if let Some(state_volume) = self.state_volume.as_deref() {
            let _ = delete_volume(state_volume);
        }
        if let Some(network_name) = self.network_name.as_deref()
            && is_runhaven_managed_network(network_name)
        {
            let _ = Command::new("container")
                .args(["network", "delete", network_name])
                .status();
        }
    }
}

fn wait_for_live_session(run_id: &str) -> ManagedSessionSummaryData {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let sessions =
            crate::commands::sessions::get_managed_sessions().expect("read live managed sessions");
        if let Some(session) = sessions
            .sessions
            .into_iter()
            .find(|session| session.run_id == run_id)
        {
            return session;
        }
        assert!(
            Instant::now() < deadline,
            "live managed session did not appear for run {run_id}"
        );
        sleep(Duration::from_millis(250));
    }
}

fn wait_for_live_input_acceptance(
    session_id: &str,
    registry: &SessionInputRegistry,
    input: &str,
) -> WriteSessionInputResponse {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        match crate::commands::sessions::write_session_input_for_test(
            WriteSessionInputRequest {
                session_id: session_id.to_string(),
                input: input.to_string(),
                confirm_send: true,
                client_target: format!("session:{session_id}"),
            },
            registry,
        ) {
            Ok(response) => return response,
            Err(error)
                if (error.contains("managed session not found")
                    || error.contains("session input transport not found"))
                    && Instant::now() < deadline =>
            {
                sleep(Duration::from_millis(250));
            }
            Err(error) => panic!("live session input failed: {error}"),
        }
    }
}

fn wait_for_live_session_output(
    session_id: &str,
    output_registry: &SessionOutputRegistry,
    expected: &str,
) {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let log = crate::commands::sessions::get_managed_session_log_snapshot_with_output_for_test(
            ManagedSessionLogSnapshotRequest {
                session_id: session_id.to_string(),
                lines: Some(40),
                confirm_sensitive_output: true,
            },
            output_registry,
        )
        .expect("read live managed session output");
        let matched = log.snapshot.text.contains(expected)
            || log
                .entries
                .iter()
                .any(|entry| entry.text.contains(expected));
        if matched {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "live managed session output did not contain {expected}"
        );
        sleep(Duration::from_millis(250));
    }
}

fn assert_live_session_output_excludes(
    session_id: &str,
    output_registry: &SessionOutputRegistry,
    forbidden: &str,
) {
    let log = crate::commands::sessions::get_managed_session_log_snapshot_with_output_for_test(
        ManagedSessionLogSnapshotRequest {
            session_id: session_id.to_string(),
            lines: Some(40),
            confirm_sensitive_output: true,
        },
        output_registry,
    )
    .expect("read live managed session output");
    assert!(
        !log.snapshot.text.contains(forbidden)
            && log
                .entries
                .iter()
                .all(|entry| !entry.text.contains(forbidden)),
        "live managed session output leaked marker {forbidden} into {session_id}"
    );
}

fn wait_for_live_session_removed(run_id: &str) {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let sessions =
            crate::commands::sessions::get_managed_sessions().expect("read live managed sessions");
        if !sessions
            .sessions
            .iter()
            .any(|session| session.run_id == run_id)
        {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "live managed session stayed active for run {run_id}"
        );
        sleep(Duration::from_millis(250));
    }
}
