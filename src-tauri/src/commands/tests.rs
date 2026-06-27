use std::path::PathBuf;

use super::*;
use crate::contracts::LogSnapshotRequest;
use runhaven_core::runtime::active::write_active_run_payload;
use runhaven_core::support::paths::{CacheRootOverride, override_cache_root_for_tests};
use serde_json::json;

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

#[test]
fn lists_agent_profiles_without_secrets() {
    let agents = agent_summaries();
    assert!(agents.iter().any(|agent| agent.name == "codex"));
    assert!(agents.iter().all(|agent| !agent.image.is_empty()));
}

#[test]
fn build_plan_response_uses_existing_runhaven_planner() {
    let (_cache, _cache_home) = isolated_cache();
    let workspace = tempfile::tempdir().expect("workspace");
    let response = build_plan_response(request(workspace.path().to_path_buf())).expect("plan");
    assert_eq!(response.profile, "codex");
    assert_eq!(response.network_mode, "provider");
    assert!(response.egress_summary.contains("provider allowlist"));
    assert_eq!(response.warnings.len(), 0);
}

#[test]
fn build_plan_response_warns_for_supported_advanced_choices() {
    let (_cache, _cache_home) = isolated_cache();
    let workspace = tempfile::tempdir().expect("workspace");
    let mut request = request(workspace.path().to_path_buf());
    request.network_mode = "internet".to_string();
    request.allow_sensitive_workspace = true;
    request.env_names = vec!["OPENAI_API_KEY".to_string()];
    request.image = Some("example/custom:1.0.0".to_string());
    let response = build_plan_response(request).expect("plan");
    let codes = response
        .warnings
        .into_iter()
        .map(|warning| warning.code)
        .collect::<Vec<_>>();
    assert_eq!(
        codes,
        vec![
            "full-internet",
            "sensitive-workspace",
            "environment",
            "custom-image"
        ]
    );
}

#[test]
fn build_plan_response_warns_for_active_runs_and_material_memory() {
    let (_cache, _cache_home) = isolated_cache();
    write_active_run("active-warning-run");
    let workspace = tempfile::tempdir().expect("workspace");

    let response = build_plan_response(request(workspace.path().to_path_buf())).expect("plan");
    let codes = response
        .warnings
        .into_iter()
        .map(|warning| warning.code)
        .collect::<Vec<_>>();

    assert!(codes.contains(&"active-runs".to_string()));
    assert!(codes.contains(&"resource-memory".to_string()));
}

#[test]
fn build_plan_response_rejects_invalid_workspace() {
    let mut request = request(PathBuf::from("/definitely/not/a/runhaven/workspace"));
    request.network_mode = "internal".to_string();
    assert!(build_plan_response(request).is_err());
}

#[test]
fn build_plan_response_rejects_oversized_ipc_fields() {
    let (_cache, _cache_home) = isolated_cache();
    let workspace = tempfile::tempdir().expect("workspace");
    let mut plan_request = request(workspace.path().to_path_buf());
    plan_request.workspace_path = "x".repeat(4097);

    let error = build_plan_response(plan_request).expect_err("workspace path cap");

    assert!(error.contains("workspace path"));

    let mut plan_request = request(workspace.path().to_path_buf());
    plan_request.provider_hosts = vec!["api.openai.com".to_string(); 51];

    let error = build_plan_response(plan_request).expect_err("provider host count cap");

    assert!(error.contains("provider hosts"));

    let mut plan_request = request(workspace.path().to_path_buf());
    plan_request.env_names = vec!["OPENAI_API_KEY".to_string(); 51];

    let error = build_plan_response(plan_request).expect_err("env name count cap");

    assert!(error.contains("environment variable names"));
}

#[test]
fn launch_run_rejects_oversized_warning_confirmations() {
    let workspace = tempfile::tempdir().expect("workspace");
    let request = LaunchRunRequest {
        plan: request(workspace.path().to_path_buf()),
        confirm_launch: true,
        confirmed_warnings: vec!["full-internet".to_string(); 17],
    };

    let error = validate_launch_confirmation(&request).expect_err("warning count cap");

    assert!(error.contains("confirmed warning codes"));
}

