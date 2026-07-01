use runhaven_core::diagnostics::auth_status_payload;
use runhaven_core::runtime::active::read_active_run_records;
use runhaven_core::ui_contracts::{
    ManagedSessionInputData, ManagedSessionListData, ManagedSessionSummaryData,
};

use super::validation::{MAX_RUN_ID_LEN, validate_text_len};
use crate::contracts::{
    LogSnapshotEntry, LogSnapshotRequest, LogSnapshotResponse, ManagedSessionLogSnapshotRequest,
    ManagedSessionLogSnapshotResponse, SessionLogEntry, SessionLogRedaction,
    WriteSessionInputRequest, WriteSessionInputResponse,
};
use crate::session_input::SessionInputRegistry;
use crate::session_output::SessionOutputRegistry;

const RAW_OUTPUT_REDACTION_MESSAGE: &str = "Raw output is not automatically redacted.";
const MAX_SESSION_INPUT_BYTES: usize = 8 * 1024;

#[tauri::command]
pub(crate) fn get_managed_sessions(
    input_registry: tauri::State<'_, SessionInputRegistry>,
) -> Result<ManagedSessionListData, String> {
    get_managed_sessions_with_input(Some(input_registry.inner()))
}

fn get_managed_sessions_with_input(
    input_registry: Option<&SessionInputRegistry>,
) -> Result<ManagedSessionListData, String> {
    managed_session_list(input_registry)
}

#[cfg(test)]
pub(crate) fn get_managed_sessions_for_test(
    input_registry: Option<&SessionInputRegistry>,
) -> Result<ManagedSessionListData, String> {
    get_managed_sessions_with_input(input_registry)
}

#[tauri::command]
pub(crate) fn get_managed_session_log_snapshot(
    request: ManagedSessionLogSnapshotRequest,
    output_registry: tauri::State<'_, SessionOutputRegistry>,
) -> Result<ManagedSessionLogSnapshotResponse, String> {
    get_managed_session_log_snapshot_with_output(request, Some(output_registry.inner()))
}

fn get_managed_session_log_snapshot_with_output(
    request: ManagedSessionLogSnapshotRequest,
    output_registry: Option<&SessionOutputRegistry>,
) -> Result<ManagedSessionLogSnapshotResponse, String> {
    let (session, run_id) = managed_session_for_id(&request.session_id)?;
    let snapshot_request = LogSnapshotRequest {
        run_id: run_id.clone(),
        lines: request.lines,
        confirm_sensitive_output: request.confirm_sensitive_output,
    };
    let lines = crate::commands::log_snapshot::validated_lines(&snapshot_request)?;
    let container_snapshot = crate::commands::log_snapshot::get_log_snapshot(LogSnapshotRequest {
        run_id,
        lines: request.lines,
        confirm_sensitive_output: request.confirm_sensitive_output,
    });
    let snapshot = match container_snapshot {
        Ok(snapshot) => snapshot,
        Err(container_error) => {
            if let Some(output_registry) = output_registry {
                if let Some(snapshot) = output_registry.snapshot(&session.run_id, lines)? {
                    snapshot
                } else {
                    return Err(container_error);
                }
            } else {
                return Err(container_error);
            }
        }
    };
    Ok(managed_session_log_snapshot_response(session, snapshot))
}

#[cfg(test)]
pub(crate) fn get_managed_session_log_snapshot_for_test(
    request: ManagedSessionLogSnapshotRequest,
) -> Result<ManagedSessionLogSnapshotResponse, String> {
    get_managed_session_log_snapshot_with_output(request, None)
}

#[cfg(test)]
pub(crate) fn get_managed_session_log_snapshot_with_output_for_test(
    request: ManagedSessionLogSnapshotRequest,
    output_registry: &SessionOutputRegistry,
) -> Result<ManagedSessionLogSnapshotResponse, String> {
    get_managed_session_log_snapshot_with_output(request, Some(output_registry))
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
    Ok(managed_session_log_snapshot_response(session, snapshot))
}

fn managed_session_log_snapshot_response(
    session: ManagedSessionSummaryData,
    snapshot: LogSnapshotResponse,
) -> ManagedSessionLogSnapshotResponse {
    let entries = snapshot
        .entries
        .iter()
        .map(|entry| session_log_entry(&session, &snapshot, entry))
        .collect();

    ManagedSessionLogSnapshotResponse {
        session,
        cursor: snapshot.cursor.clone(),
        entries,
        snapshot,
    }
}

