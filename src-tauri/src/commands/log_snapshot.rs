#[cfg(not(test))]
use tauri::AppHandle;
#[cfg(not(test))]
use tauri_plugin_dialog::MessageDialogKind;

#[cfg(not(test))]
use super::dialog::confirm_native;

#[cfg(test)]
use runhaven_core::runtime::active::log_snapshot_payload_from_stdout;
use runhaven_core::runtime::active::{
    DEFAULT_LOG_SNAPSHOT_LINES, active_run_log_snapshot_payload, validate_log_snapshot_lines,
};
use serde_json::Value;

use super::validation::{MAX_RUN_ID_LEN, validate_text_len};
use crate::contracts::{LogSnapshotEntry, LogSnapshotRequest, LogSnapshotResponse};

#[cfg(not(test))]
const RAW_OUTPUT_CONFIRM_TITLE: &str = "Confirm raw output view";
const RAW_OUTPUT_CANCELED: &str = "Raw output view canceled.";

struct PreparedLogSnapshot {
    run_id: String,
    lines: u32,
}

#[cfg(not(test))]
#[tauri::command]
pub(crate) async fn get_log_snapshot(
    app: AppHandle,
    request: LogSnapshotRequest,
) -> Result<LogSnapshotResponse, String> {
    let prepared = prepare_log_snapshot(&request)?;
    if !trusted_sensitive_output_approved(
        app,
        RAW_OUTPUT_CONFIRM_TITLE,
        trusted_run_output_message(&prepared.run_id, prepared.lines),
    )
    .await?
    {
        return Err(raw_output_canceled_error());
    }
    read_prepared_log_snapshot(prepared)
}

#[cfg(test)]
#[tauri::command]
pub(crate) fn get_log_snapshot(request: LogSnapshotRequest) -> Result<LogSnapshotResponse, String> {
    get_log_snapshot_with_approval_for_test(request, |_, _| Ok(true))
}

pub(crate) fn validated_lines(request: &LogSnapshotRequest) -> Result<u32, String> {
    Ok(prepare_log_snapshot(request)?.lines)
}

fn prepare_log_snapshot(request: &LogSnapshotRequest) -> Result<PreparedLogSnapshot, String> {
    validate_text_len("run id", &request.run_id, MAX_RUN_ID_LEN)?;
    if !request.confirm_sensitive_output {
        return Err(
            "Confirm raw log viewing before loading output that may contain secrets.".to_string(),
        );
    }
    let lines = request.lines.unwrap_or(DEFAULT_LOG_SNAPSHOT_LINES);
    validate_log_snapshot_lines(lines).map_err(|error| error.to_string())?;
    Ok(PreparedLogSnapshot {
        run_id: request.run_id.clone(),
        lines,
    })
}

fn read_prepared_log_snapshot(
    prepared: PreparedLogSnapshot,
) -> Result<LogSnapshotResponse, String> {
    read_log_snapshot(&prepared.run_id, prepared.lines)
}

pub(crate) fn read_log_snapshot(run_id: &str, lines: u32) -> Result<LogSnapshotResponse, String> {
    active_run_log_snapshot_payload(run_id, lines)
        .map_err(|error| error.to_string())
        .and_then(log_snapshot_response_from_payload)
}

pub(crate) fn raw_output_canceled_error() -> String {
    RAW_OUTPUT_CANCELED.to_string()
}

#[cfg(test)]
pub(crate) fn get_log_snapshot_with_approval_for_test(
    request: LogSnapshotRequest,
    approve: impl FnOnce(&str, u32) -> Result<bool, String>,
) -> Result<LogSnapshotResponse, String> {
    let prepared = prepare_log_snapshot(&request)?;
    if !approve(&prepared.run_id, prepared.lines)? {
        return Err(raw_output_canceled_error());
    }
    read_prepared_log_snapshot(prepared)
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

#[cfg(not(test))]
pub(crate) async fn trusted_sensitive_output_approved(
    app: AppHandle,
    title: &'static str,
    message: String,
) -> Result<bool, String> {
    confirm_native(
        app,
        title,
        "Show output",
        MessageDialogKind::Warning,
        message,
    )
    .await
}

#[cfg(not(test))]
fn trusted_run_output_message(run_id: &str, lines: u32) -> String {
    format!("Run ID: {run_id}\nLines: {lines}\n\nRaw output may include secrets. Show output?")
}

pub(crate) fn log_snapshot_response_from_payload(
    payload: Value,
) -> Result<LogSnapshotResponse, String> {
    let run_id = required_string(&payload, "run_id")?;
    let captured_at = required_string(&payload, "captured_at")?;
    let requested_lines = required_u32(&payload, "requested_lines")?;
    let text = required_string(&payload, "text")?;
    let returned_lines = required_usize(&payload, "returned_lines")?;
    let source = required_string(&payload, "source")?;

    Ok(LogSnapshotResponse {
        run_id,
        captured_at: captured_at.clone(),
        cursor: captured_at.clone(),
        requested_lines,
        entries: vec![LogSnapshotEntry {
            source: source.clone(),
            timestamp: captured_at,
            text: text.clone(),
            line_count: returned_lines,
        }],
        text,
        returned_lines,
        truncated: payload
            .get("truncated")
            .and_then(Value::as_bool)
            .ok_or_else(|| "log snapshot payload is missing truncated".to_string())?,
        source,
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