#[test]
fn launch_run_requires_explicit_confirmation() {
    let (_cache, _cache_home) = isolated_cache();
    let workspace = tempfile::tempdir().expect("workspace");
    let request = LaunchRunRequest {
        plan: request(workspace.path().to_path_buf()),
        confirm_launch: false,
        confirmed_warnings: Vec::new(),
    };

    let error = validate_launch_confirmation(&request).expect_err("confirmation required");

    assert!(error.contains("Confirm the launch"));
}

#[test]
fn launch_run_requires_each_warning_confirmation() {
    let (_cache, _cache_home) = isolated_cache();
    let workspace = tempfile::tempdir().expect("workspace");
    let mut plan = request(workspace.path().to_path_buf());
    plan.network_mode = "internet".to_string();
    let request = LaunchRunRequest {
        plan,
        confirm_launch: true,
        confirmed_warnings: Vec::new(),
    };

    let error = validate_launch_confirmation(&request).expect_err("warning required");

    assert!(error.contains("full-internet"));
}

#[test]
fn launch_run_requires_active_run_warning_confirmation() {
    let (_cache, _cache_home) = isolated_cache();
    write_active_run("active-launch-run");
    let workspace = tempfile::tempdir().expect("workspace");
    let request = LaunchRunRequest {
        plan: request(workspace.path().to_path_buf()),
        confirm_launch: true,
        confirmed_warnings: Vec::new(),
    };

    let error = validate_launch_confirmation(&request).expect_err("active warning required");

    assert!(error.contains("active-runs"));
}

#[test]
fn launch_run_response_uses_reserved_run_id_without_starting() {
    let (_cache, _cache_home) = isolated_cache();
    let workspace = tempfile::tempdir().expect("workspace");
    let response = build_launch_response(
        request(workspace.path().to_path_buf()),
        "runhaven-test-run".to_string(),
    )
    .expect("launch response");

    assert_eq!(response.run_id, "runhaven-test-run");
    assert_eq!(response.status, "started");
    assert_eq!(response.profile, "codex");
    assert_eq!(response.snapshot.run_id, "runhaven-test-run");
    assert_eq!(response.snapshot.status, "started");
    assert!(!response.snapshot.container_name.is_empty());
}

#[test]
fn launch_run_blocks_missing_bundled_image() {
    let (_cache, _cache_home) = isolated_cache();
    let workspace = tempfile::tempdir().expect("workspace");
    let request = request(workspace.path().to_path_buf());

    let error = image_readiness_error(
        &request,
        false,
        "missing",
        &request.agent,
        &request.agent,
        Some("runhaven image rebuild codex"),
    )
    .expect("image readiness error");

    assert!(error.contains("Image for codex is missing"));
    assert!(error.contains("runhaven image rebuild codex"));
}

#[test]
fn launch_run_allows_custom_image_without_bundled_image_check() {
    let (_cache, _cache_home) = isolated_cache();
    let workspace = tempfile::tempdir().expect("workspace");
    let mut request = request(workspace.path().to_path_buf());
    request.image = Some("example/custom:1.0.0".to_string());

    assert!(image_readiness_error(&request, false, "missing", "codex", "codex", None).is_none());
}

#[test]
fn run_status_response_maps_sanitized_payload_without_raw_fields() {
    let response = crate::commands::run_status::run_status_response(json!({
        "active_run": {
            "timestamp": "2026-06-16T00:00:00Z",
            "run_id": "runhaven-status-test",
            "profile": "codex",
            "workspace": "/tmp/runhaven-status",
            "network": "provider",
            "status": "running",
            "container_name": "runhaven-codex-status-test",
            "state_volume": "runhaven-codex-status-home",
            "session": "default",
            "host_pid": 12345
        },
        "container": {
            "state": "running",
            "image": "runhaven/codex:0.1.0",
            "started_at": "2026-06-16T00:00:10Z",
            "resources": {
                "cpus": 2.0,
                "memory_in_bytes": 2147483648u64
            },
            "networks": [{
                "network": "runhaven-test",
                "hostname": "runhaven-codex-status-test",
                "ipv4_address": "192.168.64.20/24",
                "ipv4_gateway": "192.168.64.1"
            }],
            "environment": ["OPENAI_API_KEY=fake-secret"],
            "mounts": [{"source": "/Users/example/private"}],
            "arguments": ["codex", "--secret-flag"]
        }
    }))
    .expect("status response");

    assert_eq!(response.run.run_id, "runhaven-status-test");
    assert_eq!(response.container.state, "running");
    assert_eq!(
        response.container.resources.memory_bytes,
        Some(2_147_483_648)
    );
    assert_eq!(
        response.container.networks[0].network.as_deref(),
        Some("runhaven-test")
    );
    let serialized = serde_json::to_string(&response).expect("json");
    assert!(!serialized.contains("fake-secret"));
    assert!(!serialized.contains("/Users/example"));
    assert!(!serialized.contains("secret-flag"));
}

