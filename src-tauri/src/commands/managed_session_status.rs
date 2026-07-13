use runhaven_core::runtime::active::{active_run_repair_available, active_run_status_payload};
use runhaven_core::ui_contracts::{ManagedSessionInputData, ManagedSessionSummaryData};

use crate::contracts::{
    ManagedSessionRecovery, ManagedSessionStatusRequest, ManagedSessionStatusResponse,
    RunStatusContainer,
};
use crate::read_guard::ReadGuards;
use crate::session_input::SessionInputRegistry;

const MANAGED_SESSION_STATUS_UNAVAILABLE_MESSAGE: &str =
    "RunHaven can’t read this session’s current status. It may have ended.";

#[tauri::command]
pub(crate) fn get_managed_session_status(
    request: ManagedSessionStatusRequest,
    input_registry: tauri::State<'_, SessionInputRegistry>,
    read_guards: tauri::State<'_, ReadGuards>,
) -> Result<ManagedSessionStatusResponse, String> {
    // Single-flight plus a 2s TTL cache keyed by session id: repeated polling
    // shells out to Apple `container`, so same-session reads coalesce into one
    // subprocess (the same local-DoS posture as `get_managed_sessions`).
    // Locking is per key, so one session's hung probe never blocks another
    // session's status read.
    read_guards
        .managed_session_status
        .run(&request.session_id, || {
            get_managed_session_status_with_input(request.clone(), Some(input_registry.inner()))
        })
}

fn get_managed_session_status_with_input(
    request: ManagedSessionStatusRequest,
    input_registry: Option<&SessionInputRegistry>,
) -> Result<ManagedSessionStatusResponse, String> {
    let (session, _) =
        super::sessions::managed_session_for_id_with_input(&request.session_id, input_registry)?;
    let (container, repair_available) = match active_run_status_payload(&session.run_id)
        .map_err(|error| error.to_string())
        .and_then(super::run_status::run_status_response)
    {
        Ok(response) => (Some(response.container), false),
        Err(_) => (None, managed_session_repair_available(&session)),
    };
    Ok(managed_session_status_response(
        session,
        container,
        repair_available,
    ))
}

#[cfg(test)]
pub(crate) fn managed_session_status_response_for_test(
    request: ManagedSessionStatusRequest,
    input_registry: Option<&SessionInputRegistry>,
    status_payload: Option<serde_json::Value>,
    repair_available: bool,
) -> Result<ManagedSessionStatusResponse, String> {
    let (session, _) =
        super::sessions::managed_session_for_id_with_input(&request.session_id, input_registry)?;
    let container = status_payload
        .map(super::run_status::run_status_response)
        .transpose()?
        .map(|response| response.container);
    Ok(managed_session_status_response(
        session,
        container,
        repair_available,
    ))
}

fn managed_session_status_response(
    mut session: ManagedSessionSummaryData,
    container: Option<RunStatusContainer>,
    repair_available: bool,
) -> ManagedSessionStatusResponse {
    if container.is_none() {
        session.input = ManagedSessionInputData::detached();
    }
    let status_unavailable = if container.is_some() {
        None
    } else {
        Some(MANAGED_SESSION_STATUS_UNAVAILABLE_MESSAGE.to_string())
    };
    ManagedSessionStatusResponse {
        recovery: managed_session_recovery(&session, container.is_some(), repair_available),
        session,
        container,
        status_unavailable,
    }
}

fn managed_session_recovery(
    session: &ManagedSessionSummaryData,
    container_available: bool,
    repair_available: bool,
) -> Option<ManagedSessionRecovery> {
    if container_available {
        return None;
    }
    match session.lifecycle.trim().to_lowercase().as_str() {
        "running" | "starting" | "stop-requested" | "kill-requested" if repair_available => {
            Some(ManagedSessionRecovery {
                status: "warning".to_string(),
                title: "Session ended unexpectedly".to_string(),
                detail: "RunHaven can no longer find this session.".to_string(),
                remedy: "Remove it from the active list, then start a new session if needed."
                    .to_string(),
                repair_available: true,
            })
        }
        "running" | "starting" | "stop-requested" | "kill-requested" => {
            Some(ManagedSessionRecovery {
                status: "attention".to_string(),
                title: "Can’t check this session".to_string(),
                detail:
                    "RunHaven can see the session but can’t confirm whether it is still running."
                        .to_string(),
                remedy: "Choose Check again. If the problem continues, open System.".to_string(),
                repair_available: false,
            })
        }
        "failed" => Some(ManagedSessionRecovery {
            status: "attention".to_string(),
            title: "Session ended with a problem".to_string(),
            detail: "This session is no longer running.".to_string(),
            remedy: "Open System to check setup, then try a new session.".to_string(),
            repair_available: false,
        }),
        "finished" | "stopped" => Some(ManagedSessionRecovery {
            status: "history".to_string(),
            title: "Session ended".to_string(),
            detail: "This session is in History, so it no longer has a live status.".to_string(),
            remedy: "Show its output or start a new session.".to_string(),
            repair_available: false,
        }),
        _ => Some(ManagedSessionRecovery {
            status: "unknown".to_string(),
            title: "Can’t check this session".to_string(),
            detail: "RunHaven can’t read this session’s current status.".to_string(),
            remedy: "Choose Check again. If the problem continues, open System.".to_string(),
            repair_available: false,
        }),
    }
}

fn managed_session_repair_available(session: &ManagedSessionSummaryData) -> bool {
    if !super::sessions::managed_session_allows_run_control(&session.lifecycle) {
        return false;
    }
    active_run_repair_available(&session.run_id).unwrap_or(false)
}
