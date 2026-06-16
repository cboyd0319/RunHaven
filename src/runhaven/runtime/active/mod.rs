use std::io::IsTerminal;
use std::process::Command;

use anyhow::{Result, bail};
use serde_json::{Value, json};

mod inspect;
mod markers;
mod repair;

pub use markers::{
    active_run_terminal_status, find_active_run_record, read_active_run_records,
    remove_active_run_record, write_active_run_payload, write_active_run_record,
};
pub use repair::runs_repair;

use crate::plans::{uses_root_identity, validate_resource_options};
use crate::validators::{require_string, validate_runhaven_container_name};
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

pub fn runs_stop(run_id: &str) -> Result<i32> {
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
    if !status.success() {
        let _ = markers::clear_active_run_stop_requested(run_id, &record);
        return Ok(status.code().unwrap_or(1));
    }
    println!("Stop requested for run {run_id} ({container_name}).");
    Ok(0)
}

pub fn runs_kill(run_id: &str) -> Result<i32> {
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
    if !status.success() {
        let _ = markers::clear_active_run_kill_requested(run_id, &record);
        return Ok(status.code().unwrap_or(1));
    }
    println!("Kill requested for run {run_id} ({container_name}).");
    Ok(0)
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
    let record = find_active_run_record(run_id)?;
    let container_name = require_string(
        record.get("container_name"),
        "active run record is missing container name",
    )?;
    validate_runhaven_container_name(container_name)?;
    let output = Command::new("container")
        .args(["inspect", container_name])
        .output()?;
    if !output.status.success() {
        eprintln!("runhaven: container inspect failed for run {run_id} ({container_name})");
        return Ok(output.status.code().unwrap_or(1));
    }
    let container = summarize_container_inspect(load_container_inspect(&output.stdout)?);
    let payload = json!({
        "active_run": public_active_run_record(&record),
        "container": container,
    });
    if json_output {
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(0);
    }
    print_runs_status(&payload);
    Ok(0)
}
