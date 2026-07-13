use std::thread::sleep;
use std::time::{Duration, Instant};

use runhaven_core::runtime::active::{
    active_run_status_payload, repair_active_run, stop_active_run,
};
use runhaven_core::runtime::network::is_runhaven_managed_network;
use runhaven_core::runtime::state::delete_volume;
use runhaven_core::ui_contracts::ManagedSessionSummaryData;
use serde::Serialize;

use crate::contracts::{
    LaunchStatusRequest, LaunchStatusResponse, ManagedSessionLogSnapshotRequest,
    ManagedSessionLogSnapshotResponse, ManagedSessionStatusRequest, ManagedSessionStatusResponse,
    StopManagedSessionRequest, WriteSessionInputRequest, WriteSessionInputResponse,
};
use crate::launch_status::LaunchStatusRegistry;
use crate::session_input::SessionInputRegistry;
use crate::session_output::SessionOutputRegistry;

pub(super) struct LiveTauriRunCleanup {
    pub(super) run_id: Option<String>,
    pub(super) state_volume: Option<String>,
    pub(super) network_name: Option<String>,
}

impl Drop for LiveTauriRunCleanup {
    fn drop(&mut self) {
        if let Some(run_id) = self.run_id.as_deref() {
            let _ = stop_active_run(run_id);
            sleep(Duration::from_secs(1));
            let _ = repair_active_run(run_id);
        }
        if let Some(state_volume) = self.state_volume.as_deref()
            && !state_volume.contains("-shared-home")
            && container_resource_exists("volume", state_volume)
        {
            let _ = delete_volume(state_volume);
        }
        if let Some(network_name) = self.network_name.as_deref()
            && is_runhaven_managed_network(network_name)
            && container_resource_exists("network", network_name)
            && let Ok(mut command) = runhaven_core::apple_container::command()
        {
            let _ = command.args(["network", "delete", network_name]).status();
        }
    }
}

