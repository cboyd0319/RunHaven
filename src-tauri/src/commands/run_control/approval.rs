use runhaven_core::ui_contracts::ManagedSessionSummaryData;
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
            Self::Kill => "Confirm the hard-stop before killing this session.",
            Self::Repair => "Confirm the repair before clearing a stale marker.",
        }
    }

    pub(super) fn canceled_error(self) -> &'static str {
        match self {
            Self::Stop => "Stop canceled before changing this session.",
            Self::Kill => "Hard-stop canceled before changing this session.",
            Self::Repair => "Repair canceled before clearing the stale marker.",
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
        match self {
            Self::Stop => "stop payload is missing return_code",
            Self::Kill => "kill payload is missing return_code",
            Self::Repair => "repair payload is missing return_code",
        }
        .to_string()
    }

    /// Error when a stop/kill container command exits non-zero. The core payload
    /// names the container, so the message stays fixed and path-free.
    pub(super) fn nonzero_exit_error(
        self,
        run_id: &str,
        container_name: &str,
        return_code: i64,
    ) -> String {
        match self {
            Self::Stop => format!(
                "could not stop run {run_id} ({container_name}); container stop exited {return_code}"
            ),
            Self::Kill => format!(
                "could not hard-stop run {run_id} ({container_name}); container kill exited {return_code}"
            ),
            Self::Repair => {
                format!("could not repair run {run_id} ({container_name}); exited {return_code}")
            }
        }
    }

    fn dialog_title(self) -> &'static str {
        match self {
            Self::Stop => "Confirm RunHaven stop",
            Self::Kill => "Confirm RunHaven hard-stop",
            Self::Repair => "Confirm RunHaven repair",
        }
    }

    fn dialog_button(self) -> &'static str {
        match self {
            Self::Stop => "Stop",
            Self::Kill => "Hard stop",
            Self::Repair => "Repair",
        }
    }

    fn dialog_label(self) -> &'static str {
        match self {
            Self::Stop => "Stop",
            Self::Kill => "Hard stop",
            Self::Repair => "Repair",
        }
    }

    fn dialog_effect(self) -> &'static str {
        match self {
            Self::Stop => "stop this active session",
            Self::Kill => "hard-stop this active session",
            Self::Repair => "clear this stale active marker",
        }
    }
}

pub(super) enum RunControlTarget<'a> {
    Run {
        run_id: &'a str,
    },
    ManagedSession {
        session: &'a ManagedSessionSummaryData,
        run_id: &'a str,
    },
}

pub(super) async fn trusted_run_control_confirmed(
    app: AppHandle,
    action: RunControlAction,
    target: RunControlTarget<'_>,
) -> Result<bool, String> {
    let message = trusted_run_control_message(action, target);
    confirm_native(
        app,
        action.dialog_title(),
        action.dialog_button(),
        MessageDialogKind::Warning,
        message,
    )
    .await
}

fn trusted_run_control_message(action: RunControlAction, target: RunControlTarget<'_>) -> String {
    match target {
        RunControlTarget::Run { run_id } => format!(
            "Action: {}\nRun ID: {}\n\nRunHaven will {} after you confirm.",
            action.dialog_label(),
            run_id,
            action.dialog_effect(),
        ),
        RunControlTarget::ManagedSession { session, run_id } => format!(
            "Action: {}\nSession: {}\nRun ID: {}\nAgent: {}\nProject: {}\nStatus: {}\n\nRunHaven will {} after you confirm.",
            action.dialog_label(),
            session.title,
            run_id,
            session.agent,
            session.project,
            session.status_label,
            action.dialog_effect(),
        ),
    }
}
