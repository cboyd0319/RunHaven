use runhaven_core::ui_contracts::ManagedSessionSummaryData;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RunStatusRequest {
    pub run_id: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RunStatusRun {
    pub run_id: String,
    pub profile: String,
    pub workspace: String,
    pub network_mode: String,
    pub status: String,
    pub timestamp: String,
    pub auth_scope: String,
    pub session: String,
    pub container_name: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RunStatusResources {
    pub cpus: Option<String>,
    pub memory_bytes: Option<u64>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RunStatusNetwork {
    pub network: Option<String>,
    pub hostname: Option<String>,
    pub ipv4_address: Option<String>,
    pub ipv4_gateway: Option<String>,
    pub ipv6_address: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RunStatusContainer {
    pub state: String,
    pub image: Option<String>,
    pub started_at: Option<String>,
    pub resources: RunStatusResources,
    pub networks: Vec<RunStatusNetwork>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RunStatusResponse {
    pub run: RunStatusRun,
    pub container: RunStatusContainer,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ManagedSessionStatusRequest {
    pub session_id: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ManagedSessionStatusResponse {
    pub session: ManagedSessionSummaryData,
    pub container: Option<RunStatusContainer>,
    pub status_unavailable: Option<String>,
    pub recovery: Option<ManagedSessionRecovery>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ManagedSessionRecovery {
    pub status: String,
    pub title: String,
    pub detail: String,
    pub remedy: String,
    pub repair_available: bool,
}
