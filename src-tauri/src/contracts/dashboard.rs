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
    /// True when the desktop app can drive this agent's sign-in fully in-app
    /// (device-code login). Claude has a core in-sandbox login too, but its
    /// sign-in needs a code pasted back into the terminal, so the GUI keeps it
    /// out of the in-app flow (it signs in from a terminal). The mutating
    /// command still fails closed regardless.
    pub supports_in_app_sign_in: bool,
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
    /// The RunHaven desktop app version, taken from the crate version at build
    /// time. A plain, non-secret string the Diagnostics view shows alongside
    /// the pinned runtime versions already in the setup checks.
    pub app_version: String,
}
