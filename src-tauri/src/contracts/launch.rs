use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RunPlanRequest {
    pub agent: String,
    pub workspace_path: String,
    pub network_mode: String,
    pub workspace_scope: String,
    #[serde(default = "default_auth_scope")]
    pub auth_scope: String,
    pub session_name: Option<String>,
    pub read_only_workspace: bool,
    pub cpus: String,
    pub memory: String,
    #[serde(default)]
    pub provider_hosts: Vec<String>,
    #[serde(default)]
    pub env_names: Vec<String>,
    #[serde(default)]
    pub api_key_broker_env: Option<String>,
    pub image: Option<String>,
    pub allow_sensitive_workspace: bool,
    pub allow_root_user: bool,
    pub user: String,
}

fn default_auth_scope() -> String {
    "agent".to_string()
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PlanWarning {
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RunPlanAccess {
    pub status: String,
    pub label: String,
    pub detail: String,
    pub remedy: String,
    pub blocking: bool,
    pub api_key_broker_env: Option<String>,
    pub api_key_broker_env_present: Option<bool>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RunPlanResponse {
    pub profile: String,
    pub workspace: String,
    pub workspace_scope: String,
    pub workspace_scope_note: Option<String>,
    pub auth_scope: String,
    pub state_volume: String,
    pub session: String,
    pub container_name: String,
    pub network_mode: String,
    pub network_name: Option<String>,
    pub egress_summary: String,
    pub image: String,
    pub provider_allowed_hosts: Vec<String>,
    pub preflight_count: usize,
    pub access: RunPlanAccess,
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
    pub auth_scope: String,
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
    pub auth_scope: String,
    pub session: String,
    pub network_mode: String,
    pub snapshot: StartedRunSnapshot,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LaunchStatusRequest {
    pub run_id: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LaunchStatusResponse {
    pub run_id: String,
    pub state: String,
    pub label: String,
    pub detail: String,
    pub updated_at: String,
}
