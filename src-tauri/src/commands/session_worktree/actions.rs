use runhaven_core::runtime::worktrees::{WorktreeLifecycle, runs_worktree_merge};
use runhaven_core::ui_contracts::ManagedSessionSummaryData;
use tauri::AppHandle;

use super::approval::{WorktreeReviewAction, trusted_worktree_action_confirmed};
use super::{
    clear_session_worktree_review, discard_owned_worktree, ensure_no_active_worktree_marker,
    require_current_review, resolve_worktree_lifecycle, review_next_steps,
};
use crate::contracts::{WorktreeActionRequest, WorktreeActionResponse, WorktreeActionResult};

#[tauri::command]
pub(crate) async fn keep_managed_session_worktree(
    app: AppHandle,
    request: WorktreeActionRequest,
) -> Result<WorktreeActionResponse, String> {
    run_worktree_action(app, request, WorktreeReviewAction::Keep).await
}

#[tauri::command]
pub(crate) async fn merge_managed_session_worktree(
    app: AppHandle,
    request: WorktreeActionRequest,
) -> Result<WorktreeActionResponse, String> {
    run_worktree_action(app, request, WorktreeReviewAction::Merge).await
}

#[tauri::command]
pub(crate) async fn recover_managed_session_worktree(
    app: AppHandle,
    request: WorktreeActionRequest,
) -> Result<WorktreeActionResponse, String> {
    run_worktree_action(app, request, WorktreeReviewAction::Recover).await
}

#[tauri::command]
pub(crate) async fn discard_managed_session_worktree(
    app: AppHandle,
    request: WorktreeActionRequest,
) -> Result<WorktreeActionResponse, String> {
    run_worktree_action(app, request, WorktreeReviewAction::Discard).await
}

/// Validate the request, resolve and RunHaven-verify the backend-owned worktree,
/// reject an active run, and require a native confirmation before any mutation.
async fn run_worktree_action(
    app: AppHandle,
    request: WorktreeActionRequest,
    action: WorktreeReviewAction,
) -> Result<WorktreeActionResponse, String> {
    authorize_worktree_action(&request, action)?;
    if !trusted_worktree_action_confirmed(app, action).await? {
        return Err(action.canceled_error().to_string());
    }
    let (session, lifecycle, review_id) = authorize_worktree_action(&request, action)?;
    execute_worktree_action(session, lifecycle, action, review_id.as_deref())
}

/// Everything that must hold before a native confirmation is shown: confirmed
/// intent, an eligible RunHaven-owned worktree, and no active run marker. No
/// mutation happens here, so denial or failure leaves the worktree untouched.
fn authorize_worktree_action(
    request: &WorktreeActionRequest,
    action: WorktreeReviewAction,
) -> Result<(ManagedSessionSummaryData, WorktreeLifecycle, Option<String>), String> {
    if !request.confirm {
        return Err(action.confirm_error().to_string());
    }
    let (session, lifecycle) = resolve_worktree_lifecycle(&request.session_id)?;
    ensure_no_active_worktree_marker(&lifecycle.run_id)?;
    let review_id = if action.mutates() {
        let review_id = request.review_id.as_deref().unwrap_or_default();
        require_current_review(&lifecycle, review_id)?;
        Some(review_id.to_string())
    } else {
        None
    };
    Ok((session, lifecycle, review_id))
}

/// Run the action after confirmation. Merge and discard mutate on-disk git state
/// through the tested core functions; keep and recover do not. The active-run
/// marker is re-checked immediately before any mutation.
fn execute_worktree_action(
    mut session: ManagedSessionSummaryData,
    lifecycle: WorktreeLifecycle,
    action: WorktreeReviewAction,
    expected_review_id: Option<&str>,
) -> Result<WorktreeActionResponse, String> {
    if let Some(review_id) = expected_review_id {
        require_current_review(&lifecycle, review_id)?;
    }
    let next_steps = match action {
        WorktreeReviewAction::Merge => {
            ensure_no_active_worktree_marker(&lifecycle.run_id)?;
            runs_worktree_merge(&lifecycle.run_id).map_err(|_| merge_failed_error())?;
            Vec::new()
        }
        WorktreeReviewAction::Discard => {
            // The shared gated discard re-checks the active-run marker and
            // re-proves ownership immediately before the force-remove.
            discard_owned_worktree(&lifecycle)?;
            Vec::new()
        }
        // Recovery returns fixed, path-free next steps and does not touch the
        // protected copy.
        WorktreeReviewAction::Recover => review_next_steps(),
        // Keep is a deliberate no-op: verify_lifecycle already proved the
        // worktree is RunHaven-owned, and keeping it leaves the branch in place.
        WorktreeReviewAction::Keep => Vec::new(),
    };
    if action.mutates() {
        clear_session_worktree_review(&mut session);
    }
    Ok(WorktreeActionResponse {
        result: WorktreeActionResult {
            run_id: lifecycle.run_id.clone(),
            action: action.as_str().to_string(),
            branch: lifecycle.branch.clone(),
            status: action.status().to_string(),
            next_steps,
        },
        session,
    })
}

/// The core merge error names host paths and recovery commands, so it is
/// collapsed to a fixed message that points at the in-app recovery steps.
fn merge_failed_error() -> String {
    "RunHaven couldn’t apply these changes automatically. The protected copy is still safe. Choose Show recovery steps."
        .to_string()
}

#[cfg(test)]
pub(crate) fn keep_managed_session_worktree_for_test(
    request: WorktreeActionRequest,
    approve: impl FnOnce(&ManagedSessionSummaryData, &str, &str) -> Result<bool, String>,
) -> Result<WorktreeActionResponse, String> {
    run_worktree_action_for_test(request, WorktreeReviewAction::Keep, approve)
}

#[cfg(test)]
pub(crate) fn merge_managed_session_worktree_for_test(
    request: WorktreeActionRequest,
    approve: impl FnOnce(&ManagedSessionSummaryData, &str, &str) -> Result<bool, String>,
) -> Result<WorktreeActionResponse, String> {
    run_worktree_action_for_test(request, WorktreeReviewAction::Merge, approve)
}

#[cfg(test)]
pub(crate) fn recover_managed_session_worktree_for_test(
    request: WorktreeActionRequest,
    approve: impl FnOnce(&ManagedSessionSummaryData, &str, &str) -> Result<bool, String>,
) -> Result<WorktreeActionResponse, String> {
    run_worktree_action_for_test(request, WorktreeReviewAction::Recover, approve)
}

#[cfg(test)]
pub(crate) fn discard_managed_session_worktree_for_test(
    request: WorktreeActionRequest,
    approve: impl FnOnce(&ManagedSessionSummaryData, &str, &str) -> Result<bool, String>,
) -> Result<WorktreeActionResponse, String> {
    run_worktree_action_for_test(request, WorktreeReviewAction::Discard, approve)
}

#[cfg(test)]
fn run_worktree_action_for_test(
    request: WorktreeActionRequest,
    action: WorktreeReviewAction,
    approve: impl FnOnce(&ManagedSessionSummaryData, &str, &str) -> Result<bool, String>,
) -> Result<WorktreeActionResponse, String> {
    let (session, lifecycle, _review_id) = authorize_worktree_action(&request, action)?;
    if !approve(&session, &lifecycle.run_id, &lifecycle.branch)? {
        return Err(action.canceled_error().to_string());
    }
    let (session, lifecycle, review_id) = authorize_worktree_action(&request, action)?;
    execute_worktree_action(session, lifecycle, action, review_id.as_deref())
}
