use tauri::AppHandle;
use tauri_plugin_dialog::MessageDialogKind;

use crate::commands::dialog::confirm_native;

/// The four worktree review lifecycle actions a user can take on a finished
/// RunHaven-owned worktree run. Merge and discard mutate on-disk git state;
/// keep and recover do not, but every action is a deliberate per-run decision
/// on a backend-owned worktree, so all four resolve the run in Rust, validate
/// RunHaven ownership, and require a native confirmation before proceeding.
#[derive(Clone, Copy)]
pub(super) enum WorktreeReviewAction {
    Keep,
    Merge,
    Recover,
    Discard,
}

impl WorktreeReviewAction {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Keep => "keep",
            Self::Merge => "merge",
            Self::Recover => "recover",
            Self::Discard => "discard",
        }
    }

    pub(super) fn status(self) -> &'static str {
        match self {
            Self::Keep => "kept",
            Self::Merge => "merged",
            Self::Recover => "recovered",
            Self::Discard => "discarded",
        }
    }

    /// True when the action changes on-disk git state. Only these clear the
    /// session's worktree review availability after they succeed.
    pub(super) fn mutates(self) -> bool {
        matches!(self, Self::Merge | Self::Discard)
    }

    pub(super) fn confirm_error(self) -> &'static str {
        match self {
            Self::Keep => "Confirm Save before RunHaven keeps the protected copy.",
            Self::Merge => "Confirm Apply changes before RunHaven changes your project.",
            Self::Recover => "Confirm Show recovery steps before RunHaven opens them.",
            Self::Discard => {
                "Confirm Discard changes before RunHaven permanently deletes the protected copy."
            }
        }
    }

    pub(super) fn canceled_error(self) -> &'static str {
        match self {
            Self::Keep => "Save canceled. Nothing changed.",
            Self::Merge => "Apply changes canceled. Nothing changed.",
            Self::Recover => "Recovery steps canceled. Nothing changed.",
            Self::Discard => "Discard changes canceled. Nothing changed.",
        }
    }

    fn dialog_title(self) -> &'static str {
        match self {
            Self::Keep => "Save changes for later?",
            Self::Merge => "Apply changes to your project?",
            Self::Recover => "Show recovery steps?",
            Self::Discard => "Discard assistant changes?",
        }
    }

    fn dialog_button(self) -> &'static str {
        match self {
            Self::Keep => "Save",
            Self::Merge => "Apply",
            Self::Recover => "Show steps",
            Self::Discard => "Discard",
        }
    }

    fn dialog_kind(self) -> MessageDialogKind {
        match self {
            // Discard deletes a branch; merge rewrites the source branch.
            Self::Merge | Self::Discard => MessageDialogKind::Warning,
            Self::Keep | Self::Recover => MessageDialogKind::Info,
        }
    }

    fn dialog_message(self) -> &'static str {
        match self {
            Self::Keep => {
                "RunHaven will keep the assistant’s protected copy for later. Your project will not change."
            }
            Self::Merge => {
                "RunHaven will add the assistant’s changes to your project, then delete the protected copy. If RunHaven finds a conflict, it will keep the copy so you can view recovery steps."
            }
            Self::Recover => {
                "RunHaven will show how to open the saved changes yourself. It won’t change or delete files."
            }
            Self::Discard => {
                "RunHaven will permanently delete the assistant’s changes and the protected copy. Your project will not change."
            }
        }
    }
}

pub(super) async fn trusted_worktree_action_confirmed(
    app: AppHandle,
    action: WorktreeReviewAction,
) -> Result<bool, String> {
    confirm_native(
        app,
        action.dialog_title(),
        action.dialog_button(),
        action.dialog_kind(),
        action.dialog_message().to_string(),
    )
    .await
}

#[cfg(test)]
pub(crate) fn worktree_confirmation_messages_for_test() -> Vec<&'static str> {
    [
        WorktreeReviewAction::Keep,
        WorktreeReviewAction::Merge,
        WorktreeReviewAction::Recover,
        WorktreeReviewAction::Discard,
    ]
    .into_iter()
    .map(WorktreeReviewAction::dialog_message)
    .collect()
}
