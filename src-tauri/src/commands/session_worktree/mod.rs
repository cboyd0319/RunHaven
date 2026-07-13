//! Worktree review lifecycle for finished managed sessions.
//!
//! A read-only status command (`get_managed_session_worktree`, scoped under the
//! read-only `main-read` capability) surfaces the RunHaven-owned branch, change
//! counts, and available actions. The mutating keep/merge/recover/discard
//! commands live under the sensitive `run-control` capability: each resolves the
//! backend-owned run from the session id, validates that the recorded worktree
//! is RunHaven-owned and not active/stale/foreign (the same `verify_lifecycle`
//! gate cleanup uses), requires a native confirmation, and returns a sanitized
//! result. No command exposes a raw host worktree path, source repository path,
//! git recovery command line, state volume, or secret.
//!
//! Security note: the app has one window, so the `main-read` / `run-control`
//! capability split is organizational, not a runtime privilege boundary; the
//! WebView can invoke any registered command. The real gate for a destructive
//! worktree op is the backend: ownership verification (`verify_lifecycle`), the
//! active-marker re-check immediately before mutation, and the Rust-owned native
//! confirmation dialog. Do not weaken any of those three on the theory that the
//! capability scoping constrains the WebView.

use runhaven_core::runtime::worktrees::{
    WorktreeLifecycle, discard_worktree_for_run, load_worktree_lifecycle,
    source_worktree_change_summary, verify_lifecycle, worktree_change_summary,
};
use runhaven_core::ui_contracts::{ManagedSessionCleanupData, ManagedSessionSummaryData};

use crate::contracts::{WorktreeReviewRequest, WorktreeReviewResponse};

pub(crate) mod actions;
mod approval;

#[cfg(test)]
pub(crate) use actions::{
    discard_managed_session_worktree_for_test, keep_managed_session_worktree_for_test,
    merge_managed_session_worktree_for_test, recover_managed_session_worktree_for_test,
};
#[cfg(test)]
pub(crate) use approval::worktree_confirmation_messages_for_test;

const ACTIVE_WORKTREE_ERROR: &str = "Wait for this session to end before you review its changes.";
const HISTORY_WORKTREE_ERROR: &str = "Saved changes are not available for this session.";
const UNSUPPORTED_WORKTREE_ERROR: &str = "Saved changes are not available for this session.";
const FAILED_WORKTREE_ERROR: &str =
    "RunHaven couldn’t verify the protected copy. It will not change or delete any files.";
const STALE_WORKTREE_REVIEW_ERROR: &str = "The protected copy changed since you reviewed it. Review the changes again before you continue.";

#[tauri::command]
pub(crate) fn get_managed_session_worktree(
    request: WorktreeReviewRequest,
) -> Result<WorktreeReviewResponse, String> {
    let (_session, lifecycle) = resolve_worktree_lifecycle(&request.session_id)?;
    build_review_response(&lifecycle)
}

#[cfg(test)]
pub(crate) use get_managed_session_worktree as get_managed_session_worktree_for_test;

/// Resolve the backend-owned run for a session id and validate that its recorded
/// worktree is finished, RunHaven-owned, and not active/stale/foreign. The
/// returned lifecycle carries raw host paths for backend use only and is never
/// serialized to the WebView.
pub(super) fn resolve_worktree_lifecycle(
    session_id: &str,
) -> Result<(ManagedSessionSummaryData, WorktreeLifecycle), String> {
    let (session, run_id) = super::sessions::managed_session_for_id(session_id)?;
    match session.lifecycle.trim().to_lowercase().as_str() {
        "finished" | "failed" | "stopped" => {}
        "running" | "starting" | "stop-requested" | "kill-requested" => {
            return Err(ACTIVE_WORKTREE_ERROR.to_string());
        }
        _ => return Err(HISTORY_WORKTREE_ERROR.to_string()),
    }
    let lifecycle = load_worktree_lifecycle(&run_id).map_err(worktree_target_error)?;
    verify_lifecycle(&lifecycle).map_err(worktree_target_error)?;
    Ok((session, lifecycle))
}

/// Collapse core lifecycle errors to fixed, path-free messages. The core bails
/// can name recorded paths or git detail, so they never reach the frontend.
fn worktree_target_error(error: impl ToString) -> String {
    if error.to_string().contains("is not a worktree run") {
        UNSUPPORTED_WORKTREE_ERROR.to_string()
    } else {
        FAILED_WORKTREE_ERROR.to_string()
    }
}

