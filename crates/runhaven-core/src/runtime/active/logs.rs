use std::io::Read;
use std::process::{Command, Stdio};
use std::thread;

use anyhow::{Result, anyhow, bail};
use serde_json::{Value, json};

pub const DEFAULT_LOG_SNAPSHOT_LINES: u32 = super::DEFAULT_LOG_FOLLOW_LINES;
pub const MAX_LOG_SNAPSHOT_LINES: u32 = 500;
pub const MAX_LOG_SNAPSHOT_BYTES: usize = 64 * 1024;
const MAX_LOG_ERROR_BYTES: usize = 4 * 1024;

const RAW_LOG_WARNING: &str = "Raw container output can contain secrets or workspace content.";

pub fn active_run_log_snapshot_payload(run_id: &str, lines: u32) -> Result<Value> {
    validate_log_snapshot_lines(lines)?;
    let (_record, container_name) = super::active_run_record_and_container(run_id)?;
    let stdout = read_container_logs(run_id, &container_name, lines)?;
    log_snapshot_payload_from_stdout(run_id, lines, &stdout, MAX_LOG_SNAPSHOT_BYTES)
}

pub fn validate_log_snapshot_lines(lines: u32) -> Result<()> {
    if !(1..=MAX_LOG_SNAPSHOT_LINES).contains(&lines) {
        bail!("log snapshot lines must be between 1 and {MAX_LOG_SNAPSHOT_LINES}");
    }
    Ok(())
}

pub fn log_snapshot_payload_from_stdout(
    run_id: &str,
    requested_lines: u32,
    stdout: &[u8],
    max_bytes: usize,
) -> Result<Value> {
    let (text, truncated) = bounded_log_text(stdout, max_bytes)?;
    Ok(json!({
        "run_id": run_id,
        "captured_at": crate::provider::observability::utc_timestamp(),
        "requested_lines": requested_lines,
        "text": text,
        "returned_lines": returned_lines(&text),
        "truncated": truncated,
        "source": "container-stdio",
        "warnings": [RAW_LOG_WARNING],
    }))
}

fn bounded_log_text(stdout: &[u8], max_bytes: usize) -> Result<(String, bool)> {
    if max_bytes == 0 {
        bail!("log snapshot byte limit must be greater than 0");
    }
    let mut text = String::from_utf8_lossy(stdout).into_owned();
    let mut truncated = false;
    if text.len() > max_bytes {
        truncated = true;
        let mut start = text.len() - max_bytes;
        while !text.is_char_boundary(start) {
            start += 1;
        }
        text = text[start..].to_string();
    }
    Ok((text, truncated))
}

fn returned_lines(text: &str) -> usize {
    if text.is_empty() {
        0
    } else {
        text.lines().count()
    }
}

fn read_container_logs(run_id: &str, container_name: &str, lines: u32) -> Result<Vec<u8>> {
    let mut child = Command::new("container")
        .args(["logs", "-n", &lines.to_string(), container_name])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("could not capture container logs stdout"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow!("could not capture container logs stderr"))?;
    let stdout_handle =
        thread::spawn(move || read_bounded(stdout, MAX_LOG_SNAPSHOT_BYTES.saturating_add(1)));
    let stderr_handle = thread::spawn(move || read_bounded(stderr, MAX_LOG_ERROR_BYTES));
    let status = child.wait()?;
    let stdout = join_reader(stdout_handle, "stdout")?;
    let _stderr = join_reader(stderr_handle, "stderr")?;
    if !status.success() {
        bail!("container logs failed for run {run_id} ({container_name})");
    }
    Ok(stdout)
}

fn read_bounded<R: Read>(mut reader: R, max_bytes: usize) -> std::io::Result<Vec<u8>> {
    let mut output = Vec::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        output.extend_from_slice(&buffer[..read]);
        if output.len() > max_bytes {
            let excess = output.len() - max_bytes;
            output.drain(0..excess);
        }
    }
    Ok(output)
}

fn join_reader(
    handle: thread::JoinHandle<std::io::Result<Vec<u8>>>,
    stream: &str,
) -> Result<Vec<u8>> {
    handle
        .join()
        .map_err(|_| anyhow!("container logs {stream} reader panicked"))?
        .map_err(Into::into)
}
