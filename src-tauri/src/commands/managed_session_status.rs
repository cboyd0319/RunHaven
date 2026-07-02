use runhaven_core::runtime::active::{active_run_repair_available, active_run_status_payload};
use runhaven_core::ui_contracts::{ManagedSessionInputData, ManagedSessionSummaryData};

use crate::contracts::{
    ManagedSessionRecovery, ManagedSessionStatusRequest, ManagedSessionStatusResponse,
    RunStatusContainer,
};
use crate::read_guard::ReadGuards;
use crate::session_input::SessionInputRegistry;
use crate::session_windows::{SessionWindowRegistry, authorize_session_read};

const MANAGED_SESSION_STATUS_UNAVAILABLE_MESSAGE: &str =
    "Container status is unavailable. The run may have stopped.";

#[tauri::command]
pub(crate) fn get_managed_session_status(
    window: tauri::Window,
    request: ManagedSessionStatusRequest,
    input_registry: tauri::State<'_, SessionInputRegistry>,
    read_guards: tauri::State<'_, ReadGuards>,
    window_registry: tauri::State<'_, SessionWindowRegistry>,
) -> Result<ManagedSessionStatusResponse, String> {
    // Per-window scope first: the main window reads any session, a monitor
    // window only the one it was opened for, anything else is denied.
    authorize_session_read(&window_registry, window.label(), &request.session_id)?;
    // Single-flight plus a 2s TTL cache keyed by session id: the main window
    // and up to six monitor windows can poll status concurrently, and each
    // probe shells out to Apple `container`, so same-session polls coalesce
    // into one subprocess (same local-DoS posture as `get_managed_sessions`).
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
                title: "Repair stale session marker".to_string(),
                detail:
                    "RunHaven still has this session marker, but the container no longer exists."
                        .to_string(),
                remedy:
                    "Refresh sessions or repair the marker before starting another session with this record."
                        .to_string(),
                repair_available: true,
            })
        }
        "running" | "starting" | "stop-requested" | "kill-requested" => {
            Some(ManagedSessionRecovery {
                status: "attention".to_string(),
                title: "Runtime status unavailable".to_string(),
                detail:
                    "RunHaven can still see this session record, but Apple container status is unavailable."
                        .to_string(),
                remedy:
                    "Refresh sessions or open diagnostics. Use session controls only after confirming the session still needs action."
                        .to_string(),
                repair_available: false,
            })
        }
        "failed" => Some(ManagedSessionRecovery {
            status: "attention".to_string(),
            title: "Run ended with an error".to_string(),
            detail: "This session is no longer running.".to_string(),
            remedy: "Review output or diagnostics. Use session controls only if the session still appears active."
                .to_string(),
            repair_available: false,
        }),
        "finished" | "stopped" => Some(ManagedSessionRecovery {
            status: "history".to_string(),
            title: "Run no longer active".to_string(),
            detail: "This session is from run history, so live runtime status is no longer available."
                .to_string(),
            remedy: "Review output or start a new run.".to_string(),
            repair_available: false,
        }),
        _ => Some(ManagedSessionRecovery {
            status: "unknown".to_string(),
            title: "Runtime status unavailable".to_string(),
            detail: "RunHaven cannot read live status for this session.".to_string(),
            remedy: "Refresh sessions or open diagnostics before taking action.".to_string(),
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
