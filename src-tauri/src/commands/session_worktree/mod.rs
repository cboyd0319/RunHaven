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
    WorktreeLifecycle, load_worktree_lifecycle, verify_lifecycle, worktree_recovery_payload,
};
use runhaven_core::ui_contracts::{ManagedSessionCleanupData, ManagedSessionSummaryData};
use serde_json::Value;

use crate::contracts::{WorktreeReviewRequest, WorktreeReviewResponse};

pub(crate) mod actions;
mod approval;

#[cfg(test)]
pub(crate) use actions::{
    discard_managed_session_worktree_for_test, keep_managed_session_worktree_for_test,
    merge_managed_session_worktree_for_test, recover_managed_session_worktree_for_test,
};

const ACTIVE_WORKTREE_ERROR: &str =
    "Worktree review is only available after this session stops running.";
const HISTORY_WORKTREE_ERROR: &str = "This session is not eligible for worktree review yet.";
const UNSUPPORTED_WORKTREE_ERROR: &str =
    "Worktree review is only available for RunHaven-owned worktree sessions.";
const FAILED_WORKTREE_ERROR: &str =
    "RunHaven could not verify a safe worktree target for this session.";

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
fn resolve_worktree_lifecycle(
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

fn ensure_no_active_worktree_marker(run_id: &str) -> Result<(), String> {
    match super::active_marker::active_run_marker_present(run_id) {
        Ok(true) => Err(ACTIVE_WORKTREE_ERROR.to_string()),
        Ok(false) => Ok(()),
        Err(()) => Err(FAILED_WORKTREE_ERROR.to_string()),
    }
}

/// The recovery payload, with paths and command lines still present. Used only
/// to derive path-free counts and static next-step labels; the payload itself
/// is never returned to the frontend.
fn worktree_review_payload(lifecycle: &WorktreeLifecycle) -> Result<Value, String> {
    worktree_recovery_payload(lifecycle).map_err(worktree_target_error)
}

fn build_review_response(lifecycle: &WorktreeLifecycle) -> Result<WorktreeReviewResponse, String> {
    let payload = worktree_review_payload(lifecycle)?;
    let worktree_changes = status_len(&payload, "worktree_status");
    let source_changes = status_len(&payload, "source_status");
    Ok(WorktreeReviewResponse {
        run_id: lifecycle.run_id.clone(),
        branch: lifecycle.branch.clone(),
        status_summary: review_status_summary(worktree_changes, source_changes),
        worktree_changes,
        source_changes,
        clean: worktree_changes == 0,
        available_actions: available_action_keys(),
    })
}

fn status_len(payload: &Value, key: &str) -> usize {
    payload
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0)
}

/// Safe, path-free manual next-step labels from the core recovery payload.
fn review_next_steps(payload: &Value) -> Vec<String> {
    payload
        .get("next_steps")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn review_status_summary(worktree_changes: usize, source_changes: usize) -> String {
    let mut summary = if worktree_changes == 0 {
        "No changes recorded in this worktree yet.".to_string()
    } else {
        format!(
            "{worktree_changes} file change{} ready to review.",
            plural(worktree_changes)
        )
    };
    if source_changes > 0 {
        summary.push_str(" The source branch also has uncommitted changes.");
    }
    summary
}

fn plural(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
}

fn available_action_keys() -> Vec<String> {
    ["merge", "keep", "recover", "discard"]
        .into_iter()
        .map(str::to_string)
        .collect()
}

/// After a mutation removes the worktree, drop the session's worktree review /
/// cleanup availability so the refreshed card no longer offers stale actions.
fn clear_session_worktree_review(session: &mut ManagedSessionSummaryData) {
    session.cleanup = ManagedSessionCleanupData {
        available: false,
        target: String::new(),
        label: String::new(),
        detail: String::new(),
    };
}
