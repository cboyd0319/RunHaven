use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

use crate::MAIN_WINDOW_LABEL;

/// The single native OK/Cancel confirmation behind every trusted command gate.
/// Parents the sheet to the main control window, runs the blocking system dialog
/// off the async runtime, and returns whether the user chose the confirm button
/// (`true`) or canceled (`false`). Each caller keeps its own title, confirm-button
/// label, severity kind, and message wording; only the dialog plumbing lives here
/// so the confirmation UX cannot be changed by editing one site.
pub(crate) async fn confirm_native(
    app: AppHandle,
    title: &str,
    confirm_label: &str,
    kind: MessageDialogKind,
    message: String,
) -> Result<bool, String> {
    let title = title.to_string();
    let confirm_label = confirm_label.to_string();
    let parent = app.get_webview_window(MAIN_WINDOW_LABEL).ok_or_else(|| {
        "RunHaven couldn’t show the confirmation. Quit and reopen RunHaven, then try again."
            .to_string()
    })?;
    let dialog = app
        .dialog()
        .message(message)
        .title(title)
        .kind(kind)
        .buttons(MessageDialogButtons::OkCancelCustom(
            confirm_label,
            "Cancel".to_string(),
        ))
        .parent(&parent);
    tauri::async_runtime::spawn_blocking(move || dialog.blocking_show())
        .await
        .map_err(|_| {
            "RunHaven couldn’t show the confirmation. Quit and reopen RunHaven, then try again."
                .to_string()
        })
}
