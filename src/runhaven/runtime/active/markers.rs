use std::fs;

use anyhow::{Result, bail};
use serde_json::{Value, json};

use crate::paths::{active_run_path, active_runs_dir};
use crate::plans::AgentRunPlan;
use crate::validators::{validate_run_id, validate_runhaven_container_name};
use crate::worktrees::worktree_record;

pub fn write_active_run_record(plan: &AgentRunPlan, run_id: &str, started_at: &str) -> Result<()> {
    let mut payload = json!({
        "timestamp": started_at,
        "run_id": run_id,
        "profile": plan.profile_name,
        "workspace": plan.workspace.display().to_string(),
        "workspace_scope": plan.workspace_scope.as_str(),
        "network": plan.network_mode.as_str(),
        "status": "running",
        "container_name": plan.container_name,
        "state_volume": plan.state_volume,
        "session": plan.session,
        "network_name": plan.network_name,
        "host_pid": std::process::id(),
    });
    if let Some(worktree) = &plan.worktree {
        payload["worktree"] = worktree_record(worktree);
    }
    write_active_run_payload(run_id, payload)
}

pub fn write_active_run_payload(run_id: &str, payload: Value) -> Result<()> {
    let path = active_run_path(run_id)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension("tmp");
    fs::write(
        &temporary,
        format!("{}\n", serde_json::to_string(&payload)?),
    )?;
    fs::rename(temporary, path)?;
    Ok(())
}

pub fn find_active_run_record(run_id: &str) -> Result<Value> {
    let path = active_run_path(run_id)?;
    if !path.exists() {
        bail!("active run not found: {run_id}");
    }
    let payload: Value = serde_json::from_str(&fs::read_to_string(path)?)?;
    if !payload.is_object() {
        bail!("active run record is invalid: {run_id}");
    }
    Ok(payload)
}

pub fn mark_active_run_stop_requested(run_id: &str, record: &Value) -> Result<()> {
    mark_active_run_status(run_id, record, "stop-requested", "stop_requested_at")
}

pub fn clear_active_run_stop_requested(run_id: &str, record: &Value) -> Result<()> {
    clear_active_run_status(run_id, record, "stop_requested_at")
}

pub fn mark_active_run_kill_requested(run_id: &str, record: &Value) -> Result<()> {
    mark_active_run_status(run_id, record, "kill-requested", "kill_requested_at")
}

pub fn clear_active_run_kill_requested(run_id: &str, record: &Value) -> Result<()> {
    clear_active_run_status(run_id, record, "kill_requested_at")
}

fn mark_active_run_status(
    run_id: &str,
    record: &Value,
    status: &str,
    timestamp_key: &str,
) -> Result<()> {
    let mut updated = record.clone();
    updated["status"] = json!(status);
    updated[timestamp_key] = json!(crate::provider_observability::utc_timestamp());
    write_active_run_payload(run_id, updated)
}

fn clear_active_run_status(run_id: &str, record: &Value, timestamp_key: &str) -> Result<()> {
    let mut updated = record.clone();
    updated["status"] = json!("running");
    if let Some(object) = updated.as_object_mut() {
        object.remove(timestamp_key);
    }
    write_active_run_payload(run_id, updated)
}

pub fn active_run_terminal_status(run_id: &str) -> Option<String> {
    let record = find_active_run_record(run_id).ok()?;
    if record
        .get("kill_requested_at")
        .and_then(Value::as_str)
        .is_some()
    {
        return Some("killed".to_string());
    }
    if record
        .get("stop_requested_at")
        .and_then(Value::as_str)
        .is_some()
    {
        return Some("stopped".to_string());
    }
    None
}

pub fn remove_active_run_record(run_id: &str) -> Result<()> {
    let path = active_run_path(run_id)?;
    let _ = fs::remove_file(path);
    Ok(())
}

pub fn read_active_run_records() -> Vec<Value> {
    let dir = active_runs_dir();
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut records = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let Ok(text) = fs::read_to_string(path) else {
            continue;
        };
        let Ok(payload) = serde_json::from_str::<Value>(&text) else {
            continue;
        };
        let Some(run_id) = payload.get("run_id").and_then(Value::as_str) else {
            continue;
        };
        let Some(container_name) = payload.get("container_name").and_then(Value::as_str) else {
            continue;
        };
        if validate_run_id(run_id).is_ok()
            && validate_runhaven_container_name(container_name).is_ok()
        {
            records.push(payload);
        }
    }
    records.sort_by_key(|record| {
        (
            record
                .get("timestamp")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            record
                .get("run_id")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
        )
    });
    records
}
