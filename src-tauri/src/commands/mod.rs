use std::collections::HashSet;
use std::path::PathBuf;
use std::thread;

use runhaven_core::doctor::collect_checks;
use runhaven_core::image::doctor::collect_image_status;
use runhaven_core::records::read_run_records;
use runhaven_core::runtime::active::read_active_run_records;
use runhaven_core::runtime::launch::{launch_run_plan, new_run_id};
use runhaven_core::runtime::plans::{
    AgentRunPlan, AuthScope, NetworkMode, RunOptions, WorkspaceScope, build_run_plan,
    normalize_provider_hosts,
};
use runhaven_core::runtime::profiles::{get_profile, profiles};
use serde_json::Value;

use crate::contracts::{
    AgentProfileSummary, CheckStatus, DashboardStatus, LaunchRunRequest, LaunchRunResponse,
    RunPlanRequest, RunPlanResponse, RunSummary, SetupStatus, StartedRunSnapshot,
};

pub(crate) mod diagnostics;
pub(crate) mod image_status;
pub(crate) mod log_snapshot;
pub(crate) mod run_control;
pub(crate) mod run_status;
mod validation;
mod warnings;

use validation::{validate_launch_request_bounds, validate_plan_request_bounds};
use warnings::plan_warnings;

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

#[tauri::command]
pub(crate) fn plan_run(request: RunPlanRequest) -> Result<RunPlanResponse, String> {
    build_plan_response(request)
}

