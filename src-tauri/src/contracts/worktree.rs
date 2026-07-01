use runhaven_core::ui_contracts::ManagedSessionSummaryData;
use serde::{Deserialize, Serialize};

/// Read-only request for the worktree review of a finished managed session.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WorktreeReviewRequest {
    pub session_id: String,
}

/// Secret-free worktree review surfaced to the untrusted WebView. Only the
/// RunHaven-owned branch name, change counts, and available action keys are
/// mapped: no raw host worktree path, source repository path, mounted workspace
/// path, git recovery command line, state volume, or token is ever included.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WorktreeReviewResponse {
    pub run_id: String,
    pub branch: String,
    pub status_summary: String,
    pub worktree_changes: usize,
    pub source_changes: usize,
    pub clean: bool,
    pub available_actions: Vec<String>,
}

/// Mutating request for a worktree review action (keep, merge, recover, or
/// discard). `confirm` gates the action the same way cleanup and run control do.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WorktreeActionRequest {
    pub session_id: String,
    pub confirm: bool,
}

/// Sanitized outcome of a worktree review action. `branch` is the RunHaven-owned
/// branch name; `next_steps` carries only the static, path-free recovery labels
/// from the core payload. No raw path, command line, or secret is included.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WorktreeActionResult {
    pub run_id: String,
    pub action: String,
    pub branch: String,
    pub status: String,
    pub next_steps: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WorktreeActionResponse {
    pub session: ManagedSessionSummaryData,
    pub result: WorktreeActionResult,
}
