use std::path::PathBuf;

use anyhow::{Result, bail};
use serde::Serialize;

use crate::runtime::profiles::AgentProfile;
use crate::support::shell;

pub const SUPPORTED_NETWORK_MODES: &[&str] = &["internet", "internal", "provider"];
pub const SUPPORTED_WORKSPACE_SCOPES: &[&str] = &["current", "git-root"];
pub const DEFAULT_ENV_PASSTHROUGH: &[&str] = &["TERM", "COLORTERM", "LANG", "LC_ALL", "NO_COLOR"];
pub const CONTAINER_PATH: &str =
    "/opt/runhaven-agent/node_modules/.bin:/home/agent/.local/bin:/usr/local/bin:/usr/bin:/bin";
pub const VOLUME_PREP_IMAGE: &str =
    "debian:trixie-slim@sha256:4e401d95de7083948053197a9c3913343cd06b706bf15eb6a0c3ccd26f436a0e";
pub const VOLUME_PREP_NETWORK: &str = "runhaven-volume-prep-internal";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NetworkMode {
    Internet,
    Internal,
    Provider,
}

impl NetworkMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Internet => "internet",
            Self::Internal => "internal",
            Self::Provider => "provider",
        }
    }
}

impl TryFrom<&str> for NetworkMode {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        match value {
            "internet" => Ok(Self::Internet),
            "internal" => Ok(Self::Internal),
            "provider" => Ok(Self::Provider),
            _ => bail!("invalid network mode: {value:?}"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkspaceScope {
    Current,
    GitRoot,
}

impl WorkspaceScope {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Current => "current",
            Self::GitRoot => "git-root",
        }
    }
}

impl TryFrom<&str> for WorkspaceScope {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        match value {
            "current" => Ok(Self::Current),
            "git-root" => Ok(Self::GitRoot),
            _ => bail!("invalid workspace scope: {value:?}"),
        }
    }
}

/// Where an agent's login and state volume live, so an OAuth login can be done
/// once and reused.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Default, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuthScope {
    /// One login per agent, reused across every workspace (log in once).
    #[default]
    Agent,
    /// A login isolated to this workspace (the per-workspace default state).
    Project,
}

impl AuthScope {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Agent => "agent",
            Self::Project => "project",
        }
    }
}

impl TryFrom<&str> for AuthScope {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        match value {
            "agent" => Ok(Self::Agent),
            "project" => Ok(Self::Project),
            _ => bail!("invalid auth scope: {value:?} (use agent or project)"),
        }
    }
}

/// Which Claude auth path a run uses through the host-side broker. This replaces
/// a bare `api_key_broker_env: Option<String>` so the OAuth token can never land
/// on a persisted plan field (design C Section 3, F1): the token is read from
/// the macOS Keychain at broker-spawn and flows straight into the broker's
/// in-memory state, never onto the plan, a marker, a record, or a log. Both
/// variants are non-secret; neither ever holds a token.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub enum ClaudeAuthSource {
    /// API-key broker: the run reads the real key from this host environment
    /// variable NAME (e.g. `ANTHROPIC_API_KEY`). Non-secret: the name, never the
    /// value. This is the only broker path that existed before design C, and it
    /// is unchanged.
    ApiKeyBrokerEnv(String),
    /// Claude subscription OAuth token brokered from the macOS Keychain. No
    /// payload: the token is fetched from the Keychain at broker-spawn and never
    /// touches the plan. A run using this variant persists no env NAME and no
    /// secret-derived value anywhere.
    KeychainOAuth,
}

impl ClaudeAuthSource {
    /// The host environment variable NAME for the API-key broker path, or `None`
    /// for the Keychain OAuth path (which reads no env var). This NEVER returns a
    /// secret: `ApiKeyBrokerEnv` carries only the variable name, and
    /// `KeychainOAuth` carries nothing. Read sites that only need the display or
    /// persistence name use this.
    pub fn broker_env_name(&self) -> Option<&str> {
        match self {
            Self::ApiKeyBrokerEnv(name) => Some(name.as_str()),
            Self::KeychainOAuth => None,
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
    /// uses the env-var broker path. `None` for the Keychain OAuth path or a run
    /// with no broker. NEVER returns a secret (see [`ClaudeAuthSource`]).
    pub fn api_key_broker_env_name(&self) -> Option<&str> {
        self.claude_auth_source
            .as_ref()
            .and_then(ClaudeAuthSource::broker_env_name)
    }

    pub fn shell_preflight(&self) -> Vec<String> {
        self.preflight
            .iter()
            .map(|command| shell::join(command))
            .collect()
    }
}
