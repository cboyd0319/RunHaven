use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CheckStatus {
    pub name: String,
    pub ok: bool,
    pub detail: String,
    pub remedy: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SetupStatus {
    pub ok: bool,
    pub checks: Vec<CheckStatus>,
    pub blocker_count: usize,
    pub ssh_available: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AgentProfileSummary {
    pub name: String,
    pub description: String,
    pub image: String,
    pub default_command: Vec<String>,
    pub provider_hosts: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RunSummary {
    pub run_id: String,
    pub profile: String,
    pub workspace: String,
    pub network: String,
    pub status: String,
    pub timestamp: String,
    pub state_volume: String,
    pub session: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DashboardStatus {
    pub setup: SetupStatus,
    pub agents: Vec<AgentProfileSummary>,
    pub active_runs: Vec<RunSummary>,
    pub recent_runs: Vec<RunSummary>,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ImageStatusRequest {
    pub agent: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProfileImageStatus {
    pub agent: String,
    pub image: String,
    pub status: String,
    pub ready: bool,
    pub expected_source_digest: String,
    pub local_source_digest: Option<String>,
    pub fix_command: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BuilderStatus {
    pub status: String,
    pub detail: String,
    pub image: Option<String>,
    pub cpus: Option<String>,
    pub memory: Option<String>,
    pub rosetta: Option<bool>,
    pub started_date: Option<String>,
    pub ipv4_address: Option<String>,
    pub warning: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ImageStatusResponse {
    pub agent: String,
    pub image: ProfileImageStatus,
    pub builder: BuilderStatus,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RunPlanRequest {
    pub agent: String,
    pub workspace_path: String,
    pub network_mode: String,
    pub workspace_scope: String,
    pub session_name: Option<String>,
    pub read_only_workspace: bool,
    pub cpus: String,
    pub memory: String,
    #[serde(default)]
    pub provider_hosts: Vec<String>,
    #[serde(default)]
    pub env_names: Vec<String>,
    pub image: Option<String>,
    pub allow_sensitive_workspace: bool,
    pub allow_root_user: bool,
    pub user: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PlanWarning {
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RunPlanResponse {
    pub profile: String,
    pub workspace: String,
    pub workspace_scope: String,
    pub workspace_scope_note: Option<String>,
    pub state_volume: String,
    pub session: String,
    pub container_name: String,
    pub network_mode: String,
    pub network_name: Option<String>,
    pub egress_summary: String,
    pub image: String,
    pub provider_allowed_hosts: Vec<String>,
    pub preflight_count: usize,
    pub warnings: Vec<PlanWarning>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LaunchRunRequest {
    pub plan: RunPlanRequest,
    pub confirm_launch: bool,
    #[serde(default)]
    pub confirmed_warnings: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StartedRunSnapshot {
    pub run_id: String,
    pub status: String,
    pub profile: String,
    pub workspace: String,
    pub state_volume: String,
    pub session: String,
    pub network_mode: String,
    pub container_name: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LaunchRunResponse {
    pub run_id: String,
    pub status: String,
    pub profile: String,
    pub workspace: String,
    pub state_volume: String,
    pub session: String,
    pub network_mode: String,
    pub snapshot: StartedRunSnapshot,
}
