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
use tauri_plugin_dialog::MessageDialogKind;

use super::dialog::confirm_native;
use crate::contracts::{NetworkPrunePreview, NetworkPruneRequest, NetworkPruneResponse};
use crate::read_guard::ReadGuards;

const PRUNE_CANCELED: &str = "Network cleanup canceled; no networks were removed.";

#[tauri::command]
pub(crate) fn get_network_prune_preview(
    read_guards: tauri::State<'_, ReadGuards>,
) -> Result<NetworkPrunePreview, String> {
    // Single-flight plus a 2s TTL cache (see `read_guard`) so concurrent
    // WebView calls cannot fork extra `container network` subprocesses. The
    // mutating prune below is never throttled or cached and re-counts fresh
    // before its confirmation dialog.
    read_guards.network_prune_preview.run("preview", || {
        managed_networks().map(|networks| NetworkPrunePreview {
            count: networks.len(),
        })
    })
}

#[tauri::command]
pub(crate) async fn prune_managed_networks(
    app: AppHandle,
    request: NetworkPruneRequest,
) -> Result<NetworkPruneResponse, String> {
    authorize_network_prune(&request)?;
    let networks = managed_networks()?;
    if !trusted_network_prune_confirmed(app, networks.len()).await? {
        return Err(PRUNE_CANCELED.to_string());
    }
    let outcome = core_prune(&networks).map_err(|_| {
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

fn managed_networks() -> Result<Vec<String>, String> {
    require_container_cli().map_err(|_| {
        "The container tool is not ready. Finish RunHaven setup before cleaning up networks."
            .to_string()
    })?;
    list_managed_networks().map_err(|_| "RunHaven could not list its managed networks.".to_string())
}

#[cfg(test)]
fn map_managed_network_count<E>(networks: Result<Vec<String>, E>) -> Result<usize, String> {
    networks
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
    confirm_native(
        app,
        "Remove old networks?",
        "Remove",
        MessageDialogKind::Warning,
        network_cleanup_confirmation_message(count),
    )
    .await
}

fn network_cleanup_confirmation_message(count: usize) -> String {
    format!(
        "RunHaven found {count} old network{}. It will remove only networks created by RunHaven. Networks used by a running session will stay in place.",
        if count == 1 { "" } else { "s" }
    )
}

#[cfg(test)]
pub(crate) fn network_cleanup_confirmation_message_for_test(count: usize) -> String {
    network_cleanup_confirmation_message(count)
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

#[cfg(test)]
pub(crate) fn managed_network_count_for_test(
    networks: Result<Vec<String>, String>,
) -> Result<usize, String> {
    map_managed_network_count(networks)
}
