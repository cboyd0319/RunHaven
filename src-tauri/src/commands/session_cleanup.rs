//! Managed-session cleanup: the "Clean up" action on a finished session card.
//!
//! Cleanup is a worktree discard with cleanup-flavored copy, so it reuses the
//! session_worktree flow rather than duplicating it: `resolve_worktree_lifecycle`
//! resolves and RunHaven-verifies the backend-owned worktree, and
//! `discard_owned_worktree` is the single gated discard that re-checks the
//! active-run marker and re-proves ownership immediately before the force-remove
//! (in case on-disk state changed while the native confirmation dialog was
//! open). Only the confirm/cancel wording and the dialog copy are cleanup
//! specific.

use runhaven_core::runtime::worktrees::WorktreeLifecycle;
use runhaven_core::ui_contracts::ManagedSessionSummaryData;
use tauri::AppHandle;
use tauri_plugin_dialog::MessageDialogKind;

use super::dialog::confirm_native;
use super::session_worktree::{
    clear_session_worktree_review, discard_owned_worktree, ensure_no_active_worktree_marker,
    resolve_worktree_lifecycle,
};
use crate::contracts::{
    CleanupManagedSessionRequest, CleanupManagedSessionResponse, CleanupManagedSessionResult,
};

const CONFIRM_CLEANUP_ERROR: &str = "Confirm the cleanup before discarding this session worktree.";
const CANCEL_CLEANUP_ERROR: &str = "Cleanup canceled before discarding this session worktree.";
const CLEANUP_TARGET_WORKTREE: &str = "worktree";
const CLEANUP_STATUS_DISCARDED: &str = "discarded";

#[tauri::command]
pub(crate) async fn cleanup_managed_session(
    app: AppHandle,
    request: CleanupManagedSessionRequest,
) -> Result<CleanupManagedSessionResponse, String> {
    let (session, lifecycle) = authorize_cleanup(&request)?;
    if !trusted_session_cleanup_confirmed(app, &session, &lifecycle.run_id).await? {
        return Err(CANCEL_CLEANUP_ERROR.to_string());
    }
    execute_cleanup(session, lifecycle)
}

#[cfg(test)]
pub(crate) fn cleanup_managed_session_for_test(
    request: CleanupManagedSessionRequest,
    approve: impl FnOnce(&ManagedSessionSummaryData, &str) -> Result<bool, String>,
) -> Result<CleanupManagedSessionResponse, String> {
    let (session, lifecycle) = authorize_cleanup(&request)?;
    if !approve(&session, &lifecycle.run_id)? {
        return Err(CANCEL_CLEANUP_ERROR.to_string());
    }
    execute_cleanup(session, lifecycle)
}

/// Everything that must hold before the native confirmation is shown: confirmed
/// intent, an eligible RunHaven-owned worktree, and no active run marker. No
/// mutation happens here, so denial or failure leaves the worktree untouched.
fn authorize_cleanup(
    request: &CleanupManagedSessionRequest,
) -> Result<(ManagedSessionSummaryData, WorktreeLifecycle), String> {
    if !request.confirm_cleanup {
        return Err(CONFIRM_CLEANUP_ERROR.to_string());
    }
    let (session, lifecycle) = resolve_worktree_lifecycle(&request.session_id)?;
    ensure_no_active_worktree_marker(&lifecycle.run_id)?;
    Ok((session, lifecycle))
}

/// Run the confirmed cleanup through the shared gated discard, then drop the
/// session's cleanup availability so the refreshed card no longer offers it.
fn execute_cleanup(
    mut session: ManagedSessionSummaryData,
    lifecycle: WorktreeLifecycle,
) -> Result<CleanupManagedSessionResponse, String> {
    discard_owned_worktree(&lifecycle)?;
    clear_session_worktree_review(&mut session);
    Ok(CleanupManagedSessionResponse {
        session,
        result: CleanupManagedSessionResult {
            run_id: lifecycle.run_id,
            target: CLEANUP_TARGET_WORKTREE.to_string(),
            status: CLEANUP_STATUS_DISCARDED.to_string(),
        },
    })
}

async fn trusted_session_cleanup_confirmed(
    app: AppHandle,
    session: &ManagedSessionSummaryData,
    run_id: &str,
) -> Result<bool, String> {
    let message = format!(
        "Action: Clean up\nSession: {}\nRun ID: {}\nAgent: {}\nProject: {}\nStatus: {}\nTarget: RunHaven worktree\n\nRunHaven will delete the recorded worktree and branch for this finished session after you confirm.",
        session.title, run_id, session.agent, session.project, session.status_label,
    );
    confirm_native(
        app,
        "Confirm RunHaven cleanup",
        "Clean up",
        MessageDialogKind::Warning,
        message,
    )
    .await
}