#[tauri::command]
pub(crate) fn launch_run(request: LaunchRunRequest) -> Result<LaunchRunResponse, String> {
    validate_launch_confirmation(&request)?;
    let setup = collect_setup_status();
    if !setup.ok {
        return Err(
            "RunHaven setup is not ready. Fix setup checks before launching a run.".to_string(),
        );
    }
    validate_bundled_image_ready(&request.plan)?;
    let run_id = new_run_id();
    let (plan, response) = prepare_launch(request.plan, run_id.clone())?;
    thread::Builder::new()
        .name(format!("runhaven-launch-{run_id}"))
        .spawn(move || {
            if let Err(error) = launch_run_plan(&plan) {
                eprintln!("runhaven launch {run_id} failed: {error}");
            }
        })
        .map_err(|error| format!("could not start background run: {error}"))?;
    Ok(response)
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
        ssh_available: false,
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

fn build_plan_response(request: RunPlanRequest) -> Result<RunPlanResponse, String> {
    validate_plan_request_bounds(&request)?;
    let active_run_count = active_run_count();
    let warnings = plan_warnings(&request, active_run_count);
    let plan = build_agent_run_plan(&request, None)?;
    Ok(RunPlanResponse {
        profile: plan.profile_name,
        workspace: plan.workspace.display().to_string(),
        workspace_scope: plan.workspace_scope.as_str().to_string(),
        workspace_scope_note: plan.workspace_scope_note,
        state_volume: plan.state_volume,
        session: plan.session,
        container_name: plan.container_name,
        network_mode: plan.network_mode.as_str().to_string(),
        network_name: plan.network_name,
        egress_summary: plan.egress_summary,
        image: plan.image,
        provider_allowed_hosts: plan.provider_allowed_hosts,
        preflight_count: plan.preflight.len(),
        warnings,
    })
}

fn build_agent_run_plan(
    request: &RunPlanRequest,
    run_id: Option<String>,
) -> Result<AgentRunPlan, String> {
    validate_plan_request_bounds(request)?;
    let profile = get_profile(&request.agent).map_err(|error| error.to_string())?;
    let network =
        NetworkMode::try_from(request.network_mode.as_str()).map_err(|error| error.to_string())?;
    let workspace_scope = WorkspaceScope::try_from(request.workspace_scope.as_str())
        .map_err(|error| error.to_string())?;
    let provider_hosts =
        normalize_provider_hosts(&request.provider_hosts).map_err(|error| error.to_string())?;
    build_run_plan(RunOptions {
        profile,
        workspace: PathBuf::from(&request.workspace_path),
        agent_args: Vec::new(),
        image: non_empty(request.image.clone()),
        cpus: defaulted(&request.cpus, "4"),
        memory: defaulted(&request.memory, "4g"),
        network,
        workspace_scope,
        session: non_empty(request.session_name.clone()),
        auth_scope: AuthScope::Agent,
        read_only_workspace: request.read_only_workspace,
        ssh: false,
        env: request.env_names.clone(),
        user: defaulted(&request.user, "agent"),
        interactive: false,
        tty: false,
        allow_sensitive_workspace: request.allow_sensitive_workspace,
        allow_root_user: request.allow_root_user,
        provider_hosts,
        api_key_broker_env: None,
        worktree: None,
        run_id,
    })
    .map_err(|error| error.to_string())
}

fn prepare_launch(
    request: RunPlanRequest,
    run_id: String,
) -> Result<(AgentRunPlan, LaunchRunResponse), String> {
    let plan = build_agent_run_plan(&request, Some(run_id.clone()))?;
    let response = LaunchRunResponse {
        run_id: run_id.clone(),
        status: "started".to_string(),
        profile: plan.profile_name.clone(),
        workspace: plan.workspace.display().to_string(),
        state_volume: plan.state_volume.clone(),
        session: plan.session.clone(),
        network_mode: plan.network_mode.as_str().to_string(),
        snapshot: StartedRunSnapshot {
            run_id,
            status: "started".to_string(),
            profile: plan.profile_name.clone(),
            workspace: plan.workspace.display().to_string(),
            state_volume: plan.state_volume.clone(),
            session: plan.session.clone(),
            network_mode: plan.network_mode.as_str().to_string(),
            container_name: plan.container_name.clone(),
        },
    };
    Ok((plan, response))
}

#[cfg(test)]
fn build_launch_response(
    request: RunPlanRequest,
    run_id: String,
) -> Result<LaunchRunResponse, String> {
    prepare_launch(request, run_id).map(|(_, response)| response)
}

fn validate_launch_confirmation(request: &LaunchRunRequest) -> Result<(), String> {
    validate_launch_request_bounds(request)?;
    if !request.confirm_launch {
        return Err("Confirm the launch before starting a run.".to_string());
    }
    let confirmed = request
        .confirmed_warnings
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    for warning in plan_warnings(&request.plan, active_run_count()) {
        if !confirmed.contains(warning.code.as_str()) {
            return Err(format!(
                "Confirm warning {} before starting a run: {}",
                warning.code, warning.message
            ));
        }
    }
    Ok(())
}

fn validate_bundled_image_ready(request: &RunPlanRequest) -> Result<(), String> {
    if request
        .image
        .as_deref()
        .is_some_and(|image| !image.trim().is_empty())
    {
        return Ok(());
    }
    let report = collect_image_status(&request.agent).map_err(|error| error.to_string())?;
    if let Some(error) = image_readiness_error(
        request,
        report.image.ready,
        &report.image.status,
        &report.image.agent,
        &report.image.image,
        report.image.fix_command.as_deref(),
    ) {
        return Err(error);
    }
    Ok(())
}

fn image_readiness_error(
    request: &RunPlanRequest,
    ready: bool,
    status: &str,
    agent: &str,
    image: &str,
    fix_command: Option<&str>,
) -> Option<String> {
    if ready
        || request
            .image
            .as_deref()
            .is_some_and(|image| !image.trim().is_empty())
    {
        return None;
    }
    let mut message = format!(
        "Image for {agent} is {status}: {image}. Rebuild the bundled image before launching."
    );
    if let Some(command) = fix_command {
        message.push_str(" Run: ");
        message.push_str(command);
    }
    Some(message)
}

fn run_summary(record: &Value) -> RunSummary {
    RunSummary {
        run_id: field(record, "run_id"),
        profile: field(record, "profile"),
        workspace: field(record, "workspace"),
        network: field(record, "network"),
        status: field(record, "status"),
        timestamp: field(record, "timestamp"),
        state_volume: field(record, "state_volume"),
        session: field(record, "session"),
    }
}

fn field(record: &Value, name: &str) -> String {
    record
        .get(name)
        .and_then(Value::as_str)
        .unwrap_or("-")
        .to_string()
}

fn defaulted(value: &str, fallback: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        trimmed.to_string()
    }
}

fn non_empty(value: Option<String>) -> Option<String> {
    value.and_then(|text| {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn active_run_count() -> usize {
    read_active_run_records().len()
}

#[cfg(test)]
mod tests;
