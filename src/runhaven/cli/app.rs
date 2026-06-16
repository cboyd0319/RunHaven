use std::io::IsTerminal;
use std::path::Path;
use std::process::Command;

use anyhow::{Result, bail};
use clap::{CommandFactory, Parser};
use serde_json::json;

use super::args::*;
use super::lock::acquire_state_lock;
use crate::active::{
    active_run_terminal_status, remove_active_run_record, runs_active, runs_attach, runs_kill,
    runs_logs_follow, runs_repair, runs_status, runs_stop, write_active_run_record,
};
use crate::diagnostics::{
    auth_explain, auth_log, auth_status, egress_log, read_auth_broker_log, read_egress_policy_log,
    why_host,
};
use crate::doctor::{collect_checks, find_on_path};
use crate::git::{capture_git_snapshot, summarize_git_change};
use crate::image_doctor;
use crate::images::build_image_plan;
use crate::plans::{
    AgentRunPlan, NetworkMode, RunOptions, WorkspaceScope, WorktreeRun, build_run_plan,
};
use crate::profiles::{get_profile, profiles};
use crate::provider_observability::utc_timestamp;
use crate::provider_runtime;
use crate::records::{RunRecordInput, runs_diff, runs_list, runs_log, runs_show, write_run_record};
use crate::runtime_state;
use crate::setup::{print_checks, print_setup_guide};
use crate::worktrees::{
    create_worktree_for_run, preview_worktree, runs_worktree_discard, runs_worktree_keep,
    runs_worktree_merge, runs_worktree_recover,
};

pub fn main_entry() -> i32 {
    match run_from(std::env::args().skip(1)) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("runhaven: {error}");
            2
        }
    }
}

pub fn run_from<I, S>(args: I) -> Result<i32>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let raw_args = args.into_iter().map(Into::into).collect::<Vec<_>>();
    let (parse_args, agent_args) = split_agent_args(&raw_args);
    let cli = match Cli::try_parse_from(std::iter::once("runhaven".to_string()).chain(parse_args)) {
        Ok(cli) => cli,
        Err(error) => {
            let code = error.exit_code();
            error.print()?;
            return Ok(code);
        }
    };
    dispatch(cli, agent_args)
}

fn dispatch(cli: Cli, agent_args: Vec<String>) -> Result<i32> {
    let Some(command) = cli.command else {
        let mut command = Cli::command();
        command.print_help()?;
        println!();
        return Ok(2);
    };
    match command {
        TopCommand::Agents => list_agents(),
        TopCommand::Doctor => doctor(),
        TopCommand::Setup { agent } => setup(&agent),
        TopCommand::Plan(args) => plan_run(&args, agent_args),
        TopCommand::Run(command) => {
            run_agent(&command.args, command.dry_run, command.worktree, agent_args)
        }
        TopCommand::Image { command } => image_command(command),
        TopCommand::Network { command } => network_command(command),
        TopCommand::State { command } => state_command(command),
        TopCommand::Runs { command } => runs_command(command, agent_args),
        TopCommand::Egress { command } => match command {
            EgressCommand::Log { limit, json } => egress_log(limit, json),
        },
        TopCommand::Auth { command } => match command {
            AuthCommand::Status { json } => auth_status(json),
            AuthCommand::Explain { agent, json } => auth_explain(&agent, json),
            AuthCommand::Log { limit, json } => auth_log(limit, json),
        },
        TopCommand::Why { command } => match command {
            WhyCommand::Host { host, port, agent } => why_host(&host, port, agent.as_deref()),
        },
    }
}

fn list_agents() -> Result<i32> {
    let agents = profiles();
    let width = agents
        .iter()
        .map(|profile| profile.name.len())
        .max()
        .unwrap_or(0);
    for profile in agents {
        println!(
            "{:<width$}  {}",
            profile.name,
            profile.description,
            width = width
        );
    }
    Ok(0)
}

fn doctor() -> Result<i32> {
    let checks = collect_checks();
    print_checks(&checks);
    Ok(if checks.iter().all(|check| check.ok) {
        0
    } else {
        1
    })
}

