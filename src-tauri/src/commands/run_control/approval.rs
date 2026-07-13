use tauri::AppHandle;
use tauri_plugin_dialog::MessageDialogKind;

use crate::commands::dialog::confirm_native;

#[derive(Clone, Copy)]
pub(super) enum RunControlAction {
    Stop,
    Kill,
    Repair,
}

impl RunControlAction {
    pub(super) fn confirm_error(self) -> &'static str {
        match self {
            Self::Stop => "Confirm the stop before stopping this session.",
            Self::Kill => "Confirm Force Stop before ending this session.",
            Self::Repair => "Confirm before removing this session from the active list.",
        }
    }

    pub(super) fn canceled_error(self) -> &'static str {
        match self {
            Self::Stop => "Stop canceled. This session was not changed.",
            Self::Kill => "Force Stop canceled. This session was not changed.",
            Self::Repair => "Remove canceled. This session was not changed.",
        }
    }

    /// The status a successful stop/kill records. Repair reads its status from the
    /// core payload instead, so its arm here is never consulted.
    pub(super) fn requested_status(self) -> &'static str {
        match self {
            Self::Stop => "stop-requested",
            Self::Kill => "kill-requested",
            Self::Repair => "removed",
        }
    }

    /// Error when a stop/kill core payload has no `return_code`.
    pub(super) fn missing_return_code_error(self) -> String {
        self.failure_message().to_string()
    }

    /// Error when a stop/kill container command exits non-zero. The core payload
    /// names the container, so the message stays fixed and path-free.
    pub(super) fn nonzero_exit_error(
        self,
        _run_id: &str,
        _container_name: &str,
        _return_code: i64,
    ) -> String {
        self.failure_message().to_string()
    }

    pub(super) fn failure_message(self) -> &'static str {
        match self {
            Self::Stop => "RunHaven couldn’t stop this session. Check its status, then try again.",
            Self::Kill => {
                "RunHaven couldn’t force stop this session. Check its status, then try again."
            }
            Self::Repair => {
                "RunHaven couldn’t remove this session from the active list. Check its status, then try again."
            }
        }
    }

    fn dialog_title(self) -> &'static str {
        match self {
            Self::Stop => "Stop this session?",
            Self::Kill => "Force stop this session?",
            Self::Repair => "Remove this inactive session?",
        }
    }

    fn dialog_button(self) -> &'static str {
        match self {
            Self::Stop => "Stop",
            Self::Kill => "Force Stop",
            Self::Repair => "Remove",
        }
    }

    fn dialog_message(self) -> &'static str {
        match self {
            Self::Stop => {
                "RunHaven will ask the assistant to finish and stop. This can take a moment."
            }
            Self::Kill => {
                "RunHaven will end this session immediately. The assistant may lose unsaved work."
            }
            Self::Repair => {
                "RunHaven will remove this session from the active list. It won’t delete project files."
            }
        }
    }
}

pub(super) async fn trusted_run_control_confirmed(
    app: AppHandle,
    action: RunControlAction,
) -> Result<bool, String> {
    confirm_native(
        app,
        action.dialog_title(),
        action.dialog_button(),
        MessageDialogKind::Warning,
        action.dialog_message().to_string(),
    )
    .await
}

#[cfg(test)]
pub(crate) fn trusted_run_control_messages_for_test() -> Vec<&'static str> {
    [
        RunControlAction::Stop,
        RunControlAction::Kill,
        RunControlAction::Repair,
    ]
    .into_iter()
    .map(RunControlAction::dialog_message)
    .collect()
}
