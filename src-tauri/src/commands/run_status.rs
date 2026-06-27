use runhaven_core::runtime::active::active_run_status_payload;
use serde_json::Value;

use super::validation::{MAX_RUN_ID_LEN, validate_text_len};
use crate::contracts::{
    RunStatusContainer, RunStatusNetwork, RunStatusRequest, RunStatusResources, RunStatusResponse,
    RunStatusRun,
};

#[tauri::command]
pub(crate) fn get_run_status(request: RunStatusRequest) -> Result<RunStatusResponse, String> {
    validate_text_len("run id", &request.run_id, MAX_RUN_ID_LEN)?;
    active_run_status_payload(&request.run_id)
        .map_err(|error| error.to_string())
        .and_then(run_status_response)
}

pub(crate) fn run_status_response(payload: Value) -> Result<RunStatusResponse, String> {
    let active_run = payload
        .get("active_run")
        .ok_or_else(|| "run status payload is missing active_run".to_string())?;
    let container = payload
        .get("container")
        .ok_or_else(|| "run status payload is missing container".to_string())?;
    Ok(RunStatusResponse {
        run: RunStatusRun {
            run_id: field(active_run, "run_id"),
            profile: field(active_run, "profile"),
            workspace: field(active_run, "workspace"),
            network_mode: field(active_run, "network"),
            status: field(active_run, "status"),
            timestamp: field(active_run, "timestamp"),
            state_volume: field(active_run, "state_volume"),
            session: field(active_run, "session"),
            container_name: field(active_run, "container_name"),
        },
        container: RunStatusContainer {
            state: field(container, "state"),
            image: optional_string(container.get("image")),
            started_at: optional_string(container.get("started_at")),
            resources: RunStatusResources {
                cpus: optional_string(container.pointer("/resources/cpus")),
                memory_bytes: container
                    .pointer("/resources/memory_in_bytes")
                    .and_then(Value::as_u64),
            },
            networks: container
                .get("networks")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .map(run_status_network)
                .collect(),
        },
    })
}

fn run_status_network(network: &Value) -> RunStatusNetwork {
    RunStatusNetwork {
        network: optional_string(network.get("network")),
        hostname: optional_string(network.get("hostname")),
        ipv4_address: optional_string(network.get("ipv4_address")),
        ipv4_gateway: optional_string(network.get("ipv4_gateway")),
        ipv6_address: optional_string(network.get("ipv6_address")),
    }
}

fn field(record: &Value, name: &str) -> String {
    optional_string(record.get(name)).unwrap_or_else(|| "-".to_string())
}

fn optional_string(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(text) if !text.is_empty() => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        _ => None,
    }
}
