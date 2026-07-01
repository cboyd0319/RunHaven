use runhaven_core::ui_contracts::ManagedSessionSummaryData;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LogSnapshotRequest {
    pub run_id: String,
    pub lines: Option<u32>,
    pub confirm_sensitive_output: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LogSnapshotEntry {
    pub source: String,
    pub timestamp: String,
    pub text: String,
    pub line_count: usize,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LogSnapshotResponse {
    pub run_id: String,
    pub captured_at: String,
    pub cursor: String,
    pub requested_lines: u32,
    pub entries: Vec<LogSnapshotEntry>,
    pub text: String,
    pub returned_lines: usize,
    pub truncated: bool,
    pub source: String,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionLogRedaction {
    pub status: String,
    pub message: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionLogEntry {
    pub session_id: String,
    pub run_id: String,
    pub source: String,
    pub cursor: String,
    pub timestamp: String,
    pub text: String,
    pub line_count: usize,
    pub redaction: SessionLogRedaction,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ManagedSessionLogSnapshotRequest {
    pub session_id: String,
    pub lines: Option<u32>,
    pub confirm_sensitive_output: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ManagedSessionLogSnapshotResponse {
    pub session: ManagedSessionSummaryData,
    pub snapshot: LogSnapshotResponse,
    pub cursor: String,
    pub entries: Vec<SessionLogEntry>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WriteSessionInputRequest {
    pub session_id: String,
    pub input: String,
    pub confirm_send: bool,
    pub client_target: String,
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
