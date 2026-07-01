use std::thread;

use runhaven_core::runtime::launch::{launch_run_plan_with_runner, new_run_id};
use runhaven_core::runtime::plans::AgentRunPlan;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::MessageDialogKind;

use crate::commands::dialog::confirm_native;
use crate::contracts::{
    LaunchRunRequest, LaunchRunResponse, PlanWarning, RunPlanRequest, RunPlanResponse,
};
use crate::launch_runner::TauriLaunchRunner;
use crate::launch_status::LaunchStatusRegistry;
use crate::session_input::SessionInputRegistry;
use crate::session_output::SessionOutputRegistry;

use super::collect_setup_status;

mod plan;

#[cfg(test)]
pub(crate) use plan::{build_agent_run_plan, build_launch_response, image_readiness_error};
pub(crate) use plan::{build_plan_response, validate_launch_confirmation};

use plan::{build_worktree_run_plan, prepare_launch, validate_bundled_image_ready};

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
    let prepared = prepare_confirmed_launch(request)?;
    if !trusted_launch_confirmed(
        app,
        &prepared.preview_plan,
        &prepared.warnings,
        prepared.worktree(),
    )
    .await?
    {
        return Err("Launch canceled before starting the run.".to_string());
    }
    // The worktree branch is created only here, after the user confirms, so no
    // git mutation happens before approval.
    launch_confirmed(prepared, launch_status, input_registry, output_registry)
}

/// Start a confirmed launch: finalize the plan (creating the worktree branch for
/// a worktree run), mark the launch as starting, and spawn the background runner.
/// Both the production command and the test seam call this so the post-approval
/// path has one implementation.
fn launch_confirmed(
    prepared: PreparedLaunch,
    launch_status: LaunchStatusRegistry,
    input_registry: SessionInputRegistry,
    output_registry: SessionOutputRegistry,
) -> Result<LaunchRunResponse, String> {
    let run_id = prepared.run_id.clone();
    let response = prepared.response.clone();
    let plan = finalize_launch_plan(prepared)?;
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

/// A confirmed, validated launch ready for the trusted confirmation dialog. The
/// preview plan and response use the source workspace (worktree runs never
/// expose the internal worktree path to the WebView); the worktree branch is
/// created later, after approval, from the carried plan request.
struct PreparedLaunch {
    run_id: String,
    request: RunPlanRequest,
    preview_plan: AgentRunPlan,
    response: LaunchRunResponse,
    warnings: Vec<PlanWarning>,
}

impl PreparedLaunch {
    fn worktree(&self) -> bool {
        self.request.worktree
    }
}

/// Build the plan RunHaven actually launches. For a worktree run this creates
/// the RunHaven-owned branch and re-plans onto the mounted worktree workspace;
/// otherwise it is the already-validated source plan.
fn finalize_launch_plan(prepared: PreparedLaunch) -> Result<AgentRunPlan, String> {
    if !prepared.request.worktree {
        return Ok(prepared.preview_plan);
    }
    build_worktree_run_plan(&prepared.request, &prepared.run_id)
}

#[cfg(test)]
pub(crate) fn launch_run_with_registries(
    request: LaunchRunRequest,
    input_registry: SessionInputRegistry,
    output_registry: SessionOutputRegistry,
) -> Result<LaunchRunResponse, String> {
    launch_run_with_registries_status_and_approval(
        request,
        input_registry,
        output_registry,
        LaunchStatusRegistry::default(),
        |_, _| Ok(true),
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
    let prepared = prepare_confirmed_launch(request)?;
    if !approve(&prepared.preview_plan, &prepared.warnings)? {
        return Err("Launch canceled before starting the run.".to_string());
    }
    launch_confirmed(prepared, launch_status, input_registry, output_registry)
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

fn prepare_confirmed_launch(request: LaunchRunRequest) -> Result<PreparedLaunch, String> {
    let warnings = validate_launch_confirmation(&request)?;
    let setup = collect_setup_status();
    if !setup.ok {
        return Err(
            "RunHaven setup is not ready. Fix setup checks before launching a run.".to_string(),
        );
    }
    validate_bundled_image_ready(&request.plan)?;
    let run_id = new_run_id();
    let plan_request = request.plan.clone();
    let (preview_plan, response) = prepare_launch(request.plan, run_id.clone())?;
    Ok(PreparedLaunch {
        run_id,
        request: plan_request,
        preview_plan,
        response,
        warnings,
    })
}

async fn trusted_launch_confirmed(
    app: AppHandle,
    plan: &AgentRunPlan,
    warnings: &[PlanWarning],
    worktree: bool,
) -> Result<bool, String> {
    let message = trusted_launch_message(plan, warnings, worktree);
    let kind = if warnings.is_empty() {
        MessageDialogKind::Info
    } else {
        MessageDialogKind::Warning
    };
    confirm_native(app, "Confirm RunHaven launch", "Launch", kind, message).await
}

fn trusted_launch_message(plan: &AgentRunPlan, warnings: &[PlanWarning], worktree: bool) -> String {
    let provider_hosts = if plan.provider_allowed_hosts.is_empty() {
        "none".to_string()
    } else {
        plan.provider_allowed_hosts.join(", ")
    };
    let api_key_broker = plan.api_key_broker_env.as_deref().unwrap_or("none");
    let worktree_line = if worktree {
        "yes (RunHaven creates a new isolated git branch; your files change only if you merge)"
    } else {
        "no (runs directly in your workspace)"
    };
    let mut message = format!(
        "Agent: {}\nWorkspace: {}\nSession: {}\nImage: {}\nAuth: {}\nNetwork: {}\nProvider hosts: {}\nAPI key broker: {}\nWorktree: {}\n\nRunHaven will start this session after you confirm.",
        plan.profile_name,
        plan.workspace.display(),
        plan.session,
        plan.image,
        plan.auth_scope.as_str(),
        plan.network_mode.as_str(),
        provider_hosts,
        api_key_broker,
        worktree_line,
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
    worktree: bool,
) -> String {
    trusted_launch_message(plan, warnings, worktree)
}
