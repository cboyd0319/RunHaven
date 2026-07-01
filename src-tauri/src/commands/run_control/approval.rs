use runhaven_core::ui_contracts::ManagedSessionSummaryData;
use tauri::AppHandle;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

#[derive(Clone, Copy)]
pub(super) enum RunControlAction {
    Stop,
    Kill,
    Repair,
}

impl RunControlAction {
    pub(super) fn confirm_error(self) -> &'static str {
        match self {
            Self::Stop => "Confirm the stop before stopping this run.",
            Self::Kill => "Confirm the hard-stop before killing this run.",
            Self::Repair => "Confirm the repair before clearing a stale marker.",
        }
    }

    pub(super) fn canceled_error(self) -> &'static str {
        match self {
            Self::Stop => "Stop canceled before changing this run.",
            Self::Kill => "Hard-stop canceled before changing this run.",
            Self::Repair => "Repair canceled before clearing the stale marker.",
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
            Self::Stop => "stop this active run",
            Self::Kill => "hard-stop this active run",
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
    let title = action.dialog_title().to_string();
    let confirm_button = action.dialog_button().to_string();
    let message = trusted_run_control_message(action, target);
    tauri::async_runtime::spawn_blocking(move || {
        app.dialog()
            .message(message)
            .title(title)
            .kind(MessageDialogKind::Warning)
            .buttons(MessageDialogButtons::OkCancelCustom(
                confirm_button,
                "Cancel".to_string(),
            ))
            .blocking_show()
    })
    .await
    .map_err(|error| format!("could not show trusted run-control confirmation: {error}"))
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
