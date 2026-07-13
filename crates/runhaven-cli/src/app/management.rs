use anyhow::Result;

use super::run_plan::workspace_scope;
use crate::args::{ImageCommand, NetworkCommand, RunsCommand, StateCommand};
use runhaven_core::diagnostics::{read_auth_broker_log, read_egress_policy_log};
use runhaven_core::doctor::require_supported_runtime;
use runhaven_core::image::build::build_image_plan;
use runhaven_core::image::doctor as image_doctor;
use runhaven_core::records::{runs_diff, runs_list, runs_log, runs_show};
use runhaven_core::runtime::active::{
    runs_active, runs_attach, runs_kill, runs_logs_follow, runs_repair, runs_status, runs_stop,
};
use runhaven_core::runtime::launch::require_container_cli;
use runhaven_core::runtime::plans::{AuthScope, NetworkMode, RunOptions, build_run_plan};
use runhaven_core::runtime::profiles::get_profile;
use runhaven_core::runtime::state as runtime_state;
use runhaven_core::runtime::worktrees::{
    runs_worktree_discard, runs_worktree_keep, runs_worktree_merge, runs_worktree_recover,
};

pub(super) fn image_command(command: ImageCommand) -> Result<i32> {
    match command {
        ImageCommand::Doctor { agent } => {
            require_container_cli()?;
            image_doctor::image_doctor(agent.as_deref())
        }
        ImageCommand::Build(args) | ImageCommand::Rebuild(args) => {
            let profile = get_profile(&args.agent)?;
            let plan = build_image_plan(&profile, args.tag.as_deref())?;
            if args.dry_run {
                println!("{}", plan.shell_command());
                return Ok(0);
            }
            require_supported_runtime()?;
            Ok(
                runhaven_core::apple_container::command_for_program(&plan.command[0])?
                    .args(&plan.command[1..])
                    .status()?
                    .code()
                    .unwrap_or(1),
            )
        }
    }
}

pub(super) fn network_command(command: NetworkCommand) -> Result<i32> {
    require_container_cli()?;
    match command {
        NetworkCommand::List => runhaven_core::runtime::network::network_list(),
        NetworkCommand::Prune { yes } => runhaven_core::runtime::network::network_prune(yes),
    }
}

pub(super) fn state_command(command: StateCommand) -> Result<i32> {
    match command {
        StateCommand::List { session } => {
            require_container_cli()?;
            runtime_state::state_list(session.as_deref())
        }
        StateCommand::Prune { session, yes } => {
            require_container_cli()?;
            runtime_state::state_prune(yes, session.as_deref())
        }
        StateCommand::Reset {
            agent,
            workspace,
            workspace_scope: workspace_scope_value,
            session,
            allow_sensitive_workspace,
            yes,
        } => state_reset(
            &agent,
            workspace,
            &workspace_scope_value,
            session,
            allow_sensitive_workspace,
            yes,
        ),
    }
}

fn state_reset(
    agent: &str,
    workspace: std::path::PathBuf,
    workspace_scope_value: &str,
    session: Option<String>,
    allow_sensitive_workspace: bool,
    yes: bool,
) -> Result<i32> {
    let profile = get_profile(agent)?;
    let plan = build_run_plan(RunOptions {
        profile,
        workspace,
        agent_args: Vec::new(),
        image: None,
        cpus: "4".to_string(),
        memory: "4g".to_string(),
        network: NetworkMode::Internet,
        workspace_scope: workspace_scope(workspace_scope_value)?,
        session,
        auth_scope: AuthScope::Project,
        read_only_workspace: false,
        ssh: false,
        env: Vec::new(),
        user: "agent".to_string(),
        interactive: false,
        tty: false,
        allow_sensitive_workspace,
        allow_root_user: false,
        provider_hosts: Vec::new(),
        claude_auth_source: None,
        worktree: None,
        run_id: None,
    })?;
    if !yes {
        println!("State volume: {}", plan.state_volume);
        println!("Session: {}", plan.session);
        println!("Rerun with --yes to delete this volume.");
        return Ok(2);
    }
    require_container_cli()?;
    match runtime_state::delete_volume_with_state_lock(&plan.state_volume)? {
        runtime_state::VolumeDeletion::Deleted => {
            println!("Deleted state volume: {}", plan.state_volume);
            Ok(0)
        }
        runtime_state::VolumeDeletion::Missing => {
            println!(
                "No state volume to reset: {} does not exist (project-scoped runs create it; the default --auth-scope agent shares one per-agent volume).",
                plan.state_volume
            );
            Ok(0)
        }
        runtime_state::VolumeDeletion::Failed => Ok(1),
    }
}

pub(super) fn runs_command(command: RunsCommand, agent_args: Vec<String>) -> Result<i32> {
    match command {
        RunsCommand::List { limit, json } => runs_list(limit, json),
        RunsCommand::Show { run_id, json } => runs_show(&run_id, json),
        RunsCommand::Log { run_id, json } => runs_log(
            &run_id,
            json,
            read_egress_policy_log(0)?,
            read_auth_broker_log(0)?,
        ),
        RunsCommand::Diff { run_id } => runs_diff(&run_id),
        RunsCommand::Recover { run_id, json } => runs_worktree_recover(&run_id, json),
        RunsCommand::Merge { run_id } => runs_worktree_merge(&run_id),
        RunsCommand::Keep { run_id } => runs_worktree_keep(&run_id),
        RunsCommand::Discard { run_id } => runs_worktree_discard(&run_id),
        RunsCommand::Active { json } => runs_active(json),
        RunsCommand::Status { run_id, json } => {
            require_container_cli()?;
            runs_status(&run_id, json)
        }
        RunsCommand::Attach {
            run_id,
            user,
            allow_root_user,
            workdir,
            tty,
        } => {
            require_container_cli()?;
            runs_attach(&run_id, &user, &workdir, &tty, allow_root_user, &agent_args)
        }
        RunsCommand::LogsFollow { run_id, lines } => {
            require_container_cli()?;
            runs_logs_follow(&run_id, lines)
        }
        RunsCommand::Stop { run_id } => {
            require_container_cli()?;
            runs_stop(&run_id)
        }
        RunsCommand::Kill { run_id } => {
            require_container_cli()?;
            runs_kill(&run_id)
        }
        RunsCommand::Repair { run_id, all, json } => {
            require_container_cli()?;
            runs_repair(run_id.as_deref(), all, json)
        }
    }
}
