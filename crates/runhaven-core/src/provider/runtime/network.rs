use std::process::Command;

use anyhow::{Context, Result, bail};
use serde_json::{Value, json};

use crate::runtime::plans::AgentRunPlan;

#[derive(Clone, Debug)]
pub struct InternalNetworkInfo {
    pub ipv4_gateway: String,
    pub ipv4_subnet: String,
}

pub fn cleanup_provider_network(plan: &AgentRunPlan) -> Result<Value> {
    let Some(name) = &plan.network_name else {
        return Ok(json!({"provider_network": "not-created", "provider_network_name": null}));
    };
    let code = delete_container_network(name)?;
    Ok(json!({
        "provider_network": if code == 0 { "deleted" } else { "delete-failed" },
        "provider_network_name": name,
        "delete_return_code": code,
    }))
}

pub fn inspect_internal_network(name: &str) -> Result<InternalNetworkInfo> {
    let output = Command::new("container")
        .args(["network", "inspect", name])
        .output()?;
    if !output.status.success() {
        bail!("container network inspect failed: {name}");
    }
    parse_internal_network_info(name, &output.stdout)
}

pub(super) fn parse_internal_network_info(
    name: &str,
    stdout: &[u8],
) -> Result<InternalNetworkInfo> {
    let payload: Value = serde_json::from_slice(stdout)
        .with_context(|| format!("could not inspect provider network {name:?}"))?;
    let item = payload
        .as_array()
        .and_then(|items| items.first())
        .ok_or_else(|| anyhow::anyhow!("could not inspect provider network {name:?}"))?;
    if item.pointer("/configuration/mode").and_then(Value::as_str) != Some("hostOnly") {
        bail!("provider network {name:?} is not host-only");
    }
    let gateway = item
        .pointer("/status/ipv4Gateway")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            anyhow::anyhow!("provider network {name:?} is missing IPv4 gateway or subnet")
        })?;
    let subnet = item
        .pointer("/status/ipv4Subnet")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            anyhow::anyhow!("provider network {name:?} is missing IPv4 gateway or subnet")
        })?;
    Ok(InternalNetworkInfo {
        ipv4_gateway: gateway.to_string(),
        ipv4_subnet: subnet.to_string(),
    })
}

pub fn delete_container_network(name: &str) -> Result<i32> {
    Ok(Command::new("container")
        .args(["network", "delete", name])
        .status()?
        .code()
        .unwrap_or(1))
}

pub fn ensure_internal_network(name: &str) -> Result<()> {
    let existing = Command::new("container")
        .args(["network", "inspect", name])
        .output()?;
    if existing.status.success() {
        let mode = inspect_network_mode(&String::from_utf8_lossy(&existing.stdout));
        if mode.as_deref() == Some("hostOnly") {
            return Ok(());
        }
        bail!(
            "existing container network {name:?} is {}, not host-only",
            mode.unwrap_or_else(|| "unknown".to_string())
        );
    }
    let status = Command::new("container")
        .args(["network", "create", "--internal", name])
        .status()?;
    if !status.success() {
        bail!("container network create failed: {name}");
    }
    Ok(())
}

pub fn inspect_network_mode(output: &str) -> Option<String> {
    let payload = serde_json::from_str::<Value>(output).ok()?;
    payload
        .as_array()?
        .first()?
        .pointer("/configuration/mode")?
        .as_str()
        .map(str::to_string)
}

pub fn run_preflight(command: &[String]) -> Result<()> {
    if command_starts_with(command, &["container", "network", "create", "--internal"])
        && let Some(name) = command.last()
    {
        return ensure_internal_network(name);
    }
    let status = Command::new(&command[0]).args(&command[1..]).status()?;
    if !status.success() {
        bail!("preflight command failed: {status}");
    }
    Ok(())
}

pub(super) fn command_starts_with(command: &[String], prefix: &[&str]) -> bool {
    command.len() >= prefix.len()
        && command
            .iter()
            .zip(prefix.iter())
            .all(|(left, right)| left == right)
}
