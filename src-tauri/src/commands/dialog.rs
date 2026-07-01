use tauri::AppHandle;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

/// The single native OK/Cancel confirmation behind every trusted command gate.
/// Runs the blocking system dialog off the async runtime and returns whether the
/// user chose the confirm button (`true`) or canceled (`false`). Each caller
/// keeps its own title, confirm-button label, severity kind, and message
/// wording; only the dialog plumbing lives here so the confirmation UX cannot be
/// changed by editing one site.
pub(crate) async fn confirm_native(
    app: AppHandle,
    title: &str,
    confirm_label: &str,
    kind: MessageDialogKind,
    message: String,
) -> Result<bool, String> {
    let title = title.to_string();
    let confirm_label = confirm_label.to_string();
    tauri::async_runtime::spawn_blocking(move || {
        app.dialog()
            .message(message)
            .title(title)
            .kind(kind)
            .buttons(MessageDialogButtons::OkCancelCustom(
                confirm_label,
                "Cancel".to_string(),
            ))
            .blocking_show()
    })
    .await
    .map_err(|error| format!("could not show confirmation dialog: {error}"))
}
