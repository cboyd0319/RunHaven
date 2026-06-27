use super::*;
use std::fs;

use runhaven_core::support::paths::{
    active_run_path, override_cache_root_for_tests, runs_log_path,
};

#[test]
fn split_agent_args_after_separator() {
    let args = ["run", "shell", "--", "echo", "--flag"]
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
    assert_eq!(
        split_agent_args(&args),
        (
            vec!["run".to_string(), "shell".to_string()],
            vec!["echo".to_string(), "--flag".to_string()]
        )
    );
}

#[test]
fn standard_run_launch_error_removes_active_marker_and_writes_record() {
    let cache = tempfile::tempdir().expect("cache");
    let _cache_home = override_cache_root_for_tests(cache.path());

    let workspace = tempfile::tempdir().expect("workspace");
    let run_id = "standard-launch-error";
    let mut plan = build_run_plan(RunOptions {
        profile: get_profile("shell").expect("profile"),
        workspace: workspace.path().to_path_buf(),
        agent_args: vec!["/bin/true".to_string()],
        image: None,
        cpus: "4".to_string(),
        memory: "4g".to_string(),
        network: NetworkMode::Internet,
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
        api_key_broker_env: None,
        worktree: None,
        run_id: Some(run_id.to_string()),
    })
    .expect("plan");
    plan.command[0] = "__runhaven_missing_container_binary__".to_string();

    let error = run_standard_agent(&plan).expect_err("launch should fail");
    assert!(
        error
            .to_string()
            .contains("__runhaven_missing_container_binary__")
    );
    assert!(!active_run_path(run_id).expect("active path").exists());

    let log = fs::read_to_string(runs_log_path()).expect("run log");
    let record: serde_json::Value =
        serde_json::from_str(log.lines().next().expect("one record")).expect("json record");
    assert_eq!(record["run_id"], run_id);
    assert_eq!(record["status"], "failed");
    assert_eq!(record["return_code"], 1);
}
