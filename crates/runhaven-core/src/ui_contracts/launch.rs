use serde::{Deserialize, Serialize};

use crate::runtime::plans::AgentRunPlan;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchPlanData {
    pub profile_name: String,
    pub workspace: String,
    pub workspace_scope: String,
    pub workspace_scope_note: Option<String>,
    pub auth_scope: String,
    pub session: String,
    pub state_volume: String,
    pub container_name: String,
    pub image: String,
    pub worktree: Option<LaunchWorktreeData>,
    pub network: LaunchNetworkData,
    pub boundary: LaunchBoundaryData,
    pub preflight_commands: Vec<String>,
    pub command: String,
    pub safety_notes: Vec<String>,
    pub confirm_required: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchWorktreeData {
    pub source_workspace: String,
    pub source_repo_root: String,
    pub worktree_root: String,
    pub mounted_workspace: String,
    pub branch: String,
    pub base_head: Option<String>,
    pub recovery_commands: Vec<RecoveryCommandData>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryCommandData {
    pub label: String,
    pub command: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchNetworkData {
    pub mode: String,
    pub name: Option<String>,
    pub summary: String,
    pub provider_allowed_hosts: Vec<String>,
    pub api_key_broker_env: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchBoundaryData {
    pub mounted_workspace: String,
    pub mounted_state_volume: String,
    pub not_shared: Vec<String>,
}

impl LaunchPlanData {
    pub fn from_plan(plan: &AgentRunPlan) -> Self {
        Self {
            profile_name: plan.profile_name.clone(),
            workspace: plan.workspace.display().to_string(),
            workspace_scope: plan.workspace_scope.as_str().to_string(),
            workspace_scope_note: plan.workspace_scope_note.clone(),
            auth_scope: plan.auth_scope.as_str().to_string(),
            session: plan.session.clone(),
            state_volume: plan.state_volume.clone(),
            container_name: plan.container_name.clone(),
            image: plan.image.clone(),
            worktree: plan.worktree.as_ref().map(|worktree| LaunchWorktreeData {
                source_workspace: worktree.source_workspace.display().to_string(),
                source_repo_root: worktree.source_repo_root.display().to_string(),
                worktree_root: worktree.worktree_root.display().to_string(),
                mounted_workspace: worktree.mounted_workspace.display().to_string(),
                branch: worktree.branch.clone(),
                base_head: worktree.base_head.clone(),
                recovery_commands: worktree
                    .recovery_commands
                    .iter()
                    .map(|(label, command)| RecoveryCommandData {
                        label: label.clone(),
                        command: command.clone(),
                    })
                    .collect(),
            }),
            network: LaunchNetworkData {
                mode: plan.network_mode.as_str().to_string(),
                name: plan.network_name.clone(),
                summary: plan.egress_summary.clone(),
                provider_allowed_hosts: plan.provider_allowed_hosts.clone(),
                api_key_broker_env: plan.api_key_broker_env.clone(),
            },
            boundary: LaunchBoundaryData {
                mounted_workspace: format!("{} -> /workspace", plan.workspace.display()),
                mounted_state_volume: format!("{} -> /home/agent", plan.state_volume),
                not_shared: vec![
                    "host home folder".to_string(),
                    "raw SSH keys".to_string(),
                    "browser profiles".to_string(),
                    "cloud credential folders".to_string(),
                    "arbitrary host environment variables".to_string(),
                ],
            },
            preflight_commands: plan.shell_preflight(),
            command: plan.shell_command(),
            safety_notes: plan.security_notices.clone(),
            confirm_required: !plan.security_notices.is_empty(),
        }
    }
}

impl From<&AgentRunPlan> for LaunchPlanData {
    fn from(plan: &AgentRunPlan) -> Self {
        Self::from_plan(plan)
    }
}
