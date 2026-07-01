use std::collections::HashSet;
use std::path::PathBuf;
use std::thread;

use runhaven_core::image::doctor::collect_image_status;
use runhaven_core::provider::runtime::validate_runtime_auth_broker_environment;
use runhaven_core::runtime::active::read_active_run_records;
use runhaven_core::runtime::launch::{launch_run_plan_with_runner, new_run_id};
use runhaven_core::runtime::plans::{
    AgentRunPlan, AuthScope, NetworkMode, RunOptions, WorkspaceScope, build_run_plan,
    normalize_provider_hosts,
};
use runhaven_core::runtime::profiles::get_profile;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

use crate::contracts::{
    LaunchRunRequest, LaunchRunResponse, PlanWarning, RunPlanRequest, RunPlanResponse,
    StartedRunSnapshot,
};
use crate::launch_runner::TauriLaunchRunner;
use crate::launch_status::LaunchStatusRegistry;
use crate::session_input::SessionInputRegistry;
use crate::session_output::SessionOutputRegistry;

use super::access::plan_access;
use super::validation::{validate_launch_request_bounds, validate_plan_request_bounds};
use super::warnings::plan_warnings;
use super::{collect_setup_status, defaulted};

#[tauri::command]
pub(crate) fn plan_run(request: RunPlanRequest) -> Result<RunPlanResponse, String> {
    build_plan_response(request)
}

#[tauri::command]
pub(crate) async fn launch_run(
    app: AppHandle,
    request: LaunchRunRequest,
    input_registry: State<'_, SessionInputRegistry>,
    output_registry: State<'_, SessionOutputRegistry>,
    launch_status: State<'_, LaunchStatusRegistry>,
) -> Result<LaunchRunResponse, String> {
    let input_registry = input_registry.inner().clone();
    let output_registry = output_registry.inner().clone();
    let launch_status = launch_status.inner().clone();
    let (run_id, plan, response, warnings) = prepare_confirmed_launch(request)?;
    if !trusted_launch_confirmed(app, &plan, &warnings).await? {
        return Err("Launch canceled before starting the run.".to_string());
    }
    launch_status.mark_starting(&run_id, &plan.profile_name, &plan.session)?;
    if let Err(error) = spawn_launch_thread(
        run_id.clone(),
        plan,
        input_registry,
        output_registry,
        launch_status.clone(),
    ) {
        let _ = launch_status.mark_failed(&run_id);
        return Err(error);
    }
    Ok(response)
}

#[cfg(test)]
pub(crate) fn launch_run_with_registries(
    request: LaunchRunRequest,
    input_registry: SessionInputRegistry,
    output_registry: SessionOutputRegistry,
) -> Result<LaunchRunResponse, String> {
    launch_run_with_registries_and_approval(request, input_registry, output_registry, |_, _| {
        Ok(true)
    })
}

#[cfg(test)]
pub(crate) fn launch_run_with_registries_and_approval(
    request: LaunchRunRequest,
    input_registry: SessionInputRegistry,
    output_registry: SessionOutputRegistry,
    approve: impl FnOnce(&AgentRunPlan, &[PlanWarning]) -> Result<bool, String>,
) -> Result<LaunchRunResponse, String> {
    launch_run_with_registries_status_and_approval(
        request,
        input_registry,
        output_registry,
        LaunchStatusRegistry::default(),
        approve,
    )
}

#[cfg(test)]
pub(crate) fn launch_run_with_registries_status_and_approval(
    request: LaunchRunRequest,
    input_registry: SessionInputRegistry,
    output_registry: SessionOutputRegistry,
    launch_status: LaunchStatusRegistry,
    approve: impl FnOnce(&AgentRunPlan, &[PlanWarning]) -> Result<bool, String>,
) -> Result<LaunchRunResponse, String> {
    let (run_id, plan, response, warnings) = prepare_confirmed_launch(request)?;
    if !approve(&plan, &warnings)? {
        return Err("Launch canceled before starting the run.".to_string());
    }
    launch_status.mark_starting(&run_id, &plan.profile_name, &plan.session)?;
    if let Err(error) = spawn_launch_thread(
        run_id.clone(),
        plan,
        input_registry,
        output_registry,
        launch_status.clone(),
    ) {
        let _ = launch_status.mark_failed(&run_id);
        return Err(error);
    }
    Ok(response)
}

