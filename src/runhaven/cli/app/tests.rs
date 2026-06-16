use super::*;
use std::ffi::OsString;
use std::fs;
use std::sync::Mutex;

use crate::paths::{active_run_path, runs_log_path};

static ENV_LOCK: Mutex<()> = Mutex::new(());

struct CacheHomeOverride {
    previous: Option<OsString>,
}

impl CacheHomeOverride {
    fn set(path: &std::path::Path) -> Self {
        let previous = std::env::var_os("RUNHAVEN_CACHE_HOME");
        // SAFETY: tests using this helper hold ENV_LOCK while mutating the
        // process environment, and Drop restores the previous value.
        unsafe {
            std::env::set_var("RUNHAVEN_CACHE_HOME", path);
        }
        Self { previous }
    }
}

impl Drop for CacheHomeOverride {
    fn drop(&mut self) {
        // SAFETY: caller holds ENV_LOCK until this guard is dropped.
        unsafe {
            if let Some(value) = &self.previous {
                std::env::set_var("RUNHAVEN_CACHE_HOME", value);
            } else {
                std::env::remove_var("RUNHAVEN_CACHE_HOME");
            }
        }
    }
}

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
    let _guard = ENV_LOCK.lock().expect("env lock");
    let cache = tempfile::tempdir().expect("cache");
    let _cache_home = CacheHomeOverride::set(cache.path());

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
        read_only_workspace: false,
        ssh: false,
        env: Vec::new(),
        user: "agent".to_string(),
        interactive: false,
        tty: false,
        allow_sensitive_workspace: false,
        allow_root_user: false,
        provider_hosts: Vec::new(),
        codex_api_key_broker_env: None,
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
