use anyhow::{Result, bail};
use serde_json::json;

use crate::doctor::require_supported_runtime;
use crate::provider::observability::utc_timestamp;
use crate::provider::runtime as provider_runtime;
use crate::records::{RunRecordInput, find_run_record, write_run_record};
use crate::runtime::active::{
    active_run_terminal_status, remove_active_run_record, write_active_run_record,
};
use crate::runtime::plans::{AgentRunPlan, NetworkMode, WorktreeRun};
use crate::runtime::volume_attachment::launch_with_state_volume_guard;
#[cfg(test)]
use crate::runtime::volume_attachment::{
    ensure_state_volume_detached_from_container_list, launch_under_state_lock_with_attachment_check,
};
use crate::support::git::{GitChange, capture_git_snapshot, summarize_git_change};

#[cfg(target_os = "macos")]
mod signals;

pub struct AgentCommandContext<'a> {
    pub plan: &'a AgentRunPlan,
    pub run_id: &'a str,
    pub started_at: &'a str,
}

pub trait AgentCommandRunner {
    fn run_agent_command(
        &mut self,
        command: &[String],
        env: Option<(&str, &str)>,
        context: AgentCommandContext<'_>,
    ) -> Result<i32>;
}

pub(crate) struct AgentCommandExecution {
    pub(crate) started_at: String,
    pub(crate) finished_at: String,
    pub(crate) return_code: i32,
    pub(crate) terminal_status: Option<String>,
    pub(crate) git: GitChange,
    pub(crate) command_error: Option<anyhow::Error>,
}

pub(crate) struct StatusAgentCommandRunner;

impl StatusAgentCommandRunner {
    pub(crate) fn new() -> Result<Self> {
        Ok(Self)
    }
}

impl AgentCommandRunner for StatusAgentCommandRunner {
    fn run_agent_command(
        &mut self,
        command: &[String],
        env: Option<(&str, &str)>,
        _context: AgentCommandContext<'_>,
    ) -> Result<i32> {
        let mut agent_command = crate::apple_container::command_for_program(&command[0])?;
        agent_command.args(&command[1..]);
        if let Some((name, value)) = env {
            agent_command.env(name, value);
        }
        #[cfg(target_os = "macos")]
        {
            // Install only around the owned child. Dropping this local state
            // immediately restores normal signal behavior before resource and
            // record cleanup, so a second Ctrl-C can terminate a stuck cleanup.
            let signal_state = signals::install_cleanup_signal_handlers()?;
            let mut child = match agent_command.spawn() {
                Ok(child) => child,
                Err(error) => {
                    signal_state.finish_child();
                    return Err(anyhow::anyhow!(
                        "could not launch agent command {:?}: {error}",
                        command[0]
                    ));
                }
            };
            let result = signals::wait_for_child(&mut child, signal_state).map_err(|error| {
                anyhow::anyhow!("could not wait for agent command {:?}: {error}", command[0])
            });
            signal_state.finish_child();
            result
        }
        #[cfg(not(target_os = "macos"))]
        let status = agent_command.status().map_err(|error| {
            anyhow::anyhow!("could not launch agent command {:?}: {error}", command[0])
        })?;
        #[cfg(not(target_os = "macos"))]
        Ok(command_exit_code(status))
    }
}

fn command_exit_code(status: std::process::ExitStatus) -> i32 {
    if let Some(code) = status.code() {
        return code;
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            return 128 + signal;
        }
    }
    1
}

pub fn new_run_id() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}

pub fn launch_run_plan(plan: &AgentRunPlan) -> Result<i32> {
    let mut runner = StatusAgentCommandRunner::new()?;
    launch_run_plan_with_runner(plan, &mut runner)
}

/// Launch `plan` and run `on_success` after all child, cleanup, and record work
/// has succeeded, while the plan's shared-state lock is still held.
pub fn launch_run_plan_with_completion(
    plan: &AgentRunPlan,
    on_success: impl FnOnce() -> Result<()>,
) -> Result<i32> {
    let mut runner = StatusAgentCommandRunner::new()?;
    launch_run_plan_with_runner_and_completion(plan, &mut runner, on_success)
}

pub fn launch_run_plan_with_runner(
    plan: &AgentRunPlan,
    runner: &mut impl AgentCommandRunner,
) -> Result<i32> {
    launch_run_plan_with_runner_and_completion(plan, runner, || Ok(()))
}

pub fn launch_run_plan_with_runner_and_completion(
    plan: &AgentRunPlan,
    runner: &mut impl AgentCommandRunner,
    on_success: impl FnOnce() -> Result<()>,
) -> Result<i32> {
    launch_run_plan_with_runtime_check_runner_and_completion(
        plan,
        runner,
        require_supported_runtime,
        on_success,
    )
}