fn container_resource_exists(kind: &str, name: &str) -> bool {
    let Ok(mut command) = runhaven_core::apple_container::command() else {
        return true;
    };
    let Ok(output) = command.args([kind, "list", "--quiet"]).output() else {
        return true;
    };
    if !output.status.success() {
        return true;
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .any(|line| line.trim() == name)
}

pub(super) fn wait_for_live_session(run_id: &str) -> ManagedSessionSummaryData {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let sessions = crate::commands::sessions::get_managed_sessions_for_test(None)
            .expect("read live managed sessions");
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

pub(super) fn wait_for_live_monitor_session(run_id: &str) -> ManagedSessionSummaryData {
    let deadline = Instant::now() + Duration::from_secs(45);
    loop {
        let session = wait_for_live_session(run_id);
        match (
            session.lifecycle.as_str(),
            session.next_action.kind.as_str(),
        ) {
            ("running", "monitor") => return session,
            (_, "auth_attention") => {
                panic!("live real-agent session needs auth before it can be monitored")
            }
            ("failed" | "finished" | "stopped", _) => {
                panic!(
                    "live real-agent session ended before monitor state: {}",
                    session.lifecycle
                )
            }
            _ if Instant::now() < deadline => sleep(Duration::from_millis(250)),
            _ => panic!(
                "live real-agent session did not reach monitor state for run {run_id}; last lifecycle={}, action={}",
                session.lifecycle, session.next_action.kind
            ),
        }
    }
}

pub(super) fn wait_for_live_launch_status(
    run_id: &str,
    launch_status: &LaunchStatusRegistry,
    expected_states: &[&str],
) -> LaunchStatusResponse {
    let deadline = Instant::now() + Duration::from_secs(45);
    let mut last_state = None;
    loop {
        let status = crate::commands::launch_status::get_launch_status_for_test(
            LaunchStatusRequest {
                run_id: run_id.to_string(),
            },
            launch_status,
        )
        .unwrap_or_else(|error| panic!("live launch status unavailable for {run_id}: {error}"));
        if expected_states.iter().any(|state| status.state == *state) {
            return status;
        }
        match status.state.as_str() {
            "failed" | "finished" => {
                panic!(
                    "live launch status reached {} before {:?} for run {run_id}",
                    status.state, expected_states
                );
            }
            _ if Instant::now() < deadline => {
                last_state = Some(status.state);
                sleep(Duration::from_millis(250));
            }
            _ => panic!(
                "live launch status did not reach {:?} for run {run_id}; last state={}",
                expected_states,
                last_state.unwrap_or(status.state)
            ),
        }
    }
}

pub(super) fn wait_for_live_managed_session_status(
    session_id: &str,
    input_registry: &SessionInputRegistry,
) -> ManagedSessionStatusResponse {
    let deadline = Instant::now() + Duration::from_secs(45);
    let mut last_error = None;
    loop {
        let (session, _) = crate::commands::sessions::managed_session_for_id_with_input(
            session_id,
            Some(input_registry),
        )
        .unwrap_or_else(|error| panic!("live managed session lookup failed: {error}"));
        match active_run_status_payload(&session.run_id) {
            Ok(payload) => {
                return crate::commands::sessions::managed_session_status_response_for_test(
                    ManagedSessionStatusRequest {
                        session_id: session_id.to_string(),
                    },
                    Some(input_registry),
                    Some(payload),
                    false,
                )
                .expect("live managed session status");
            }
            Err(error) if Instant::now() < deadline => {
                last_error = Some(error.to_string());
                sleep(Duration::from_millis(250));
            }
            Err(error) => panic!(
                "live managed session status was unavailable for {session_id}: {}",
                last_error.unwrap_or_else(|| error.to_string())
            ),
        }
    }
}

pub(super) fn wait_for_live_input_acceptance(
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

pub(super) fn wait_for_live_session_output(
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
        if log.snapshot.text.contains(expected) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "live managed session output did not contain {expected}"
        );
        sleep(Duration::from_millis(250));
    }
}

pub(super) fn wait_for_live_session_output_available(
    session_id: &str,
    output_registry: &SessionOutputRegistry,
) -> ManagedSessionLogSnapshotResponse {
    let deadline = Instant::now() + Duration::from_secs(45);
    let mut last_error = None;
    loop {
        match crate::commands::sessions::get_managed_session_log_snapshot_with_output_for_test(
            ManagedSessionLogSnapshotRequest {
                session_id: session_id.to_string(),
                lines: Some(40),
                confirm_sensitive_output: true,
            },
            output_registry,
        ) {
            Ok(log) if session_output_has_text(&log) => return log,
            Ok(_) if Instant::now() < deadline => sleep(Duration::from_millis(250)),
            Ok(_) => panic!("live managed session output stayed empty for {session_id}"),
            Err(error) if Instant::now() < deadline => {
                last_error = Some(error);
                sleep(Duration::from_millis(250));
            }
            Err(error) => panic!(
                "live managed session output was unavailable for {session_id}: {}",
                last_error.unwrap_or(error)
            ),
        }
    }
}

fn session_output_has_text(log: &ManagedSessionLogSnapshotResponse) -> bool {
    !log.snapshot.text.trim().is_empty()
}

pub(super) fn assert_live_response_excludes<T: Serialize>(
    label: &str,
    value: &T,
    forbidden: &[(&str, String)],
) {
    let encoded = serde_json::to_string(value).expect("serialize live response");
    for (forbidden_label, fragment) in forbidden {
        assert!(
            !fragment.is_empty() && !encoded.contains(fragment),
            "{label} leaked forbidden value labelled {forbidden_label}"
        );
    }
}

pub(super) fn assert_live_session_output_excludes(
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
        !log.snapshot.text.contains(forbidden),
        "live managed session output leaked marker {forbidden} into {session_id}"
    );
}

pub(super) fn wait_for_live_session_inactive(run_id: &str) -> ManagedSessionSummaryData {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let sessions = crate::commands::sessions::get_managed_sessions_for_test(None)
            .expect("read live managed sessions");
        if let Some(session) = sessions
            .sessions
            .into_iter()
            .find(|session| session.run_id == run_id)
            && !matches!(
                session.lifecycle.as_str(),
                "running" | "starting" | "stop-requested" | "kill-requested"
            )
        {
            return session;
        }
        assert!(
            Instant::now() < deadline,
            "live managed session stayed running for run {run_id}"
        );
        sleep(Duration::from_millis(250));
    }
}

pub(super) fn stop_live_session_if_active(session: &ManagedSessionSummaryData) {
    if !matches!(
        session.lifecycle.as_str(),
        "running" | "starting" | "stop-requested" | "kill-requested"
    ) {
        return;
    }
    let stopped = crate::commands::run_control::stop_managed_session_for_test(
        StopManagedSessionRequest {
            session_id: session.id.clone(),
            confirm_stop: true,
        },
        |_, _| Ok(true),
        crate::commands::run_control::stop_run_checked_for_test,
    )
    .expect("stop live managed session");
    assert_eq!(stopped.result.run_id, session.run_id);
    assert_eq!(stopped.result.status, "stop-requested");
}
