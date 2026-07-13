use std::thread;

use runhaven_core::runtime::launch::{
    launch_run_plan_with_runner, new_run_id, write_worktree_setup_failure_record,
};
use runhaven_core::runtime::plans::AgentRunPlan;
use runhaven_core::runtime::worktrees::preview_worktree;
use tauri::{AppHandle, State};

use crate::contracts::{
    LaunchRunRequest, LaunchRunResponse, PlanWarning, RunPlanRequest, RunPlanResponse,
};
use crate::launch_runner::TauriLaunchRunner;
use crate::launch_status::LaunchStatusRegistry;
use crate::session_input::SessionInputRegistry;
use crate::session_output::SessionOutputRegistry;

use super::collect_setup_status;
use super::validation::validate_plan_request_bounds;

pub(super) const LAUNCH_STATE_CHANGED: &str = "The project or session settings changed before RunHaven could start. Check your folder and options, then try again.";

mod confirmation;
mod plan;

use confirmation::trusted_launch_confirmed;

#[cfg(test)]
pub(crate) use confirmation::{
    trusted_launch_kind_for_test, trusted_launch_message_for_test, trusted_launch_title_for_test,
};

pub(crate) use plan::build_plan_response;
#[cfg(test)]
pub(crate) use plan::{build_agent_run_plan, build_launch_response, image_readiness_error};

use plan::{
    build_worktree_run_plan, current_launch_warnings, prepare_launch,
    sanitize_worktree_creation_error, validate_bundled_image_ready,
};

#[tauri::command]
pub(crate) fn plan_run(request: RunPlanRequest) -> Result<RunPlanResponse, String> {
    build_plan_response(request)
}

/// Returns `None` when the user cancels the native confirmation. Cancellation
/// is a successful no-op; preparation, dialog, and launch failures remain errors.
#[tauri::command]
pub(crate) async fn launch_run(
    app: AppHandle,
    request: LaunchRunRequest,
    input_registry: State<'_, SessionInputRegistry>,
    output_registry: State<'_, SessionOutputRegistry>,
    launch_status: State<'_, LaunchStatusRegistry>,
) -> Result<Option<LaunchRunResponse>, String> {
    let input_registry = input_registry.inner().clone();
    let output_registry = output_registry.inner().clone();
    let launch_status = launch_status.inner().clone();
    let prepared = prepare_confirmed_launch(request)?;
    if !trusted_launch_confirmed(
        app,
        &prepared.preview_plan,
        &prepared.warnings,
        prepared.worktree(),
        &prepared.request,
    )
    .await?
    {
        return Ok(None);
    }
    // The worktree branch is created only here, after the user confirms, so no
    // git mutation happens before approval.
    let response = launch_confirmed(prepared, launch_status, input_registry, output_registry)?;
    Ok(Some(response))
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
    let plan = finalize_confirmed_launch_plan(prepared)?;
    if let Err(error) = launch_status.mark_starting(&run_id, &plan.profile_name, &plan.session) {
        record_unstarted_worktree_failure(&plan)?;
        return Err(error);
    }
    let recovery_plan = plan.worktree.as_ref().map(|_| plan.clone());
    if let Err(error) = spawn_launch_thread(
        run_id.clone(),
        plan,
        input_registry,
        output_registry,
        launch_status.clone(),
    ) {
        let _ = launch_status.mark_failed(&run_id);
        if let Some(plan) = recovery_plan.as_ref() {
            record_unstarted_worktree_failure(plan)?;
        }
        return Err(error);
    }
    Ok(response)
}

pub(crate) fn record_unstarted_worktree_failure(plan: &AgentRunPlan) -> Result<(), String> {
    let Some(worktree) = plan.worktree.as_ref() else {
        return Ok(());
    };
    write_worktree_setup_failure_record(plan, worktree).map_err(|_| {
        "RunHaven could not save recovery information for the isolated worktree.".to_string()
    })
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
    worktree_preview: Option<(std::path::PathBuf, std::path::PathBuf, String)>,
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
    build_worktree_run_plan(&prepared.request, &prepared.run_id, &prepared.preview_plan)
}

