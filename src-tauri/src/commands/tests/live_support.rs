use std::process::Command;
use std::thread::sleep;
use std::time::{Duration, Instant};

use runhaven_core::runtime::active::{repair_active_run, stop_active_run};
use runhaven_core::runtime::network::is_runhaven_managed_network;
use runhaven_core::runtime::state::delete_volume;
use runhaven_core::ui_contracts::ManagedSessionSummaryData;

use crate::contracts::{
    ManagedSessionLogSnapshotRequest, ManagedSessionLogSnapshotResponse, StopManagedSessionRequest,
    WriteSessionInputRequest, WriteSessionInputResponse,
};
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
        {
            let _ = Command::new("container")
                .args(["network", "delete", network_name])
                .status();
        }
    }
}

fn container_resource_exists(kind: &str, name: &str) -> bool {
    let Ok(output) = Command::new("container")
        .args([kind, "list", "--quiet"])
        .output()
    else {
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
        || log
            .entries
            .iter()
            .any(|entry| !entry.text.trim().is_empty())
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
        !log.snapshot.text.contains(forbidden)
            && log
                .entries
                .iter()
                .all(|entry| !entry.text.contains(forbidden)),
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
    let stopped = crate::commands::run_control::stop_managed_session(StopManagedSessionRequest {
        session_id: session.id.clone(),
        confirm_stop: true,
    })
    .expect("stop live managed session");
    assert_eq!(stopped.result.run_id, session.run_id);
    assert_eq!(stopped.result.status, "stop-requested");
}
