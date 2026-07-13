use runhaven_core::provider::agent_sign_in_record::AgentReadiness;
use serde::Serialize;

use super::SetupAction;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CheckStatus {
    pub name: String,
    pub ok: bool,
    pub detail: String,
    pub remedy: String,
    /// A supported Rust-owned recovery, when this exact check can be fixed in
    /// the app. No executable, arguments, or host path cross into the WebView.
    pub action: Option<SetupAction>,
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
    /// True when the desktop app can drive this agent's native sign-in through
    /// its in-app terminal. The mutating command still validates the exact
    /// supported agent set and fails closed regardless.
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
    /// Per-agent sign-in readiness for the Home "Agent setup" panel. Describes
    /// each agent's own state ("ready" | "signed-in" | "needs-sign-in"), never
    /// implies RunHaven stores the credential.
    pub agent_readiness: Vec<AgentReadiness>,
    pub active_runs: Vec<RunSummary>,
    pub recent_runs: Vec<RunSummary>,
    pub warnings: Vec<String>,
    /// The RunHaven desktop app version, taken from the crate version at build
    /// time. A plain, non-secret string the Diagnostics view shows alongside
    /// the pinned runtime versions already in the setup checks.
    pub app_version: String,
}
