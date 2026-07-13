use anyhow::{Result, bail};

use crate::runtime::lock::acquire_state_lock;
use crate::runtime::plans::AgentRunPlan;

pub(super) const STATE_VOLUME_IN_USE_MESSAGE: &str = "This agent's saved state is already in use by another container. Stop the other run. If RunHaven quit during it, open Cleanup and remove leftover resources, then try again.";

pub(super) fn launch_with_state_volume_guard(
    plan: &AgentRunPlan,
    launch: impl FnOnce() -> Result<i32>,
    on_success: impl FnOnce() -> Result<()>,
) -> Result<i32> {
    launch_under_state_lock_with_attachment_check(
        plan,
        ensure_state_volume_detached,
        launch,
        on_success,
    )
}

pub(super) fn launch_under_state_lock_with_attachment_check(
    plan: &AgentRunPlan,
    attachment_check: impl FnOnce(&str) -> Result<()>,
    launch: impl FnOnce() -> Result<i32>,
    on_success: impl FnOnce() -> Result<()>,
) -> Result<i32> {
    let _state_lock = acquire_state_lock(&plan.state_volume)?;
    attachment_check(&plan.state_volume)?;
    let code = launch()?;
    if code == 0 {
        on_success()?;
    }
    Ok(code)
}

pub(super) fn ensure_state_volume_detached(state_volume: &str) -> Result<()> {
    let output = crate::apple_container::command()?
        .args(["list", "--format", "json"])
        .output()
        .map_err(|_| anyhow::anyhow!(state_volume_check_error()))?;
    ensure_state_volume_detached_from_container_list(
        state_volume,
        output.status.success(),
        &output.stdout,
    )
}

pub(super) fn ensure_state_volume_detached_from_container_list(
    state_volume: &str,
    list_succeeded: bool,
    stdout: &[u8],
) -> Result<()> {
    if !list_succeeded {
        bail!("{}", state_volume_check_error());
    }
    let containers: serde_json::Value =
        serde_json::from_slice(stdout).map_err(|_| anyhow::anyhow!(state_volume_check_error()))?;
    let Some(containers) = containers.as_array() else {
        bail!("{}", state_volume_check_error());
    };
    let attached = containers.iter().any(|container| {
        container
            .pointer("/configuration/mounts")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|mounts| {
                mounts.iter().any(|mount| {
                    mount
                        .pointer("/type/volume/name")
                        .and_then(serde_json::Value::as_str)
                        == Some(state_volume)
                })
            })
    });
    if attached {
        bail!(STATE_VOLUME_IN_USE_MESSAGE);
    }
    Ok(())
}

fn state_volume_check_error() -> &'static str {
    "RunHaven could not verify whether this agent's saved state is available. Make sure Apple container is running, then try again."
}
