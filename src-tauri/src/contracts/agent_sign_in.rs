use serde::{Deserialize, Serialize};

/// Request to start an in-app device-code sign-in for an agent. `confirm` must
/// be true; the Rust command still shows a native confirmation before any
/// sign-in starts. The desktop app only triggers sign-in: RunHaven core owns the
/// resulting credential (unchanged `0600` path), and no token or code is ever
/// returned here.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AgentSignInRequest {
    pub agent: String,
    #[serde(default)]
    pub confirm: bool,
}

/// Acknowledgement that a background sign-in was accepted. The WebView polls
/// `get_agent_sign_in_status` with `sign_in_id` for sanitized progress. No
/// secret is present in this response.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AgentSignInStartResponse {
    pub sign_in_id: String,
    pub agent: String,
    pub state: String,
    pub label: String,
    pub detail: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AgentSignInStatusRequest {
    pub sign_in_id: String,
}

/// Sanitized sign-in progress. `output_tail` is a bounded, secret-redacted tail
/// of the agent's login output so the sign-in URL and short-lived device code
/// stay visible while any token or key is scrubbed. No raw host paths, no token,
/// no unbounded logs. The signed-in truth stays with `get_auth_status`; a
/// `succeeded` state is the desktop app's cue to refresh it.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AgentSignInStatusResponse {
    pub sign_in_id: String,
    pub agent: String,
    pub state: String,
    pub label: String,
    pub detail: String,
    pub output_tail: String,
    pub updated_at: String,
}
