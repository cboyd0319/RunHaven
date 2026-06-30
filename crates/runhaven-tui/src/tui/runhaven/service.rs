use std::path::Path;
use std::path::PathBuf;

use runhaven_core::diagnostics::auth_status_payload;
use runhaven_core::diagnostics::read_auth_broker_log_tail_bounded;
use runhaven_core::diagnostics::read_egress_policy_log_tail_bounded;
use runhaven_core::records::read_run_records_tail_bounded;
use runhaven_core::runtime::active::active_run_log_snapshot_payload;
use runhaven_core::runtime::active::read_active_run_records;
use runhaven_core::runtime::active::validate_log_snapshot_lines;
use runhaven_core::runtime::plans::AgentRunPlan;
use runhaven_core::runtime::plans::AuthScope;
use runhaven_core::runtime::plans::NetworkMode;
use runhaven_core::runtime::plans::RunOptions;
use runhaven_core::runtime::plans::WorkspaceScope;
use runhaven_core::runtime::plans::build_run_plan;
use runhaven_core::runtime::plans::default_network_mode;
use runhaven_core::runtime::profiles::profiles;
use runhaven_core::support::git::git_repo_root;
use runhaven_core::ui_contracts::ActiveRunListData;
use runhaven_core::ui_contracts::ActiveRunLogSnapshotData;
use runhaven_core::ui_contracts::AgentCatalogData;
use runhaven_core::ui_contracts::AgentCatalogItemData;
use runhaven_core::ui_contracts::LaunchPlanData;
use runhaven_core::ui_contracts::RunHavenDiagnosticsData;
use runhaven_core::ui_contracts::RunHistoryListData;
use serde_json::Value;

use super::protocol::ClientRequest;
use super::protocol::UnsupportedFamily;
use super::protocol::ValidateWorkspaceResponse;

const DIAGNOSTICS_LOG_TAIL_BYTES: u64 = 2 * 1024 * 1024;
const RUN_HISTORY_LOG_TAIL_BYTES: u64 = 4 * 1024 * 1024;
pub(crate) const CURRENT_DIRECTORY_WORKSPACE_LABEL: &str = "Current directory";
pub(crate) const GIT_REPOSITORY_ROOT_WORKSPACE_LABEL: &str = "Git repository root";
pub(crate) const GIT_REPOSITORY_ROOT_WORKSPACE_DESCRIPTION: &str =
    "Mount the full repository instead of only the nested folder.";

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct LaunchPolicySelection {
    pub(crate) network: NetworkPolicySelection,
    pub(crate) auth_scope: AuthScope,
}

impl Default for LaunchPolicySelection {
    fn default() -> Self {
        Self {
            network: NetworkPolicySelection::Default,
            auth_scope: AuthScope::Agent,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum NetworkPolicySelection {
    Default,
    Fixed(NetworkMode),
}

impl NetworkPolicySelection {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Fixed(NetworkMode::Provider) => "provider",
            Self::Fixed(NetworkMode::Internal) => "internal",
            Self::Fixed(NetworkMode::Internet) => "internet",
        }
    }

    fn resolve(self, profile: &runhaven_core::runtime::profiles::AgentProfile) -> NetworkMode {
        match self {
            Self::Default => default_network_mode(profile),
            Self::Fixed(mode) => mode,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct AgentLaunchPreview {
    pub(crate) agent: AgentCatalogItemData,
    pub(crate) plan: Result<PreparedLaunch, LaunchPreviewError>,
}

#[derive(Debug, Clone)]
pub(crate) struct PreparedLaunch {
    pub(crate) data: LaunchPlanData,
    pub(crate) executable: AgentRunPlan,
    pub(crate) policy: LaunchPolicySelection,
}

impl PreparedLaunch {
    fn from_agent_run_plan(executable: AgentRunPlan, policy: LaunchPolicySelection) -> Self {
        let data = LaunchPlanData::from(&executable);
        Self {
            data,
            executable,
            policy,
        }
    }

    #[cfg(test)]
    pub(crate) fn from_parts_for_tests(data: LaunchPlanData, executable: AgentRunPlan) -> Self {
        Self {
            data,
            executable,
            policy: LaunchPolicySelection::default(),
        }
    }
}

impl PartialEq for PreparedLaunch {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl Eq for PreparedLaunch {}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum LaunchPreviewError {
    PlanBuildFailed { detail: String },
}

impl LaunchPreviewError {
    fn plan_build_failed(error: anyhow::Error) -> Self {
        Self::PlanBuildFailed {
            detail: error.to_string(),
        }
    }

    pub(crate) fn reason(&self) -> &'static str {
        match self {
            Self::PlanBuildFailed { .. } => "Plan could not be built.",
        }
    }

    pub(crate) fn detail(&self) -> &str {
        match self {
            Self::PlanBuildFailed { detail } => detail,
        }
    }
}

impl std::fmt::Display for LaunchPreviewError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.reason(), self.detail())
    }
}

#[derive(Debug)]
pub(crate) struct LaunchPreviewPayload {
    pub(crate) workspace: PathBuf,
    pub(crate) previews: Vec<AgentLaunchPreview>,
}

#[derive(Debug)]
pub(crate) struct WorkspaceLaunchPreview {
    pub(crate) label: String,
    pub(crate) description: String,
    pub(crate) payload: LaunchPreviewPayload,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum RunHavenServiceError {
    Unsupported {
        method: String,
        family: UnsupportedFamily,
    },
    Validation {
        method: String,
        message: String,
    },
    Backend {
        method: String,
        message: String,
    },
}

impl std::fmt::Display for RunHavenServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported { method, .. } => {
                write!(f, "{method} is not supported by the RunHaven TUI backend")
            }
            Self::Validation { method, message } => {
                write!(f, "{method} validation error: {message}")
            }
            Self::Backend { method, message } => {
                write!(f, "{method} backend error: {message}")
            }
        }
    }
}

