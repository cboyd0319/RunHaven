use runhaven_core::ui_contracts::ManagedSessionSummaryData;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ManagedSessionDiagnosticsRequest {
    pub session_id: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EgressLogEntry {
    pub timestamp: String,
    pub profile: String,
    pub decision: String,
    pub host: String,
    pub port: u32,
    pub count: u64,
    pub reason: String,
    pub matched_rule: String,
    pub run_id: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EgressLogResponse {
    pub entries: Vec<EgressLogEntry>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AuthLogEntry {
    pub timestamp: String,
    pub profile: String,
    pub broker: String,
    pub decision: String,
    pub method: String,
    pub path: String,
    pub upstream_status: Option<u32>,
    pub count: u64,
    pub reason: String,
    pub run_id: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AuthLogResponse {
    pub entries: Vec<AuthLogEntry>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AuthProfileStatus {
    pub name: String,
    pub status: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AuthStatusResponse {
    pub status: String,
    pub runtime: String,
    pub profiles: Vec<AuthProfileStatus>,
}

/// Per-agent auth explanation exposed to the untrusted WebView. Only the static,
/// secret-free description fields from the core profile tables are surfaced: no
/// credential value, environment value, token, or raw host filesystem path.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AuthExplainResponse {
    pub name: String,
    pub status: String,
    pub runtime: String,
    pub supported_auth: Vec<String>,
    pub host_keeps: Vec<String>,
    pub guest_receives: Vec<String>,
    pub provider_hosts: Vec<String>,
    pub current_safe_path: String,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ManagedSessionDiagnosticsResponse {
    pub session: ManagedSessionSummaryData,
    pub auth_status: AuthProfileStatus,
    pub auth_log_entries: Vec<AuthLogEntry>,
    pub egress_log_entries: Vec<EgressLogEntry>,
}
