use std::path::PathBuf;

use runhaven_core::runtime::plans::AgentRunPlan;
use runhaven_core::runtime::plans::AuthScope;
use runhaven_core::runtime::plans::NetworkMode;
use runhaven_core::runtime::plans::WorkspaceScope;
use runhaven_core::ui_contracts::AgentCatalogItemData;
use runhaven_core::ui_contracts::LaunchBoundaryData;
use runhaven_core::ui_contracts::LaunchNetworkData;
use runhaven_core::ui_contracts::LaunchPlanData;

use super::AgentLaunchPreview;
use super::PreparedLaunch;

pub(crate) fn confirm_required_preview_for_tests() -> AgentLaunchPreview {
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
        claude_auth_source: None,
        security_notices: vec!["This plan uses a less safe launch option.".to_string()],
    };

    AgentLaunchPreview {
        agent,
        plan: Ok(PreparedLaunch::from_parts_for_tests(data, executable)),
    }
}
