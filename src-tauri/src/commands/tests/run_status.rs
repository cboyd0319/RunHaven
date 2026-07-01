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
    assert!(!serialized.contains("/Users/example"));
    assert!(!serialized.contains("secret-flag"));
}
