use anyhow::{Result, bail};
use serde_json::{Value, json};

const ACTIVE_RUN_PUBLIC_FIELDS: &[&str] = &[
    "timestamp",
    "run_id",
    "profile",
    "workspace",
    "network",
    "status",
    "container_name",
    "state_volume",
    "session",
    "network_name",
    "host_pid",
    "stop_requested_at",
    "kill_requested_at",
];

pub fn public_active_run_record(record: &Value) -> Value {
    let mut payload = serde_json::Map::new();
    for key in ACTIVE_RUN_PUBLIC_FIELDS {
        if let Some(value) = record.get(*key) {
            payload.insert((*key).to_string(), value.clone());
        }
    }
    Value::Object(payload)
}

pub fn load_container_inspect(stdout: &[u8]) -> Result<Value> {
    let payload: Value = serde_json::from_slice(stdout)?;
    if let Some(items) = payload.as_array() {
        return items
            .first()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("container inspect returned no records"));
    }
    if payload.is_object() {
        return Ok(payload);
    }
    bail!("container inspect returned an invalid record")
}

pub fn summarize_container_inspect(record: Value) -> Value {
    let mut container = serde_json::Map::new();
    if let Some(id) = record.get("id").and_then(Value::as_str) {
        container.insert("id".to_string(), json!(id));
    }
    if let Some(image) = record
        .pointer("/configuration/image/reference")
        .and_then(Value::as_str)
    {
        container.insert("image".to_string(), json!(image));
    }
    if let Some(resources) =
        summarize_container_resources(record.pointer("/configuration/resources"))
    {
        container.insert("resources".to_string(), resources);
    }
    if let Some(state) = record.pointer("/status/state").and_then(Value::as_str) {
        container.insert("state".to_string(), json!(state));
    } else if let Some(state) = record.get("status").and_then(Value::as_str) {
        container.insert("state".to_string(), json!(state));
    }
    if let Some(started) = record
        .pointer("/status/startedDate")
        .and_then(Value::as_str)
    {
        container.insert("started_at".to_string(), json!(started));
    }
    let networks = summarize_container_networks(
        record
            .pointer("/status/networks")
            .or_else(|| record.get("networks")),
    );
    if !networks.is_empty() {
        container.insert("networks".to_string(), Value::Array(networks));
    }
    Value::Object(container)
}

fn summarize_container_resources(resources: Option<&Value>) -> Option<Value> {
    let resources = resources?.as_object()?;
    let mut summary = serde_json::Map::new();
    if let Some(cpus) = resources.get("cpus").and_then(Value::as_f64) {
        summary.insert("cpus".to_string(), json!(cpus));
    }
    if let Some(memory) = resources.get("memoryInBytes").and_then(Value::as_u64) {
        summary.insert("memory_in_bytes".to_string(), json!(memory));
    }
    if summary.is_empty() {
        None
    } else {
        Some(Value::Object(summary))
    }
}

fn summarize_container_networks(networks: Option<&Value>) -> Vec<Value> {
    let Some(networks) = networks.and_then(Value::as_array) else {
        return Vec::new();
    };
    networks
        .iter()
        .filter_map(|network| {
            let mut summary = serde_json::Map::new();
            for (source, output) in [
                ("network", "network"),
                ("hostname", "hostname"),
                ("ipv4Address", "ipv4_address"),
                ("ipv4Gateway", "ipv4_gateway"),
                ("ipv6Address", "ipv6_address"),
            ] {
                if let Some(value) = network.get(source).and_then(Value::as_str) {
                    summary.insert(output.to_string(), json!(value));
                }
            }
            for (source, output) in [("address", "ipv4_address"), ("gateway", "ipv4_gateway")] {
                if !summary.contains_key(output)
                    && let Some(value) = network.get(source).and_then(Value::as_str)
                {
                    summary.insert(output.to_string(), json!(value));
                }
            }
            if summary.is_empty() {
                None
            } else {
                Some(Value::Object(summary))
            }
        })
        .collect()
}

pub fn print_runs_status(payload: &Value) {
    let active_run = &payload["active_run"];
    let container = &payload["container"];
    println!(
        "Run id: {}",
        active_run
            .get("run_id")
            .and_then(Value::as_str)
            .unwrap_or("-")
    );
    println!(
        "Profile: {}",
        active_run
            .get("profile")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
    );
    println!(
        "Workspace: {}",
        active_run
            .get("workspace")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
    );
    println!(
        "Network: {}",
        active_run
            .get("network")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
    );
    println!(
        "Marker status: {}",
        active_run
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
    );
    println!(
        "Container: {}",
        active_run
            .get("container_name")
            .and_then(Value::as_str)
            .unwrap_or("-")
    );
    println!(
        "Container state: {}",
        container
            .get("state")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
    );
    println!(
        "Container started: {}",
        container
            .get("started_at")
            .and_then(Value::as_str)
            .unwrap_or("-")
    );
    if let Some(image) = container.get("image").and_then(Value::as_str) {
        println!("Container image: {image}");
    }
    if let Some(networks) = container.get("networks").and_then(Value::as_array) {
        for network in networks {
            println!("Container network: {}", format_container_network(network));
        }
    }
}

