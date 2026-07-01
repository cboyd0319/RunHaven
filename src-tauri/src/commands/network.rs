//! Network cleanup commands: a read-only prunable-count preview and a mutating
//! prune that removes only RunHaven-managed stale networks.
//!
//! `container network delete` fails for in-use networks, so a busy network is
//! reported as `failed` and skipped rather than forced. Both commands expose
//! only counts; network names and raw `container` errors stay in Rust. The
//! prune fails closed on an unconfirmed request and requires a Rust-owned native
//! confirmation showing the count before any deletion.

use runhaven_core::runtime::launch::require_container_cli;
use runhaven_core::runtime::network::{
    NetworkPruneOutcome, list_managed_networks, prune_managed_networks as core_prune,
};
use tauri::AppHandle;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

use crate::contracts::{NetworkPrunePreview, NetworkPruneRequest, NetworkPruneResponse};

const PRUNE_CANCELED: &str = "Network cleanup canceled; no networks were removed.";

#[tauri::command]
pub(crate) fn get_network_prune_preview() -> Result<NetworkPrunePreview, String> {
    let count = managed_network_count()?;
    Ok(NetworkPrunePreview { count })
}

#[tauri::command]
pub(crate) async fn prune_managed_networks(
    app: AppHandle,
    request: NetworkPruneRequest,
) -> Result<NetworkPruneResponse, String> {
    authorize_network_prune(&request)?;
    let count = managed_network_count()?;
    if !trusted_network_prune_confirmed(app, count).await? {
        return Err(PRUNE_CANCELED.to_string());
    }
    let outcome = core_prune().map_err(|_| {
        "RunHaven could not remove the managed networks. Open diagnostics or try again.".to_string()
    })?;
    Ok(prune_response(outcome))
}

/// Validate intent before anything is shown or deleted. Fails closed on an
/// unconfirmed request; no mutation happens here.
fn authorize_network_prune(request: &NetworkPruneRequest) -> Result<(), String> {
    if !request.confirm {
        return Err(
            "Confirm the cleanup before RunHaven removes its managed networks.".to_string(),
        );
    }
    Ok(())
}

fn managed_network_count() -> Result<usize, String> {
    require_container_cli().map_err(|_| {
        "The container tool is not ready. Finish RunHaven setup before cleaning up networks."
            .to_string()
    })?;
    list_managed_networks()
        .map(|networks| networks.len())
        .map_err(|_| "RunHaven could not list its managed networks.".to_string())
}

fn prune_response(outcome: NetworkPruneOutcome) -> NetworkPruneResponse {
    NetworkPruneResponse {
        removed: outcome.removed,
        failed: outcome.failed,
    }
}

async fn trusted_network_prune_confirmed(app: AppHandle, count: usize) -> Result<bool, String> {
    let message = format!(
        "RunHaven-managed networks to remove: {count}\n\nRunHaven will delete only its own stale networks after you confirm. Networks still in use by a running session are skipped."
    );
    tauri::async_runtime::spawn_blocking(move || {
        app.dialog()
            .message(message)
            .title("Confirm RunHaven network cleanup")
            .kind(MessageDialogKind::Warning)
            .buttons(MessageDialogButtons::OkCancelCustom(
                "Remove".to_string(),
                "Cancel".to_string(),
            ))
            .blocking_show()
    })
    .await
    .map_err(|error| format!("could not show network cleanup confirmation: {error}"))
}

#[cfg(test)]
pub(crate) fn prune_managed_networks_for_test(
    request: NetworkPruneRequest,
    count: usize,
    approve: impl FnOnce(usize) -> Result<bool, String>,
    prune: impl FnOnce() -> Result<NetworkPruneOutcome, String>,
) -> Result<NetworkPruneResponse, String> {
    authorize_network_prune(&request)?;
    if !approve(count)? {
        return Err(PRUNE_CANCELED.to_string());
    }
    Ok(prune_response(prune()?))
}