/// Run every non-mutating launch check that can be completed before creating a
/// worktree. Launch repeats these checks because runtime and image state can
/// change between planning and execution.
pub fn validate_run_plan_before_worktree(plan: &AgentRunPlan) -> Result<()> {
    validate_run_plan_before_worktree_with_runtime_check(plan, require_supported_runtime)
}

fn validate_run_plan_before_worktree_with_runtime_check(
    plan: &AgentRunPlan,
    runtime_check: impl FnOnce() -> Result<()>,
) -> Result<()> {
    require_plan_run_id(plan)?;
    runtime_check()?;
    provider_runtime::validate_runtime_auth_broker_environment(plan)?;
    require_container_cli()?;
    ensure_agent_image_built(plan)
}

#[cfg(test)]
fn launch_run_plan_with_runtime_check_and_runner(
    plan: &AgentRunPlan,
    runner: &mut impl AgentCommandRunner,
    runtime_check: impl FnOnce() -> Result<()>,
) -> Result<i32> {
    launch_run_plan_with_runtime_check_runner_and_completion(plan, runner, runtime_check, || Ok(()))
}

fn launch_run_plan_with_runtime_check_runner_and_completion(
    plan: &AgentRunPlan,
    runner: &mut impl AgentCommandRunner,
    runtime_check: impl FnOnce() -> Result<()>,
    on_success: impl FnOnce() -> Result<()>,
) -> Result<i32> {
    let result = (|| {
        validate_run_plan_before_worktree_with_runtime_check(plan, runtime_check)?;
        launch_with_state_volume_guard(
            plan,
            || {
                if plan.network_mode == NetworkMode::Provider {
                    return provider_runtime::run_provider_agent_with_runner(plan, runner);
                }
                run_standard_launch_with_ops(
                    plan,
                    runner,
                    provider_runtime::run_preflight,
                    provider_runtime::cleanup_run_network,
                )
            },
            on_success,
        )
    })();
    match result {
        Ok(code) => Ok(code),
        Err(error) => {
            if let Err(record_error) = record_missing_worktree_failure(plan) {
                return Err(error.context(format!(
                    "could not save worktree recovery information: {record_error}"
                )));
            }
            Err(error)
        }
    }
}

/// Persist a failed run with worktree recovery metadata when launch aborts
/// before the normal lifecycle writes a complete record. Existing records win,
/// so a runner error that already recorded execution is never duplicated.
fn record_missing_worktree_failure(plan: &AgentRunPlan) -> Result<()> {
    if plan.worktree.is_none() {
        return Ok(());
    }
    let run_id = require_plan_run_id(plan)?;
    if find_run_record(run_id).is_ok() {
        return Ok(());
    }
    let timestamp = utc_timestamp();
    write_run_record(RunRecordInput {
        plan,
        run_id,
        started_at: &timestamp,
        finished_at: &timestamp,
        return_code: 1,
        status: Some("failed"),
        provider_decisions: &[],
        auth_decisions: None,
        cleanup: json!({
            "provider_network": "launch-failed",
            "worktree": "kept-for-recovery",
        }),
        git: GitChange::Unavailable {
            reason: "launch-failed-without-complete-git-record".to_string(),
            repo_root: None,
        },
    })
}

/// Record a worktree created during final plan construction when that
/// construction fails before any agent process can start.
pub fn write_worktree_setup_failure_record(
    preview_plan: &AgentRunPlan,
    worktree: &WorktreeRun,
) -> Result<()> {
    let mut recovery_plan = preview_plan.clone();
    recovery_plan.workspace = worktree.mounted_workspace.clone();
    recovery_plan.worktree = Some(worktree.clone());
    record_missing_worktree_failure(&recovery_plan)
}

/// Fail early with an actionable message when the agent image is not built,
/// instead of letting `container run` try to pull a RunHaven image and return a
/// confusing `registry-1.docker.io 401`.
fn ensure_agent_image_built(plan: &AgentRunPlan) -> Result<()> {
    if crate::image::doctor::image_is_built(&plan.image)? {
        return Ok(());
    }
    bail!(
        "{}",
        image_not_built_message(&plan.image, &plan.profile_name)
    );
}

fn image_not_built_message(image: &str, agent: &str) -> String {
    if image.starts_with("runhaven/") {
        format!(
            "The {agent} image is not built yet, so the sandbox cannot start. Build it once with:\n  runhaven image build {agent}"
        )
    } else {
        format!("The image {image} is not available locally. Build or load it before running.")
    }
}

pub fn run_standard_agent(plan: &AgentRunPlan) -> Result<i32> {
    let mut runner = StatusAgentCommandRunner::new()?;
    run_standard_agent_with_runner(plan, &mut runner)
}