/// Re-prove RunHaven ownership and git consistency immediately before a
/// destructive mutation, in case on-disk state changed during the native
/// confirmation dialog. `verify_lifecycle` checks ownership, branch, and path
/// consistency (not cleanliness), so a legitimately dirty owned worktree still
/// passes and can be discarded.
fn reverify_worktree_owned(lifecycle: &WorktreeLifecycle) -> Result<(), String> {
    verify_lifecycle(lifecycle).map_err(worktree_target_error)
}

pub(super) fn ensure_no_active_worktree_marker(run_id: &str) -> Result<(), String> {
    match super::active_marker::active_run_marker_present(run_id) {
        Ok(true) => Err(ACTIVE_WORKTREE_ERROR.to_string()),
        Ok(false) => Ok(()),
        Err(()) => Err(FAILED_WORKTREE_ERROR.to_string()),
    }
}

/// The single gated worktree discard, shared by the worktree Discard action and
/// managed-session cleanup. Immediately before the force-remove it re-checks the
/// active-run marker and re-proves RunHaven ownership, in case on-disk state
/// changed while the native confirmation dialog was open, then deletes the
/// worktree and branch through the tested core function. Every discard path runs
/// these three gates here, so no caller can force-remove a worktree without
/// them.
pub(super) fn discard_owned_worktree(lifecycle: &WorktreeLifecycle) -> Result<(), String> {
    ensure_no_active_worktree_marker(&lifecycle.run_id)?;
    reverify_worktree_owned(lifecycle)?;
    discard_worktree_for_run(&lifecycle.run_id).map_err(worktree_target_error)
}

fn build_review_response(lifecycle: &WorktreeLifecycle) -> Result<WorktreeReviewResponse, String> {
    let summary = current_worktree_change_summary(lifecycle)?;
    let source_summary =
        source_worktree_change_summary(lifecycle).map_err(|_| FAILED_WORKTREE_ERROR.to_string())?;
    Ok(WorktreeReviewResponse {
        run_id: lifecycle.run_id.clone(),
        branch: lifecycle.branch.clone(),
        status_summary: review_status_summary(summary.total_count, source_summary.total_count),
        worktree_changes: summary.total_count,
        source_changes: source_summary.total_count,
        clean: summary.total_count == 0,
        file_changes: summary.file_changes,
        file_changes_truncated: summary.file_changes_truncated,
        source_file_changes: source_summary.file_changes,
        source_file_changes_truncated: source_summary.file_changes_truncated,
        review_id: summary.review_id,
        available_actions: available_action_keys(),
    })
}

fn current_worktree_change_summary(
    lifecycle: &WorktreeLifecycle,
) -> Result<runhaven_core::runtime::worktrees::WorktreeChangeSummary, String> {
    worktree_change_summary(lifecycle).map_err(|_| FAILED_WORKTREE_ERROR.to_string())
}

pub(super) fn require_current_review(
    lifecycle: &WorktreeLifecycle,
    expected_review_id: &str,
) -> Result<(), String> {
    if expected_review_id.len() != 64
        || !expected_review_id
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(STALE_WORKTREE_REVIEW_ERROR.to_string());
    }
    let current = current_worktree_change_summary(lifecycle)?;
    if current.review_id != expected_review_id {
        return Err(STALE_WORKTREE_REVIEW_ERROR.to_string());
    }
    Ok(())
}

/// Fixed, path-free recovery steps shown only after an action fails.
fn review_next_steps() -> Vec<String> {
    [
        "Keep the protected copy so nothing is lost.",
        "Finish or undo other changes in your project.",
        "Return to this review and choose Apply changes again.",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn review_status_summary(worktree_changes: usize, source_changes: usize) -> String {
    let mut summary = if worktree_changes == 0 {
        "The assistant didn’t change any files.".to_string()
    } else {
        format!(
            "{worktree_changes} file change{} ready to review.",
            plural(worktree_changes)
        )
    };
    if source_changes > 0 {
        summary.push_str(" Your project also has changes that must be finished or undone first.");
    }
    summary
}

fn plural(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
}

fn available_action_keys() -> Vec<String> {
    ["merge", "recover", "discard"]
        .into_iter()
        .map(str::to_string)
        .collect()
}

/// After a mutation removes the worktree, drop the session's worktree review /
/// cleanup availability so the refreshed card no longer offers stale actions.
pub(super) fn clear_session_worktree_review(session: &mut ManagedSessionSummaryData) {
    session.cleanup = ManagedSessionCleanupData {
        available: false,
        target: String::new(),
        label: String::new(),
        detail: String::new(),
    };
}
