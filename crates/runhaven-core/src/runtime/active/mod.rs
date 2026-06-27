use std::io::IsTerminal;
use std::process::Command;

use anyhow::{Result, bail};
use serde_json::{Value, json};

mod inspect;
mod logs;
mod markers;
mod repair;

pub use logs::{
    DEFAULT_LOG_SNAPSHOT_LINES, MAX_LOG_SNAPSHOT_BYTES, MAX_LOG_SNAPSHOT_LINES,
    active_run_log_snapshot_payload, log_snapshot_payload_from_stdout, validate_log_snapshot_lines,
};
pub use markers::{
    active_run_terminal_status, find_active_run_record, read_active_run_records,
    remove_active_run_record, write_active_run_payload, write_active_run_record,
};
pub use repair::{repair_active_run, runs_repair};

use crate::runtime::plans::{uses_root_identity, validate_resource_options};
use crate::support::validators::{
    require_string, validate_run_id, validate_runhaven_container_name,
};
use inspect::{
    load_container_inspect, print_runs_status, public_active_run_record,
    summarize_container_inspect,
};

pub const DEFAULT_LOG_FOLLOW_LINES: u32 = 200;
pub const DEFAULT_ATTACH_COMMAND: &[&str] = &["/bin/bash"];

pub fn runs_active(json_output: bool) -> Result<i32> {
    let records = read_active_run_records();
    if json_output {
        println!("{}", serde_json::to_string_pretty(&records)?);
        return Ok(0);
    }
    if records.is_empty() {
        println!("No active RunHaven runs found.");
        return Ok(0);
    }
    for record in records {
        println!(
            "{}  {}  {}  {}  run={}  workspace={}  container={}",
            record
                .get("timestamp")
                .and_then(Value::as_str)
                .unwrap_or("<unknown>"),
            record
                .get("profile")
                .and_then(Value::as_str)
                .unwrap_or("unknown"),
            record
                .get("network")
                .and_then(Value::as_str)
                .unwrap_or("unknown"),
            record
                .get("status")
                .and_then(Value::as_str)
                .unwrap_or("unknown"),
            record.get("run_id").and_then(Value::as_str).unwrap_or("-"),
            record
                .get("workspace")
                .and_then(Value::as_str)
                .unwrap_or("-"),
            record
                .get("container_name")
                .and_then(Value::as_str)
                .unwrap_or("-"),
        );
    }
    Ok(0)
}

pub fn runs_attach(
    run_id: &str,
    user: &str,
    workdir: &str,
    tty_mode: &str,
    allow_root_user: bool,
    command_args: &[String],
) -> Result<i32> {
    let record = find_active_run_record(run_id)?;
    let container_name = require_string(
        record.get("container_name"),
        "active run record is missing container name",
    )?;
    validate_runhaven_container_name(container_name)?;
    validate_resource_options("1", "1g", user)?;
    if uses_root_identity(user) && !allow_root_user {
        bail!("root user or group requires --allow-root-user");
    }
    validate_attach_workdir(workdir)?;
    validate_attach_command(command_args)?;
    let mut command = vec!["exec", "--interactive"];
    if tty_mode == "always"
        || (tty_mode == "auto" && std::io::stdin().is_terminal() && std::io::stdout().is_terminal())
    {
        command.push("--tty");
    }
    command.extend(["--user", user, "--workdir", workdir, container_name]);
    let mut process = Command::new("container");
    process.args(command);
    if command_args.is_empty() {
        process.args(DEFAULT_ATTACH_COMMAND);
    } else {
        process.args(command_args);
    }
    Ok(process.status()?.code().unwrap_or(1))
}

pub fn runs_logs_follow(run_id: &str, lines: u32) -> Result<i32> {
    if lines < 1 {
        bail!("--lines must be 1 or greater");
    }
    let record = find_active_run_record(run_id)?;
    let container_name = require_string(
        record.get("container_name"),
        "active run record is missing container name",
    )?;
    validate_runhaven_container_name(container_name)?;
    Ok(Command::new("container")
        .args(["logs", "--follow", "-n", &lines.to_string(), container_name])
        .status()?
        .code()
        .unwrap_or(1))
}