#[cfg(test)]
pub(crate) fn launch_prepared_with_approval_for_test(
    run_id: String,
    plan: AgentRunPlan,
    response: LaunchRunResponse,
    warnings: Vec<PlanWarning>,
    input_registry: SessionInputRegistry,
    output_registry: SessionOutputRegistry,
    approve: impl FnOnce(&AgentRunPlan, &[PlanWarning]) -> Result<bool, String>,
) -> Result<LaunchRunResponse, String> {
    if !approve(&plan, &warnings)? {
        return Err("Launch canceled before starting the run.".to_string());
    }
    let launch_status = LaunchStatusRegistry::default();
    launch_status.mark_starting(&run_id, &plan.profile_name, &plan.session)?;
    if let Err(error) = spawn_launch_thread(
        run_id.clone(),
        plan,
        input_registry,
        output_registry,
        launch_status.clone(),
    ) {
        let _ = launch_status.mark_failed(&run_id);
        return Err(error);
    }
    Ok(response)
}

fn spawn_launch_thread(
    run_id: String,
    plan: AgentRunPlan,
    input_registry: SessionInputRegistry,
    output_registry: SessionOutputRegistry,
    launch_status: LaunchStatusRegistry,
) -> Result<(), String> {
    thread::Builder::new()
        .name(format!("runhaven-launch-{run_id}"))
        .spawn(move || {
            let mut runner =
                TauriLaunchRunner::new(input_registry, output_registry, launch_status.clone());
            match launch_run_plan_with_runner(&plan, &mut runner) {
                Ok(code) => {
                    if let Err(error) = launch_status.mark_finished(&run_id, code) {
                        eprintln!("runhaven launch {run_id} status update failed: {error}");
                    }
                }
                Err(error) => {
                    if let Err(status_error) = launch_status.mark_failed(&run_id) {
                        eprintln!("runhaven launch {run_id} status update failed: {status_error}");
                    }
                    eprintln!("runhaven launch {run_id} failed: {error}");
                }
            }
        })
        .map_err(|error| format!("could not start background run: {error}"))?;
    Ok(())
}

fn prepare_confirmed_launch(
    request: LaunchRunRequest,
) -> Result<(String, AgentRunPlan, LaunchRunResponse, Vec<PlanWarning>), String> {
    let warnings = validate_launch_confirmation(&request)?;
    let setup = collect_setup_status();
    if !setup.ok {
        return Err(
            "RunHaven setup is not ready. Fix setup checks before launching a run.".to_string(),
        );
    }
    validate_bundled_image_ready(&request.plan)?;
    let run_id = new_run_id();
    let (plan, response) = prepare_launch(request.plan, run_id.clone())?;
    Ok((run_id, plan, response, warnings))
}

async fn trusted_launch_confirmed(
    app: AppHandle,
    plan: &AgentRunPlan,
    warnings: &[PlanWarning],
) -> Result<bool, String> {
    let message = trusted_launch_message(plan, warnings);
    let has_warnings = !warnings.is_empty();
    tauri::async_runtime::spawn_blocking(move || {
        app.dialog()
            .message(message)
            .title("Confirm RunHaven launch")
            .kind(if has_warnings {
                MessageDialogKind::Warning
            } else {
                MessageDialogKind::Info
            })
            .buttons(MessageDialogButtons::OkCancelCustom(
                "Launch".to_string(),
                "Cancel".to_string(),
            ))
            .blocking_show()
    })
    .await
    .map_err(|error| format!("could not show launch confirmation: {error}"))
}

fn trusted_launch_message(plan: &AgentRunPlan, warnings: &[PlanWarning]) -> String {
    let provider_hosts = if plan.provider_allowed_hosts.is_empty() {
        "none".to_string()
    } else {
        plan.provider_allowed_hosts.join(", ")
    };
    let api_key_broker = plan.api_key_broker_env.as_deref().unwrap_or("none");
    let mut message = format!(
        "Agent: {}\nWorkspace: {}\nSession: {}\nImage: {}\nAuth: {}\nNetwork: {}\nProvider hosts: {}\nAPI key broker: {}\n\nRunHaven will start this session after you confirm.",
        plan.profile_name,
        plan.workspace.display(),
        plan.session,
        plan.image,
        plan.auth_scope.as_str(),
        plan.network_mode.as_str(),
        provider_hosts,
        api_key_broker,
    );
    if !warnings.is_empty() {
        message.push_str("\n\nWarnings:");
        for warning in warnings {
            message.push_str("\n- ");
            message.push_str(&warning.message);
        }
    }
    message
}

