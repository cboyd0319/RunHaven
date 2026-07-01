use std::process::Command;

use anyhow::{Result, bail};

use crate::runtime::plans::VOLUME_PREP_NETWORK;

pub fn network_list() -> Result<i32> {
    let networks = list_managed_networks()?;
    if networks.is_empty() {
        println!("No RunHaven managed networks found.");
        return Ok(0);
    }
    for network in networks {
        println!("{network}");
    }
    Ok(0)
}

pub fn network_prune(confirm: bool) -> Result<i32> {
    let networks = list_managed_networks()?;
    if networks.is_empty() {
        println!("No RunHaven managed networks found.");
        return Ok(0);
    }
    if !confirm {
        for network in &networks {
            println!("{network}");
        }
        println!("Rerun with --yes to delete these networks.");
        return Ok(2);
    }
    let outcome = prune_networks(&networks)?;
    if outcome.failed > 0 {
        return Ok(1);
    }
    Ok(0)
}

/// Structured result of deleting RunHaven-managed networks. Counts only; the
/// network names and any raw `container` errors are deliberately withheld so a
/// caller (for example the untrusted GUI WebView) never receives them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NetworkPruneOutcome {
    pub removed: usize,
    pub failed: usize,
}

/// Delete every RunHaven-managed stale network and return only how many were
/// removed versus left in place. `container network delete` fails for in-use
/// networks, so a busy network is counted as `failed` and skipped rather than
/// forced. Structured counterpart to `network_prune`, which keeps the CLI's
/// print-and-exit-code shape.
pub fn prune_managed_networks() -> Result<NetworkPruneOutcome> {
    let networks = list_managed_networks()?;
    prune_networks(&networks)
}

fn prune_networks(networks: &[String]) -> Result<NetworkPruneOutcome> {
    let mut outcome = NetworkPruneOutcome::default();
    for network in networks {
        let status = Command::new("container")
            .args(["network", "delete", network])
            .status()?;
        if status.success() {
            outcome.removed += 1;
        } else {
            outcome.failed += 1;
        }
    }
    Ok(outcome)
}

pub fn list_managed_networks() -> Result<Vec<String>> {
    let output = Command::new("container")
        .args(["network", "list", "--quiet"])
        .output()?;
    if !output.status.success() {
        return Ok(output
            .status
            .code()
            .unwrap_or(1)
            .to_string()
            .lines()
            .map(str::to_string)
            .collect());
    }
    let networks = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| is_runhaven_managed_network(line))
        .map(str::to_string)
        .collect();
    Ok(networks)
}

pub fn is_runhaven_managed_network(name: &str) -> bool {
    if name == VOLUME_PREP_NETWORK {
        return true;
    }
    let project = "[0-9a-f]{16}";
    let internal =
        regex::Regex::new(&format!(r"^runhaven-{project}-internal$")).expect("valid regex");
    let provider = regex::Regex::new(&format!(
        r"^runhaven-(?:antigravity|claude|codex|copilot|gemini|shell)-{project}-provider$"
    ))
    .expect("valid regex");
    internal.is_match(name) || provider.is_match(name)
}

pub fn ensure_network_command_succeeded(status: std::process::ExitStatus) -> Result<()> {
    if status.success() {
        Ok(())
    } else {
        bail!("container network command failed: {status}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn managed_network_detection_gates_only_runhaven_owned_names() {
        assert!(is_runhaven_managed_network(VOLUME_PREP_NETWORK));
        assert!(is_runhaven_managed_network(
            "runhaven-0123456789abcdef-internal"
        ));
        assert!(is_runhaven_managed_network(
            "runhaven-codex-0123456789abcdef-provider"
        ));
        // Foreign or malformed names must never be pruned.
        assert!(!is_runhaven_managed_network("bridge"));
        assert!(!is_runhaven_managed_network("runhaven-internal"));
        assert!(!is_runhaven_managed_network(
            "runhaven-unknownagent-0123456789abcdef-provider"
        ));
    }

    #[test]
    fn network_prune_outcome_defaults_to_zero() {
        let outcome = NetworkPruneOutcome::default();
        assert_eq!(outcome.removed, 0);
        assert_eq!(outcome.failed, 0);
    }
}
