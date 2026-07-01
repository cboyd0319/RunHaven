use std::path::PathBuf;

use super::*;
use crate::contracts::{
    KillManagedSessionRequest, KillRunResponse, LogSnapshotRequest,
    ManagedSessionLogSnapshotRequest, RepairManagedSessionRequest, RepairRunResponse,
    StopManagedSessionRequest, StopRunResponse, WriteSessionInputRequest,
};
use crate::session_input::SessionInputRegistry;
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
fn launch_plan_for_desktop_keeps_container_stdin_open() {
    let (_cache, _cache_home) = isolated_cache();
    let workspace = tempfile::tempdir().expect("workspace");

    let plan = build_agent_run_plan(
        &request(workspace.path().to_path_buf()),
        Some("desktop-stdin-plan".to_string()),
    )
    .expect("desktop launch plan");

    assert!(plan.command.iter().any(|arg| arg == "--interactive"));
    assert!(plan.command.iter().any(|arg| arg == "--tty"));
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
    assert_eq!(response.cursor, response.captured_at);
    assert_eq!(response.entries.len(), 1);
    let entry = &response.entries[0];
    assert_eq!(entry.source, "container-stdio");
    assert_eq!(entry.timestamp, response.captured_at);
    assert_eq!(entry.text, response.text);
    assert_eq!(entry.line_count, response.returned_lines);
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
fn managed_sessions_response_is_read_only_and_secret_free() {
    let (_cache, _cache_home) = isolated_cache();
    write_active_run_payload(
        "managed-session-run",
        json!({
            "timestamp": "2026-06-30T02:00:00Z",
            "run_id": "managed-session-run",
            "profile": "codex",
            "workspace": "/sensitive/workspaces/project",
            "network": "provider",
            "status": "running",
            "container_name": "runhaven-codex-managed-session",
            "state_volume": "runhaven-codex-secret-home",
            "session": "project",
            "token": "should-not-leak"
        }),
    )
    .expect("active run payload");

    let response =
        crate::commands::sessions::get_managed_sessions().expect("managed sessions response");

    assert_eq!(response.sessions.len(), 1);
    let session = &response.sessions[0];
    assert_eq!(session.run_id, "managed-session-run");
    assert_eq!(session.agent, "codex");
    assert_eq!(session.project, "project");
    assert_eq!(session.status, "running");
    assert_eq!(session.auth_status, "api-key-broker");
    assert_eq!(session.lifecycle, "running");
    assert_eq!(session.next_action.kind, "monitor");
    assert_eq!(
        session
            .events
            .iter()
            .map(|event| event.source.as_str())
            .collect::<Vec<_>>(),
        ["run", "auth", "network"]
    );

    let encoded = serde_json::to_string(&response).expect("serialize managed sessions");
    for forbidden in [
        "/sensitive/workspaces",
        "runhaven-codex-secret-home",
        "should-not-leak",
        "write_session_input",
        "attach_session",
    ] {
        assert!(
            !encoded.contains(forbidden),
            "managed sessions response leaked forbidden value {forbidden:?}"
        );
    }
}

#[test]
fn managed_session_log_snapshot_resolves_session_id_and_stays_read_only() {
    let (_cache, _cache_home) = isolated_cache();
    write_active_run_payload(
        "managed-session-log",
        json!({
            "timestamp": "2026-06-30T03:00:00Z",
            "run_id": "managed-session-log",
            "profile": "codex",
            "workspace": "/sensitive/workspaces/project",
            "network": "provider",
            "status": "running",
            "container_name": "runhaven-codex-managed-session-log",
            "state_volume": "runhaven-codex-hidden-home",
            "session": "project",
            "token": "should-not-leak"
        }),
    )
    .expect("active run payload");

    let missing = crate::commands::sessions::managed_session_log_snapshot_response_for_test(
        ManagedSessionLogSnapshotRequest {
            session_id: "missing-session".to_string(),
            lines: Some(20),
            confirm_sensitive_output: true,
        },
        b"",
        1024,
    )
    .expect_err("unknown session rejected");

    assert!(missing.contains("managed session not found"));

    let response = crate::commands::sessions::managed_session_log_snapshot_response_for_test(
        ManagedSessionLogSnapshotRequest {
            session_id: "managed-session-log".to_string(),
            lines: Some(20),
            confirm_sensitive_output: true,
        },
        b"entry one\nentry two\n",
        1024,
    )
    .expect("managed session log snapshot");

    assert_eq!(response.session.id, "managed-session-log");
    assert_eq!(response.session.project, "project");
    assert_eq!(response.snapshot.run_id, "managed-session-log");
    assert_eq!(response.snapshot.requested_lines, 20);
    assert_eq!(response.snapshot.entries.len(), 1);
    assert_eq!(response.snapshot.entries[0].text, "entry one\nentry two\n");
    assert_eq!(response.cursor, response.snapshot.cursor);
    assert_eq!(response.entries.len(), 1);
    let session_entry = &response.entries[0];
    assert_eq!(session_entry.session_id, "managed-session-log");
    assert_eq!(session_entry.run_id, "managed-session-log");
    assert_eq!(session_entry.source, "container-stdio");
    assert_eq!(session_entry.cursor, response.snapshot.cursor);
    assert_eq!(session_entry.timestamp, response.snapshot.captured_at);
    assert_eq!(session_entry.text, "entry one\nentry two\n");
    assert_eq!(session_entry.line_count, 2);
    assert_eq!(session_entry.redaction.status, "notApplied");
    assert!(
        session_entry
            .redaction
            .message
            .contains("not automatically redacted")
    );

    let encoded = serde_json::to_string(&response).expect("serialize response");
    for forbidden in [
        "/sensitive/workspaces",
        "runhaven-codex-hidden-home",
        "should-not-leak",
        "write_session_input",
        "attach_session",
        "resize_session",
    ] {
        assert!(
            !encoded.contains(forbidden),
            "managed session log snapshot leaked forbidden value {forbidden:?}"
        );
    }
}

#[test]
fn managed_session_run_control_resolves_session_id_and_stays_confirmed() {
    let (_cache, _cache_home) = isolated_cache();
    write_active_run_payload(
        "managed-session-control",
        json!({
            "timestamp": "2026-06-30T04:00:00Z",
            "run_id": "managed-session-control",
            "profile": "codex",
            "workspace": "/sensitive/workspaces/project",
            "network": "provider",
            "status": "running",
            "container_name": "runhaven-codex-managed-session-control",
            "state_volume": "runhaven-codex-hidden-home",
            "session": "project",
            "token": "should-not-leak"
        }),
    )
    .expect("active run payload");

    let missing = crate::commands::run_control::stop_managed_session_for_test(
        StopManagedSessionRequest {
            session_id: "missing-session".to_string(),
            confirm_stop: true,
        },
        |_| unreachable!("unknown sessions must fail before stop"),
    )
    .expect_err("unknown session rejected");
    assert!(missing.contains("managed session not found"));

    let unconfirmed = crate::commands::run_control::stop_managed_session_for_test(
        StopManagedSessionRequest {
            session_id: "managed-session-control".to_string(),
            confirm_stop: false,
        },
        |_| unreachable!("unconfirmed stop must fail before mutation"),
    )
    .expect_err("confirmation required");
    assert!(unconfirmed.contains("Confirm the stop"));

    let stop = crate::commands::run_control::stop_managed_session_for_test(
        StopManagedSessionRequest {
            session_id: "managed-session-control".to_string(),
            confirm_stop: true,
        },
        |run_id| {
            assert_eq!(run_id, "managed-session-control");
            Ok(StopRunResponse {
                run_id: run_id.to_string(),
                container_name: "runhaven-codex-managed-session-control".to_string(),
                status: "stop-requested".to_string(),
            })
        },
    )
    .expect("stop managed session");
    assert_eq!(stop.session.id, "managed-session-control");
    assert_eq!(stop.result.status, "stop-requested");

    let kill = crate::commands::run_control::kill_managed_session_for_test(
        KillManagedSessionRequest {
            session_id: "managed-session-control".to_string(),
            confirm_kill: true,
        },
        |run_id| {
            assert_eq!(run_id, "managed-session-control");
            Ok(KillRunResponse {
                run_id: run_id.to_string(),
                container_name: "runhaven-codex-managed-session-control".to_string(),
                status: "kill-requested".to_string(),
            })
        },
    )
    .expect("kill managed session");
    assert_eq!(kill.session.id, "managed-session-control");
    assert_eq!(kill.result.status, "kill-requested");

    let repair = crate::commands::run_control::repair_managed_session_for_test(
        RepairManagedSessionRequest {
            session_id: "managed-session-control".to_string(),
            confirm_repair: true,
        },
        |run_id| {
            assert_eq!(run_id, "managed-session-control");
            Ok(RepairRunResponse {
                run_id: run_id.to_string(),
                container_name: "runhaven-codex-managed-session-control".to_string(),
                status: "removed".to_string(),
                marker_removed: true,
            })
        },
    )
    .expect("repair managed session");
    assert_eq!(repair.session.id, "managed-session-control");
    assert!(repair.result.marker_removed);

    let encoded = serde_json::to_string(&repair).expect("serialize repair response");
    for forbidden in [
        "/sensitive/workspaces",
        "runhaven-codex-hidden-home",
        "should-not-leak",
        "write_session_input",
        "attach_session",
    ] {
        assert!(
            !encoded.contains(forbidden),
            "managed session run-control leaked forbidden value {forbidden:?}"
        );
    }
}

#[test]
fn managed_session_input_contract_rejects_before_write() {
    let (_cache, _cache_home) = isolated_cache();
    let registry = SessionInputRegistry::default();
    write_active_run_payload(
        "managed-session-input",
        json!({
            "timestamp": "2026-06-30T05:00:00Z",
            "run_id": "managed-session-input",
            "profile": "codex",
            "workspace": "/sensitive/workspaces/project",
            "network": "provider",
            "status": "running",
            "container_name": "runhaven-codex-managed-session-input",
            "state_volume": "runhaven-codex-hidden-home",
            "session": "project",
            "token": "should-not-leak"
        }),
    )
    .expect("active run payload");

    let missing = crate::commands::sessions::write_session_input_for_test(
        WriteSessionInputRequest {
            session_id: "missing-session".to_string(),
            input: "hello".to_string(),
            confirm_send: true,
            client_target: "session:missing-session".to_string(),
        },
        &registry,
    )
    .expect_err("unknown session rejected");
    assert!(missing.contains("managed session not found"));

    let oversized_id = crate::commands::sessions::write_session_input_for_test(
        WriteSessionInputRequest {
            session_id: "s".repeat(129),
            input: "hello".to_string(),
            confirm_send: true,
            client_target: format!("session:{}", "s".repeat(129)),
        },
        &registry,
    )
    .expect_err("oversized session id rejected");
    assert!(oversized_id.contains("session id is too long"));

    let unconfirmed = crate::commands::sessions::write_session_input_for_test(
        WriteSessionInputRequest {
            session_id: "managed-session-input".to_string(),
            input: "hello".to_string(),
            confirm_send: false,
            client_target: "session:managed-session-input".to_string(),
        },
        &registry,
    )
    .expect_err("confirmation required");
    assert!(unconfirmed.contains("Confirm the send"));

    for (label, input) in [
        ("empty input", ""),
        ("oversized input", &"a".repeat(8 * 1024 + 1)),
        ("nul input", "hello\0there"),
        ("escape input", "hello\x1b[31mthere"),
    ] {
        let error = crate::commands::sessions::write_session_input_for_test(
            WriteSessionInputRequest {
                session_id: "managed-session-input".to_string(),
                input: input.to_string(),
                confirm_send: true,
                client_target: "session:managed-session-input".to_string(),
            },
            &registry,
        )
        .expect_err(label);
        assert!(
            error.contains("input"),
            "{label} returned unexpected error: {error}"
        );
    }

    let stale_target = crate::commands::sessions::write_session_input_for_test(
        WriteSessionInputRequest {
            session_id: "managed-session-input".to_string(),
            input: "hello".to_string(),
            confirm_send: true,
            client_target: "session:older-session".to_string(),
        },
        &registry,
    )
    .expect_err("stale target rejected");
    assert!(stale_target.contains("Selected session changed"));
}

#[test]
fn managed_session_input_contract_accepts_only_ready_session() {
    let (_cache, _cache_home) = isolated_cache();
    let registry = SessionInputRegistry::default();
    write_active_run_payload(
        "managed-session-ready-input",
        json!({
            "timestamp": "2026-06-30T06:00:00Z",
            "run_id": "managed-session-ready-input",
            "profile": "codex",
            "workspace": "/sensitive/workspaces/ready",
            "network": "provider",
            "status": "running",
            "container_name": "runhaven-codex-ready-input",
            "state_volume": "runhaven-codex-hidden-home",
            "session": "ready",
            "token": "should-not-leak"
        }),
    )
    .expect("ready active run payload");
    write_active_run_payload(
        "managed-session-auth-input",
        json!({
            "timestamp": "2026-06-30T07:00:00Z",
            "run_id": "managed-session-auth-input",
            "profile": "antigravity",
            "workspace": "/sensitive/workspaces/auth",
            "network": "provider",
            "status": "running",
            "container_name": "runhaven-antigravity-auth-input",
            "state_volume": "runhaven-antigravity-hidden-home",
            "session": "auth",
            "token": "should-not-leak"
        }),
    )
    .expect("auth active run payload");

    let auth_blocked = crate::commands::sessions::write_session_input_for_test(
        WriteSessionInputRequest {
            session_id: "managed-session-auth-input".to_string(),
            input: "hello".to_string(),
            confirm_send: true,
            client_target: "session:managed-session-auth-input".to_string(),
        },
        &registry,
    )
    .expect_err("auth attention rejected");
    assert!(auth_blocked.contains("auth"));

    registry
        .register_writer(
            "managed-session-ready-input",
            "managed-session-ready-input",
            |input| {
                assert_eq!(String::from_utf8_lossy(input), "hello\nthere");
                Ok(input.len())
            },
        )
        .expect("register ready session writer");
    let response = crate::commands::sessions::write_session_input_for_test(
        WriteSessionInputRequest {
            session_id: "managed-session-ready-input".to_string(),
            input: "hello\nthere".to_string(),
            confirm_send: true,
            client_target: "session:managed-session-ready-input".to_string(),
        },
        &registry,
    )
    .expect("ready session input");

    assert!(response.accepted);
    assert_eq!(response.accepted_bytes, "hello\nthere".len());
    assert_eq!(response.session.id, "managed-session-ready-input");
    assert_eq!(response.session.run_id, "managed-session-ready-input");
    assert_eq!(response.status, "accepted");

    let encoded = serde_json::to_string(&response).expect("serialize response");
    for forbidden in [
        "/sensitive/workspaces",
        "runhaven-codex-hidden-home",
        "should-not-leak",
        "attach_session",
        "resize_session",
        "filesystem_rpc",
    ] {
        assert!(
            !encoded.contains(forbidden),
            "managed session input response leaked forbidden value {forbidden:?}"
        );
    }
}

#[test]
fn managed_session_input_requires_backend_owned_transport() {
    let (_cache, _cache_home) = isolated_cache();
    write_active_run_payload(
        "managed-session-transport-a",
        json!({
            "timestamp": "2026-06-30T08:00:00Z",
            "run_id": "managed-session-transport-a",
            "profile": "codex",
            "workspace": "/sensitive/workspaces/a",
            "network": "provider",
            "status": "running",
            "container_name": "runhaven-codex-transport-a",
            "state_volume": "runhaven-codex-hidden-home-a",
            "session": "a"
        }),
    )
    .expect("active run payload a");
    write_active_run_payload(
        "managed-session-transport-b",
        json!({
            "timestamp": "2026-06-30T08:01:00Z",
            "run_id": "managed-session-transport-b",
            "profile": "codex",
            "workspace": "/sensitive/workspaces/b",
            "network": "provider",
            "status": "running",
            "container_name": "runhaven-codex-transport-b",
            "state_volume": "runhaven-codex-hidden-home-b",
            "session": "b"
        }),
    )
    .expect("active run payload b");

    let registry = SessionInputRegistry::default();
    let missing = crate::commands::sessions::write_session_input_for_test(
        WriteSessionInputRequest {
            session_id: "managed-session-transport-a".to_string(),
            input: "hello".to_string(),
            confirm_send: true,
            client_target: "session:managed-session-transport-a".to_string(),
        },
        &registry,
    )
    .expect_err("unregistered transport rejected");
    assert!(missing.contains("session input transport not found"));

    registry
        .register_writer("managed-session-transport-a", "older-run", |_| Ok(1))
        .expect("register mismatched transport");
    let mismatch = crate::commands::sessions::write_session_input_for_test(
        WriteSessionInputRequest {
            session_id: "managed-session-transport-a".to_string(),
            input: "hello".to_string(),
            confirm_send: true,
            client_target: "session:managed-session-transport-a".to_string(),
        },
        &registry,
    )
    .expect_err("mismatched run rejected");
    assert!(mismatch.contains("session input transport no longer matches"));

    let session_a_writes = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let session_b_writes = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let session_a_capture = std::sync::Arc::clone(&session_a_writes);
    registry
        .register_writer(
            "managed-session-transport-a",
            "managed-session-transport-a",
            move |input| {
                session_a_capture
                    .lock()
                    .expect("session a writes lock")
                    .push(String::from_utf8_lossy(input).to_string());
                Ok(input.len())
            },
        )
        .expect("register session a transport");
    let session_b_capture = std::sync::Arc::clone(&session_b_writes);
    registry
        .register_writer(
            "managed-session-transport-b",
            "managed-session-transport-b",
            move |input| {
                session_b_capture
                    .lock()
                    .expect("session b writes lock")
                    .push(String::from_utf8_lossy(input).to_string());
                Ok(input.len())
            },
        )
        .expect("register session b transport");

    let response = crate::commands::sessions::write_session_input_for_test(
        WriteSessionInputRequest {
            session_id: "managed-session-transport-a".to_string(),
            input: "message for a".to_string(),
            confirm_send: true,
            client_target: "session:managed-session-transport-a".to_string(),
        },
        &registry,
    )
    .expect("session a input accepted");

    assert!(response.accepted);
    assert_eq!(response.accepted_bytes, "message for a".len());
    assert_eq!(
        *session_a_writes.lock().expect("session a writes lock"),
        vec!["message for a".to_string()]
    );
    assert!(
        session_b_writes
            .lock()
            .expect("session b writes lock")
            .is_empty(),
        "session b must not receive session a input"
    );
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

#[test]
#[ignore = "requires RUNHAVEN_LIVE_MANAGED_SESSION_RUN_ID and a live RunHaven container"]
fn managed_session_live_smoke_reads_output_and_stops_run() {
    let run_id =
        std::env::var("RUNHAVEN_LIVE_MANAGED_SESSION_RUN_ID").expect("live managed session run id");
    let expected =
        std::env::var("RUNHAVEN_LIVE_MANAGED_SESSION_EXPECT").expect("expected log marker");
    let forbidden_workspace = std::env::var("RUNHAVEN_LIVE_MANAGED_SESSION_FORBIDDEN").ok();

    let sessions = crate::commands::sessions::get_managed_sessions().expect("managed sessions");
    let session = sessions
        .sessions
        .iter()
        .find(|session| session.run_id == run_id)
        .expect("live managed session")
        .clone();
    let session_id = session.id.clone();

    let encoded_session = serde_json::to_string(&session).expect("serialize managed session");
    if let Some(workspace) = forbidden_workspace.as_deref() {
        assert!(
            !encoded_session.contains(workspace),
            "managed session leaked workspace path"
        );
    }
    assert!(!encoded_session.contains("write_session_input"));
    assert!(!encoded_session.contains("attach_session"));

    let log = crate::commands::sessions::get_managed_session_log_snapshot(
        ManagedSessionLogSnapshotRequest {
            session_id: session_id.clone(),
            lines: Some(20),
            confirm_sensitive_output: true,
        },
    )
    .expect("managed session log snapshot");
    assert_eq!(log.session.id, session_id);
    assert_eq!(log.snapshot.run_id, run_id);
    assert!(log.snapshot.text.contains(&expected));

    let stopped = crate::commands::run_control::stop_managed_session(StopManagedSessionRequest {
        session_id,
        confirm_stop: true,
    })
    .expect("stop managed session");
    assert_eq!(stopped.result.run_id, run_id);
    assert_eq!(stopped.result.status, "stop-requested");
}
