use super::*;
use crate::contracts::{
    KillManagedSessionRequest, KillRunResponse, LogSnapshotRequest,
    ManagedSessionLogSnapshotRequest, RepairManagedSessionRequest, RepairRunResponse,
    StopManagedSessionRequest, StopRunResponse, WriteSessionInputRequest,
    WriteSessionInputResponse,
};
use crate::session_input::SessionInputRegistry;
use crate::session_output::SessionOutputRegistry;
use runhaven_core::runtime::active::write_active_run_payload;
use runhaven_core::support::paths::{CacheRootOverride, override_cache_root_for_tests};
use serde_json::json;
use std::path::PathBuf;

mod image;
mod live;
mod log_snapshot;
mod plan;
mod run_control;
mod run_status;
mod session_input;
mod sessions;

fn isolated_cache() -> (tempfile::TempDir, CacheRootOverride) {
    let cache = tempfile::tempdir().expect("cache");
    let cache_home = override_cache_root_for_tests(cache.path());
    (cache, cache_home)
}

fn request(workspace: PathBuf) -> RunPlanRequest {
    RunPlanRequest {
        agent: "codex".to_string(),
        workspace_path: workspace.display().to_string(),
        network_mode: "provider".to_string(),
        workspace_scope: "current".to_string(),
        session_name: None,
        read_only_workspace: false,
        cpus: "4".to_string(),
        memory: "4g".to_string(),
        provider_hosts: Vec::new(),
        env_names: Vec::new(),
        api_key_broker_env: None,
        image: None,
        allow_sensitive_workspace: false,
        allow_root_user: false,
        user: "agent".to_string(),
    }
}

fn shell_request(workspace: PathBuf, session: String) -> RunPlanRequest {
    RunPlanRequest {
        agent: "shell".to_string(),
        workspace_path: workspace.display().to_string(),
        network_mode: "internal".to_string(),
        workspace_scope: "current".to_string(),
        session_name: Some(session),
        read_only_workspace: true,
        cpus: "1".to_string(),
        memory: "512m".to_string(),
        provider_hosts: Vec::new(),
        env_names: Vec::new(),
        api_key_broker_env: None,
        image: None,
        allow_sensitive_workspace: false,
        allow_root_user: false,
        user: "agent".to_string(),
    }
}

fn write_active_run(run_id: &str) {
    write_active_run_payload(
        run_id,
        json!({
            "timestamp": "2026-06-16T00:00:00Z",
            "run_id": run_id,
            "profile": "codex",
            "workspace": "/tmp/runhaven-active",
            "network": "provider",
            "status": "running",
            "container_name": format!("runhaven-codex-{run_id}"),
            "state_volume": "runhaven-codex-active-home",
            "session": "default"
        }),
    )
    .expect("active run payload");
}
