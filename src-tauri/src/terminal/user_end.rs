//! Explicit, natively confirmed user request to end one live terminal.

use tauri::{AppHandle, State};
use tauri_plugin_dialog::MessageDialogKind;

use super::TerminalSessionRegistry;
use crate::commands::dialog::confirm_native;

const END_TERMINAL_UNAVAILABLE: &str =
    "This terminal is no longer available. Check the session, then try again.";
const END_TERMINAL_MESSAGE: &str = "RunHaven will end this live terminal session. The assistant may lose unsaved work. Cancel to keep the terminal running.";

struct UserEndTarget<'a> {
    session_id: &'a str,
    generation: u64,
}

fn prepare_user_end<'a>(
    registry: &TerminalSessionRegistry,
    session_id: &'a str,
) -> Result<UserEndTarget<'a>, String> {
    let generation = registry
        .live_generation(session_id)
        .map_err(|_| END_TERMINAL_UNAVAILABLE.to_string())?;
    Ok(UserEndTarget {
        session_id,
        generation,
    })
}

fn finish_user_end(
    registry: &TerminalSessionRegistry,
    target: UserEndTarget<'_>,
) -> Result<bool, String> {
    registry
        .close_live_generation(target.session_id, target.generation)
        .map_err(|_| END_TERMINAL_UNAVAILABLE.to_string())?;
    Ok(true)
}

/// End one currently live terminal after a native Warning confirmation.
/// Cancellation is a successful no-op; stale or replaced targets fail closed.
#[tauri::command]
pub(crate) async fn end_terminal_session(
    app: AppHandle,
    registry: State<'_, TerminalSessionRegistry>,
    session_id: String,
) -> Result<bool, String> {
    let target = prepare_user_end(&registry, &session_id)?;
    if !confirm_user_end(app).await? {
        return Ok(false);
    }
    finish_user_end(&registry, target)
}

async fn confirm_user_end(app: AppHandle) -> Result<bool, String> {
    confirm_native(
        app,
        "End this terminal session?",
        "End Session",
        MessageDialogKind::Warning,
        END_TERMINAL_MESSAGE.to_string(),
    )
    .await
}

#[cfg(test)]
pub(super) fn end_terminal_session_with_approval_for_test(
    registry: &TerminalSessionRegistry,
    session_id: &str,
    approve: impl FnOnce(&str) -> Result<bool, String>,
) -> Result<bool, String> {
    let target = prepare_user_end(registry, session_id)?;
    if !approve(session_id)? {
        return Ok(false);
    }
    finish_user_end(registry, target)
}
