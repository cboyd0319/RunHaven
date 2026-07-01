use runhaven_core::ui_contracts::{ManagedSessionInputData, ManagedSessionSummaryData};
use tauri::AppHandle;
use tauri_plugin_dialog::MessageDialogKind;

use super::dialog::confirm_native;
use super::validation::{MAX_RUN_ID_LEN, validate_text_len};
use crate::contracts::{WriteSessionInputRequest, WriteSessionInputResponse};
use crate::session_input::{MAX_SESSION_INPUT_BYTES, SessionInputRegistry};

#[tauri::command]
pub(crate) async fn write_session_input(
    app: AppHandle,
    request: WriteSessionInputRequest,
    registry: tauri::State<'_, SessionInputRegistry>,
) -> Result<WriteSessionInputResponse, String> {
    let registry = registry.inner().clone();
    let prepared = prepare_session_input(request, &registry)?;
    if !registry.input_approved(&prepared.session.id, &prepared.run_id)? {
        if !trusted_session_input_approved(app, &prepared.session, &prepared.run_id).await? {
            return Err("Session input was not approved.".to_string());
        }
        registry.approve_input(&prepared.session.id, &prepared.run_id)?;
    }
    write_prepared_session_input(prepared, &registry)
}

struct PreparedSessionInput {
    request: WriteSessionInputRequest,
    session: ManagedSessionSummaryData,
    run_id: String,
}

fn prepare_session_input(
    request: WriteSessionInputRequest,
    registry: &SessionInputRegistry,
) -> Result<PreparedSessionInput, String> {
    validate_text_len("session id", &request.session_id, MAX_RUN_ID_LEN)?;
    validate_session_input_request(&request)?;

    let (session, run_id) =
        super::sessions::managed_session_for_id_with_input(&request.session_id, Some(registry))?;
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

    Ok(PreparedSessionInput {
        request,
        session,
        run_id,
    })
}

fn write_prepared_session_input(
    prepared: PreparedSessionInput,
    registry: &SessionInputRegistry,
) -> Result<WriteSessionInputResponse, String> {
    let PreparedSessionInput {
        request,
        mut session,
        run_id,
    } = prepared;
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
    let prepared = prepare_session_input(request, registry)?;
    registry.approve_input(&prepared.session.id, &prepared.run_id)?;
    write_prepared_session_input(prepared, registry)
}

#[cfg(test)]
pub(crate) fn write_session_input_with_approval_for_test(
    request: WriteSessionInputRequest,
    registry: &SessionInputRegistry,
    approve: impl FnOnce(&ManagedSessionSummaryData, &str) -> Result<bool, String>,
) -> Result<WriteSessionInputResponse, String> {
    let prepared = prepare_session_input(request, registry)?;
    if !registry.input_approved(&prepared.session.id, &prepared.run_id)? {
        if !approve(&prepared.session, &prepared.run_id)? {
            return Err("Session input was not approved.".to_string());
        }
        registry.approve_input(&prepared.session.id, &prepared.run_id)?;
    }
    write_prepared_session_input(prepared, registry)
}

async fn trusted_session_input_approved(
    app: AppHandle,
    session: &ManagedSessionSummaryData,
    run_id: &str,
) -> Result<bool, String> {
    let message = trusted_session_input_message(session, run_id);
    confirm_native(
        app,
        "Allow session input",
        "Allow",
        MessageDialogKind::Warning,
        message,
    )
    .await
}

fn trusted_session_input_message(session: &ManagedSessionSummaryData, run_id: &str) -> String {
    format!(
        "Allow messages to be sent to this session while it is running?\n\nSession: {}\nRun: {run_id}",
        session.title
    )
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
