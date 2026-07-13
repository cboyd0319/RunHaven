use runhaven_core::ui_contracts::{
    ManagedSessionInputData, ManagedSessionSummaryData, strip_terminal_controls,
};
use tauri::AppHandle;
use tauri_plugin_dialog::MessageDialogKind;

use super::dialog::confirm_native;
use super::validation::{MAX_RUN_ID_LEN, validate_text_len};
use crate::contracts::{WriteSessionInputRequest, WriteSessionInputResponse};
use crate::session_input::{MAX_SESSION_INPUT_BYTES, SessionInputRegistry};

const MAX_CONFIRMATION_PREVIEW_SCALARS: usize = 240;
const MAX_CONFIRMATION_PREVIEW_LINES: usize = 6;

#[tauri::command]
pub(crate) async fn write_session_input(
    app: AppHandle,
    request: WriteSessionInputRequest,
    registry: tauri::State<'_, SessionInputRegistry>,
) -> Result<WriteSessionInputResponse, String> {
    let registry = registry.inner().clone();
    let _flight = registry.begin_input_send()?;
    let prepared = prepare_session_input(request, &registry)?;
    let message =
        session_input_confirmation_message(&prepared.session.title, &prepared.request.input);
    if !trusted_session_input_approved(app, message).await? {
        return Err("Message send canceled. Nothing was sent.".to_string());
    }
    let prepared = revalidate_prepared_session_input(prepared, &registry)?;
    write_prepared_session_input(prepared, &registry)
}

struct PreparedSessionInput {
    request: WriteSessionInputRequest,
    session: ManagedSessionSummaryData,
    run_id: String,
    writer_id: u64,
}

fn prepare_session_input(
    request: WriteSessionInputRequest,
    registry: &SessionInputRegistry,
) -> Result<PreparedSessionInput, String> {
    validate_text_len("session id", &request.session_id, MAX_RUN_ID_LEN).map_err(|_| {
        "This session ended, so it can’t receive messages. Start a new session to continue."
            .to_string()
    })?;
    validate_session_input_request(&request)?;

    let (session, run_id) =
        super::sessions::managed_active_session_for_id_with_input(&request.session_id, registry)
            .map_err(|_| {
                "This session ended, so it can’t receive messages. Start a new session to continue."
                    .to_string()
            })?;
    if session.next_action.kind == "auth_attention" {
        return Err("Finish signing in before you send a message.".to_string());
    }
    if session.next_action.kind != "monitor" {
        return Err(
            "This session can’t receive messages now. Check its status before you try again."
                .to_string(),
        );
    }
    let writer_id = registry
        .writer_identity(&session.id, &run_id)
        .map_err(|_| {
            "This session can’t receive messages now. Check its status before you try again."
                .to_string()
        })?;

    Ok(PreparedSessionInput {
        request,
        session,
        run_id,
        writer_id,
    })
}

fn revalidate_prepared_session_input(
    prepared: PreparedSessionInput,
    registry: &SessionInputRegistry,
) -> Result<PreparedSessionInput, String> {
    let expected_run_id = prepared.run_id.clone();
    let expected_writer_id = prepared.writer_id;
    let current = prepare_session_input(prepared.request, registry)?;
    if current.run_id != expected_run_id || current.writer_id != expected_writer_id {
        return Err(
            "The session changed before RunHaven could send the message. Nothing was sent. Check the session status, then try again."
                .to_string(),
        );
    }
    Ok(current)
}

