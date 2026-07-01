use runhaven_core::doctor::collect_checks;
use runhaven_core::provider::auth_profiles::{agent_broker, agent_sign_in};
use runhaven_core::records::read_run_records;
use runhaven_core::runtime::active::read_active_run_records;
use runhaven_core::runtime::login::supports_sandbox_login;
use runhaven_core::runtime::profiles::profiles;
use runhaven_core::ui_contracts::project_label;
use serde_json::Value;

#[cfg(test)]
pub(crate) use runhaven_core::runtime::plans::AuthScope;
#[cfg(test)]
pub(crate) use runhaven_core::runtime::profiles::get_profile;

use crate::contracts::{
    AgentProfileSummary, CheckStatus, DashboardStatus, RunSummary, SetupStatus,
};

#[cfg(test)]
pub(crate) use crate::contracts::{LaunchRunRequest, RunPlanRequest};

mod access;
mod active_marker;
pub(crate) mod agent_sign_in;
pub(crate) mod agent_sign_in_status;
pub(crate) mod diagnostics;
pub(crate) mod diagnostics_why;
mod dialog;
pub(crate) mod image_build;
pub(crate) mod image_build_status;
pub(crate) mod image_status;
pub(crate) mod launch;
pub(crate) mod launch_status;
pub(crate) mod log_snapshot;
pub(crate) mod managed_session_input;
pub(crate) mod managed_session_output;
pub(crate) mod managed_session_status;
pub(crate) mod network;
pub(crate) mod run_control;
pub(crate) mod run_status;
pub(crate) mod session_cleanup;
pub(crate) mod session_diagnostics;
pub(crate) mod session_worktree;
pub(crate) mod sessions;
mod validation;
mod warnings;

#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use launch::{
    build_agent_run_plan, build_launch_response, build_plan_response, image_readiness_error,
    launch_prepared_with_approval_for_test, launch_run_with_registries,
    launch_run_with_registries_status_and_approval, trusted_launch_message_for_test,
    validate_launch_confirmation,
};

#[tauri::command]
pub(crate) fn get_setup_status() -> SetupStatus {
    collect_setup_status()
}

#[tauri::command]
pub(crate) fn list_agents() -> Vec<AgentProfileSummary> {
    agent_summaries()
}

#[tauri::command]
pub(crate) fn get_dashboard_status() -> DashboardStatus {
    collect_dashboard_status()
}

fn collect_setup_status() -> SetupStatus {
    let checks = collect_checks()
        .into_iter()
        .map(|check| CheckStatus {
            name: check.name,
            ok: check.ok,
            detail: check.detail,
            remedy: check.remedy,
        })
        .collect::<Vec<_>>();
    let blocker_count = checks.iter().filter(|check| !check.ok).count();
    SetupStatus {
        ok: blocker_count == 0,
        checks,
        blocker_count,
    }
}

fn agent_summaries() -> Vec<AgentProfileSummary> {
    profiles()
        .into_iter()
        .map(|profile| AgentProfileSummary {
            name: profile.name.to_string(),
            description: profile.description.to_string(),
            image: profile.image.to_string(),
            default_command: profile
                .command
                .iter()
                .map(|arg| (*arg).to_string())
                .collect(),
            provider_hosts: profile
                .provider_hosts
                .iter()
                .map(|host| (*host).to_string())
                .collect(),
            sign_in: agent_sign_in(profile.name).to_string(),
            broker: agent_broker(profile.name).to_string(),
            // Claude has a core in-sandbox login, but its code paste-back cannot
            // run in-app, so the GUI keeps it out of the in-app sign-in flow
            // (matching the explicit refusal in authorize_agent_sign_in).
            supports_in_app_sign_in: profile.name != "claude"
                && supports_sandbox_login(profile.name),
        })
        .collect()
}

fn collect_dashboard_status() -> DashboardStatus {
    let mut warnings = Vec::new();
    let recent_runs = match read_run_records(10) {
        Ok(records) => records.iter().map(run_summary).collect(),
        Err(error) => {
            warnings.push(format!("Run history is unavailable: {error}"));
            Vec::new()
        }
    };
    DashboardStatus {
        setup: collect_setup_status(),
        agents: agent_summaries(),
        active_runs: read_active_run_records().iter().map(run_summary).collect(),
        recent_runs,
        warnings,
    }
}

fn run_summary(record: &Value) -> RunSummary {
    RunSummary {
        run_id: string_field(record, "run_id"),
        profile: string_field(record, "profile"),
        workspace: project_label(&string_field(record, "workspace")),
        network: string_field(record, "network"),
        status: string_field(record, "status"),
        timestamp: string_field(record, "timestamp"),
        auth_scope: string_field(record, "auth_scope"),
        session: string_field(record, "session"),
    }
}

/// Canonical string-field extractor for command responses. Every value here is
/// rendered in the WebView, so control characters (terminal escapes included)
/// are stripped. Command modules under `commands` share this single helper
/// instead of keeping their own raw copies.
pub(super) fn string_field(record: &Value, name: &str) -> String {
    crate::registry_support::strip_control_chars(
        record.get(name).and_then(Value::as_str).unwrap_or("-"),
    )
}

fn defaulted(value: &str, fallback: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests;
