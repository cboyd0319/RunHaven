use super::*;
use crate::doctor::Check;

#[test]
fn diagnostics_contract_maps_secret_free_fields_only() {
    let data = RunHavenDiagnosticsData::from_payloads(
        [
            Check {
                name: "container\u{1b}".to_string(),
                ok: false,
                detail: "not found\u{7}".to_string(),
                remedy: "Install Apple container 1.0.0.".to_string(),
            },
            Check {
                name: "Apple container CLI".to_string(),
                ok: true,
                detail: "/Users/example/bin/container".to_string(),
                remedy: "Install Apple container 1.0.0.".to_string(),
            },
        ],
        serde_json::json!({
            "status": "available",
            "runtime": "host-broker",
            "credential_stores_inspected": false,
            "environment_values_inspected": false,
            "secrets_printed": false,
            "profiles": [{"name": "codex", "status": "brokered"}],
            "token": "should-not-leak"
        }),
        [serde_json::json!({
            "timestamp": "2026-06-29T00:00:00Z",
            "profile": "codex",
            "decision": "denied",
            "host": "example.com",
            "port": 443,
            "count": 2,
            "reason": "not-in-allowlist",
            "matched_rule": "",
            "run_id": "run-123",
            "workspace": "/Users/c/secret/project"
        })],
        [serde_json::json!({
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
        })],
    );

    assert_eq!(data.auth_status.status, "available");
    assert_eq!(data.doctor_checks[0].name, "container");
    assert_eq!(data.doctor_checks[0].detail, "not found");
    assert_eq!(data.doctor_checks[1].detail, "found on PATH");
    assert_eq!(data.auth_status.profiles[0].name, "codex");
    assert_eq!(data.egress_log[0].host, "example.com");
    assert_eq!(data.auth_log[0].path, "/v1/responses");
    assert_eq!(data.auth_log[0].upstream_status, Some(200));

    let serialized = serde_json::to_string(&data).expect("serialize");
    for forbidden in [
        "/Users/c/secret/project",
        "Bearer secret",
        "token=secret",
        "should-not-leak",
        "credential_stores_inspected",
        "environment_values_inspected",
    ] {
        assert!(
            !serialized.contains(forbidden),
            "diagnostics payload leaked forbidden field/value {forbidden:?}"
        );
    }
}
