use std::process::Command;

use anyhow::{Result, bail};
use serde_json::{Value, json};

use super::{find_active_run_record, read_active_run_records, remove_active_run_record};
use crate::support::validators::{
    require_string, validate_run_id, validate_runhaven_container_name,
};

pub fn runs_repair(run_id: Option<&str>, repair_all: bool, json_output: bool) -> Result<i32> {
    if repair_all && run_id.is_some() {
        bail!("pass either a run id or --all, not both");
    }
    if !repair_all && run_id.is_none() {
        bail!("pass a run id or --all");
    }
    let records = if repair_all {
        read_active_run_records()
    } else {
        vec![find_active_run_record(run_id.expect("checked"))?]
    };
    let mut results = Vec::new();
    for record in records {
        results.push(repair_one_marker(&record)?);
    }
    let removed = results
        .iter()
        .filter(|result| result["status"].as_str() == Some("removed"))
        .count();
    let kept = results
        .iter()
        .filter(|result| result["status"].as_str() == Some("kept"))
        .count();
    let unverified = results
        .iter()
        .filter(|result| result["status"].as_str() == Some("unverified"))
        .count();
    let exit_code = if unverified > 0 {
        if repair_all {
            1
        } else {
            results[0]["inspect_return_code"].as_i64().unwrap_or(1) as i32
        }
    } else if !repair_all && kept > 0 {
        1
    } else {
        0
    };
    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "mode": if repair_all { "all" } else { "single" },
                "summary": {"kept": kept, "removed": removed, "unverified": unverified},
                "exit_code": exit_code,
                "results": results,
            }))?
        );
        return Ok(exit_code);
    }
    for result in &results {
        let run_id = result["run_id"].as_str().unwrap_or("-");
        let container_name = result["container_name"].as_str().unwrap_or("-");
        match result["status"].as_str().unwrap_or("unverified") {
            "removed" => {
                println!("Removed stale active marker for run {run_id} ({container_name}).")
            }
            "kept" => eprintln!(
                "runhaven: kept active marker for run {run_id}; container still exists ({container_name})"
            ),
            _ => eprintln!(
                "runhaven: could not confirm whether container exists for run {run_id} ({container_name})"
            ),
        }
    }
    if repair_all {
        println!("Repair summary: removed={removed} kept={kept} unverified={unverified}");
    }
    Ok(exit_code)
}

fn repair_one_marker(record: &Value) -> Result<Value> {
    let run_id = require_string(record.get("run_id"), "active run record is missing run id")?;
    let container_name = require_string(
        record.get("container_name"),
        "active run record is missing container name",
    )?;
    validate_run_id(run_id)?;
    validate_runhaven_container_name(container_name)?;
    let output = Command::new("container")
        .args(["inspect", container_name])
        .output()?;
    let code = output.status.code().unwrap_or(1);
    if output.status.success() {
        return Ok(json!({
            "run_id": run_id,
            "container_name": container_name,
            "inspect_return_code": code,
            "marker_removed": false,
            "status": "kept",
        }));
    }
    if is_missing_container_stderr(&output.stderr) {
        remove_active_run_record(run_id)?;
        return Ok(json!({
            "run_id": run_id,
            "container_name": container_name,
            "inspect_return_code": code,
            "marker_removed": true,
            "status": "removed",
        }));
    }
    Ok(json!({
        "run_id": run_id,
        "container_name": container_name,
        "inspect_return_code": code,
        "marker_removed": false,
        "status": "unverified",
    }))
}

fn is_missing_container_stderr(stderr: &[u8]) -> bool {
    let stderr = String::from_utf8_lossy(stderr).to_ascii_lowercase();
    stderr.contains("container not found:")
        || (stderr.contains("container with id ") && stderr.contains(" not found"))
}

/// Repair the stale active marker for a single run, returning the result value
/// (`run_id`, `container_name`, `status`, `marker_removed`). Backs the Tauri
/// `repair_run` command; the CLI `runs_repair` keeps its own summary output.
pub fn repair_active_run(run_id: &str) -> Result<Value> {
    validate_run_id(run_id)?;
    let record = find_active_run_record(run_id)?;
    repair_one_marker(&record)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_container_stderr_accepts_current_cli_wording() {
        assert!(is_missing_container_stderr(
            b"Error: container not found: runhaven-nonexistent-fixture\n"
        ));
    }

    #[test]
    fn missing_container_stderr_accepts_source_not_found_wording() {
        assert!(is_missing_container_stderr(
            b"Error: container with ID runhaven-shell-run not found\n"
        ));
    }

    #[test]
    fn missing_container_stderr_rejects_unverified_errors() {
        assert!(!is_missing_container_stderr(
            b"Error: failed to connect to container service\n"
        ));
        assert!(!is_missing_container_stderr(
            b"Error: network not found: runhaven-fixture\n"
        ));
        assert!(!is_missing_container_stderr(
            b"Error: image not found: runhaven/base:0.1.0\n"
        ));
    }
}
