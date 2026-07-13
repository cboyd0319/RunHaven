use runhaven_core::doctor::collect_checks;
use runhaven_core::provider::agent_sign_in_record::agent_setup_readiness;
use runhaven_core::provider::auth_profiles::{agent_broker, agent_sign_in};
use runhaven_core::records::read_run_records;
use runhaven_core::runtime::active::read_active_run_records;
use runhaven_core::runtime::login::supports_sandbox_login;
use runhaven_core::runtime::profiles::agent_profiles;
use runhaven_core::runtime::state::list_state_volumes;
use runhaven_core::ui_contracts::project_label;
use serde_json::Value;

#[cfg(test)]
pub(crate) use runhaven_core::runtime::plans::AuthScope;
#[cfg(test)]
pub(crate) use runhaven_core::runtime::profiles::get_profile;

use crate::contracts::{
    AgentProfileSummary, CheckStatus, DashboardStatus, RunSummary, SetupAction, SetupActionId,
    SetupStatus,
};

#[cfg(test)]
pub(crate) use crate::contracts::{LaunchRunRequest, RunPlanRequest};

mod access;
mod active_marker;
pub(crate) mod attention_badge;
pub(crate) mod diagnostics;
pub(crate) mod diagnostics_why;
pub(crate) mod dialog;
pub(crate) mod image_build;
pub(crate) mod image_build_status;
pub(crate) mod image_status;
pub(crate) mod launch;
pub(crate) mod launch_status;
pub(crate) mod log_snapshot;
pub(crate) mod managed_session_input;
pub(crate) mod managed_session_output;
pub(crate) mod managed_session_status;
pub(crate) mod menu_sync;
pub(crate) mod network;
pub(crate) mod run_control;
pub(crate) mod run_status;
pub(crate) mod session_cleanup;
pub(crate) mod session_diagnostics;
pub(crate) mod session_window;
pub(crate) mod session_worktree;
pub(crate) mod sessions;
pub(crate) mod setup_recovery;
mod validation;
mod warnings;

#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use launch::{
    build_agent_run_plan, build_launch_response, build_plan_response,
    build_worktree_run_plan_with_failure_for_test, image_readiness_error,
    launch_prepared_with_approval_for_test, launch_run_with_registries,
    launch_run_with_registries_status_and_approval, trusted_launch_message_for_test,
    validate_launch_confirmation,
};

#[tauri::command]
pub(crate) fn get_dashboard_status() -> DashboardStatus {
    collect_dashboard_status()
}

fn collect_setup_status() -> SetupStatus {
    let checks = desktop_setup_checks(collect_checks());
    let blocker_count = checks.iter().filter(|check| !check.ok).count();
    SetupStatus {
        ok: blocker_count == 0,
        checks,
        blocker_count,
    }
}

fn desktop_setup_checks(checks: Vec<runhaven_core::doctor::Check>) -> Vec<CheckStatus> {
    checks
        .into_iter()
        .filter(|check| check.name != "rust")
        .map(check_status)
        .collect()
}

fn check_status(check: runhaven_core::doctor::Check) -> CheckStatus {
    let action = if check.ok {
        None
    } else if matches!(
        check.name.as_str(),
        "Apple container CLI" | "Apple container version"
    ) {
        Some(SetupAction {
            id: SetupActionId::OpenAppleContainerInstaller,
            label: if check.name == "Apple container version" {
                "Download required version".to_string()
            } else {
                "Download Apple container".to_string()
            },
        })
    } else if check.name == "container system" {
        Some(SetupAction {
            id: SetupActionId::StartContainerSystem,
            label: "Start container service".to_string(),
        })
    } else {
        None
    };
    let (detail, remedy) = desktop_check_copy(&check.name, check.ok);
    CheckStatus {
        name: check.name,
        ok: check.ok,
        detail,
        remedy,
        action,
    }
}

/// Doctor output is a developer surface and can contain host paths, exact
/// commands, URLs, and runtime dumps. The desktop contract needs only a plain
/// result and next step, so raw detail never crosses into the WebView.
fn desktop_check_copy(name: &str, ok: bool) -> (String, String) {
    if ok {
        return ("Ready.".to_string(), "No action needed.".to_string());
    }
    let (detail, remedy) = match name {
        "operating system" => (
            "RunHaven runs only on macOS.",
            "Use a supported Mac, then check again.",
        ),
        "macOS" => (
            "This macOS version is not supported.",
            "Update to macOS 26 or newer, then check again.",
        ),
        "architecture" => (
            "RunHaven needs a Mac with Apple silicon.",
            "Use a supported Apple silicon Mac.",
        ),
        "Apple container CLI" => (
            "Apple container is not installed.",
            "Download Apple container, install it, then check again.",
        ),
        "Apple container version" => (
            "The installed Apple container version is not supported.",
            "Download the required version, install it, then check again.",
        ),
        "container system" => (
            "Apple container's local service is not running.",
            "Start the service, then check again.",
        ),
        "Apple container runtime commit"
        | "Apple container builder image"
        | "Apple container vminit image"
        | "Apple container kernel" => (
            "Apple container's runtime files do not match the supported version.",
            "Reinstall the supported Apple container version, then check again.",
        ),
        _ => (
            "A required setup check did not pass.",
            "Check the supported setup, then try again.",
        ),
    };
    (detail.to_string(), remedy.to_string())
}

fn agent_summaries() -> Vec<AgentProfileSummary> {
    agent_profiles()
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
            supports_in_app_sign_in: supports_sandbox_login(profile.name),
        })
        .collect()
}

fn collect_dashboard_status() -> DashboardStatus {
    let mut warnings = Vec::new();
    let setup = collect_setup_status();
    let recent_runs = match read_run_records(10) {
        Ok(records) => records.iter().map(run_summary).collect(),
        Err(error) => {
            warnings.push(format!("Run history is unavailable: {error}"));
            Vec::new()
        }
    };
    let names = agent_profiles()
        .iter()
        .map(|profile| profile.name)
        .collect::<Vec<_>>();
    let state_volumes = if setup.ok {
        match list_state_volumes(None) {
            Ok(volumes) => volumes,
            Err(_) => {
                warnings
                    .push("Agent sign-in status is unavailable. Choose Check again.".to_string());
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };
    let agent_readiness = agent_setup_readiness(&names, &state_volumes);
    DashboardStatus {
        setup,
        agents: agent_summaries(),
        agent_readiness,
        active_runs: read_active_run_records().iter().map(run_summary).collect(),
        recent_runs,
        warnings,
        app_version: desktop_app_version(),
    }
}

fn desktop_app_version() -> String {
    let version = env!("CARGO_PKG_VERSION");
    if cfg!(debug_assertions) {
        format!("{version} Debug")
    } else {
        version.to_string()
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
