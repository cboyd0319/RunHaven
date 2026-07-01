use super::*;
use crate::tui::runhaven::protocol::ClientRequest;
use runhaven_core::support::paths::{
    ensure_private_parent, override_cache_root_for_tests, runs_log_path,
};
use std::io::Write;

#[path = "service_launch_tests.rs"]
mod launch;

#[test]
fn run_history_payload_omits_workspace_paths() {
    let cache = tempfile::tempdir().expect("cache");
    let _cache_home = override_cache_root_for_tests(cache.path());
    ensure_private_parent(&runs_log_path()).expect("runs log parent");
    let mut file = std::fs::File::create(runs_log_path()).expect("runs log file");
    writeln!(
        file,
        "{}",
        serde_json::json!({
            "timestamp": "2026-06-30T01:00:00Z",
            "started_at": "2026-06-30T00:00:00Z",
            "finished_at": "2026-06-30T01:00:00Z",
            "run_id": "run-\u{1b}123",
            "profile": "codex",
            "workspace": "/Users/c/secret/project",
            "workspace_scope": "current",
            "state_volume": "runhaven-codex-shared-home",
            "session": "none",
            "network": "provider",
            "status": "succeeded",
            "return_code": 0,
            "provider_policy": {"allowed": 3, "denied": 1},
            "auth_broker": {"allowed": 2, "denied": 0},
            "cleanup": {"provider_network": "removed"},
            "git": {"available": "false", "reason": "not-a-git-worktree"}
        })
    )
    .expect("write run record");

    let history = RunHavenTuiService::new()
        .run_history_payload(10)
        .expect("run history");
    let encoded = serde_json::to_string(&history).expect("serialize history");

    assert_eq!(history.runs.len(), 1);
    assert_eq!(history.runs[0].run_id, "run-123");
    assert_eq!(history.runs[0].return_code, Some(0));
    assert_eq!(history.runs[0].provider_denied, 1);
    assert!(!encoded.contains("/Users/c/secret/project"));
    assert!(!encoded.contains("\\u001b"));
}

#[test]
fn active_runs_payload_omits_workspace_paths() {
    use runhaven_core::runtime::active::write_active_run_payload;
    use runhaven_core::support::paths::override_cache_root_for_tests;

    let cache = tempfile::tempdir().expect("cache");
    let _cache_home = override_cache_root_for_tests(cache.path());
    write_active_run_payload(
        "run-123",
        serde_json::json!({
            "timestamp": "2026-06-29T00:00:00Z",
            "run_id": "run-123",
            "profile": "codex",
            "workspace": "/Users/c/secret/project",
            "network": "provider",
            "status": "running",
            "container_name": "runhaven-codex-project-run",
            "state_volume": "runhaven-codex-shared-home",
            "session": "none"
        }),
    )
    .expect("active marker");

    let payload = RunHavenTuiService::new().active_runs_payload();

    assert_eq!(payload.runs.len(), 1);
    assert_eq!(payload.runs[0].run_id, "run-123");
    let serialized = serde_json::to_string(&payload).expect("serialize");
    assert!(!serialized.contains("/Users/c/secret/project"));
}

#[test]
fn diagnostics_payload_maps_secret_free_log_metadata() {
    use runhaven_core::support::paths::auth_broker_log_path;
    use runhaven_core::support::paths::egress_policy_log_path;
    use runhaven_core::support::paths::ensure_private_parent;
    use runhaven_core::support::paths::override_cache_root_for_tests;
    use std::io::Write;

    let cache = tempfile::tempdir().expect("cache");
    let _cache_home = override_cache_root_for_tests(cache.path());
    ensure_private_parent(&egress_policy_log_path()).expect("egress parent");
    ensure_private_parent(&auth_broker_log_path()).expect("auth parent");
    {
        let mut file = std::fs::File::create(egress_policy_log_path()).expect("egress log file");
        writeln!(
            file,
            "{}",
            serde_json::json!({
                "timestamp": "2026-06-29T00:00:00Z",
                "profile": "codex",
                "decision": "denied",
                "host": "example.com",
                "port": 443,
                "count": 1,
                "reason": "not-in-allowlist",
                "matched_rule": "",
                "run_id": "run-123",
                "workspace": "/Users/c/secret/project"
            })
        )
        .expect("egress write");
    }
    {
        let mut file = std::fs::File::create(auth_broker_log_path()).expect("auth log file");
        writeln!(
            file,
            "{}",
            serde_json::json!({
                "timestamp": "2026-06-29T00:00:00Z",
                "profile": "codex",
                "broker": "api-key",
                "decision": "allowed",
                "method": "POST",
                "path": "/v1/responses?token=secret#fragment",
                "upstream_status": 200,
                "count": 1,
                "reason": "-",
                "run_id": "run-123",
                "authorization": "Bearer secret"
            })
        )
        .expect("auth write");
    }

    let payload = RunHavenTuiService::new()
        .diagnostics_payload(10)
        .expect("diagnostics");

    assert!(!payload.doctor_checks.is_empty());
    assert_eq!(payload.egress_log[0].host, "example.com");
    assert_eq!(payload.auth_log[0].path, "/v1/responses");
    assert_eq!(payload.auth_log[0].upstream_status, Some(200));
    let serialized = serde_json::to_string(&payload).expect("serialize");
    assert!(!serialized.contains("/Users/c/secret/project"));
    assert!(!serialized.contains("Bearer secret"));
    assert!(!serialized.contains("token=secret"));
}

