use anyhow::Result;
use clap::{CommandFactory, Parser};

use super::args::*;
use super::diagnostics::{
    auth_explain, auth_log, auth_status, egress_log, why_host, why_network, why_state,
    why_workspace,
};
use crate::setup::{print_checks, print_setup_guide};
use runhaven_core::doctor::collect_checks;
use runhaven_core::provider::auth_profiles::{agent_broker, agent_sign_in};
use runhaven_core::runtime::plans::default_network_mode;
use runhaven_core::runtime::profiles::{get_profile, profiles};

mod management;
#[path = "plan_output.rs"]
mod plan_output;
mod run_plan;

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
        TopCommand::Plan(args) => run_plan::plan_run(&args, agent_args),
        TopCommand::Run(command) => {
            run_plan::run_agent(&command.args, command.dry_run, command.worktree, agent_args)
        }
        TopCommand::Login { agent, clear } => {
            get_profile(&agent)?;
            if clear {
                runhaven_core::runtime::login::logout(&agent)
            } else {
                runhaven_core::runtime::login::login(&agent)
            }
        }
        TopCommand::Image { command } => management::image_command(command),
        TopCommand::Network { command } => management::network_command(command),
        TopCommand::State { command } => management::state_command(command),
        TopCommand::Runs { command } => management::runs_command(command, agent_args),
        TopCommand::Egress { command } => match command {
            EgressCommand::Log { limit, json } => egress_log(limit, json),
        },
        TopCommand::Auth { command } => match command {
            AuthCommand::Status { json } => auth_status(json),
            AuthCommand::Explain { agent, json } => auth_explain(&agent, json),
            AuthCommand::Log { limit, json } => auth_log(limit, json),
        },
        TopCommand::Why { command } => match command {
            WhyCommand::Host {
                host,
                port,
                agent,
                json,
            } => why_host(&host, port, agent.as_deref(), json),
            WhyCommand::Workspace {
                path,
                workspace_scope,
                allow_sensitive_workspace,
                json,
            } => why_workspace(&path, &workspace_scope, allow_sensitive_workspace, json),
            WhyCommand::Network { mode, json } => why_network(&mode, json),
            WhyCommand::State { agent, json } => why_state(&agent, json),
        },
    }
}

fn list_agents() -> Result<i32> {
    let agents = profiles();
    let name_width = agents
        .iter()
        .map(|profile| profile.name.len())
        .max()
        .unwrap_or(0)
        .max("AGENT".len());
    println!(
        "{:<name_width$}  {:<14}  {:<8}  {:<6}  DESCRIPTION",
        "AGENT",
        "SIGN-IN",
        "NETWORK",
        "BROKER",
        name_width = name_width
    );
    for profile in &agents {
        println!(
            "{:<name_width$}  {:<14}  {:<8}  {:<6}  {}",
            profile.name,
            agent_sign_in(profile.name),
            default_network_mode(profile).as_str(),
            agent_broker(profile.name),
            profile.description,
            name_width = name_width
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

fn split_agent_args(argv: &[String]) -> (Vec<String>, Vec<String>) {
    let Some(separator) = argv.iter().position(|arg| arg == "--") else {
        return (argv.to_vec(), Vec::new());
    };
    (argv[..separator].to_vec(), argv[separator + 1..].to_vec())
}

#[cfg(test)]
mod tests;
