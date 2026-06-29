use std::path::Path;
use std::path::PathBuf;

use runhaven_core::diagnostics::auth_status_payload;
use runhaven_core::diagnostics::read_auth_broker_log_tail_bounded;
use runhaven_core::diagnostics::read_egress_policy_log_tail_bounded;
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
use serde_json::Value;

use super::protocol::ClientRequest;
use super::protocol::UnsupportedFamily;
use super::protocol::ValidateWorkspaceResponse;

const DIAGNOSTICS_LOG_TAIL_BYTES: u64 = 2 * 1024 * 1024;

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
            label: "Current directory".to_string(),
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
                    label: "Git repository root".to_string(),
                    description: "Mount the full repository instead of only the nested folder."
                        .to_string(),
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
mod tests {
    use super::*;

    fn preview<'a>(payload: &'a LaunchPreviewPayload, name: &str) -> &'a AgentLaunchPreview {
        payload
            .previews
            .iter()
            .find(|preview| preview.agent.name == name)
            .unwrap_or_else(|| panic!("missing {name} preview"))
    }

    fn launch<'a>(preview: &'a AgentLaunchPreview, name: &str) -> &'a PreparedLaunch {
        preview
            .plan
            .as_ref()
            .unwrap_or_else(|error| panic!("{name} plan failed: {error}"))
    }

    fn plan<'a>(preview: &'a AgentLaunchPreview, name: &str) -> &'a LaunchPlanData {
        &launch(preview, name).data
    }

    fn run_git(args: &[&str], cwd: &Path) {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(cwd)
            .output()
            .expect("run git");
        assert!(
            output.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn launch_preview_payload_maps_profiles_to_plan_payloads_by_name() {
        let workspace = tempfile::tempdir().expect("workspace");
        let payload = RunHavenTuiService::new().launch_preview_payload(workspace.path());

        assert_eq!(payload.workspace, workspace.path());
        assert!(payload.previews.len() >= 5);
        for name in [
            "antigravity",
            "claude",
            "codex",
            "copilot",
            "gemini",
            "shell",
        ] {
            let preview = preview(&payload, name);
            let plan = plan(preview, name);
            assert_eq!(preview.agent.name, name);
            assert_eq!(plan.profile_name, name);
            assert_eq!(
                plan.command,
                launch(preview, name).executable.shell_command()
            );
            assert!(!preview.agent.description.trim().is_empty());
            assert!(!preview.agent.image.trim().is_empty());
            assert!(!plan.command.trim().is_empty());
        }
    }

    #[test]
    fn prepared_launch_display_data_is_derived_from_executable_plan() {
        let workspace = tempfile::tempdir().expect("workspace");
        let payload = RunHavenTuiService::new().launch_preview_payload(workspace.path());

        for preview in payload.previews {
            let launch = preview.plan.expect("launch should be prepared");
            assert_eq!(
                launch.data,
                LaunchPlanData::from(&launch.executable),
                "{} display data must stay derived from the executable plan",
                preview.agent.name
            );
        }
    }

    #[test]
    fn launch_preview_payload_uses_default_network_and_agent_auth_scope() {
        let workspace = tempfile::tempdir().expect("workspace");
        let payload = RunHavenTuiService::new().launch_preview_payload(workspace.path());

        let shell = preview(&payload, "shell");
        let shell_plan = plan(shell, "shell");
        assert_eq!(shell.agent.default_network, "internet");
        assert_eq!(shell_plan.network.mode, "internet");
        assert_eq!(shell_plan.auth_scope, "agent");
        assert!(shell_plan.confirm_required);

        let codex = preview(&payload, "codex");
        let codex_plan = plan(codex, "codex");
        assert_eq!(codex.agent.default_network, "provider");
        assert_eq!(codex_plan.network.mode, "provider");
        assert_eq!(codex_plan.auth_scope, "agent");
        assert!(!codex_plan.boundary.not_shared.is_empty());
    }

    #[test]
    fn launch_preview_payload_applies_policy_overrides_to_executable_plan() {
        let workspace = tempfile::tempdir().expect("workspace");
        let payload = RunHavenTuiService::new().launch_preview_payload_with_policy(
            workspace.path(),
            LaunchPolicySelection {
                network: NetworkPolicySelection::Fixed(NetworkMode::Internal),
                auth_scope: AuthScope::Project,
            },
        );

        let codex = preview(&payload, "codex");
        let codex_launch = launch(codex, "codex");

        assert_eq!(codex_launch.data.network.mode, "internal");
        assert_eq!(codex_launch.executable.network_mode, NetworkMode::Internal);
        assert_eq!(codex_launch.data.auth_scope, "project");
        assert_eq!(codex_launch.executable.auth_scope, AuthScope::Project);
        assert_eq!(
            codex_launch.data,
            LaunchPlanData::from(&codex_launch.executable),
            "display policy data must stay derived from executable plan"
        );
    }

    #[test]
    fn provider_policy_override_fails_closed_for_profiles_without_provider_hosts() {
        let workspace = tempfile::tempdir().expect("workspace");
        let payload = RunHavenTuiService::new().launch_preview_payload_with_policy(
            workspace.path(),
            LaunchPolicySelection {
                network: NetworkPolicySelection::Fixed(NetworkMode::Provider),
                auth_scope: AuthScope::Agent,
            },
        );

        let shell = preview(&payload, "shell");
        let error = shell.plan.as_ref().expect_err("shell provider plan");

        assert!(error.detail().contains("provider hosts are required"));
    }

    #[test]
    fn launch_preview_payload_maps_auth_and_provider_metadata() {
        let workspace = tempfile::tempdir().expect("workspace");
        let payload = RunHavenTuiService::new().launch_preview_payload(workspace.path());

        let shell = preview(&payload, "shell");
        assert_eq!(shell.agent.sign_in, "n/a");
        assert_eq!(shell.agent.broker, "n/a");
        assert_eq!(shell.agent.provider_host_count, 0);

        let codex = preview(&payload, "codex");
        let codex_plan = plan(codex, "codex");
        assert_eq!(codex.agent.sign_in, "runhaven login");
        assert_eq!(codex.agent.broker, "yes");
        assert_eq!(codex.agent.provider_host_count, 3);
        assert_eq!(codex_plan.network.mode, "provider");
        assert_eq!(
            codex_plan.network.provider_allowed_hosts,
            ["api.openai.com", "chatgpt.com", "auth.openai.com"]
        );
        assert!(!codex_plan.confirm_required);

        let claude = preview(&payload, "claude");
        let claude_plan = plan(claude, "claude");
        assert_eq!(claude.agent.sign_in, "runhaven login");
        assert_eq!(claude.agent.broker, "yes");
        assert_eq!(claude_plan.network.mode, "provider");
        assert!(
            claude_plan
                .network
                .provider_allowed_hosts
                .contains(&"api.anthropic.com".to_string())
        );
    }

    #[test]
    fn launch_preview_payload_surfaces_internet_confirmation_for_shell() {
        let workspace = tempfile::tempdir().expect("workspace");
        let payload = RunHavenTuiService::new().launch_preview_payload(workspace.path());

        let shell = preview(&payload, "shell");
        let shell_plan = plan(shell, "shell");

        assert_eq!(shell_plan.network.mode, "internet");
        assert!(shell_plan.confirm_required);
        assert!(shell_plan.safety_notes.iter().any(|note| {
            note.contains("Unrestricted internet egress") && note.contains("Use --network provider")
        }));
    }

    #[test]
    fn launch_preview_payload_uses_agent_shared_state_volume() {
        let workspace = tempfile::tempdir().expect("workspace");
        let payload = RunHavenTuiService::new().launch_preview_payload(workspace.path());

        for name in ["codex", "shell"] {
            let preview = preview(&payload, name);
            let plan = plan(preview, name);

            assert_eq!(plan.auth_scope, "agent");
            assert_eq!(plan.state_volume, format!("runhaven-{name}-shared-home"));
            assert_eq!(
                plan.boundary.mounted_state_volume,
                format!("runhaven-{name}-shared-home -> /home/agent")
            );
        }
    }

    #[test]
    fn launch_preview_payload_preserves_nested_git_workspace_note() {
        let repo = tempfile::tempdir().expect("repo");
        run_git(&["init", "-q"], repo.path());
        let nested = repo.path().join("nested");
        std::fs::create_dir(&nested).expect("nested workspace");

        let payload = RunHavenTuiService::new().launch_preview_payload(&nested);
        let codex = preview(&payload, "codex");
        let codex_plan = plan(codex, "codex");
        let note = codex_plan
            .workspace_scope_note
            .as_ref()
            .expect("workspace scope note");

        assert!(note.contains("selected workspace is inside git repository root"));
        assert!(note.contains("RunHaven mounts only the selected directory"));
    }

    #[test]
    fn launch_workspace_choices_offer_current_and_git_root_for_nested_repo() {
        let repo = tempfile::tempdir().expect("repo");
        run_git(&["init", "-q"], repo.path());
        let nested = repo.path().join("nested");
        std::fs::create_dir(&nested).expect("nested workspace");

        let choices = RunHavenTuiService::new().launch_workspace_choices(&nested);

        assert_eq!(choices.len(), 2);
        assert_eq!(choices[0].label, "Current directory");
        assert_eq!(choices[0].payload.workspace, nested);
        assert_eq!(choices[1].label, "Git repository root");
        assert_eq!(
            choices[1].payload.workspace,
            repo.path().canonicalize().expect("canonical repo")
        );
        assert!(
            choices[1]
                .description
                .contains("Mount the full repository instead of only the nested folder")
        );

        let current_codex = launch(preview(&choices[0].payload, "codex"), "codex");
        let root_codex = launch(preview(&choices[1].payload, "codex"), "codex");
        assert_eq!(
            current_codex.executable.workspace,
            choices[0]
                .payload
                .workspace
                .canonicalize()
                .expect("canonical current workspace")
        );
        assert_eq!(
            root_codex.executable.workspace,
            choices[1]
                .payload
                .workspace
                .canonicalize()
                .expect("canonical root workspace")
        );
        assert_ne!(
            current_codex.executable.workspace,
            root_codex.executable.workspace
        );
    }

    #[test]
    fn launch_preview_payload_keeps_plan_errors_per_agent() {
        let root = tempfile::tempdir().expect("root");
        let missing_workspace = root.path().join("missing-workspace");
        let payload = RunHavenTuiService::new().launch_preview_payload(&missing_workspace);

        assert_eq!(payload.workspace, missing_workspace);
        assert!(payload.previews.len() >= 5);
        assert!(payload.previews.iter().all(|preview| preview.plan.is_err()));
        let codex = preview(&payload, "codex");
        assert_eq!(
            codex.agent.description,
            "OpenAI Codex CLI with workspace-write sandboxing inside the container."
        );
        assert_eq!(codex.agent.sign_in, "runhaven login");
        assert_eq!(codex.agent.default_network, "provider");
        assert!(
            payload
                .previews
                .iter()
                .all(|preview| preview.plan.as_ref().err().is_some_and(|error| {
                    error.reason() == "Plan could not be built."
                        && (error.detail().contains("could not resolve workspace path")
                            || error.detail().contains("workspace does not exist"))
                }))
        );
    }

    #[test]
    fn active_runs_payload_omits_workspace_paths() {
        use runhaven_core::runtime::active::write_active_run_payload;
        use runhaven_core::support::paths::override_cache_root_for_tests;

        let cache = tempfile::tempdir().expect("cache");
        let _cache_home = override_cache_root_for_tests(cache.path());
        write_active_run_payload(
            "run-123",
            serde_json::json!({
                "timestamp": "2026-06-29T00:00:00Z",
                "run_id": "run-123",
                "profile": "codex",
                "workspace": "/Users/c/secret/project",
                "network": "provider",
                "status": "running",
                "container_name": "runhaven-codex-project-run",
                "state_volume": "runhaven-codex-shared-home",
                "session": "none"
            }),
        )
        .expect("active marker");

        let payload = RunHavenTuiService::new().active_runs_payload();

        assert_eq!(payload.runs.len(), 1);
        assert_eq!(payload.runs[0].run_id, "run-123");
        let serialized = serde_json::to_string(&payload).expect("serialize");
        assert!(!serialized.contains("/Users/c/secret/project"));
    }

    #[test]
    fn diagnostics_payload_maps_secret_free_log_metadata() {
        use runhaven_core::support::paths::auth_broker_log_path;
        use runhaven_core::support::paths::egress_policy_log_path;
        use runhaven_core::support::paths::ensure_private_parent;
        use runhaven_core::support::paths::override_cache_root_for_tests;
        use std::io::Write;

        let cache = tempfile::tempdir().expect("cache");
        let _cache_home = override_cache_root_for_tests(cache.path());
        ensure_private_parent(&egress_policy_log_path()).expect("egress parent");
        ensure_private_parent(&auth_broker_log_path()).expect("auth parent");
        {
            let mut file =
                std::fs::File::create(egress_policy_log_path()).expect("egress log file");
            writeln!(
                file,
                "{}",
                serde_json::json!({
                    "timestamp": "2026-06-29T00:00:00Z",
                    "profile": "codex",
                    "decision": "denied",
                    "host": "example.com",
                    "port": 443,
                    "count": 1,
                    "reason": "not-in-allowlist",
                    "matched_rule": "",
                    "run_id": "run-123",
                    "workspace": "/Users/c/secret/project"
                })
            )
            .expect("egress write");
        }
        {
            let mut file = std::fs::File::create(auth_broker_log_path()).expect("auth log file");
            writeln!(
                file,
                "{}",
                serde_json::json!({
                    "timestamp": "2026-06-29T00:00:00Z",
                    "profile": "codex",
                    "broker": "api-key",
                    "decision": "allowed",
                    "method": "POST",
                    "path": "/v1/responses?token=secret#fragment",
                    "upstream_status": 200,
                    "count": 1,
                    "reason": "-",
                    "run_id": "run-123",
                    "authorization": "Bearer secret"
                })
            )
            .expect("auth write");
        }

        let payload = RunHavenTuiService::new()
            .diagnostics_payload(10)
            .expect("diagnostics");

        assert_eq!(payload.egress_log[0].host, "example.com");
        assert_eq!(payload.auth_log[0].path, "/v1/responses");
        assert_eq!(payload.auth_log[0].upstream_status, Some(200));
        let serialized = serde_json::to_string(&payload).expect("serialize");
        assert!(!serialized.contains("/Users/c/secret/project"));
        assert!(!serialized.contains("Bearer secret"));
        assert!(!serialized.contains("token=secret"));
    }

    #[tokio::test]
    async fn log_snapshot_rejects_invalid_line_count_before_backend_lookup() {
        let error = RunHavenTuiService::new()
            .handle_request(ClientRequest::RunHavenRunLogSnapshot {
                request_id: 42,
                run_id: "not-a-real-run".to_string(),
                lines: 0,
                confirm_sensitive_output: true,
            })
            .await
            .expect_err("invalid line count should fail validation");

        match error {
            RunHavenServiceError::Validation { method, message } => {
                assert_eq!(method, "runhaven/run/logSnapshot");
                assert!(message.contains("between 1 and 500"));
            }
            other => panic!("expected validation error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn log_snapshot_requires_sensitive_output_confirmation_before_backend_lookup() {
        let error = RunHavenTuiService::new()
            .handle_request(ClientRequest::RunHavenRunLogSnapshot {
                request_id: 42,
                run_id: "not-a-real-run".to_string(),
                lines: 100,
                confirm_sensitive_output: false,
            })
            .await
            .expect_err("missing confirmation should fail validation");

        match error {
            RunHavenServiceError::Validation { method, message } => {
                assert_eq!(method, "runhaven/run/logSnapshot");
                assert!(message.contains("Confirm raw log viewing"));
            }
            other => panic!("expected validation error, got {other:?}"),
        }
    }
}
