use std::collections::BTreeSet;

use runhaven_core::diagnostics::auth_status_payload;
use runhaven_core::records::read_run_records_tail_bounded;
use runhaven_core::runtime::active::{active_run_repair_available, read_active_run_records};
use runhaven_core::ui_contracts::{
    ManagedSessionInputData, ManagedSessionListData, ManagedSessionSummaryData,
};

use super::validation::{MAX_RUN_ID_LEN, validate_text_len};
use crate::session_input::SessionInputRegistry;

#[cfg(test)]
pub(crate) use super::managed_session_input::{
    write_session_input_for_test, write_session_input_with_approval_for_test,
};
#[cfg(test)]
pub(crate) use super::managed_session_output::{
    get_managed_session_log_snapshot_for_test,
    get_managed_session_log_snapshot_with_approval_for_test,
    get_managed_session_log_snapshot_with_output_for_test,
    managed_session_log_snapshot_response_for_test,
};
#[cfg(test)]
pub(crate) use super::managed_session_status::managed_session_status_response_for_test;

const MANAGED_SESSION_HISTORY_LIMIT: usize = 20;
const MANAGED_SESSION_HISTORY_TAIL_BYTES: u64 = 256 * 1024;

#[tauri::command]
pub(crate) fn get_managed_sessions(
    input_registry: tauri::State<'_, SessionInputRegistry>,
) -> Result<ManagedSessionListData, String> {
    get_managed_sessions_with_input(Some(input_registry.inner()))
}

fn get_managed_sessions_with_input(
    input_registry: Option<&SessionInputRegistry>,
) -> Result<ManagedSessionListData, String> {
    let mut sessions = managed_session_list(input_registry)?;
    reconcile_stale_sessions(&mut sessions, &container_marker_gone);
    Ok(sessions)
}

/// Production reconcile signal: an active RunHaven marker is stale only when the
/// authoritative Apple `container inspect` proves its container is gone (the same
/// check marker repair uses). Any other outcome (container present, `container`
/// CLI missing, transient error) is treated as not-stale so live sessions are
/// never falsely flagged.
fn container_marker_gone(run_id: &str) -> Result<bool, String> {
    active_run_repair_available(run_id).map_err(|error| error.to_string())
}

/// Downgrade active sessions whose container is proven gone to a recovery state.
/// The container check is injected so tests stay hermetic (no real `container`
/// subprocess) and fail-safe: a checker error leaves the session untouched.
pub(crate) fn reconcile_stale_sessions(
    sessions: &mut ManagedSessionListData,
    container_gone: &dyn Fn(&str) -> Result<bool, String>,
) {
    // lean: this issues one `container inspect` per active session on each list
    // poll (~4s) with no per-call timeout; N is small in normal use. If a
    // degraded container service or large N becomes real, bound it with an
    // inspect timeout or a cheap pre-filter before adding more machinery.
    let mut stale = BTreeSet::new();
    for session in &sessions.sessions {
        if matches!(session.lifecycle.as_str(), "running" | "starting")
            && matches!(container_gone(&session.run_id), Ok(true))
        {
            stale.insert(session.run_id.clone());
        }
    }
    sessions.mark_stale_sessions(&stale);
}

#[cfg(test)]
pub(crate) fn get_managed_sessions_for_test(
    input_registry: Option<&SessionInputRegistry>,
) -> Result<ManagedSessionListData, String> {
    managed_session_list(input_registry)
}

fn managed_session_list(
    input_registry: Option<&SessionInputRegistry>,
) -> Result<ManagedSessionListData, String> {
    let active_runs = read_active_run_records();
    let recent_runs = read_run_records_tail_bounded(
        MANAGED_SESSION_HISTORY_LIMIT,
        MANAGED_SESSION_HISTORY_TAIL_BYTES,
    )
    .map_err(|error| error.to_string())?;
    let mut sessions = ManagedSessionListData::from_active_and_recent_runs(
        active_runs,
        recent_runs,
        auth_status_payload(),
    );
    if let Some(input_registry) = input_registry {
        mark_session_input_transports(&mut sessions, input_registry)?;
    }
    Ok(sessions)
}

fn mark_session_input_transports(
    sessions: &mut ManagedSessionListData,
    input_registry: &SessionInputRegistry,
) -> Result<(), String> {
    for session in &mut sessions.sessions {
        if managed_session_accepts_input(session)
            && input_registry.has_writer(&session.id, &session.run_id)?
        {
            session.input = ManagedSessionInputData::attached();
        }
    }
    Ok(())
}

pub(crate) fn managed_session_accepts_input(session: &ManagedSessionSummaryData) -> bool {
    session.next_action.kind == "monitor"
}

pub(crate) fn managed_session_for_id(
    session_id: &str,
) -> Result<(ManagedSessionSummaryData, String), String> {
    managed_session_for_id_with_input(session_id, None)
}

pub(crate) fn managed_session_for_run_control(
    session_id: &str,
) -> Result<(ManagedSessionSummaryData, String), String> {
    let (session, run_id) = managed_session_for_id(session_id)?;
    if !managed_session_allows_run_control(&session.lifecycle) {
        return Err(
            "managed session is no longer active; refresh sessions and start a new run instead"
                .to_string(),
        );
    }
    Ok((session, run_id))
}

pub(crate) fn managed_session_for_id_with_input(
    session_id: &str,
    input_registry: Option<&SessionInputRegistry>,
) -> Result<(ManagedSessionSummaryData, String), String> {
    validate_text_len("session id", session_id, MAX_RUN_ID_LEN)?;
    if session_id.is_empty() {
        return Err("managed session id is required".to_string());
    }
    let session = managed_session_list(input_registry)?
        .sessions
        .into_iter()
        .find(|session| session.id == session_id)
        .ok_or_else(|| "managed session not found".to_string())?;
    let run_id = session.run_id.clone();
    Ok((session, run_id))
}

pub(crate) fn managed_session_allows_run_control(lifecycle: &str) -> bool {
    matches!(
        lifecycle.trim().to_lowercase().as_str(),
        "running" | "starting" | "stop-requested" | "kill-requested"
    )
}