impl std::error::Error for RunHavenServiceError {}

#[derive(Debug, Default, Clone)]
pub(crate) struct RunHavenTuiService;

impl RunHavenTuiService {
    pub(crate) fn new() -> Self {
        Self
    }

    pub(crate) async fn handle_request(
        &self,
        request: ClientRequest,
    ) -> Result<Value, RunHavenServiceError> {
        let method = request.method().to_string();
        match request {
            ClientRequest::RunHavenAgentList { .. } => {
                serde_json::to_value(self.agent_catalog_payload()).map_err(|error| {
                    RunHavenServiceError::Backend {
                        method,
                        message: error.to_string(),
                    }
                })
            }
            ClientRequest::RunHavenActiveRuns { .. } => {
                serde_json::to_value(self.active_runs_payload()).map_err(|error| {
                    RunHavenServiceError::Backend {
                        method,
                        message: error.to_string(),
                    }
                })
            }
            ClientRequest::RunHavenDiagnostics { limit, .. } => {
                serde_json::to_value(self.diagnostics_payload(limit).map_err(|error| {
                    RunHavenServiceError::Backend {
                        method: method.clone(),
                        message: error.to_string(),
                    }
                })?)
                .map_err(|error| RunHavenServiceError::Backend {
                    method,
                    message: error.to_string(),
                })
            }
            ClientRequest::RunHavenValidateWorkspace { workspace, .. } => {
                self.validate_workspace(&workspace, &method)?;
                serde_json::to_value(ValidateWorkspaceResponse {
                    workspace: workspace.display().to_string(),
                })
                .map_err(|error| RunHavenServiceError::Backend {
                    method,
                    message: error.to_string(),
                })
            }
            ClientRequest::RunHavenRunLogSnapshot {
                run_id,
                lines,
                confirm_sensitive_output,
                ..
            } => serde_json::to_value(self.active_run_log_snapshot_data(
                &run_id,
                lines,
                confirm_sensitive_output,
                &method,
            )?)
            .map_err(|error| RunHavenServiceError::Backend {
                method,
                message: error.to_string(),
            }),
            ClientRequest::Unsupported { method, .. } => Err(RunHavenServiceError::Unsupported {
                method: method.method().to_string(),
                family: method.family(),
            }),
            #[cfg(test)]
            ClientRequest::BackendFailureForTest { message, .. } => {
                Err(RunHavenServiceError::Backend { method, message })
            }
        }
    }

    pub(crate) fn agent_catalog_payload(&self) -> AgentCatalogData {
        AgentCatalogData::from_profiles(profiles())
    }

    pub(crate) fn active_runs_payload(&self) -> ActiveRunListData {
        ActiveRunListData::from_active_run_records(read_active_run_records())
    }

    pub(crate) fn run_history_payload(&self, limit: usize) -> anyhow::Result<RunHistoryListData> {
        Ok(RunHistoryListData::from_run_records(
            read_run_records_tail_bounded(limit, RUN_HISTORY_LOG_TAIL_BYTES)?,
        ))
    }

    pub(crate) fn diagnostics_payload(
        &self,
        limit: usize,
    ) -> anyhow::Result<RunHavenDiagnosticsData> {
        Ok(RunHavenDiagnosticsData::from_payloads(
            auth_status_payload(),
            read_egress_policy_log_tail_bounded(limit, DIAGNOSTICS_LOG_TAIL_BYTES)?,
            read_auth_broker_log_tail_bounded(limit, DIAGNOSTICS_LOG_TAIL_BYTES)?,
        ))
    }