fn format_container_network(network: &Value) -> String {
    let mut parts = vec![
        network
            .get("network")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string(),
    ];
    for (key, label) in [
        ("ipv4_address", "ipv4"),
        ("ipv6_address", "ipv6"),
        ("hostname", "hostname"),
    ] {
        if let Some(value) = network.get(key).and_then(Value::as_str) {
            parts.push(format!("{label}={value}"));
        }
    }
    parts.join(" ")
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    const CONTAINER_INSPECT_CURRENT: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/apple_container/container-inspect-current.json"
    ));
    const CONTAINER_INSPECT_LEGACY_ATTACHMENT: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/apple_container/container-inspect-legacy-attachment.json"
    ));

    #[test]
    fn container_status_summary_omits_command_environment_mounts_and_secrets() {
        let summary = summarize_container_inspect(json!({
            "id": "runhaven-shell-abc-run",
            "configuration": {
                "image": {"reference": "runhaven/base:0.1.0"},
                "resources": {"cpus": 2, "memoryInBytes": 1073741824},
                "initProcess": {
                    "arguments": ["agent", "--secret-flag"],
                    "environment": ["OPENAI_API_KEY=fake-secret-value"]
                },
                "mounts": [{"source": "/host/private", "destination": "/workspace"}]
            },
            "status": {
                "state": "running",
                "startedDate": "2026-06-15T00:00:10Z",
                "networks": [{
                    "network": "default",
                    "hostname": "runhaven-shell-abc-run",
                    "ipv4Address": "192.168.64.20/24",
                    "ipv4Gateway": "192.168.64.1"
                }]
            }
        }));

        assert_eq!(summary["image"], "runhaven/base:0.1.0");
        assert_eq!(summary["state"], "running");
        assert_eq!(summary["resources"]["cpus"], 2.0);
        assert_eq!(summary["networks"][0]["ipv4_address"], "192.168.64.20/24");
        let serialized = serde_json::to_string(&summary).expect("json");
        assert!(!serialized.contains("fake-secret-value"));
        assert!(!serialized.contains("OPENAI_API_KEY"));
        assert!(!serialized.contains("secret-flag"));
        assert!(!serialized.contains("/host/private"));
    }

    #[test]
    fn parses_current_apple_container_inspect_shape() {
        let record = load_container_inspect(CONTAINER_INSPECT_CURRENT).expect("inspect record");
        let summary = summarize_container_inspect(record);

        assert_eq!(
            summary["image"],
            "ghcr.io/apple/container-builder-shim/builder:0.12.0"
        );
        assert_eq!(summary["state"], "running");
        assert_eq!(summary["started_at"], "2026-06-14T02:50:37Z");
        assert_eq!(summary["resources"]["memory_in_bytes"], 2147483648u64);
        assert_eq!(summary["networks"][0]["network"], "default");
        assert_eq!(summary["networks"][0]["ipv4_gateway"], "192.168.64.1");
        let serialized = serde_json::to_string(&summary).expect("json");
        assert!(!serialized.contains("SECRET_TOKEN"));
        assert!(!serialized.contains("/Users/example"));
    }

    #[test]
    fn parses_legacy_apple_network_attachment_aliases() {
        let record =
            load_container_inspect(CONTAINER_INSPECT_LEGACY_ATTACHMENT).expect("inspect record");
        let summary = summarize_container_inspect(record);

        assert_eq!(summary["state"], "running");
        assert_eq!(summary["networks"][0]["ipv4_address"], "192.168.70.2/24");
        assert_eq!(summary["networks"][0]["ipv4_gateway"], "192.168.70.1");
    }

    #[test]
    fn accepts_single_object_container_inspect_shape() {
        let record = load_container_inspect(br#"{"id":"runhaven-shell-fixture-run"}"#)
            .expect("single object");

        assert_eq!(record["id"], "runhaven-shell-fixture-run");
    }

    #[test]
    fn rejects_empty_container_inspect_array() {
        let error = load_container_inspect(b"[]").expect_err("empty inspect");

        assert!(error.to_string().contains("returned no records"));
    }

    #[test]
    fn rejects_non_object_container_inspect_shape() {
        let error = load_container_inspect(b"false").expect_err("invalid inspect");

        assert!(error.to_string().contains("invalid record"));
    }
}