/// Validate and stop one active RunHaven run, returning a structured outcome.
///
/// Shared by the CLI `runs_stop` and the Tauri `stop_run` command so both go
/// through the same run-id, active-marker, and RunHaven-owned-container checks
/// before any mutation. The payload carries `run_id`, `container_name`, and the
/// `container stop` `return_code`.
pub fn stop_active_run(run_id: &str) -> Result<Value> {
    validate_run_id(run_id)?;
    let record = find_active_run_record(run_id)?;
    let container_name = require_string(
        record.get("container_name"),
        "active run record is missing container name",
    )?;
    validate_runhaven_container_name(container_name)?;
    markers::mark_active_run_stop_requested(run_id, &record)?;
    let status = Command::new("container")
        .args(["stop", container_name])
        .status()?;
    let code = status.code().unwrap_or(1);
    if !status.success() {
        let _ = markers::clear_active_run_stop_requested(run_id, &record);
    }
    Ok(json!({
        "run_id": run_id,
        "container_name": container_name,
        "return_code": code,
    }))
}

pub fn runs_stop(run_id: &str) -> Result<i32> {
    let payload = stop_active_run(run_id)?;
    let code = payload
        .get("return_code")
        .and_then(Value::as_i64)
        .unwrap_or(1) as i32;
    if code == 0 {
        let container = payload
            .get("container_name")
            .and_then(Value::as_str)
            .unwrap_or("-");
        println!("Stop requested for run {run_id} ({container}).");
    }
    Ok(code)
}

/// Validate and hard-stop one active RunHaven run, returning a structured
/// outcome. Shares the same checks as `stop_active_run` and backs both the CLI
/// `runs_kill` and the Tauri `kill_run` command.
pub fn kill_active_run(run_id: &str) -> Result<Value> {
    validate_run_id(run_id)?;
    let record = find_active_run_record(run_id)?;
    let container_name = require_string(
        record.get("container_name"),
        "active run record is missing container name",
    )?;
    validate_runhaven_container_name(container_name)?;
    markers::mark_active_run_kill_requested(run_id, &record)?;
    let status = Command::new("container")
        .args(["kill", container_name])
        .status()?;
    let code = status.code().unwrap_or(1);
    if !status.success() {
        let _ = markers::clear_active_run_kill_requested(run_id, &record);
    }
    Ok(json!({
        "run_id": run_id,
        "container_name": container_name,
        "return_code": code,
    }))
}

pub fn runs_kill(run_id: &str) -> Result<i32> {
    let payload = kill_active_run(run_id)?;
    let code = payload
        .get("return_code")
        .and_then(Value::as_i64)
        .unwrap_or(1) as i32;
    if code == 0 {
        let container = payload
            .get("container_name")
            .and_then(Value::as_str)
            .unwrap_or("-");
        println!("Kill requested for run {run_id} ({container}).");
    }
    Ok(code)
}

fn validate_attach_workdir(workdir: &str) -> Result<()> {
    if workdir.is_empty()
        || !workdir.starts_with('/')
        || workdir.chars().any(|c| matches!(c, '\0' | '\r' | '\n'))
    {
        bail!("invalid attach workdir: {workdir:?}");
    }
    Ok(())
}

fn validate_attach_command(command: &[String]) -> Result<()> {
    if command
        .iter()
        .any(|arg| arg.is_empty() || arg.contains('\0'))
    {
        bail!("attach command arguments cannot be empty");
    }
    Ok(())
}

pub fn runs_status(run_id: &str, json_output: bool) -> Result<i32> {
    let (record, container_name) = active_run_record_and_container(run_id)?;
    let output = Command::new("container")
        .args(["inspect", container_name.as_str()])
        .output()?;
    if !output.status.success() {
        eprintln!("runhaven: container inspect failed for run {run_id} ({container_name})");
        return Ok(output.status.code().unwrap_or(1));
    }
    let payload = active_run_status_payload_from_stdout(&record, &output.stdout)?;
    if json_output {
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(0);
    }
    print_runs_status(&payload);
    Ok(0)
}

pub fn active_run_status_payload(run_id: &str) -> Result<Value> {
    let (record, container_name) = active_run_record_and_container(run_id)?;
    let output = Command::new("container")
        .args(["inspect", container_name.as_str()])
        .output()?;
    if !output.status.success() {
        bail!("container inspect failed for run {run_id} ({container_name})");
    }
    active_run_status_payload_from_stdout(&record, &output.stdout)
}

pub(super) fn active_run_record_and_container(run_id: &str) -> Result<(Value, String)> {
    let record = find_active_run_record(run_id)?;
    let container_name = require_string(
        record.get("container_name"),
        "active run record is missing container name",
    )?
    .to_string();
    validate_runhaven_container_name(&container_name)?;
    Ok((record, container_name))
}

fn active_run_status_payload_from_stdout(record: &Value, stdout: &[u8]) -> Result<Value> {
    let container = summarize_container_inspect(load_container_inspect(stdout)?);
    Ok(json!({
        "active_run": public_active_run_record(record),
        "container": container,
    }))
}