#[tokio::test]
async fn log_snapshot_rejects_invalid_line_count_before_backend_lookup() {
    let error = RunHavenTuiService::new()
        .handle_request(ClientRequest::RunHavenRunLogSnapshot {
            request_id: 42,
            run_id: "not-a-real-run".to_string(),
            lines: 0,
            confirm_sensitive_output: true,
        })
        .await
        .expect_err("invalid line count should fail validation");

    match error {
        RunHavenServiceError::Validation { method, message } => {
            assert_eq!(method, "runhaven/run/logSnapshot");
            assert!(message.contains("between 1 and 500"));
        }
        other => panic!("expected validation error, got {other:?}"),
    }
}

#[tokio::test]
async fn log_snapshot_requires_sensitive_output_confirmation_before_backend_lookup() {
    let error = RunHavenTuiService::new()
        .handle_request(ClientRequest::RunHavenRunLogSnapshot {
            request_id: 42,
            run_id: "not-a-real-run".to_string(),
            lines: 100,
            confirm_sensitive_output: false,
        })
        .await
        .expect_err("missing confirmation should fail validation");

    match error {
        RunHavenServiceError::Validation { method, message } => {
            assert_eq!(method, "runhaven/run/logSnapshot");
            assert!(message.contains("Confirm raw log viewing"));
        }
        other => panic!("expected validation error, got {other:?}"),
    }
}

#[tokio::test]
async fn run_diff_requires_sensitive_output_confirmation_before_backend_lookup() {
    let error = RunHavenTuiService::new()
        .handle_request(ClientRequest::RunHavenRunDiff {
            request_id: 46,
            run_id: "not-a-real-run".to_string(),
            confirm_sensitive_output: false,
        })
        .await
        .expect_err("missing confirmation should fail validation");

    match error {
        RunHavenServiceError::Validation { method, message } => {
            assert_eq!(method, "runhaven/run/diff");
            assert!(message.contains("Confirm diff viewing"));
            assert!(message.contains("workspace file contents"));
        }
        other => panic!("expected validation error, got {other:?}"),
    }
}

#[tokio::test]
async fn run_control_requires_typed_confirmation_before_backend_lookup() {
    let service = RunHavenTuiService::new();
    for (request, method, expected) in [
        (
            ClientRequest::RunHavenRunStop {
                request_id: 43,
                run_id: "not-a-real-run".to_string(),
                confirm_stop: false,
            },
            "runhaven/run/stop",
            "Confirm stop",
        ),
        (
            ClientRequest::RunHavenRunKill {
                request_id: 44,
                run_id: "not-a-real-run".to_string(),
                confirm_kill: false,
            },
            "runhaven/run/kill",
            "Confirm hard stop",
        ),
        (
            ClientRequest::RunHavenRunRepair {
                request_id: 45,
                run_id: "not-a-real-run".to_string(),
                confirm_repair: false,
            },
            "runhaven/run/repair",
            "Confirm repair",
        ),
    ] {
        let error = service
            .handle_request(request)
            .await
            .expect_err("missing run-control confirmation should fail validation");

        match error {
            RunHavenServiceError::Validation {
                method: actual,
                message,
            } => {
                assert_eq!(actual, method);
                assert!(message.contains(expected), "{message}");
            }
            other => panic!("expected validation error, got {other:?}"),
        }
    }
}
