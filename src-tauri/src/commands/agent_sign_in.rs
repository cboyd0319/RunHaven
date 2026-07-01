//! Mutating in-app agent sign-in command.
//!
//! Signing in is how a beginner authenticates an agent without dropping to a
//! terminal. It reuses the tested core device-code login plan (no login logic
//! lives here), fails closed on unconfirmed, unknown, or non-device-code input,
//! and requires a Rust-owned native confirmation naming the exact agent before
//! any sign-in runs. The sign-in itself runs on a background thread; the command
//! returns a sign-in id the WebView polls through `get_agent_sign_in_status`.
//!
//! The desktop app only ever triggers the sign-in and watches sanitized
//! progress. RunHaven core owns the resulting credential (unchanged `0600`
//! path); nothing here reads, stores, or returns a token. Claude is deliberately
//! not an in-app target: its host `claude setup-token` needs a token pasted into
//! stdin, so it stays a terminal step (see docs/DESKTOP_APP.md).

use runhaven_core::runtime::launch::{new_run_id, require_container_cli};
use runhaven_core::runtime::login::{build_sandbox_login_plan, supports_sandbox_login};
use runhaven_core::runtime::plans::AgentRunPlan;
use runhaven_core::runtime::profiles::get_profile;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

use super::validation::{MAX_AGENT_NAME_LEN, validate_text_len};
use crate::agent_sign_in_runner::spawn_agent_sign_in;
use crate::agent_sign_in_status::AgentSignInStatusRegistry;
use crate::contracts::{AgentSignInRequest, AgentSignInStartResponse};

const SIGN_IN_CANCELED: &str = "Sign-in canceled before it started.";

#[tauri::command]
pub(crate) async fn start_agent_sign_in(
    app: AppHandle,
    request: AgentSignInRequest,
    agent_sign_in_status: State<'_, AgentSignInStatusRegistry>,
) -> Result<AgentSignInStartResponse, String> {
    let registry = agent_sign_in_status.inner().clone();
    let prepared = authorize_agent_sign_in(&request)?;
    if !trusted_sign_in_confirmed(app, &prepared.agent).await? {
        return Err(SIGN_IN_CANCELED.to_string());
    }
    start_sign_in_flow(prepared, &registry)
}

/// A validated, ownership-checked sign-in ready for the confirmation dialog.
/// Building this starts nothing, so a denial or failure after this point leaves
/// no sign-in behind and touches no credential.
struct PreparedSignIn {
    agent: String,
    plan: AgentRunPlan,
}

/// Validate intent and input before anything is shown or run. Fails closed on an
/// unconfirmed request, an over-long or unknown agent, or an agent without an
/// in-app device-code sign-in (claude is redirected to the terminal). The
/// canonical `profile.name` is used everywhere downstream so arbitrary WebView
/// input is never reflected back.
fn authorize_agent_sign_in(request: &AgentSignInRequest) -> Result<PreparedSignIn, String> {
    if !request.confirm {
        return Err("Confirm before RunHaven starts sign-in for this agent.".to_string());
    }
    validate_text_len("agent", &request.agent, MAX_AGENT_NAME_LEN)?;
    let profile =
        get_profile(&request.agent).map_err(|_| "Choose a known agent to sign in.".to_string())?;
    let agent = profile.name.to_string();
    if agent == "claude" {
        return Err(
            "Claude sign-in needs a code pasted in a terminal. Run `runhaven login claude` \
             in your terminal, then come back."
                .to_string(),
        );
    }
    if !supports_sandbox_login(&agent) {
        return Err("This agent does not support in-app sign-in yet.".to_string());
    }
    // build_sandbox_login_plan can surface host paths or template names on error,
    // so it is collapsed to a fixed, path-free message.
    let plan = build_sandbox_login_plan(&agent)
        .map_err(|_| "RunHaven could not prepare sign-in for this agent.".to_string())?;
    Ok(PreparedSignIn { agent, plan })
}

fn start_sign_in_flow(
    prepared: PreparedSignIn,
    registry: &AgentSignInStatusRegistry,
) -> Result<AgentSignInStartResponse, String> {
    // The container CLI is the last gate before the sign-in runs, re-checked
    // after confirmation so a denied or canceled sign-in never starts anything.
    require_container_cli().map_err(|_| {
        "The container tool is not ready. Finish RunHaven setup before signing in.".to_string()
    })?;
    let sign_in_id = new_run_id();
    registry.mark_starting(&sign_in_id, &prepared.agent)?;
    let mut plan = prepared.plan;
    plan.run_id = Some(sign_in_id.clone());
    if let Err(error) = spawn_agent_sign_in(
        sign_in_id.clone(),
        prepared.agent.clone(),
        plan,
        registry.clone(),
    ) {
        let _ = registry.mark_failed(&sign_in_id, "RunHaven could not start sign-in.".to_string());
        return Err(error);
    }
    Ok(AgentSignInStartResponse {
        sign_in_id,
        agent: prepared.agent,
        state: "starting".to_string(),
        label: "Starting".to_string(),
        detail: "RunHaven started signing you in. Watch below for the link and code.".to_string(),
    })
}

async fn trusted_sign_in_confirmed(app: AppHandle, agent: &str) -> Result<bool, String> {
    let message = format!(
        "Agent: {agent}\n\nStart sign-in for {agent}? A browser step will be needed: RunHaven \
         opens a device sign-in in the sandbox and shows a link and a short code to finish in \
         your browser. RunHaven never sees or stores the token."
    );
    tauri::async_runtime::spawn_blocking(move || {
        app.dialog()
            .message(message)
            .title("Confirm RunHaven sign-in")
            .kind(MessageDialogKind::Info)
            .buttons(MessageDialogButtons::OkCancelCustom(
                "Start sign-in".to_string(),
                "Cancel".to_string(),
            ))
            .blocking_show()
    })
    .await
    .map_err(|error| format!("could not show sign-in confirmation: {error}"))
}

#[cfg(test)]
pub(crate) fn start_agent_sign_in_for_test(
    request: AgentSignInRequest,
    registry: &AgentSignInStatusRegistry,
    approve: impl FnOnce(&str) -> Result<bool, String>,
) -> Result<AgentSignInStartResponse, String> {
    let prepared = authorize_agent_sign_in(&request)?;
    if !approve(&prepared.agent)? {
        return Err(SIGN_IN_CANCELED.to_string());
    }
    start_sign_in_flow(prepared, registry)
}
