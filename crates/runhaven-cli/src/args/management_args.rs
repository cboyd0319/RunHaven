use std::path::PathBuf;

use clap::{Args, Subcommand};

use runhaven_core::runtime::active::DEFAULT_LOG_FOLLOW_LINES;

#[derive(Subcommand, Debug)]
pub(crate) enum ImageCommand {
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
pub(crate) struct ImageBuildArgs {
    #[arg(help = "agent image template to build")]
    pub(crate) agent: String,
    #[arg(long, help = "override the image tag")]
    pub(crate) tag: Option<String>,
    #[arg(long, help = "print the build command")]
    pub(crate) dry_run: bool,
}

#[derive(Subcommand, Debug)]
pub(crate) enum NetworkCommand {
    #[command(about = "list RunHaven managed Apple container networks")]
    List,
    #[command(about = "remove RunHaven managed Apple container networks")]
    Prune {
        #[arg(long, help = "delete listed networks")]
        yes: bool,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum StateCommand {
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
pub(crate) enum RunsCommand {
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
            value_parser = ["auto", "always", "never"],
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
