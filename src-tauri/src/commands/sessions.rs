use runhaven_core::diagnostics::auth_status_payload;
use runhaven_core::runtime::active::read_active_run_records;
use runhaven_core::ui_contracts::{ManagedSessionListData, ManagedSessionSummaryData};

use super::validation::{MAX_RUN_ID_LEN, validate_text_len};
use crate::contracts::{
    LogSnapshotRequest, ManagedSessionLogSnapshotRequest, ManagedSessionLogSnapshotResponse,
};

#[tauri::command]
pub(crate) fn get_managed_sessions() -> Result<ManagedSessionListData, String> {
    Ok(managed_session_list())
}

#[tauri::command]
pub(crate) fn get_managed_session_log_snapshot(
    request: ManagedSessionLogSnapshotRequest,
) -> Result<ManagedSessionLogSnapshotResponse, String> {
    let (session, run_id) = managed_session_for_id(&request.session_id)?;
    let snapshot = crate::commands::log_snapshot::get_log_snapshot(LogSnapshotRequest {
        run_id,
        lines: request.lines,
        confirm_sensitive_output: request.confirm_sensitive_output,
    })?;
    Ok(ManagedSessionLogSnapshotResponse { session, snapshot })
}

#[cfg(test)]
pub(crate) fn managed_session_log_snapshot_response_for_test(
    request: ManagedSessionLogSnapshotRequest,
    stdout: &[u8],
    max_bytes: usize,
) -> Result<ManagedSessionLogSnapshotResponse, String> {
    let (session, run_id) = managed_session_for_id(&request.session_id)?;
    let snapshot_request = LogSnapshotRequest {
        run_id: run_id.clone(),
        lines: request.lines,
        confirm_sensitive_output: request.confirm_sensitive_output,
    };
    let lines = crate::commands::log_snapshot::validated_lines(&snapshot_request)?;
    let snapshot =
        crate::commands::log_snapshot::log_snapshot_response(&run_id, lines, stdout, max_bytes)?;
    Ok(ManagedSessionLogSnapshotResponse { session, snapshot })
}

fn managed_session_list() -> ManagedSessionListData {
    let active_runs = read_active_run_records();
    ManagedSessionListData::from_active_runs_and_auth_status(active_runs, auth_status_payload())
}

pub(crate) fn managed_session_for_id(
    session_id: &str,
) -> Result<(ManagedSessionSummaryData, String), String> {
    validate_text_len("session id", session_id, MAX_RUN_ID_LEN)?;
    if session_id.is_empty() {
        return Err("managed session id is required".to_string());
    }
    let session = managed_session_list()
        .sessions
        .into_iter()
        .find(|session| session.id == session_id)
        .ok_or_else(|| "managed session not found".to_string())?;
    let run_id = session.run_id.clone();
    Ok((session, run_id))
}
