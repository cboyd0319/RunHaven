use std::path::Path;
use std::path::PathBuf;

use runhaven_core::runtime::plans::AuthScope;
use runhaven_core::runtime::plans::RunOptions;
use runhaven_core::runtime::plans::WorkspaceScope;
use runhaven_core::runtime::plans::build_run_plan;
use runhaven_core::runtime::plans::default_network_mode;
use runhaven_core::runtime::profiles::profiles;
use runhaven_core::ui_contracts::AgentCatalogData;
use runhaven_core::ui_contracts::AgentCatalogItemData;
use runhaven_core::ui_contracts::LaunchPlanData;
use serde_json::Value;

use super::protocol::ClientRequest;
use super::protocol::UnsupportedFamily;
use super::protocol::ValidateWorkspaceResponse;

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct AgentLaunchPreview {
    pub(crate) agent: AgentCatalogItemData,
    pub(crate) plan: Result<LaunchPlanData, LaunchPreviewError>,
}

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

pub(crate) struct LaunchPreviewPayload {
    pub(crate) workspace: PathBuf,
    pub(crate) previews: Vec<AgentLaunchPreview>,
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

    pub(crate) fn launch_preview_payload(
        &self,
        workspace: impl AsRef<Path>,
    ) -> LaunchPreviewPayload {
        let workspace = workspace.as_ref().to_path_buf();
        let previews = profiles()
            .into_iter()
            .map(|profile| {
                let network = default_network_mode(&profile);
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
                    auth_scope: AuthScope::Agent,
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
                .map(|plan| LaunchPlanData::from(&plan))
                .map_err(LaunchPreviewError::plan_build_failed);

                AgentLaunchPreview { agent, plan }
            })
            .collect();

        LaunchPreviewPayload {
            workspace,
            previews,
        }
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
}

#[cfg(test)]
pub(crate) fn confirm_required_preview_for_tests() -> AgentLaunchPreview {
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
    let plan = LaunchPlanData {
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

    AgentLaunchPreview {
        agent,
        plan: Ok(plan),
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

    fn plan<'a>(preview: &'a AgentLaunchPreview, name: &str) -> &'a LaunchPlanData {
        preview
            .plan
            .as_ref()
            .unwrap_or_else(|error| panic!("{name} plan failed: {error}"))
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
            assert!(!preview.agent.description.trim().is_empty());
            assert!(!preview.agent.image.trim().is_empty());
            assert!(!plan.command.trim().is_empty());
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
}
