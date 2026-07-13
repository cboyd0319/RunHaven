use super::*;
use crate::runtime::profiles::get_profile;

#[test]
fn leading_agent_flags_use_the_profile_command() {
    let workspace = tempfile::tempdir().expect("workspace");
    let plan = build_run_plan(RunOptions {
        profile: get_profile("shell").expect("profile"),
        workspace: workspace.path().to_path_buf(),
        agent_args: vec!["-lc".to_string(), "printf changed > seed.txt".to_string()],
        image: None,
        cpus: "4".to_string(),
        memory: "4g".to_string(),
        network: NetworkMode::Internal,
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
    .expect("plan");

    assert!(plan.command.ends_with(&[
        "runhaven/base:0.7.0".to_string(),
        "/bin/bash".to_string(),
        "-lc".to_string(),
        "printf changed > seed.txt".to_string(),
    ]));
}