#[test]
fn log_snapshot_requires_sensitive_output_confirmation() {
    let request = LogSnapshotRequest {
        run_id: "runhaven-log-test".to_string(),
        lines: Some(200),
        confirm_sensitive_output: false,
    };

    let error = crate::commands::log_snapshot::validated_lines(&request)
        .expect_err("confirmation required");

    assert!(error.contains("Confirm raw log viewing"));
}

#[test]
fn log_snapshot_response_caps_lines_and_bytes() {
    let too_many_lines = LogSnapshotRequest {
        run_id: "runhaven-log-test".to_string(),
        lines: Some(501),
        confirm_sensitive_output: true,
    };

    let error = crate::commands::log_snapshot::validated_lines(&too_many_lines)
        .expect_err("line cap required");

    assert!(error.contains("500"));

    let response = crate::commands::log_snapshot::log_snapshot_response(
        "runhaven-log-test",
        200,
        b"first line\nsecond line\nthird line\n",
        18,
    )
    .expect("snapshot response");

    assert_eq!(response.run_id, "runhaven-log-test");
    assert_eq!(response.requested_lines, 200);
    assert!(response.truncated);
    assert!(response.text.len() <= 18);
    assert!(
        response
            .warnings
            .iter()
            .any(|warning| warning.contains("Raw"))
    );
}

#[test]
fn log_snapshot_rejects_invalid_run_id_before_container_access() {
    let request = LogSnapshotRequest {
        run_id: "../not-a-run".to_string(),
        lines: Some(200),
        confirm_sensitive_output: true,
    };

    let error =
        crate::commands::log_snapshot::get_log_snapshot(request).expect_err("invalid run id");

    assert!(error.contains("invalid run id"));
}

#[test]
fn log_snapshot_rejects_non_runhaven_container_names() {
    let (_cache, _cache_home) = isolated_cache();
    write_active_run_payload(
        "bad-container-log-run",
        json!({
            "timestamp": "2026-06-16T00:00:00Z",
            "run_id": "bad-container-log-run",
            "profile": "codex",
            "workspace": "/tmp/runhaven-active",
            "network": "provider",
            "status": "running",
            "container_name": "other-project-container",
            "state_volume": "runhaven-codex-active-home",
            "session": "default"
        }),
    )
    .expect("active run payload");
    let request = LogSnapshotRequest {
        run_id: "bad-container-log-run".to_string(),
        lines: Some(200),
        confirm_sensitive_output: true,
    };

    let error =
        crate::commands::log_snapshot::get_log_snapshot(request).expect_err("container rejected");

    assert!(error.contains("RunHaven-owned"));
}

#[test]
#[ignore = "requires RUNHAVEN_LIVE_LOG_RUN_ID and a live RunHaven container"]
fn log_snapshot_reads_live_active_run() {
    let run_id = std::env::var("RUNHAVEN_LIVE_LOG_RUN_ID").expect("live run id");
    let expected = std::env::var("RUNHAVEN_LIVE_LOG_EXPECT").expect("expected log marker");
    let expect_truncated = std::env::var("RUNHAVEN_LIVE_LOG_EXPECT_TRUNCATED")
        .ok()
        .as_deref()
        == Some("1");

    let response = crate::commands::log_snapshot::get_log_snapshot(LogSnapshotRequest {
        run_id,
        lines: Some(20),
        confirm_sensitive_output: true,
    })
    .expect("live log snapshot");

    assert!(response.text.contains(&expected));
    if expect_truncated {
        assert!(response.truncated);
    }
}
