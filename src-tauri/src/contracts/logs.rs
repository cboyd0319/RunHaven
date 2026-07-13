use runhaven_core::ui_contracts::ManagedSessionSummaryData;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LogSnapshotRequest {
    pub run_id: String,
    pub lines: Option<u32>,
    /// Fail closed: omitted means `false`, yielding the friendly confirm error.
    #[serde(default)]
    pub confirm_sensitive_output: bool,
}

/// One bounded, redacted output snapshot. `captured_at` is the capture
/// timestamp and doubles as the change marker the UI compares between polls.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LogSnapshotResponse {
    pub run_id: String,
    pub captured_at: String,
    pub requested_lines: u32,
    pub text: String,
    pub returned_lines: usize,
    pub truncated: bool,
    pub source: String,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ManagedSessionLogSnapshotRequest {
    pub session_id: String,
    pub lines: Option<u32>,
    /// Fail closed: omitted means `false`, yielding the friendly confirm error.
    #[serde(default)]
    pub confirm_sensitive_output: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ManagedSessionLogSnapshotResponse {
    pub session: ManagedSessionSummaryData,
    pub snapshot: LogSnapshotResponse,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct WriteSessionInputRequest {
    pub session_id: String,
    pub input: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WriteSessionInputResponse {
    pub session: ManagedSessionSummaryData,
    pub accepted: bool,
    pub accepted_bytes: usize,
    pub cursor: String,
    pub status: String,
}