    pub(crate) fn active_run_log_snapshot_data(
        &self,
        run_id: &str,
        lines: u32,
        confirm_sensitive_output: bool,
        method: &str,
    ) -> Result<ActiveRunLogSnapshotData, RunHavenServiceError> {
        self.validate_sensitive_log_confirmation(confirm_sensitive_output, method)?;
        self.validate_log_snapshot_lines(lines, method)?;
        self.active_run_log_snapshot_payload(run_id, lines, method)
    }

    pub(crate) fn launch_preview_payload(
        &self,
        workspace: impl AsRef<Path>,
    ) -> LaunchPreviewPayload {
        self.launch_preview_payload_with_policy(workspace, LaunchPolicySelection::default())
    }

    pub(crate) fn launch_preview_payload_with_policy(
        &self,
        workspace: impl AsRef<Path>,
        policy: LaunchPolicySelection,
    ) -> LaunchPreviewPayload {
        let workspace = workspace.as_ref().to_path_buf();
        let previews = profiles()
            .into_iter()
            .map(|profile| {
                let network = policy.network.resolve(&profile);
                let agent = AgentCatalogItemData::from_profile(&profile);
                let plan = build_run_plan(RunOptions {
                    profile,
                    workspace: workspace.clone(),
                    agent_args: Vec::new(),
                    image: None,
                    cpus: "4".to_string(),
                    memory: "4g".to_string(),
                    network,
                    workspace_scope: WorkspaceScope::Current,
                    session: None,
                    auth_scope: policy.auth_scope,
                    read_only_workspace: false,
                    ssh: false,
                    env: Vec::new(),
                    user: "agent".to_string(),
                    interactive: true,
                    tty: true,
                    allow_sensitive_workspace: false,
                    allow_root_user: false,
                    provider_hosts: Vec::new(),
                    api_key_broker_env: None,
                    worktree: None,
                    run_id: None,
                })
                .map(|executable| PreparedLaunch::from_agent_run_plan(executable, policy))
                .map_err(LaunchPreviewError::plan_build_failed);

                AgentLaunchPreview { agent, plan }
            })
            .collect();

        LaunchPreviewPayload {
            workspace,
            previews,
        }
    }

    pub(crate) fn launch_workspace_choices(
        &self,
        workspace: impl AsRef<Path>,
    ) -> Vec<WorkspaceLaunchPreview> {
        self.launch_workspace_choices_from(workspace, |service, workspace| {
            service.launch_preview_payload(workspace)
        })
    }

    pub(crate) fn launch_workspace_choices_with_policy(
        &self,
        workspace: impl AsRef<Path>,
        policy: LaunchPolicySelection,
    ) -> Vec<WorkspaceLaunchPreview> {
        self.launch_workspace_choices_from(workspace, |service, workspace| {
            service.launch_preview_payload_with_policy(workspace, policy)
        })
    }

    fn launch_workspace_choices_from(
        &self,
        workspace: impl AsRef<Path>,
        mut payload_for: impl FnMut(&Self, &Path) -> LaunchPreviewPayload,
    ) -> Vec<WorkspaceLaunchPreview> {
        let workspace = workspace.as_ref();
        let mut choices = vec![WorkspaceLaunchPreview {
            label: CURRENT_DIRECTORY_WORKSPACE_LABEL.to_string(),
            description: workspace.display().to_string(),
            payload: payload_for(self, workspace),
        }];

        let (repo_root, _) = git_repo_root(workspace);
        if let Some(repo_root) = repo_root {
            let repo_root = PathBuf::from(repo_root);
            let current = workspace
                .canonicalize()
                .unwrap_or_else(|_| workspace.to_path_buf());
            if repo_root != current {
                choices.push(WorkspaceLaunchPreview {
                    label: GIT_REPOSITORY_ROOT_WORKSPACE_LABEL.to_string(),
                    description: GIT_REPOSITORY_ROOT_WORKSPACE_DESCRIPTION.to_string(),
                    payload: payload_for(self, &repo_root),
                });
            }
        }

        choices
    }

    fn validate_workspace(
        &self,
        workspace: &Path,
        method: &str,
    ) -> Result<(), RunHavenServiceError> {
        if workspace.exists() {
            return Ok(());
        }

        Err(RunHavenServiceError::Validation {
            method: method.to_string(),
            message: format!("workspace does not exist: {}", workspace.display()),
        })
    }

    fn validate_log_snapshot_lines(
        &self,
        lines: u32,
        method: &str,
    ) -> Result<(), RunHavenServiceError> {
        validate_log_snapshot_lines(lines).map_err(|error| RunHavenServiceError::Validation {
            method: method.to_string(),
            message: error.to_string(),
        })
    }

