use runhaven_core::runtime::worktrees::{WorktreeLifecycle, runs_worktree_merge};
use runhaven_core::ui_contracts::ManagedSessionSummaryData;
use tauri::AppHandle;

use super::approval::{WorktreeReviewAction, trusted_worktree_action_confirmed};
use super::{
    clear_session_worktree_review, discard_owned_worktree, ensure_no_active_worktree_marker,
    resolve_worktree_lifecycle, review_next_steps, worktree_review_payload,
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
    let (session, lifecycle) = authorize_worktree_action(&request, action)?;
    if !trusted_worktree_action_confirmed(
        app,
        action,
        &session,
        &lifecycle.run_id,
        &lifecycle.branch,
    )
    .await?
    {
        return Err(action.canceled_error().to_string());
    }
    execute_worktree_action(session, lifecycle, action)
}

/// Everything that must hold before a native confirmation is shown: confirmed
/// intent, an eligible RunHaven-owned worktree, and no active run marker. No
/// mutation happens here, so denial or failure leaves the worktree untouched.
fn authorize_worktree_action(
    request: &WorktreeActionRequest,
    action: WorktreeReviewAction,
) -> Result<(ManagedSessionSummaryData, WorktreeLifecycle), String> {
    if !request.confirm {
        return Err(action.confirm_error().to_string());
    }
    let (session, lifecycle) = resolve_worktree_lifecycle(&request.session_id)?;
    ensure_no_active_worktree_marker(&lifecycle.run_id)?;
    Ok((session, lifecycle))
}

/// Run the action after confirmation. Merge and discard mutate on-disk git state
/// through the tested core functions; keep and recover do not. The active-run
/// marker is re-checked immediately before any mutation.
fn execute_worktree_action(
    mut session: ManagedSessionSummaryData,
    lifecycle: WorktreeLifecycle,
    action: WorktreeReviewAction,
) -> Result<WorktreeActionResponse, String> {
    let next_steps = match action {
        WorktreeReviewAction::Merge => {
            ensure_no_active_worktree_marker(&lifecycle.run_id)?;
            runs_worktree_merge(&lifecycle.run_id).map_err(|_| merge_failed_error(&lifecycle))?;
            Vec::new()
        }
        WorktreeReviewAction::Discard => {
            // The shared gated discard re-checks the active-run marker and
            // re-proves ownership immediately before the force-remove.
            discard_owned_worktree(&lifecycle)?;
            Vec::new()
        }
        // Recover reads the tested core payload for the safe, path-free manual
        // next steps; it does not touch the worktree.
        WorktreeReviewAction::Recover => review_next_steps(&worktree_review_payload(&lifecycle)?),
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

/// The core merge error names host paths and git recovery command lines, so it
/// is collapsed to a fixed message that points at the safe CLI recovery flow.
fn merge_failed_error(lifecycle: &WorktreeLifecycle) -> String {
    format!(
        "RunHaven could not merge this worktree automatically. Open the terminal and run `runhaven runs recover {}` to see the safe next steps.",
        lifecycle.run_id
    )
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
    let (session, lifecycle) = authorize_worktree_action(&request, action)?;
    if !approve(&session, &lifecycle.run_id, &lifecycle.branch)? {
        return Err(action.canceled_error().to_string());
    }
    execute_worktree_action(session, lifecycle, action)
}
