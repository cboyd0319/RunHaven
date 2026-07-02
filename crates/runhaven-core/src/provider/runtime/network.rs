use std::io;
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde_json::{Value, json};

use crate::runtime::active::repair::is_missing_container_stderr;
use crate::runtime::plans::AgentRunPlan;

#[derive(Clone, Debug)]
pub struct InternalNetworkInfo {
    pub ipv4_gateway: String,
    pub ipv4_subnet: String,
}

/// One captured `container` teardown result: exit code plus stderr bytes. stderr
/// is the only stream teardown needs, to tell "already gone" from a real failure
/// and to surface a delete anomaly.
#[derive(Clone)]
pub(crate) struct ContainerOutcome {
    pub code: i32,
    pub stderr: Vec<u8>,
}

/// Runs the `container` teardown subcommands. The seam lets teardown tests assert
/// the exact command sequence (force-remove the run container before deleting the
/// network) without a live Apple `container` daemon; production uses
/// [`SystemContainerCli`]. This mirrors the crate's other `*_with_runner` seams.
pub(crate) trait ContainerCli {
    fn run(&mut self, args: &[&str]) -> io::Result<ContainerOutcome>;
}

struct SystemContainerCli;

impl ContainerCli for SystemContainerCli {
    fn run(&mut self, args: &[&str]) -> io::Result<ContainerOutcome> {
        let output = Command::new("container").args(args).output()?;
        Ok(ContainerOutcome {
            code: output.status.code().unwrap_or(1),
            stderr: output.stderr,
        })
    }
}

pub fn cleanup_provider_network(plan: &AgentRunPlan) -> Result<Value> {
    cleanup_provider_network_with_cli(plan, &mut SystemContainerCli)
}

/// Tear down the provider network. A network cannot be deleted while a container
/// still references it, and `container run --rm` removes the guest only on a
/// clean client exit: when the login or attach PTY client is killed mid-flow
/// (terminal closed, app quit, or the login process died) the guest lingers and
/// blocks the delete, leaking both the container and the network. So force-remove
/// the run container first, then delete the network. The force-remove is
/// idempotent: on the normal clean path the container is already gone, which the
/// runtime reports as "not found" and this treats as success, so a good run still
/// records `provider_network: deleted`, never a spurious failure.
pub(crate) fn cleanup_provider_network_with_cli(
    plan: &AgentRunPlan,
    cli: &mut impl ContainerCli,
) -> Result<Value> {
    let Some(name) = &plan.network_name else {
        return Ok(json!({"provider_network": "not-created", "provider_network_name": null}));
    };
    let run_container = force_remove_run_container(cli, &plan.container_name)?;
    let delete = cli.run(&["network", "delete", name])?;
    if delete.code != 0 {
        // Surface the runtime's own error (previously shown via inherited stdio)
        // so a delete-failed teardown is visible, not silently swallowed. After
        // the force-remove above, a "referring containers" failure here is a real
        // anomaly worth seeing.
        report_stderr(
            &format!("could not delete provider network {name}"),
            &delete.stderr,
        );
    }
    Ok(json!({
        "provider_network": if delete.code == 0 { "deleted" } else { "delete-failed" },
        "provider_network_name": name,
        "delete_return_code": delete.code,
        "run_container": run_container,
    }))
}

/// Force-remove the run container so the provider network has no referring
/// container. `container rm --force` stops and deletes it in one step even if it
/// is still running (verified against Apple `container` 1.0.0). Returns a status
/// string for the run record: `removed` (a lingering guest was cleaned up),
/// `already-gone` (the clean `--rm` path; the runtime reports "not found", which
/// is success, not failure), or `remove-failed` (an unexpected error; teardown
/// still proceeds to the network delete, which then surfaces any real
/// referring-container anomaly).
fn force_remove_run_container(cli: &mut impl ContainerCli, name: &str) -> Result<Value> {
    let outcome = cli.run(&["rm", "--force", name])?;
    if outcome.code == 0 {
        return Ok(json!("removed"));
    }
    if is_missing_container_stderr(&outcome.stderr) {
        return Ok(json!("already-gone"));
    }
    report_stderr(
        &format!("could not remove run container {name}"),
        &outcome.stderr,
    );
    Ok(json!("remove-failed"))
}

fn report_stderr(prefix: &str, stderr: &[u8]) {
    let stderr = String::from_utf8_lossy(stderr);
    let stderr = stderr.trim();
    if stderr.is_empty() {
        eprintln!("runhaven: {prefix}");
    } else {
        eprintln!("runhaven: {prefix}: {stderr}");
    }
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
