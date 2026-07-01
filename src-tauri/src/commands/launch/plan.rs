use std::collections::HashSet;
use std::path::{Path, PathBuf};

use runhaven_core::image::doctor::collect_image_status;
use runhaven_core::provider::runtime::validate_runtime_auth_broker_environment;
use runhaven_core::runtime::active::read_active_run_records;
use runhaven_core::runtime::plans::{
    AgentRunPlan, AuthScope, NetworkMode, RunOptions, WorkspaceScope, WorktreeRun, build_run_plan,
    normalize_provider_hosts,
};
use runhaven_core::runtime::profiles::get_profile;
use runhaven_core::runtime::worktrees::create_worktree_for_run;

use crate::contracts::{
    LaunchRunRequest, LaunchRunResponse, PlanWarning, RunPlanRequest, RunPlanResponse,
    StartedRunSnapshot,
};

use super::super::access::plan_access;
use super::super::defaulted;
use super::super::validation::{validate_launch_request_bounds, validate_plan_request_bounds};
use super::super::warnings::plan_warnings;

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
        worktree: request.worktree,
        worktree_note: worktree_plan_note(request.worktree),
    })
}

/// Beginner-friendly note shown in the plan review when the user opts into a
/// worktree run. Path-free and secret-free.
fn worktree_plan_note(worktree: bool) -> Option<String> {
    if !worktree {
        return None;
    }
    Some(
        "RunHaven will run this agent in a new, isolated git branch (a worktree). \
         Your workspace must be a clean git repository. Your original files are not \
         changed until you choose to merge."
            .to_string(),
    )
}

pub(crate) fn build_agent_run_plan(
    request: &RunPlanRequest,
    run_id: Option<String>,
) -> Result<AgentRunPlan, String> {
    validate_plan_request_bounds(request)?;
    let options = agent_run_options(
        request,
        run_id,
        PathBuf::from(&request.workspace_path),
        None,
    )?;
    build_run_plan(options).map_err(|error| error.to_string())
}

/// Build the run plan for a confirmed worktree launch: create the RunHaven-owned
/// worktree branch through the tested core function, then plan onto the mounted
/// worktree workspace. Creation errors are collapsed to fixed, path-free guidance.
pub(super) fn build_worktree_run_plan(
    request: &RunPlanRequest,
    run_id: &str,
) -> Result<AgentRunPlan, String> {
    validate_plan_request_bounds(request)?;
    let workspace_scope = WorkspaceScope::try_from(request.workspace_scope.as_str())
        .map_err(|error| error.to_string())?;
    let worktree = create_worktree_for_run(
        Path::new(&request.workspace_path),
        workspace_scope,
        request.allow_sensitive_workspace,
        &request.agent,
        run_id,
    )
    .map_err(sanitize_worktree_creation_error)?;
    let mounted_workspace = worktree.mounted_workspace.clone();
    let options = agent_run_options(
        request,
        Some(run_id.to_string()),
        mounted_workspace,
        Some(worktree),
    )?;
    build_run_plan(options).map_err(|error| error.to_string())
}

/// Map worktree creation failures to fixed, path-free, beginner-friendly guidance.
/// The raw core error can name the source repo or internal worktree path, so it
/// never reaches the WebView.
fn sanitize_worktree_creation_error(error: impl ToString) -> String {
    let message = error.to_string();
    if message.contains("clean source git worktree") {
        "Worktree mode needs a clean git workspace. Commit or stash your changes, then try again."
            .to_string()
    } else if message.contains("committed HEAD") {
        "Worktree mode needs a git repository with at least one commit.".to_string()
    } else if message.contains("requires a git worktree") {
        "Worktree mode needs the workspace to be a git repository.".to_string()
    } else {
        "RunHaven could not create an isolated worktree for this run.".to_string()
    }
}

fn agent_run_options(
    request: &RunPlanRequest,
    run_id: Option<String>,
    workspace: PathBuf,
    worktree: Option<WorktreeRun>,
) -> Result<RunOptions, String> {
    let profile = get_profile(&request.agent).map_err(|error| error.to_string())?;
    let network =
        NetworkMode::try_from(request.network_mode.as_str()).map_err(|error| error.to_string())?;
    let workspace_scope = WorkspaceScope::try_from(request.workspace_scope.as_str())
        .map_err(|error| error.to_string())?;
    let auth_scope =
        AuthScope::try_from(request.auth_scope.as_str()).map_err(|error| error.to_string())?;
    let provider_hosts =
        normalize_provider_hosts(&request.provider_hosts).map_err(|error| error.to_string())?;
    Ok(RunOptions {
        profile,
        workspace,
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
        worktree,
        run_id,
    })
}

pub(super) fn prepare_launch(
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

pub(super) fn validate_bundled_image_ready(request: &RunPlanRequest) -> Result<(), String> {
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