fn setup(agent: &str) -> Result<i32> {
    print_setup_guide(agent, &collect_checks())
}

fn plan_run(args: &RunArgs, agent_args: Vec<String>) -> Result<i32> {
    let plan = make_run_plan(args, agent_args, None, None, None)?;
    print_run_plan(&plan);
    Ok(0)
}

fn run_agent(
    args: &RunArgs,
    dry_run: bool,
    worktree_enabled: bool,
    agent_args: Vec<String>,
) -> Result<i32> {
    if worktree_enabled && dry_run {
        return print_worktree_dry_run(args, agent_args);
    }

    let mut plan = make_run_plan(args, agent_args.clone(), None, None, None)?;
    if worktree_enabled {
        provider_runtime::validate_runtime_auth_broker_environment(&plan)?;
        require_container_cli()?;
        let run_id = uuid::Uuid::new_v4().simple().to_string();
        let workspace_scope = workspace_scope(&args.workspace_scope)?;
        let worktree = create_worktree_for_run(
            &args.workspace,
            workspace_scope,
            args.allow_sensitive_workspace,
            &args.agent,
            &run_id,
        )?;
        let mounted = worktree.mounted_workspace.clone();
        plan = make_run_plan(
            args,
            agent_args,
            Some(&mounted),
            Some(worktree),
            Some(run_id),
        )?;
    }
    if dry_run {
        print_run_plan(&plan);
        return Ok(0);
    }

    provider_runtime::validate_runtime_auth_broker_environment(&plan)?;
    require_container_cli()?;
    let _state_lock = acquire_state_lock(&plan.state_volume)?;
    if plan.network_mode == NetworkMode::Provider {
        return provider_runtime::run_provider_agent(&plan);
    }
    for command in &plan.preflight {
        provider_runtime::run_preflight(command)?;
    }
    run_standard_agent(&plan)
}

fn run_standard_agent(plan: &AgentRunPlan) -> Result<i32> {
    let run_id = plan
        .run_id
        .clone()
        .unwrap_or_else(|| uuid::Uuid::new_v4().simple().to_string());
    let git_before = capture_git_snapshot(&plan.workspace);
    let started_at = utc_timestamp();
    eprintln!("Run id: {run_id}");
    write_active_run_record(plan, &run_id, &started_at)?;
    let status_result = Command::new(&plan.command[0])
        .args(&plan.command[1..])
        .status();
    let terminal_status = active_run_terminal_status(&run_id);
    let _ = remove_active_run_record(&run_id);
    let status = status_result?;
    let finished_at = utc_timestamp();
    let return_code = status.code().unwrap_or(1);
    let git = summarize_git_change(git_before, capture_git_snapshot(&plan.workspace));
    write_run_record(RunRecordInput {
        plan,
        run_id: &run_id,
        started_at: &started_at,
        finished_at: &finished_at,
        return_code,
        status: terminal_status.as_deref(),
        provider_decisions: &[],
        auth_decisions: None,
        cleanup: json!({"provider_network": "not-applicable"}),
        git,
    })?;
    Ok(return_code)
}

fn image_command(command: ImageCommand) -> Result<i32> {
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
            require_container_cli()?;
            Ok(Command::new(&plan.command[0])
                .args(&plan.command[1..])
                .status()?
                .code()
                .unwrap_or(1))
        }
    }
}

fn network_command(command: NetworkCommand) -> Result<i32> {
    require_container_cli()?;
    match command {
        NetworkCommand::List => crate::network::network_list(),
        NetworkCommand::Prune { yes } => crate::network::network_prune(yes),
    }
}

