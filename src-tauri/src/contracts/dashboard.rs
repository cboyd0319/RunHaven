use serde::Serialize;

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
    pub sign_in: String,
    pub broker: String,
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
    pub auth_scope: String,
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
