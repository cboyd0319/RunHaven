use super::*;
use crate::runtime::profiles::get_profile;

fn isolated_resource_plan(
    workspace: &std::path::Path,
    network: NetworkMode,
    auth_scope: AuthScope,
    run_id: Option<&str>,
) -> AgentRunPlan {
    build_run_plan(RunOptions {
        profile: get_profile("codex").expect("profile"),
        workspace: workspace.to_path_buf(),
        agent_args: Vec::new(),
        image: None,
        cpus: "4".to_string(),
        memory: "4g".to_string(),
        network,
        workspace_scope: WorkspaceScope::Current,
        session: None,
        auth_scope,
        read_only_workspace: false,
        ssh: false,
        env: Vec::new(),
        user: "agent".to_string(),
        interactive: false,
        tty: false,
        allow_sensitive_workspace: false,
        allow_root_user: false,
        provider_hosts: Vec::new(),
        claude_auth_source: None,
        worktree: None,
        run_id: run_id.map(str::to_string),
    })
    .expect("plan")
}

#[test]
fn runtime_resources_are_distinct_across_run_ids() {
    let workspace = tempfile::tempdir().expect("workspace");

    for network in [NetworkMode::Internal, NetworkMode::Provider] {
        let alpha = isolated_resource_plan(
            workspace.path(),
            network,
            AuthScope::Project,
            Some("run-alpha"),
        );
        let beta = isolated_resource_plan(
            workspace.path(),
            network,
            AuthScope::Project,
            Some("run-beta"),
        );

        assert_ne!(alpha.container_name, beta.container_name);
        assert_ne!(alpha.network_name, beta.network_name);
    }
}

#[test]
fn runtime_resources_are_distinct_across_auth_scopes_without_run_ids() {
    let workspace = tempfile::tempdir().expect("workspace");

    for network in [NetworkMode::Internal, NetworkMode::Provider] {
        let agent = isolated_resource_plan(workspace.path(), network, AuthScope::Agent, None);
        let project = isolated_resource_plan(workspace.path(), network, AuthScope::Project, None);

        assert_ne!(agent.container_name, project.container_name);
        assert_ne!(agent.network_name, project.network_name);
    }
}
