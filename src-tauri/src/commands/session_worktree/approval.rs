use runhaven_core::ui_contracts::ManagedSessionSummaryData;
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
            Self::Keep => {
                "Confirm keeping this worktree before RunHaven leaves the branch in place."
            }
            Self::Merge => "Confirm the merge before RunHaven merges this worktree branch.",
            Self::Recover => "Confirm the review before RunHaven shows recovery steps.",
            Self::Discard => "Confirm the discard before RunHaven deletes this worktree branch.",
        }
    }

    pub(super) fn canceled_error(self) -> &'static str {
        match self {
            Self::Keep => "Keep canceled; the worktree was left unchanged.",
            Self::Merge => "Merge canceled before changing this worktree.",
            Self::Recover => "Recovery review canceled.",
            Self::Discard => "Discard canceled; the worktree was left in place.",
        }
    }

    fn dialog_title(self) -> &'static str {
        match self {
            Self::Keep => "Confirm RunHaven keep",
            Self::Merge => "Confirm RunHaven merge",
            Self::Recover => "Confirm RunHaven recovery review",
            Self::Discard => "Confirm RunHaven discard",
        }
    }

    fn dialog_button(self) -> &'static str {
        match self {
            Self::Keep => "Keep",
            Self::Merge => "Merge",
            Self::Recover => "Show steps",
            Self::Discard => "Discard",
        }
    }

    fn dialog_label(self) -> &'static str {
        match self {
            Self::Keep => "Keep worktree",
            Self::Merge => "Merge worktree",
            Self::Recover => "Recovery review",
            Self::Discard => "Discard worktree",
        }
    }

    fn dialog_kind(self) -> MessageDialogKind {
        match self {
            // Discard deletes a branch; merge rewrites the source branch.
            Self::Merge | Self::Discard => MessageDialogKind::Warning,
            Self::Keep | Self::Recover => MessageDialogKind::Info,
        }
    }

    fn dialog_effect(self) -> &'static str {
        match self {
            Self::Keep => "leave this worktree and branch in place for manual review",
            Self::Merge => {
                "merge this worktree branch into the source branch, then remove the worktree"
            }
            Self::Recover => "show the safe manual recovery steps for this worktree",
            Self::Discard => "delete the recorded worktree and branch for this finished session",
        }
    }
}

pub(super) async fn trusted_worktree_action_confirmed(
    app: AppHandle,
    action: WorktreeReviewAction,
    session: &ManagedSessionSummaryData,
    run_id: &str,
    branch: &str,
) -> Result<bool, String> {
    let message = format!(
        "Action: {}\nSession: {}\nRun ID: {}\nAgent: {}\nProject: {}\nBranch: {}\n\nRunHaven will {} after you confirm.",
        action.dialog_label(),
        session.title,
        run_id,
        session.agent,
        session.project,
        branch,
        action.dialog_effect(),
    );
    confirm_native(
        app,
        action.dialog_title(),
        action.dialog_button(),
        action.dialog_kind(),
        message,
    )
    .await
}
