use std::path::PathBuf;

use serde::Serialize;

use super::modes::{AuthScope, NetworkMode, WorkspaceScope};
use crate::runtime::profiles::AgentProfile;
use crate::support::shell;

pub const DEFAULT_ENV_PASSTHROUGH: &[&str] = &["TERM", "COLORTERM", "LANG", "LC_ALL", "NO_COLOR"];
pub const CONTAINER_PATH: &str =
    "/opt/runhaven-agent/node_modules/.bin:/home/agent/.local/bin:/usr/local/bin:/usr/bin:/bin";
pub const VOLUME_PREP_IMAGE: &str =
    "debian:trixie-slim@sha256:28de0877c2189802884ccd20f15ee41c203573bd87bb6b883f5f46362d24c5c2";
pub const VOLUME_PREP_NETWORK: &str = "runhaven-volume-prep-internal";

/// An explicitly configured host-side API-key broker selector. The value is the
/// environment-variable name only, never its secret value.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub enum ClaudeAuthSource {
    /// API-key broker: the run reads the real key from this host environment
    /// variable NAME (e.g. `ANTHROPIC_API_KEY`). Non-secret: the name, never the
    /// value. This is the only broker path that existed before design C, and it
    /// is unchanged.
    ApiKeyBrokerEnv(String),
}

