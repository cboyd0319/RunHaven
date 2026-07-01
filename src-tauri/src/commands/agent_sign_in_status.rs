//! Read-only progress for a background agent sign-in. Returns only the sanitized
//! status the `AgentSignInStatusRegistry` records (state, label, detail, and a
//! bounded, secret-redacted output tail with the sign-in URL and device code).
//! The signed-in truth stays with `get_auth_status`; a `succeeded` state is the
//! desktop app's cue to refresh it.

use tauri::State;

use crate::contracts::{AgentSignInStatusRequest, AgentSignInStatusResponse};
use crate::tail_status::AgentSignInStatusRegistry;

#[tauri::command]
pub(crate) fn get_agent_sign_in_status(
    request: AgentSignInStatusRequest,
    agent_sign_in_status: State<'_, AgentSignInStatusRegistry>,
) -> Result<AgentSignInStatusResponse, String> {
    agent_sign_in_status.get(&request.sign_in_id)
}

#[cfg(test)]
pub(crate) fn get_agent_sign_in_status_for_test(
    request: AgentSignInStatusRequest,
    registry: &AgentSignInStatusRegistry,
) -> Result<AgentSignInStatusResponse, String> {
    registry.get(&request.sign_in_id)
}
