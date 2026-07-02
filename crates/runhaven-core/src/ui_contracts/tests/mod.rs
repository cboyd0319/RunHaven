use super::*;
use crate::runtime::plans::{
    AgentRunPlan, AuthScope, NetworkMode, RunOptions, WorkspaceScope, build_run_plan,
};
use crate::runtime::profiles::{get_profile, profiles};

fn test_plan(network: NetworkMode) -> AgentRunPlan {
    let workspace = tempfile::tempdir().expect("workspace");
    build_run_plan(RunOptions {
        profile: get_profile("shell").expect("profile"),
        workspace: workspace.path().to_path_buf(),
        agent_args: vec![
            "/bin/bash".to_string(),
            "-lc".to_string(),
            "echo hello".to_string(),
        ],
        image: None,
        cpus: "4".to_string(),
        memory: "4g".to_string(),
        network,
        workspace_scope: WorkspaceScope::Current,
        session: None,
        auth_scope: AuthScope::Project,
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
        run_id: None,
    })
    .expect("plan")
}

mod active_runs;
mod agent_catalog;
mod diagnostics;
mod helpers;
mod launch;
mod payload;
mod review;
mod session_cleanup;
mod session_recovery;
mod sessions;