fn write_prepared_session_input(
    prepared: PreparedSessionInput,
    registry: &SessionInputRegistry,
) -> Result<WriteSessionInputResponse, String> {
    let PreparedSessionInput {
        request,
        mut session,
        run_id,
        writer_id,
    } = prepared;
    let accepted_bytes = registry.write_exact(&session.id, &run_id, writer_id, &request.input)?;
    if accepted_bytes > request.input.len() {
        return Err(
            "RunHaven couldn’t send this message. Nothing was sent. Try again.".to_string(),
        );
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
    let _flight = registry.begin_input_send()?;
    let prepared = prepare_session_input(request, registry)?;
    write_prepared_session_input(prepared, registry)
}

#[cfg(test)]
pub(crate) fn write_session_input_with_approval_for_test(
    request: WriteSessionInputRequest,
    registry: &SessionInputRegistry,
    approve: impl FnOnce(&ManagedSessionSummaryData, &str) -> Result<bool, String>,
) -> Result<WriteSessionInputResponse, String> {
    let _flight = registry.begin_input_send()?;
    let prepared = prepare_session_input(request, registry)?;
    if !approve(&prepared.session, &prepared.run_id)? {
        return Err("Message send canceled. Nothing was sent.".to_string());
    }
    let prepared = revalidate_prepared_session_input(prepared, registry)?;
    write_prepared_session_input(prepared, registry)
}

async fn trusted_session_input_approved(app: AppHandle, message: String) -> Result<bool, String> {
    confirm_native(
        app,
        "Send this message?",
        "Send",
        MessageDialogKind::Warning,
        message,
    )
    .await
}

#[cfg(test)]
pub(crate) fn session_input_confirmation_message_for_test(target: &str, input: &str) -> String {
    session_input_confirmation_message(target, input)
}

fn session_input_confirmation_message(target: &str, input: &str) -> String {
    let preview = bounded_confirmation_preview(input);
    let (heading, closing) = if preview.shortened {
        (
            "Message preview (shortened):",
            "RunHaven will send the full message exactly as entered, including text not shown in this preview. The assistant can act on it in this session.",
        )
    } else {
        (
            "Message:",
            "RunHaven will send this exact message to the running assistant. The assistant can act on it in this session.",
        )
    };
    format!(
        "To: {}\nMessage size: {} bytes\n\n{heading}\n{}\n\n{closing}",
        safe_confirmation_target(target),
        input.len(),
        preview.text,
    )
}

fn safe_confirmation_target(target: &str) -> String {
    let controls_as_spaces = target
        .chars()
        .map(|character| {
            if character.is_control() || is_unsafe_message_character(character) {
                ' '
            } else {
                character
            }
        })
        .collect::<String>();
    let one_line = strip_terminal_controls(&controls_as_spaces)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if one_line.is_empty() {
        return "Selected session".to_string();
    }
    one_line.chars().take(120).collect()
}

struct ConfirmationPreview {
    text: String,
    shortened: bool,
}

fn bounded_confirmation_preview(input: &str) -> ConfirmationPreview {
    let lines = input.split('\n').collect::<Vec<_>>();
    let shortened = lines.len() > MAX_CONFIRMATION_PREVIEW_LINES
        || input.chars().count() > MAX_CONFIRMATION_PREVIEW_SCALARS;
    if !shortened {
        return ConfirmationPreview {
            text: input.to_string(),
            shortened: false,
        };
    }
    let (head_source, tail_source) = if lines.len() > 4 {
        (lines[..2].join("\n"), lines[lines.len() - 2..].join("\n"))
    } else {
        (input.to_string(), input.to_string())
    };
    ConfirmationPreview {
        text: bounded_head_tail(&head_source, &tail_source, MAX_CONFIRMATION_PREVIEW_SCALARS),
        shortened: true,
    }
}

fn bounded_head_tail(head_source: &str, tail_source: &str, max_scalars: usize) -> String {
    const MARKER: &str = "\n[middle omitted]\n";
    let marker_len = MARKER.chars().count();
    let head_len = (max_scalars - marker_len) / 2;
    let tail_len = max_scalars - head_len - marker_len;
    let head = head_source.chars().take(head_len).collect::<String>();
    let mut tail = tail_source.chars().rev().take(tail_len).collect::<Vec<_>>();
    tail.reverse();
    format!("{head}{MARKER}{}", tail.into_iter().collect::<String>())
}

fn validate_session_input_request(request: &WriteSessionInputRequest) -> Result<(), String> {
    if request.input.is_empty() {
        return Err("Enter a message before you choose Send.".to_string());
    }
    if request.input.len() > MAX_SESSION_INPUT_BYTES {
        return Err(format!(
            "This message is too long. Shorten it to {MAX_SESSION_INPUT_BYTES} bytes or less, then try again."
        ));
    }
    if contains_unsafe_message_formatting(&request.input) {
        return Err(
            "This message contains hidden formatting that RunHaven can’t send safely. Paste it again as plain text, then try again."
                .to_string(),
        );
    }

    Ok(())
}

fn contains_unsafe_message_formatting(input: &str) -> bool {
    input.chars().any(is_unsafe_message_character)
}

fn is_unsafe_message_character(character: char) -> bool {
    (character != '\n' && character.is_control())
        || matches!(
            character,
            '\u{00ad}'
                | '\u{034f}'
                | '\u{061c}'
                | '\u{180e}'
                | '\u{200b}'..='\u{200f}'
                | '\u{2028}'..='\u{202e}'
                | '\u{2060}'..='\u{2069}'
                | '\u{feff}'
        )
}
