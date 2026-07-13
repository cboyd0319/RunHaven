use runhaven_core::doctor::PINNED_APPLE_CONTAINER_VERSION;
use tauri::AppHandle;
use tauri_plugin_dialog::MessageDialogKind;

use super::CONFIRM_FAILED;
use crate::commands::dialog::confirm_native;
use crate::contracts::SetupActionId;

pub(super) async fn trusted_setup_recovery_confirmed(
    app: AppHandle,
    action: SetupActionId,
) -> Result<bool, String> {
    match action {
        SetupActionId::OpenAppleContainerInstaller => confirm_native(
            app,
            "Download Apple container?",
            "Download",
            MessageDialogKind::Info,
            format!(
                "RunHaven will open Apple’s signed Apple container {PINNED_APPLE_CONTAINER_VERSION} installer in your web browser. If a different version is already installed, review Apple’s replacement warning before you continue. Open the downloaded package and follow Apple’s instructions. macOS may ask for your administrator password."
            ),
        )
        .await
        .map_err(|_| CONFIRM_FAILED.to_string()),
        SetupActionId::StartContainerSystem => confirm_native(
            app,
            "Start the container service?",
            "Start",
            MessageDialogKind::Info,
            "RunHaven will start Apple container on this Mac. The first start may install Apple system components. This doesn’t change RunHaven’s security settings."
                .to_string(),
        )
        .await
        .map_err(|_| CONFIRM_FAILED.to_string()),
        SetupActionId::RestartContainerSystem => confirm_native(
            app,
            "Restart the container service?",
            "Restart",
            MessageDialogKind::Warning,
            restart_system_confirmation_message().to_string(),
        )
        .await
        .map_err(|_| CONFIRM_FAILED.to_string()),
    }
}

fn restart_system_confirmation_message() -> &'static str {
    "RunHaven will stop and restart Apple container. Current RunHaven sessions will end. Apple containers running outside RunHaven will also stop. This often fixes internet access in new sessions."
}

#[cfg(test)]
pub(crate) fn restart_system_confirmation_message_for_test() -> &'static str {
    restart_system_confirmation_message()
}
