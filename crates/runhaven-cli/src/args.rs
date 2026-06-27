use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use runhaven_core::runtime::active::DEFAULT_LOG_FOLLOW_LINES;

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
        #[arg(help = "agent profile to log in (currently: claude)")]
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

#[derive(Subcommand, Debug)]
pub(super) enum ImageCommand {
    #[command(about = "build a bundled agent image")]
    Build(ImageBuildArgs),
    #[command(about = "rebuild a bundled agent image")]
    Rebuild(ImageBuildArgs),
    #[command(about = "diagnose bundled agent image availability")]
    Doctor {
        #[arg(help = "agent image to diagnose; defaults to all bundled agents")]
        agent: Option<String>,
    },
}

#[derive(Args, Debug)]
pub(super) struct ImageBuildArgs {
    #[arg(help = "agent image template to build")]
    pub(super) agent: String,
    #[arg(long, help = "override the image tag")]
    pub(super) tag: Option<String>,
    #[arg(long, help = "print the build command")]
    pub(super) dry_run: bool,
}

#[derive(Subcommand, Debug)]
pub(super) enum NetworkCommand {
    #[command(about = "list RunHaven managed Apple container networks")]
    List,
    #[command(about = "remove RunHaven managed Apple container networks")]
    Prune {
        #[arg(long, help = "delete listed networks")]
        yes: bool,
    },
}

#[derive(Subcommand, Debug)]
pub(super) enum StateCommand {
    #[command(about = "list RunHaven agent home volumes")]
    List {
        #[arg(long, help = "only list volumes for this named session")]
        session: Option<String>,
    },
    #[command(about = "remove RunHaven agent home volumes")]
    Prune {
        #[arg(long, help = "only prune volumes for this named session")]
        session: Option<String>,
        #[arg(long, help = "delete listed volumes")]
        yes: bool,
    },
    #[command(about = "delete one planned project/profile/session state volume")]
    Reset {
        #[arg(help = "agent profile to reset")]
        agent: String,
        #[arg(
            long,
            default_value = ".",
            help = "host project directory whose state volume should be reset"
        )]
        workspace: PathBuf,
        #[arg(
            long,
            default_value = "current",
            help = "current resets the selected directory state; git-root explicitly expands to the containing git repository root"
        )]
        workspace_scope: String,
        #[arg(long, help = "named session to reset")]
        session: Option<String>,
        #[arg(
            long,
            help = "allow resolving broad or credential-bearing host paths for reset targeting"
        )]
        allow_sensitive_workspace: bool,
        #[arg(long, help = "delete the planned volume")]
        yes: bool,
    },
}

#[derive(Subcommand, Debug)]
pub(super) enum RunsCommand {
    #[command(about = "show recent RunHaven runs")]
    List {
        #[arg(
            long,
            default_value_t = 20,
            help = "maximum entries to show; use 0 for all entries"
        )]
        limit: usize,
        #[arg(long, help = "print JSON output")]
        json: bool,
    },
    #[command(about = "show one RunHaven run record")]
    Show {
        run_id: String,
        #[arg(long, help = "print JSON output")]
        json: bool,
    },
    #[command(about = "show one run with related provider and auth events")]
    Log {
        run_id: String,
        #[arg(long, help = "print JSON output")]
        json: bool,
    },
    #[command(about = "show live git diff for one RunHaven run")]
    Diff { run_id: String },
    #[command(about = "print manual recovery steps for a RunHaven worktree run")]
    Recover {
        run_id: String,
        #[arg(long, help = "print JSON output")]
        json: bool,
    },
    #[command(about = "merge a RunHaven worktree run back into the source checkout")]
    Merge { run_id: String },
    #[command(about = "keep a RunHaven worktree run for manual review")]
    Keep { run_id: String },
    #[command(about = "discard a RunHaven worktree run and delete its branch")]
    Discard { run_id: String },
    #[command(about = "show currently active RunHaven runs")]
    Active {
        #[arg(long, help = "print JSON output")]
        json: bool,
    },
    #[command(about = "show sanitized status for an active RunHaven run")]
    Status {
        run_id: String,
        #[arg(long, help = "print JSON output")]
        json: bool,
    },
    #[command(
        about = "open a shell or command in an active RunHaven run",
        after_help = "Use -- before a custom command, for example: runhaven runs attach RUN_ID -- pwd"
    )]
    Attach {
        run_id: String,
        #[arg(
            long,
            default_value = "agent",
            help = "container user for the attached process; defaults to non-root agent"
        )]
        user: String,
        #[arg(long, help = "allow attaching as root inside the active container")]
        allow_root_user: bool,
        #[arg(
            long,
            default_value = "/workspace",
            help = "container working directory for the attached process"
        )]
        workdir: String,
        #[arg(
            long,
            default_value = "auto",
            help = "allocate a TTY for the attached process; auto follows the current terminal"
        )]
        tty: String,
    },
    #[command(
        name = "logs-follow",
        about = "follow logs from an active RunHaven run"
    )]
    LogsFollow {
        run_id: String,
        #[arg(long, default_value_t = DEFAULT_LOG_FOLLOW_LINES, help = "recent log lines to show before following")]
        lines: u32,
    },
    #[command(about = "stop an active RunHaven run")]
    Stop { run_id: String },
    #[command(about = "hard-stop an active RunHaven run")]
    Kill { run_id: String },
    #[command(about = "remove a stale active marker after confirming its container is gone")]
    Repair {
        run_id: Option<String>,
        #[arg(
            long,
            help = "inspect all active markers and remove only confirmed-stale markers"
        )]
        all: bool,
        #[arg(long, help = "print JSON output")]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
pub(super) enum EgressCommand {
    #[command(about = "show recent provider proxy policy decisions")]
    Log {
        #[arg(
            long,
            default_value_t = 20,
            help = "maximum entries to show; use 0 for all entries"
        )]
        limit: usize,
        #[arg(long, help = "print JSON output")]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
pub(super) enum AuthCommand {
    #[command(about = "show auth broker status without reading secrets")]
    Status {
        #[arg(long, help = "print JSON output")]
        json: bool,
    },
    #[command(about = "explain the auth broker boundary for an agent")]
    Explain {
        agent: String,
        #[arg(long, help = "print JSON output")]
        json: bool,
    },
    #[command(about = "show recent auth broker decisions without secrets")]
    Log {
        #[arg(
            long,
            default_value_t = 20,
            help = "maximum entries to show; use 0 for all entries"
        )]
        limit: usize,
        #[arg(long, help = "print JSON output")]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
pub(super) enum WhyCommand {
    #[command(about = "explain provider-host allowlist behavior")]
    Host {
        host: String,
        #[arg(long, default_value_t = 443, help = "provider port to check")]
        port: u16,
        #[arg(
            long,
            help = "agent profile whose bundled provider hosts should be checked"
        )]
        agent: Option<String>,
    },
    #[command(about = "explain workspace mount validation")]
    Workspace {
        path: PathBuf,
        #[arg(
            long,
            default_value = "current",
            help = "workspace scope to evaluate: current or git-root"
        )]
        workspace_scope: String,
        #[arg(long, help = "evaluate with the sensitive-workspace override enabled")]
        allow_sensitive_workspace: bool,
    },
    #[command(about = "explain network mode behavior")]
    Network { mode: String },
    #[command(about = "explain agent state-volume isolation")]
    State { agent: String },
}
