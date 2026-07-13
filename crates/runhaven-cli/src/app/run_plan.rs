use std::io::IsTerminal;
use std::path::Path;

use anyhow::{Result, bail};

use super::plan_output;
use crate::args::RunArgs;
use runhaven_core::provider::runtime as provider_runtime;
use runhaven_core::runtime::launch::{
    launch_run_plan, new_run_id, validate_run_plan_before_worktree,
    write_worktree_setup_failure_record,
};
use runhaven_core::runtime::plans::{
    AgentRunPlan, AuthScope, ClaudeAuthSource, NetworkMode, RunOptions, WorkspaceScope,
    WorktreeRun, build_run_plan, default_network_mode,
};
use runhaven_core::runtime::profiles::{AgentProfile, get_profile};
use runhaven_core::runtime::worktrees::{create_worktree_for_run, preview_worktree};

pub(super) fn plan_run(args: &RunArgs, agent_args: Vec<String>) -> Result<i32> {
    let plan = make_run_plan(args, agent_args, None, None, None)?;
    plan_output::print_run_plan(&plan);
    plan_output::eprint_security_notices(&plan);
    Ok(0)
}

pub(super) fn run_agent(
    args: &RunArgs,
    dry_run: bool,
    worktree_enabled: bool,
    agent_args: Vec<String>,
) -> Result<i32> {
    if worktree_enabled && dry_run {
        return print_worktree_dry_run(args, agent_args);
    }

    let launch_run_id = (!dry_run).then(new_run_id);
    let mut plan = make_run_plan(args, agent_args.clone(), None, None, launch_run_id.clone())?;
    if worktree_enabled {
        validate_run_plan_before_worktree(&plan)?;
        let run_id = launch_run_id
            .ok_or_else(|| anyhow::anyhow!("worktree launch is missing its reserved run id"))?;
        let workspace_scope = workspace_scope(&args.workspace_scope)?;
        let worktree = create_worktree_for_run(
            &args.workspace,
            workspace_scope,
            args.allow_sensitive_workspace,
            &args.agent,
            &run_id,
        )?;
        let mounted = worktree.mounted_workspace.clone();
        let final_plan = make_run_plan_with_auth_source(
            args,
            agent_args,
            Some(&mounted),
            Some(worktree.clone()),
            Some(run_id),
            plan.claude_auth_source.clone(),
        );
        plan = match final_plan {
            Ok(plan) => plan,
            Err(error) => {
                if let Err(record_error) = write_worktree_setup_failure_record(&plan, &worktree) {
                    return Err(error.context(format!(
                        "could not save worktree recovery information: {record_error}"
                    )));
                }
                return Err(error);
            }
        };
    }
    if dry_run {
        plan_output::print_run_plan(&plan);
        plan_output::eprint_security_notices(&plan);
        return Ok(0);
    }

    plan_output::eprint_security_notices(&plan);
    launch_run_plan(&plan)
}

fn print_worktree_dry_run(args: &RunArgs, agent_args: Vec<String>) -> Result<i32> {
    let _ = make_run_plan(args, agent_args, None, None, None)?;
    let workspace_scope = workspace_scope(&args.workspace_scope)?;
    let (source_workspace, repo_root, base_head) = preview_worktree(
        &args.workspace,
        workspace_scope,
        args.allow_sensitive_workspace,
    )?;
    println!("Worktree: enabled");
    println!("Source workspace: {}", source_workspace.display());
    println!("Source repo root: {}", repo_root.display());
    println!("Base HEAD: {base_head}");
    println!("RunHaven will create a branch named runhaven/<agent>/<run-id>.");
    println!("RunHaven will keep the worktree after the run and record recovery commands.");
    Ok(0)
}

pub(super) fn make_run_plan(
    args: &RunArgs,
    agent_args: Vec<String>,
    workspace_override: Option<&Path>,
    worktree: Option<WorktreeRun>,
    run_id: Option<String>,
) -> Result<AgentRunPlan> {
    let profile = get_profile(&args.agent)?;
    let network = match &args.network {
        Some(value) => network_mode(value)?,
        None => default_network_mode(&profile),
    };
    let claude_auth_source = provider_runtime::select_api_key_broker_source_for_network(
        args.api_key_broker_env.clone(),
        network,
    )?;
    make_run_plan_from_resolved(
        args,
        agent_args,
        workspace_override,
        worktree,
        run_id,
        (profile, network, claude_auth_source),
    )
}

pub(super) fn make_run_plan_with_auth_source(
    args: &RunArgs,
    agent_args: Vec<String>,
    workspace_override: Option<&Path>,
    worktree: Option<WorktreeRun>,
    run_id: Option<String>,
    claude_auth_source: Option<ClaudeAuthSource>,
) -> Result<AgentRunPlan> {
    let profile = get_profile(&args.agent)?;
    let network = match &args.network {
        Some(value) => network_mode(value)?,
        None => default_network_mode(&profile),
    };
    make_run_plan_from_resolved(
        args,
        agent_args,
        workspace_override,
        worktree,
        run_id,
        (profile, network, claude_auth_source),
    )
}

fn make_run_plan_from_resolved(
    args: &RunArgs,
    agent_args: Vec<String>,
    workspace_override: Option<&Path>,
    worktree: Option<WorktreeRun>,
    run_id: Option<String>,
    resolved: (AgentProfile, NetworkMode, Option<ClaudeAuthSource>),
) -> Result<AgentRunPlan> {
    let (profile, network, claude_auth_source) = resolved;
    let tty = match args.tty.as_str() {
        "always" => true,
        "never" => false,
        "auto" => std::io::stdin().is_terminal() && std::io::stdout().is_terminal(),
        other => bail!("invalid --tty value: {other:?}"),
    };
    build_run_plan(RunOptions {
        profile,
        workspace: workspace_override.unwrap_or(&args.workspace).to_path_buf(),
        agent_args,
        image: args.image.clone(),
        cpus: args.cpus.clone(),
        memory: args.memory.clone(),
        network,
        workspace_scope: workspace_scope(&args.workspace_scope)?,
        session: args.session.clone(),
        auth_scope: AuthScope::try_from(args.auth_scope.as_str())?,
        read_only_workspace: args.read_only_workspace,
        ssh: args.ssh,
        env: args.env.clone(),
        user: args.user.clone(),
        interactive: !args.no_interactive,
        tty,
        allow_sensitive_workspace: args.allow_sensitive_workspace,
        allow_root_user: args.allow_root_user,
        provider_hosts: args.provider_host.clone(),
        claude_auth_source,
        worktree,
        run_id,
    })
}

fn network_mode(value: &str) -> Result<NetworkMode> {
    NetworkMode::try_from(value)
}

pub(super) fn workspace_scope(value: &str) -> Result<WorkspaceScope> {
    WorkspaceScope::try_from(value)
}
