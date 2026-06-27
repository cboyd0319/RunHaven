#[cfg(test)]
use runhaven_core::runtime::active::log_snapshot_payload_from_stdout;
use runhaven_core::runtime::active::{
    DEFAULT_LOG_SNAPSHOT_LINES, active_run_log_snapshot_payload, validate_log_snapshot_lines,
};
use serde_json::Value;

use super::validation::{MAX_RUN_ID_LEN, validate_text_len};
use crate::contracts::{LogSnapshotRequest, LogSnapshotResponse};

#[tauri::command]
pub(crate) fn get_log_snapshot(request: LogSnapshotRequest) -> Result<LogSnapshotResponse, String> {
    let lines = validated_lines(&request)?;
    active_run_log_snapshot_payload(&request.run_id, lines)
        .map_err(|error| error.to_string())
        .and_then(log_snapshot_response_from_payload)
}

pub(crate) fn validated_lines(request: &LogSnapshotRequest) -> Result<u32, String> {
    validate_text_len("run id", &request.run_id, MAX_RUN_ID_LEN)?;
    if !request.confirm_sensitive_output {
        return Err(
            "Confirm raw log viewing before loading output that may contain secrets.".to_string(),
        );
    }
    let lines = request.lines.unwrap_or(DEFAULT_LOG_SNAPSHOT_LINES);
    validate_log_snapshot_lines(lines).map_err(|error| error.to_string())?;
    Ok(lines)
}

#[cfg(test)]
pub(crate) fn log_snapshot_response(
    run_id: &str,
    requested_lines: u32,
    stdout: &[u8],
    max_bytes: usize,
) -> Result<LogSnapshotResponse, String> {
    log_snapshot_payload_from_stdout(run_id, requested_lines, stdout, max_bytes)
        .map_err(|error| error.to_string())
        .and_then(log_snapshot_response_from_payload)
}

fn log_snapshot_response_from_payload(payload: Value) -> Result<LogSnapshotResponse, String> {
    Ok(LogSnapshotResponse {
        run_id: required_string(&payload, "run_id")?,
        captured_at: required_string(&payload, "captured_at")?,
        requested_lines: required_u32(&payload, "requested_lines")?,
        text: required_string(&payload, "text")?,
        returned_lines: required_usize(&payload, "returned_lines")?,
        truncated: payload
            .get("truncated")
            .and_then(Value::as_bool)
            .ok_or_else(|| "log snapshot payload is missing truncated".to_string())?,
        source: required_string(&payload, "source")?,
        warnings: payload
            .get("warnings")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect(),
    })
}

fn required_string(payload: &Value, name: &str) -> Result<String, String> {
    payload
        .get(name)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("log snapshot payload is missing {name}"))
}

fn required_u32(payload: &Value, name: &str) -> Result<u32, String> {
    let value = payload
        .get(name)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("log snapshot payload is missing {name}"))?;
    u32::try_from(value).map_err(|_| format!("log snapshot payload {name} is too large"))
}

fn required_usize(payload: &Value, name: &str) -> Result<usize, String> {
    let value = payload
        .get(name)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("log snapshot payload is missing {name}"))?;
    usize::try_from(value).map_err(|_| format!("log snapshot payload {name} is too large"))
}
