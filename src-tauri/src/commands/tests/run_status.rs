use super::*;

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
            "auth_scope": "project",
            "container_name": "runhaven-codex-status-test",
            "state_volume": "runhaven-codex-status-home",
            "session": "default",
            "host_pid": 12345
        },
        "container": {
            "state": "running",
            "image": "runhaven/codex:0.7.0",
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
    assert_eq!(response.run.workspace, "runhaven-status");
    assert_eq!(response.run.auth_scope, "project");
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
    assert!(!serialized.contains("/tmp/runhaven-status"));
    assert!(!serialized.contains("stateVolume"));
    assert!(!serialized.contains("runhaven-codex-status-home"));
    assert!(!serialized.contains("/Users/example"));
    assert!(!serialized.contains("secret-flag"));
}

#[test]
fn dashboard_status_omits_raw_workspace_paths_and_state_volumes() {
    let (_cache, _cache_home) = isolated_cache();
    write_active_run_payload(
        "dashboard-active-run",
        json!({
            "timestamp": "2026-07-01T12:00:00Z",
            "run_id": "dashboard-active-run",
            "profile": "codex",
            "workspace": "/sensitive/workspaces/active-project",
            "network": "provider",
            "status": "running",
            "container_name": "runhaven-codex-dashboard-active",
            "state_volume": "runhaven-codex-active-secret-home",
            "auth_scope": "project",
            "session": "active-session"
        }),
    )
    .expect("active run payload");
    runhaven_core::support::paths::ensure_private_parent(
        &runhaven_core::support::paths::runs_log_path(),
    )
    .expect("runs log parent");
    let mut file =
        std::fs::File::create(runhaven_core::support::paths::runs_log_path()).expect("runs log");
    use std::io::Write;
    writeln!(
        file,
        "{}",
        json!({
            "timestamp": "2026-07-01T12:30:00Z",
            "started_at": "2026-07-01T12:00:00Z",
            "finished_at": "2026-07-01T12:30:00Z",
            "run_id": "dashboard-recent-run",
            "profile": "codex",
            "workspace": "/sensitive/workspaces/recent-project",
            "network": "provider",
            "status": "succeeded",
            "auth_scope": "project",
            "state_volume": "runhaven-codex-recent-secret-home",
            "session": "recent-session"
        })
    )
    .expect("write run record");

    let dashboard = collect_dashboard_status();

    assert_eq!(dashboard.active_runs[0].workspace, "active-project");
    assert_eq!(dashboard.recent_runs[0].workspace, "recent-project");
    let serialized = serde_json::to_string(&dashboard).expect("serialize dashboard");
    for forbidden in [
        "/sensitive/workspaces",
        "stateVolume",
        "runhaven-codex-active-secret-home",
        "runhaven-codex-recent-secret-home",
    ] {
        assert!(
            !serialized.contains(forbidden),
            "dashboard leaked forbidden value {forbidden:?}"
        );
    }
}
