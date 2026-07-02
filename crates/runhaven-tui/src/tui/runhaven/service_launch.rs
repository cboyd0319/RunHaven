use std::path::Path;
use std::path::PathBuf;

use runhaven_core::runtime::plans::RunOptions;
use runhaven_core::runtime::plans::WorkspaceScope;
use runhaven_core::runtime::plans::build_run_plan;
use runhaven_core::runtime::profiles::profiles;
use runhaven_core::support::git::git_repo_root;
use runhaven_core::ui_contracts::AgentCatalogItemData;

use super::AgentLaunchPreview;
use super::CURRENT_DIRECTORY_WORKSPACE_LABEL;
use super::GIT_REPOSITORY_ROOT_WORKSPACE_DESCRIPTION;
use super::GIT_REPOSITORY_ROOT_WORKSPACE_LABEL;
use super::LaunchPolicySelection;
use super::LaunchPreviewError;
use super::LaunchPreviewPayload;
use super::PreparedLaunch;
use super::RunHavenTuiService;
use super::WorkspaceLaunchPreview;

impl RunHavenTuiService {
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
                    claude_auth_source: None,
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
}
