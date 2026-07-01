use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WhyHostRequest {
    pub host: String,
    pub port: u16,
    pub agent: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WhyWorkspaceRequest {
    pub path: String,
    pub workspace_scope: String,
    pub allow_sensitive_workspace: bool,
}

// Read-only "why is this safe" explanations exposed to the untrusted WebView.
// Every field is static or caller-supplied description text. No credential
// value, environment value, token, or raw host filesystem path is surfaced. The
// workspace response carries only the path-free verdict, never the resolved,
// mounted, or input path (those stay in the CLI-only payload).

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NetworkExplanationResponse {
    pub mode: String,
    pub behavior: String,
    pub boundary: String,
    pub use_when: String,
    pub dns_safety: String,
    pub inspect: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StateExplanationResponse {
    pub profile: String,
    pub default_session: String,
    pub home_mount: String,
    pub state_volume_pattern: String,
    pub named_session_pattern: String,
    pub boundary: String,
    pub project_id: String,
    pub manage: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HostExplanationResponse {
    pub host: String,
    pub port: u32,
    pub decision: String,
    pub reason: String,
    pub next_action: String,
    pub agent: String,
    pub profile: String,
    pub matched_rules: Vec<String>,
    pub bundled_hosts: Vec<String>,
    pub endpoint_notes: Vec<String>,
    pub dns_safety: String,
}

/// Workspace boundary verdict. Only the path-free decision, verdict, next
/// action, and boundary text are exposed; no absolute host path is ever mapped.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WorkspaceExplanationResponse {
    pub scope: String,
    pub decision: String,
    pub verdict: String,
    pub next_action: String,
    pub boundary: String,
    pub sensitive_override_used: bool,
}