fn session_log_entry(
    session: &ManagedSessionSummaryData,
    snapshot: &LogSnapshotResponse,
    entry: &LogSnapshotEntry,
) -> SessionLogEntry {
    SessionLogEntry {
        session_id: session.id.clone(),
        run_id: session.run_id.clone(),
        source: entry.source.clone(),
        cursor: snapshot.cursor.clone(),
        timestamp: entry.timestamp.clone(),
        text: entry.text.clone(),
        line_count: entry.line_count,
        redaction: SessionLogRedaction {
            status: "notApplied".to_string(),
            message: RAW_OUTPUT_REDACTION_MESSAGE.to_string(),
        },
    }
}

fn managed_session_list(
    input_registry: Option<&SessionInputRegistry>,
) -> Result<ManagedSessionListData, String> {
    let active_runs = read_active_run_records();
    let mut sessions = ManagedSessionListData::from_active_runs_and_auth_status(
        active_runs,
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
        if input_registry.has_writer(&session.id, &session.run_id)? {
            session.input = ManagedSessionInputData::attached();
        }
    }
    Ok(())
}

pub(crate) fn managed_session_for_id(
    session_id: &str,
) -> Result<(ManagedSessionSummaryData, String), String> {
    validate_text_len("session id", session_id, MAX_RUN_ID_LEN)?;
    if session_id.is_empty() {
        return Err("managed session id is required".to_string());
    }
    let session = managed_session_list(None)?
        .sessions
        .into_iter()
        .find(|session| session.id == session_id)
        .ok_or_else(|| "managed session not found".to_string())?;
    let run_id = session.run_id.clone();
    Ok((session, run_id))
}

#[tauri::command]
pub(crate) fn write_session_input(
    request: WriteSessionInputRequest,
    registry: tauri::State<'_, SessionInputRegistry>,
) -> Result<WriteSessionInputResponse, String> {
    write_session_input_with_registry(request, registry.inner())
}

fn write_session_input_with_registry(
    request: WriteSessionInputRequest,
    registry: &SessionInputRegistry,
) -> Result<WriteSessionInputResponse, String> {
    validate_text_len("session id", &request.session_id, MAX_RUN_ID_LEN)?;
    validate_session_input_request(&request)?;

    let (mut session, run_id) = managed_session_for_id(&request.session_id)?;
    if session.next_action.kind == "auth_attention" {
        return Err(
            "Session auth is not ready. Finish auth setup before sending input.".to_string(),
        );
    }
    if session.next_action.kind != "monitor" {
        return Err(format!(
            "Session cannot accept input while action is {}.",
            session.next_action.kind
        ));
    }

    let accepted_bytes = registry.write(&session.id, &run_id, &request.input)?;
    if accepted_bytes > request.input.len() {
        return Err("session input write reported more bytes than were sent".to_string());
    }
    session.input = ManagedSessionInputData::attached();

    Ok(WriteSessionInputResponse {
        cursor: format!("session:{}:input:{accepted_bytes}", session.id),
        session,
        accepted: true,
        accepted_bytes,
        status: "accepted".to_string(),
    })
}

#[cfg(test)]
pub(crate) fn write_session_input_for_test(
    request: WriteSessionInputRequest,
    registry: &SessionInputRegistry,
) -> Result<WriteSessionInputResponse, String> {
    write_session_input_with_registry(request, registry)
}

fn validate_session_input_request(request: &WriteSessionInputRequest) -> Result<(), String> {
    if !request.confirm_send {
        return Err("Confirm the send before sending input.".to_string());
    }
    if request.input.is_empty() {
        return Err("session input is required".to_string());
    }
    if request.input.len() > MAX_SESSION_INPUT_BYTES {
        return Err(format!(
            "session input is too long; keep each send at or below {MAX_SESSION_INPUT_BYTES} bytes"
        ));
    }
    if request.input.contains('\0') {
        return Err("session input contains a NUL byte".to_string());
    }
    if contains_terminal_escape(&request.input) {
        return Err("session input contains a terminal escape sequence".to_string());
    }

    let expected_target = format!("session:{}", request.session_id);
    if request.client_target != expected_target {
        return Err(
            "Selected session changed. Choose the session again before sending input.".to_string(),
        );
    }

    Ok(())
}

fn contains_terminal_escape(input: &str) -> bool {
    input.chars().any(|ch| matches!(ch, '\u{1b}' | '\u{9b}'))
}
