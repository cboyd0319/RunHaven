use tauri::AppHandle;
use tauri_plugin_dialog::MessageDialogKind;

use crate::commands::dialog::confirm_native;

pub(super) const TITLE: &str = "Create a support report?";
pub(super) const BUTTON: &str = "Create report";
pub(super) const MESSAGE: &str = "RunHaven will create one text file that only your macOS account can read. It will open a public GitHub support form and select the file in Finder. Nothing is uploaded or submitted automatically. If you attach the file, GitHub uploads it before you submit the issue.\n\nIncluded: RunHaven and macOS versions, Apple container and setup results, and recent RunHaven action names, times, outcomes, durations, and exit codes.\n\nNot included: raw logs, terminal output, prompts, project names or paths, project files or changes, environment values, credentials, account identifiers, or browser data.\n\nReview the file before you attach it.";

pub(super) async fn trusted_support_report_confirmed(app: AppHandle) -> Result<bool, String> {
    confirm_native(
        app,
        TITLE,
        BUTTON,
        MessageDialogKind::Info,
        MESSAGE.to_string(),
    )
    .await
    .map_err(|_| {
        "RunHaven could not show the support report confirmation. No report was created."
            .to_string()
    })
}

#[cfg(test)]
pub(crate) fn confirmation_copy_for_test() -> (&'static str, &'static str, &'static str) {
    (TITLE, BUTTON, MESSAGE)
}