    fn validate_sensitive_log_confirmation(
        &self,
        confirmed: bool,
        method: &str,
    ) -> Result<(), RunHavenServiceError> {
        if confirmed {
            return Ok(());
        }

        Err(RunHavenServiceError::Validation {
            method: method.to_string(),
            message: "Confirm raw log viewing before loading output that may contain secrets."
                .to_string(),
        })
    }

    fn active_run_log_snapshot_payload(
        &self,
        run_id: &str,
        lines: u32,
        method: &str,
    ) -> Result<ActiveRunLogSnapshotData, RunHavenServiceError> {
        let payload = active_run_log_snapshot_payload(run_id, lines).map_err(|error| {
            RunHavenServiceError::Backend {
                method: method.to_string(),
                message: error.to_string(),
            }
        })?;
        let data = ActiveRunLogSnapshotData::from_active_log_snapshot_payload(payload).map_err(
            |error| RunHavenServiceError::Backend {
                method: method.to_string(),
                message: error.to_string(),
            },
        )?;

        Ok(data)
    }
}

#[cfg(test)]
pub(crate) fn confirm_required_preview_for_tests() -> AgentLaunchPreview {
    use runhaven_core::runtime::plans::AgentRunPlan;
    use runhaven_core::runtime::plans::AuthScope;
    use runhaven_core::runtime::plans::NetworkMode;
    use runhaven_core::runtime::plans::WorkspaceScope;
    use runhaven_core::ui_contracts::LaunchBoundaryData;
    use runhaven_core::ui_contracts::LaunchNetworkData;

    let agent = AgentCatalogItemData {
        name: "codex".to_string(),
        description: "Codex test profile".to_string(),
        image: "runhaven/codex:0.1.0".to_string(),
        sign_in: "runhaven login codex".to_string(),
        broker: "no".to_string(),
        default_network: "provider".to_string(),
        provider_host_count: 1,
    };
    let data = LaunchPlanData {
        profile_name: "codex".to_string(),
        workspace: "/tmp/project".to_string(),
        workspace_scope: "current".to_string(),
        workspace_scope_note: None,
        auth_scope: "agent".to_string(),
        session: "none".to_string(),
        state_volume: "runhaven-codex-shared-home".to_string(),
        container_name: "runhaven-codex".to_string(),
        image: "runhaven/codex:0.1.0".to_string(),
        worktree: None,
        network: LaunchNetworkData {
            mode: "provider".to_string(),
            name: Some("runhaven-provider".to_string()),
            summary: "provider allowlist".to_string(),
            provider_allowed_hosts: vec!["api.openai.com".to_string()],
            api_key_broker_env: None,
        },
        boundary: LaunchBoundaryData {
            mounted_workspace: "/tmp/project -> /workspace".to_string(),
            mounted_state_volume: "runhaven-codex-shared-home -> /home/agent".to_string(),
            not_shared: vec![
                "host home folder".to_string(),
                "raw SSH keys".to_string(),
                "browser profiles".to_string(),
            ],
        },
        preflight_commands: Vec::new(),
        command: "container run --name runhaven-codex runhaven/codex:0.1.0".to_string(),
        safety_notes: vec!["This plan uses a less safe launch option.".to_string()],
        confirm_required: true,
    };
    let executable = AgentRunPlan {
        command: vec![
            "container".to_string(),
            "run".to_string(),
            "--name".to_string(),
            "runhaven-codex".to_string(),
            "runhaven/codex:0.1.0".to_string(),
        ],
        preflight: Vec::new(),
        workspace: PathBuf::from("/tmp/project"),
        state_volume: "runhaven-codex-shared-home".to_string(),
        session: "none".to_string(),
        container_name: "runhaven-codex".to_string(),
        profile_name: "codex".to_string(),
        workspace_scope: WorkspaceScope::Current,
        workspace_scope_note: None,
        auth_scope: AuthScope::Agent,
        worktree: None,
        run_id: None,
        network_name: Some("runhaven-provider".to_string()),
        network_mode: NetworkMode::Provider,
        egress_summary: "provider allowlist".to_string(),
        image: "runhaven/codex:0.1.0".to_string(),
        provider_allowed_hosts: vec!["api.openai.com".to_string()],
        api_key_broker_env: None,
        security_notices: vec!["This plan uses a less safe launch option.".to_string()],
    };

    AgentLaunchPreview {
        agent,
        plan: Ok(PreparedLaunch::from_parts_for_tests(data, executable)),
    }
}

#[cfg(test)]
#[path = "service_tests.rs"]
mod tests;
