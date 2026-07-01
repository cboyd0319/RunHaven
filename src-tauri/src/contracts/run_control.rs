use runhaven_core::ui_contracts::ManagedSessionSummaryData;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StopRunRequest {
    pub run_id: String,
    pub confirm_stop: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StopRunResponse {
    pub run_id: String,
    pub container_name: String,
    pub status: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StopManagedSessionRequest {
    pub session_id: String,
    pub confirm_stop: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StopManagedSessionResponse {
    pub session: ManagedSessionSummaryData,
    pub result: StopRunResponse,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct KillRunRequest {
    pub run_id: String,
    pub confirm_kill: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct KillRunResponse {
    pub run_id: String,
    pub container_name: String,
    pub status: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct KillManagedSessionRequest {
    pub session_id: String,
    pub confirm_kill: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct KillManagedSessionResponse {
    pub session: ManagedSessionSummaryData,
    pub result: KillRunResponse,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RepairRunRequest {
    pub run_id: String,
    pub confirm_repair: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RepairRunResponse {
    pub run_id: String,
    pub container_name: String,
    pub status: String,
    pub marker_removed: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RepairManagedSessionRequest {
    pub session_id: String,
    pub confirm_repair: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RepairManagedSessionResponse {
    pub session: ManagedSessionSummaryData,
    pub result: RepairRunResponse,
}