fn finalize_confirmed_launch_plan(prepared: PreparedLaunch) -> Result<AgentRunPlan, String> {
    if current_launch_warnings(&prepared.request) != prepared.warnings {
        return Err(LAUNCH_STATE_CHANGED.to_string());
    }
    if let Some(expected) = &prepared.worktree_preview {
        let current = preview_worktree(
            std::path::Path::new(&prepared.request.workspace_path),
            prepared.preview_plan.workspace_scope,
            prepared.request.allow_sensitive_workspace,
        )
        .map_err(|_| LAUNCH_STATE_CHANGED.to_string())?;
        if &current != expected {
            return Err(LAUNCH_STATE_CHANGED.to_string());
        }
    }
    finalize_launch_plan(prepared)
}

#[cfg(test)]
pub(crate) use plan::{
    build_worktree_run_plan_with_failure_for_test, sanitize_worktree_creation_error_for_test,
};

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
pub(crate) fn finalize_launch_with_approval_for_test(
    request: LaunchRunRequest,
    approve: impl FnOnce(&AgentRunPlan, &[PlanWarning]) -> Result<bool, String>,
) -> Result<Option<AgentRunPlan>, String> {
    let prepared = prepare_launch_for_confirmation(request, |_| Ok(()))?;
    if !approve(&prepared.preview_plan, &prepared.warnings)? {
        return Ok(None);
    }
    finalize_confirmed_launch_plan(prepared).map(Some)
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
) -> Result<Option<LaunchRunResponse>, String> {
    if !approve(&plan, &warnings)? {
        return Ok(None);
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
    Ok(Some(response))
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
                        log::error!("run {run_id}: launch status update failed: {error}");
                    }
                }
                Err(error) => {
                    if let Err(status_error) = launch_status.mark_failed(&run_id) {
                        log::error!("run {run_id}: launch status update failed: {status_error}");
                    }
                    log::error!("run {run_id}: launch failed: {error}");
                }
            }
        })
        .map_err(|error| format!("could not start background run: {error}"))?;
    Ok(())
}

/// Prepare, natively confirm, and finalize a launch for the in-app terminal run
/// path (task #23, Option A). Reuses `launch_run`'s exact plan build, presence-
/// based Keychain OAuth selection, native launch confirmation, and worktree
/// finalize; only the caller differs: it runs the returned plan under a host PTY
/// (`terminal::open_agent_run_terminal`) instead of the background captured runner
/// in [`spawn_launch_thread`]. Returns `Ok(None)` when the user cancels the native
/// confirmation, so nothing is spawned. Any worktree branch is created only here,
/// after approval, exactly as in `launch_run`, so no git mutation happens before
/// consent.
pub(crate) async fn prepare_confirmed_terminal_run(
    app: AppHandle,
    request: LaunchRunRequest,
) -> Result<Option<AgentRunPlan>, String> {
    let prepared = prepare_confirmed_launch(request)?;
    if !trusted_launch_confirmed(
        app,
        &prepared.preview_plan,
        &prepared.warnings,
        prepared.worktree(),
        &prepared.request,
    )
    .await?
    {
        return Ok(None);
    }
    let plan = finalize_confirmed_launch_plan(prepared)?;
    Ok(Some(plan))
}

fn prepare_confirmed_launch(request: LaunchRunRequest) -> Result<PreparedLaunch, String> {
    prepare_launch_for_confirmation(request, |request| {
        let setup = collect_setup_status();
        if !setup.ok {
            return Err(
                "RunHaven setup is not ready. Fix setup checks before launching a run.".to_string(),
            );
        }
        validate_bundled_image_ready(request)
    })
}

fn prepare_launch_for_confirmation(
    request: LaunchRunRequest,
    validate_readiness: impl FnOnce(&RunPlanRequest) -> Result<(), String>,
) -> Result<PreparedLaunch, String> {
    validate_plan_request_bounds(&request.plan)?;
    validate_readiness(&request.plan)?;
    let run_id = new_run_id();
    let plan_request = request.plan.clone();
    let (preview_plan, response) = prepare_launch(request.plan, run_id.clone())?;
    let worktree_preview = if plan_request.worktree {
        Some(
            preview_worktree(
                std::path::Path::new(&plan_request.workspace_path),
                preview_plan.workspace_scope,
                plan_request.allow_sensitive_workspace,
            )
            .map_err(sanitize_worktree_creation_error)?,
        )
    } else {
        None
    };
    let warnings = current_launch_warnings(&plan_request);
    Ok(PreparedLaunch {
        run_id,
        request: plan_request,
        preview_plan,
        response,
        warnings,
        worktree_preview,
    })
}