impl ClaudeAuthSource {
    /// The host environment-variable name. This never returns its value.
    pub fn broker_env_name(&self) -> Option<&str> {
        match self {
            Self::ApiKeyBrokerEnv(name) => Some(name.as_str()),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct WorktreeRun {
    pub source_workspace: PathBuf,
    pub source_repo_root: PathBuf,
    pub worktree_root: PathBuf,
    pub mounted_workspace: PathBuf,
    pub branch: String,
    pub base_head: Option<String>,
    pub recovery_commands: Vec<(String, String)>,
}

#[derive(Clone, Debug)]
pub struct RunOptions {
    pub profile: AgentProfile,
    pub workspace: PathBuf,
    pub agent_args: Vec<String>,
    pub image: Option<String>,
    pub cpus: String,
    pub memory: String,
    pub network: NetworkMode,
    pub workspace_scope: WorkspaceScope,
    pub session: Option<String>,
    pub auth_scope: AuthScope,
    pub read_only_workspace: bool,
    pub ssh: bool,
    pub env: Vec<String>,
    pub user: String,
    pub interactive: bool,
    pub tty: bool,
    pub allow_sensitive_workspace: bool,
    pub allow_root_user: bool,
    pub provider_hosts: Vec<String>,
    pub claude_auth_source: Option<ClaudeAuthSource>,
    pub worktree: Option<WorktreeRun>,
    pub run_id: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AgentRunPlan {
    pub command: Vec<String>,
    pub preflight: Vec<Vec<String>>,
    pub workspace: PathBuf,
    pub state_volume: String,
    pub session: String,
    pub container_name: String,
    pub profile_name: String,
    pub workspace_scope: WorkspaceScope,
    pub workspace_scope_note: Option<String>,
    pub auth_scope: AuthScope,
    pub worktree: Option<WorktreeRun>,
    pub run_id: Option<String>,
    pub network_name: Option<String>,
    pub network_mode: NetworkMode,
    pub egress_summary: String,
    pub image: String,
    pub provider_allowed_hosts: Vec<String>,
    pub claude_auth_source: Option<ClaudeAuthSource>,
    pub security_notices: Vec<String>,
}

impl AgentRunPlan {
    pub fn shell_command(&self) -> String {
        shell::join(&self.command)
    }

    /// The API-key broker host environment variable NAME for this run, if it
    /// uses the env-var broker path. `None` for a run with no broker.
    pub fn api_key_broker_env_name(&self) -> Option<&str> {
        self.claude_auth_source
            .as_ref()
            .and_then(ClaudeAuthSource::broker_env_name)
    }

    /// A provider-neutral, non-secret label for host API-key broker records.
    pub fn broker_label(&self) -> &'static str {
        "api-key"
    }

    pub fn shell_preflight(&self) -> Vec<String> {
        self.preflight
            .iter()
            .map(|command| shell::join(command))
            .collect()
    }

    /// Ensure this plan's `container run` argv runs the agent under a host PTY by
    /// carrying both `--interactive` and `--tty`. The desktop in-app run-terminal
    /// path calls this on the plan a normal launch built, which is `--interactive`
    /// but not `--tty` (the background captured runner drives the agent through
    /// pipes). An interactive agent such as Claude Code needs a real TTY: with
    /// none it drops to `--print` (headless) and errors when no piped prompt is
    /// supplied, so a terminal run must run the interactive TUI.
    ///
    /// Idempotent and argv-only: it touches no other argument (image, `--user`,
    /// mounts, network, env are unchanged). A plan that already carries `--tty` is
    /// left as-is. In the normal path `--interactive` is present, so `--tty` is
    /// inserted right after it, among the container-run option flags before the
    /// positional image. The defensive branch (no `--interactive`, not produced by
    /// the GUI launch path) inserts both right after the `run` subcommand so they
    /// still precede the image.
    pub fn make_interactive_tty(&mut self) {
        if self.command.iter().any(|arg| arg == "--tty") {
            return;
        }
        match self.command.iter().position(|arg| arg == "--interactive") {
            Some(pos) => self.command.insert(pos + 1, "--tty".to_string()),
            None => {
                let run_pos = self
                    .command
                    .iter()
                    .position(|arg| arg == "run")
                    .map_or(0, |pos| pos + 1);
                self.command.insert(run_pos, "--tty".to_string());
                self.command.insert(run_pos, "--interactive".to_string());
            }
        }
    }

    /// True when this plan runs the agent under an interactive host PTY, i.e. its
    /// argv carries `--tty`. The desktop run-terminal path adds `--tty` via
    /// [`AgentRunPlan::make_interactive_tty`]; background captured runs stay
    /// `--interactive` only, so this cleanly distinguishes an interactive TUI run.
    /// The user ends such a run by closing the TUI, so a non-zero exit is normal
    /// and its exit code is not a success/failure signal.
    pub fn is_interactive_tty(&self) -> bool {
        self.command.iter().any(|arg| arg == "--tty")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::plans::build_run_plan;
    use crate::runtime::profiles::get_profile;

    /// A GUI-built run plan is `--interactive` but not `--tty` (the background
    /// captured runner drives the agent through pipes). The desktop terminal-run
    /// path calls `make_interactive_tty` before running the plan under a host PTY
    /// via `open_gui_run_pty`; prove it yields an interactive TTY run that still
    /// stays on the non-root `agent` user, so the agent starts its TUI instead of
    /// headless `--print`. Hermetic: it builds the plan through the normal seam
    /// and touches no container.
    #[test]
    fn make_interactive_tty_forces_a_tty_run_and_keeps_it_non_root() {
        let workspace = tempfile::tempdir().expect("workspace");
        let mut plan = build_run_plan(RunOptions {
            profile: get_profile("shell").expect("profile"),
            workspace: workspace.path().to_path_buf(),
            agent_args: vec![
                "/bin/bash".to_string(),
                "-lc".to_string(),
                "true".to_string(),
            ],
            image: None,
            cpus: "4".to_string(),
            memory: "4g".to_string(),
            network: NetworkMode::Internet,
            workspace_scope: WorkspaceScope::Current,
            session: None,
            auth_scope: AuthScope::Agent,
            read_only_workspace: true,
            ssh: false,
            env: Vec::new(),
            user: "agent".to_string(),
            interactive: true,
            tty: false,
            allow_sensitive_workspace: false,
            allow_root_user: false,
            provider_hosts: Vec::new(),
            claude_auth_source: None,
            worktree: None,
            run_id: Some("term-run".to_string()),
        })
        .expect("plan");

        // Baseline: the launch path emits --interactive but not --tty.
        assert!(plan.command.iter().any(|arg| arg == "--interactive"));
        assert!(!plan.command.iter().any(|arg| arg == "--tty"));

        plan.make_interactive_tty();

        // --tty lands right after --interactive, among the container-run flags.
        let interactive_pos = plan
            .command
            .iter()
            .position(|arg| arg == "--interactive")
            .expect("--interactive present");
        assert_eq!(
            plan.command.get(interactive_pos + 1).map(String::as_str),
            Some("--tty"),
            "--tty must sit right after --interactive"
        );
        // Non-root is preserved: still --user agent, and no bare root argument.
        assert!(plan.command.windows(2).any(|w| w == ["--user", "agent"]));
        assert!(!plan.command.iter().any(|arg| arg == "root"));
        // Read-only rootfs and cap-drop are untouched by the argv rewrite.
        assert!(plan.command.iter().any(|arg| arg == "--read-only"));
        assert!(plan.command.windows(2).any(|w| w == ["--cap-drop", "ALL"]));

        // Idempotent: a second call adds no duplicate --tty.
        plan.make_interactive_tty();
        assert_eq!(
            plan.command.iter().filter(|arg| *arg == "--tty").count(),
            1,
            "make_interactive_tty must be idempotent"
        );
    }

    #[test]
    fn is_interactive_tty_distinguishes_terminal_runs_from_background_runs() {
        let workspace = tempfile::tempdir().expect("workspace");
        let mut plan = build_run_plan(RunOptions {
            profile: get_profile("shell").expect("profile"),
            workspace: workspace.path().to_path_buf(),
            agent_args: vec![
                "/bin/bash".to_string(),
                "-lc".to_string(),
                "true".to_string(),
            ],
            image: None,
            cpus: "4".to_string(),
            memory: "4g".to_string(),
            network: NetworkMode::Internet,
            workspace_scope: WorkspaceScope::Current,
            session: None,
            auth_scope: AuthScope::Agent,
            read_only_workspace: true,
            ssh: false,
            env: Vec::new(),
            user: "agent".to_string(),
            interactive: true,
            tty: false,
            allow_sensitive_workspace: false,
            allow_root_user: false,
            provider_hosts: Vec::new(),
            claude_auth_source: None,
            worktree: None,
            run_id: Some("tty-probe".to_string()),
        })
        .expect("plan");

        // A background --interactive-only run (the captured runner) is not an
        // interactive TTY run.
        assert!(!plan.is_interactive_tty());

        // The run-terminal path adds --tty, marking it an interactive TUI run.
        plan.make_interactive_tty();
        assert!(plan.is_interactive_tty());
    }

    #[test]
    fn broker_label_names_the_auth_source_not_a_hardcoded_codex() {
        let workspace = tempfile::tempdir().expect("workspace");
        let mut plan = build_run_plan(RunOptions {
            profile: get_profile("shell").expect("profile"),
            workspace: workspace.path().to_path_buf(),
            agent_args: vec![
                "/bin/bash".to_string(),
                "-lc".to_string(),
                "true".to_string(),
            ],
            image: None,
            cpus: "4".to_string(),
            memory: "4g".to_string(),
            network: NetworkMode::Internet,
            workspace_scope: WorkspaceScope::Current,
            session: None,
            auth_scope: AuthScope::Agent,
            read_only_workspace: true,
            ssh: false,
            env: Vec::new(),
            user: "agent".to_string(),
            interactive: true,
            tty: false,
            allow_sensitive_workspace: false,
            allow_root_user: false,
            provider_hosts: Vec::new(),
            claude_auth_source: None,
            worktree: None,
            run_id: Some("label".to_string()),
        })
        .expect("plan");
        // Broker labels remain provider-neutral and never include a secret.\n        assert_eq!(plan.broker_label(), "api-key");
        plan.claude_auth_source = Some(ClaudeAuthSource::ApiKeyBrokerEnv(
            "ANTHROPIC_API_KEY".to_string(),
        ));
        assert_eq!(plan.broker_label(), "api-key");
    }
}
