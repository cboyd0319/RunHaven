use std::path::PathBuf;

use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub(crate) enum EgressCommand {
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
pub(crate) enum AuthCommand {
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
pub(crate) enum WhyCommand {
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
        #[arg(long, help = "print JSON output")]
        json: bool,
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
        #[arg(long, help = "print JSON output")]
        json: bool,
    },
    #[command(about = "explain network mode behavior")]
    Network {
        mode: String,
        #[arg(long, help = "print JSON output")]
        json: bool,
    },
    #[command(about = "explain agent state-volume isolation")]
    State {
        agent: String,
        #[arg(long, help = "print JSON output")]
        json: bool,
    },
}