fn state_command(command: StateCommand) -> Result<i32> {
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
        } => {
            let profile = get_profile(&agent)?;
            let plan = build_run_plan(RunOptions {
                profile,
                workspace,
                agent_args: Vec::new(),
                image: None,
                cpus: "4".to_string(),
                memory: "4g".to_string(),
                network: NetworkMode::Internet,
                workspace_scope: workspace_scope(&workspace_scope_value)?,
                session,
                read_only_workspace: false,
                ssh: false,
                env: Vec::new(),
                user: "agent".to_string(),
                interactive: false,
                tty: false,
                allow_sensitive_workspace,
                allow_root_user: false,
                provider_hosts: Vec::new(),
                codex_api_key_broker_env: None,
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
            let status = Command::new("container")
                .args(["volume", "delete", &plan.state_volume])
                .status()?;
            if status.success() {
                println!("Deleted state volume: {}", plan.state_volume);
            }
            Ok(status.code().unwrap_or(1))
        }
    }
}

fn runs_command(command: RunsCommand, agent_args: Vec<String>) -> Result<i32> {
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

fn make_run_plan(
    args: &RunArgs,
    agent_args: Vec<String>,
    workspace_override: Option<&Path>,
    worktree: Option<WorktreeRun>,
    run_id: Option<String>,
) -> Result<AgentRunPlan> {
    let profile = get_profile(&args.agent)?;
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
        network: network_mode(&args.network)?,
        workspace_scope: workspace_scope(&args.workspace_scope)?,
        session: args.session.clone(),
        read_only_workspace: args.read_only_workspace,
        ssh: args.ssh,
        env: args.env.clone(),
        user: args.user.clone(),
        interactive: !args.no_interactive,
        tty,
        allow_sensitive_workspace: args.allow_sensitive_workspace,
        allow_root_user: args.allow_root_user,
        provider_hosts: args.provider_host.clone(),
        codex_api_key_broker_env: args.codex_api_key_broker_env.clone(),
        worktree,
        run_id,
    })
}

fn print_run_plan(plan: &AgentRunPlan) {
    println!("Workspace: {}", plan.workspace.display());
    println!("Workspace scope: {}", plan.workspace_scope.as_str());
    if let Some(note) = &plan.workspace_scope_note {
        println!("Workspace note: {note}");
    }
    if let Some(worktree) = &plan.worktree {
        println!("Worktree: {}", worktree.worktree_root.display());
        println!("Worktree branch: {}", worktree.branch);
    }
    println!("Session: {}", plan.session);
    println!("State volume: {}", plan.state_volume);
    println!(
        "Network: {}",
        plan.network_name
            .as_deref()
            .unwrap_or("default internet network")
    );
    println!("Egress: {}", plan.egress_summary);
    if plan.network_mode == NetworkMode::Provider {
        println!("Provider hosts: {}", plan.provider_allowed_hosts.join(", "));
        println!("Provider proxy: RunHaven injects proxy environment variables at runtime.");
    }
    if let Some(name) = &plan.codex_api_key_broker_env {
        println!(
            "Codex API key broker: enabled from host environment variable {name}; value is not printed or planned."
        );
    }
    if !plan.preflight.is_empty() {
        println!("Preflight:");
        for command in plan.shell_preflight() {
            println!("  {command}");
        }
    }
    println!("Run:");
    println!("  {}", plan.shell_command());
}

fn network_mode(value: &str) -> Result<NetworkMode> {
    NetworkMode::try_from(value)
}

fn workspace_scope(value: &str) -> Result<WorkspaceScope> {
    WorkspaceScope::try_from(value)
}

fn split_agent_args(argv: &[String]) -> (Vec<String>, Vec<String>) {
    let Some(separator) = argv.iter().position(|arg| arg == "--") else {
        return (argv.to_vec(), Vec::new());
    };
    (argv[..separator].to_vec(), argv[separator + 1..].to_vec())
}

fn require_container_cli() -> Result<()> {
    if find_on_path("container").is_some() {
        return Ok(());
    }
    bail!(
        "Apple container CLI was not found. Install it from https://github.com/apple/container/releases and run `container system start`."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_agent_args_after_separator() {
        let args = ["run", "shell", "--", "echo", "--flag"]
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>();
        assert_eq!(
            split_agent_args(&args),
            (
                vec!["run".to_string(), "shell".to_string()],
                vec!["echo".to_string(), "--flag".to_string()]
            )
        );
    }
}
