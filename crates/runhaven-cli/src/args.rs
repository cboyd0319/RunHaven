use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

mod diagnostics_args;
mod management_args;

pub(super) use diagnostics_args::{AuthCommand, EgressCommand, WhyCommand};
pub(super) use management_args::{ImageCommand, NetworkCommand, RunsCommand, StateCommand};

#[derive(Parser, Debug)]
#[command(
    name = "runhaven",
    about = "Run AI coding agents inside Apple container on macOS.",
    version
)]
pub(super) struct Cli {
    #[command(subcommand)]
    pub(super) command: Option<TopCommand>,
}

#[derive(Subcommand, Debug)]
pub(super) enum TopCommand {
    #[command(about = "list bundled agent profiles")]
    Agents,
    #[command(about = "check local runtime prerequisites")]
    Doctor,
    #[command(about = "guide first-run prerequisites and next commands")]
    Setup {
        #[arg(
            long,
            default_value = "claude",
            help = "agent profile to prepare; defaults to claude"
        )]
        agent: String,
    },
    #[command(about = "print the Apple container run plan", after_help = AGENT_ARGS_HELP)]
    Plan(RunArgs),
    #[command(about = "run an agent through Apple container", after_help = AGENT_ARGS_HELP)]
    Run(RunCommand),
    #[command(about = "log in to an agent once and reuse the login across runs")]
    Login {
        #[arg(help = "agent profile to log in (claude, codex, copilot, antigravity)")]
        agent: String,
        #[arg(long, help = "clear the stored login for this agent")]
        clear: bool,
    },
    #[command(about = "manage local agent images")]
    Image {
        #[command(subcommand)]
        command: ImageCommand,
    },
    #[command(about = "inspect or remove RunHaven managed networks")]
    Network {
        #[command(subcommand)]
        command: NetworkCommand,
    },
    #[command(about = "inspect or remove RunHaven state volumes")]
    State {
        #[command(subcommand)]
        command: StateCommand,
    },
    #[command(about = "inspect RunHaven run history")]
    Runs {
        #[command(subcommand)]
        command: RunsCommand,
    },
    #[command(about = "inspect provider egress policy logs")]
    Egress {
        #[command(subcommand)]
        command: EgressCommand,
    },
    #[command(about = "inspect provider auth broker status")]
    Auth {
        #[command(subcommand)]
        command: AuthCommand,
    },
    #[command(about = "explain RunHaven safety decisions")]
    Why {
        #[command(subcommand)]
        command: WhyCommand,
    },
}

const AGENT_ARGS_HELP: &str =
    "Use -- before flags meant for the agent, for example:\n  runhaven run claude -- --version";

#[derive(Args, Clone, Debug)]
pub(super) struct RunArgs {
    #[arg(help = "agent profile to run")]
    pub(super) agent: String,
    #[arg(
        long,
        default_value = ".",
        help = "host project directory to mount at /workspace"
    )]
    pub(super) workspace: PathBuf,
    #[arg(
        long,
        default_value = "current",
        help = "current mounts the selected directory; git-root explicitly expands to the containing git repository root"
    )]
    pub(super) workspace_scope: String,
    #[arg(
        long,
        help = "named reusable project session for the agent home volume"
    )]
    pub(super) session: Option<String>,
    #[arg(
        long,
        default_value = "agent",
        help = "agent reuses one login per agent across all your projects (log in once); project isolates the login to this workspace"
    )]
    pub(super) auth_scope: String,
    #[arg(long, help = "override the profile image")]
    pub(super) image: Option<String>,
    #[arg(long, default_value = "4", help = "virtual CPUs for the container")]
    pub(super) cpus: String,
    #[arg(long, default_value = "4g", help = "memory limit for the container")]
    pub(super) memory: String,
    #[arg(
        long,
        default_value = "auto",
        help = "allocate a container TTY; auto follows the current terminal"
    )]
    pub(super) tty: String,
    #[arg(long, help = "do not keep container standard input open")]
    pub(super) no_interactive: bool,
    #[arg(
        long,
        help = "egress mode: provider routes through a runtime allowlist proxy, internal is host-only, internet is unrestricted. Default is provider for profiles with bundled provider hosts, otherwise internet"
    )]
    pub(super) network: Option<String>,
    #[arg(
        long,
        value_name = "HOST",
        help = "additional fully qualified HTTPS host allowed by --network provider"
    )]
    pub(super) provider_host: Vec<String>,
    #[arg(
        long,
        alias = "codex-api-key-broker-env",
        value_name = "NAME",
        help = "read this host environment variable at run time and broker the selected agent's API requests (Codex, Claude, Gemini) without placing the raw value in the guest"
    )]
    pub(super) api_key_broker_env: Option<String>,
    #[arg(
        long,
        help = "mount the workspace read-only so the agent can inspect but not edit it"
    )]
    pub(super) read_only_workspace: bool,
    #[arg(
        long,
        help = "currently disabled: SSH forwarding fails closed until Apple container non-root forwarding is verified"
    )]
    pub(super) ssh: bool,
    #[arg(
        long,
        value_name = "NAME",
        help = "inherit a single host environment variable by name"
    )]
    pub(super) env: Vec<String>,
    #[arg(
        long,
        default_value = "agent",
        help = "container user to run as; bundled images provide the non-root agent user"
    )]
    pub(super) user: String,
    #[arg(long, help = "allow mounting broad or credential-bearing host paths")]
    pub(super) allow_sensitive_workspace: bool,
    #[arg(
        long,
        help = "allow running the agent process as root inside the container"
    )]
    pub(super) allow_root_user: bool,
}

#[derive(Args, Debug)]
pub(super) struct RunCommand {
    #[command(flatten)]
    pub(super) args: RunArgs,
    #[arg(long, help = "print the plan instead of running")]
    pub(super) dry_run: bool,
    #[arg(
        long,
        help = "create a RunHaven-owned git worktree for this run and mount it"
    )]
    pub(super) worktree: bool,
}