pub fn run_standard_agent_with_runner(
    plan: &AgentRunPlan,
    runner: &mut impl AgentCommandRunner,
) -> Result<i32> {
    run_standard_agent_with_runner_and_cleanup(plan, runner, |_| {
        Ok(json!({"provider_network": "not-applicable"}))
    })
}

fn run_standard_launch_with_ops(
    plan: &AgentRunPlan,
    runner: &mut impl AgentCommandRunner,
    mut run_preflight: impl FnMut(&[String]) -> Result<()>,
    cleanup_run_network: impl Fn(&AgentRunPlan) -> Result<serde_json::Value>,
) -> Result<i32> {
    let mut run_network_ready = false;
    for command in &plan.preflight {
        if let Err(error) = run_preflight(command) {
            if run_network_ready && let Err(cleanup_error) = cleanup_run_network(plan) {
                eprintln!(
                    "runhaven: could not clean up the run network after preflight failed: {cleanup_error}"
                );
            }
            return Err(error);
        }
        run_network_ready |= is_internal_run_network_create(plan, command);
    }
    run_standard_agent_with_runner_and_cleanup(plan, runner, cleanup_run_network)
}

fn is_internal_run_network_create(plan: &AgentRunPlan, command: &[String]) -> bool {
    plan.network_mode == NetworkMode::Internal
        && command
            .get(0..4)
            .is_some_and(|prefix| prefix == ["container", "network", "create", "--internal"])
        && plan.network_name.as_ref() == command.last()
}

fn run_standard_agent_with_runner_and_cleanup(
    plan: &AgentRunPlan,
    runner: &mut impl AgentCommandRunner,
    cleanup_run_network: impl FnOnce(&AgentRunPlan) -> Result<serde_json::Value>,
) -> Result<i32> {
    let run_id = require_plan_run_id(plan)?.to_string();
    let execution = run_agent_command_lifecycle(plan, &run_id, plan.command.clone(), runner);
    // The `container run --rm` client can be killed before it removes its guest.
    // Always clean the exact run container after a spawned standard lifecycle,
    // even when the run used Internet mode and owned no network.
    let cleanup = cleanup_run_network(plan).unwrap_or_else(|error| {
        json!({
            "provider_network": "cleanup-error",
            "provider_network_name": plan.network_name,
            "error": error.to_string(),
        })
    });
    let execution = execution?;
    let AgentCommandExecution {
        started_at,
        finished_at,
        return_code,
        terminal_status,
        git,
        command_error,
    } = execution;
    write_run_record(RunRecordInput {
        plan,
        run_id: &run_id,
        started_at: &started_at,
        finished_at: &finished_at,
        return_code,
        status: terminal_status.as_deref(),
        provider_decisions: &[],
        auth_decisions: None,
        cleanup,
        git,
    })?;
    if let Some(error) = command_error {
        return Err(error);
    }
    Ok(return_code)
}

pub(crate) fn require_plan_run_id(plan: &AgentRunPlan) -> Result<&str> {
    plan.run_id
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("run plan is missing its reserved run id"))
}

pub(crate) fn run_agent_command_lifecycle(
    plan: &AgentRunPlan,
    run_id: &str,
    command: Vec<String>,
    runner: &mut impl AgentCommandRunner,
) -> Result<AgentCommandExecution> {
    let git_before = capture_git_snapshot(&plan.workspace);
    let started_at = utc_timestamp();
    eprintln!("Run id: {run_id}");
    write_active_run_record(plan, run_id, &started_at)?;
    // No agent token is injected: every agent logs in in-sandbox and its
    // credential lives in the isolated home volume, never in the run env.
    let status_result = runner.run_agent_command(
        &command,
        None,
        AgentCommandContext {
            plan,
            run_id,
            started_at: &started_at,
        },
    );
    let terminal_status = active_run_terminal_status(run_id);
    let marker_removal = remove_active_run_record(run_id);
    let finished_at = utc_timestamp();
    let mut command_error = None;
    let mut return_code = match status_result {
        Ok(code) => code,
        Err(error) => {
            command_error = Some(error);
            1
        }
    };
    if let Err(error) = marker_removal {
        return_code = 1;
        command_error = Some(match command_error {
            Some(command_error) => command_error.context(error.to_string()),
            None => error,
        });
    }
    let git = summarize_git_change(git_before, capture_git_snapshot(&plan.workspace));
    Ok(AgentCommandExecution {
        started_at,
        finished_at,
        return_code,
        terminal_status,
        git,
        command_error,
    })
}

pub fn require_container_cli() -> Result<()> {
    if crate::apple_container::resolve_program().is_some() {
        return Ok(());
    }
    bail!(
        "Apple container CLI was not found. In the desktop app, choose Download Apple container. For the CLI, install Apple's signed 1.1.0 package and run `container system start`."
    )
}

#[cfg(test)]
mod tests;