#[cfg(test)]
pub(crate) fn trusted_launch_message_for_test(
    plan: &AgentRunPlan,
    warnings: &[PlanWarning],
) -> String {
    trusted_launch_message(plan, warnings)
}

pub(crate) fn build_plan_response(request: RunPlanRequest) -> Result<RunPlanResponse, String> {
    validate_plan_request_bounds(&request)?;
    let active_run_count = active_run_count();
    let warnings = plan_warnings(&request, active_run_count);
    let plan = build_agent_run_plan(&request, None)?;
    let access = plan_access(&plan);
    Ok(RunPlanResponse {
        profile: plan.profile_name,
        workspace: plan.workspace.display().to_string(),
        workspace_scope: plan.workspace_scope.as_str().to_string(),
        workspace_scope_note: plan.workspace_scope_note,
        auth_scope: plan.auth_scope.as_str().to_string(),
        state_volume: plan.state_volume,
        session: plan.session,
        container_name: plan.container_name,
        network_mode: plan.network_mode.as_str().to_string(),
        network_name: plan.network_name,
        egress_summary: plan.egress_summary,
        image: plan.image,
        provider_allowed_hosts: plan.provider_allowed_hosts,
        preflight_count: plan.preflight.len(),
        access,
        warnings,
    })
}

pub(crate) fn build_agent_run_plan(
    request: &RunPlanRequest,
    run_id: Option<String>,
) -> Result<AgentRunPlan, String> {
    validate_plan_request_bounds(request)?;
    let profile = get_profile(&request.agent).map_err(|error| error.to_string())?;
    let network =
        NetworkMode::try_from(request.network_mode.as_str()).map_err(|error| error.to_string())?;
    let workspace_scope = WorkspaceScope::try_from(request.workspace_scope.as_str())
        .map_err(|error| error.to_string())?;
    let auth_scope =
        AuthScope::try_from(request.auth_scope.as_str()).map_err(|error| error.to_string())?;
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
        auth_scope,
        read_only_workspace: request.read_only_workspace,
        ssh: false,
        env: request.env_names.clone(),
        user: defaulted(&request.user, "agent"),
        interactive: true,
        tty: false,
        allow_sensitive_workspace: request.allow_sensitive_workspace,
        allow_root_user: request.allow_root_user,
        provider_hosts,
        api_key_broker_env: non_empty(request.api_key_broker_env.clone()),
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
    validate_runtime_auth_broker_environment(&plan).map_err(|error| error.to_string())?;
    let response = LaunchRunResponse {
        run_id: run_id.clone(),
        status: "started".to_string(),
        profile: plan.profile_name.clone(),
        workspace: plan.workspace.display().to_string(),
        state_volume: plan.state_volume.clone(),
        auth_scope: plan.auth_scope.as_str().to_string(),
        session: plan.session.clone(),
        network_mode: plan.network_mode.as_str().to_string(),
        snapshot: StartedRunSnapshot {
            run_id,
            status: "started".to_string(),
            profile: plan.profile_name.clone(),
            workspace: plan.workspace.display().to_string(),
            state_volume: plan.state_volume.clone(),
            auth_scope: plan.auth_scope.as_str().to_string(),
            session: plan.session.clone(),
            network_mode: plan.network_mode.as_str().to_string(),
            container_name: plan.container_name.clone(),
        },
    };
    Ok((plan, response))
}

#[cfg(test)]
pub(crate) fn build_launch_response(
    request: RunPlanRequest,
    run_id: String,
) -> Result<LaunchRunResponse, String> {
    prepare_launch(request, run_id).map(|(_, response)| response)
}

pub(crate) fn validate_launch_confirmation(
    request: &LaunchRunRequest,
) -> Result<Vec<PlanWarning>, String> {
    validate_launch_request_bounds(request)?;
    if !request.confirm_launch {
        return Err("Confirm the launch before starting a run.".to_string());
    }
    let confirmed = request
        .confirmed_warnings
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    let warnings = plan_warnings(&request.plan, active_run_count());
    for warning in &warnings {
        if !confirmed.contains(warning.code.as_str()) {
            return Err(format!(
                "Confirm warning {} before starting a run: {}",
                warning.code, warning.message
            ));
        }
    }
    Ok(warnings)
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

pub(crate) fn image_readiness_error(
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

fn active_run_count() -> usize {
    read_active_run_records().len()
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
