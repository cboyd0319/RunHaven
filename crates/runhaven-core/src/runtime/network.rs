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

/// Delete the exact RunHaven-managed network snapshot the caller reviewed.
/// `container network delete` fails for in-use networks, so a busy network is
/// counted as `failed` and skipped rather than forced. Every supplied name is
/// revalidated before deletion so this API cannot target a foreign network.
pub fn prune_managed_networks(networks: &[String]) -> Result<NetworkPruneOutcome> {
    if networks
        .iter()
        .any(|network| !is_runhaven_managed_network(network))
    {
        bail!("network cleanup snapshot contains a foreign network");
    }
    prune_networks(networks)
}

fn prune_networks(networks: &[String]) -> Result<NetworkPruneOutcome> {
    let mut outcome = NetworkPruneOutcome::default();
    for network in networks {
        let output = crate::apple_container::command()?
            .args(["network", "delete", network])
            .output()?;
        if output.status.success() {
            outcome.removed += 1;
        } else {
            outcome.failed += 1;
        }
    }
    Ok(outcome)
}

pub fn list_managed_networks() -> Result<Vec<String>> {
    let output = crate::apple_container::command()?
        .args(["network", "list", "--quiet"])
        .output()?;
    managed_networks_from_command(output.status.success(), &output.stdout, &output.stderr)
}

fn managed_networks_from_command(
    success: bool,
    stdout: &[u8],
    _stderr: &[u8],
) -> Result<Vec<String>> {
    if !success {
        bail!("container network list failed");
    }
    let networks = String::from_utf8_lossy(stdout)
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
    let runtime = "[0-9a-f]{16}";
    let internal = regex::Regex::new(&format!(r"^runhaven-{project}(?:-{runtime})?-internal$"))
        .expect("valid regex");
    let provider = regex::Regex::new(&format!(
        // Keep the retired Gemini prefix recognizable so Cleanup can remove
        // networks created by earlier local builds. No new Gemini profile can
        // create one.
        r"^runhaven-(?:antigravity|claude|codex|copilot|gemini|shell)-{project}(?:-{runtime})?-provider$"
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
        assert!(is_runhaven_managed_network(
            "runhaven-0123456789abcdef-fedcba9876543210-internal"
        ));
        assert!(is_runhaven_managed_network(
            "runhaven-codex-0123456789abcdef-fedcba9876543210-provider"
        ));
        assert!(is_runhaven_managed_network(
            "runhaven-gemini-0123456789abcdef-provider"
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

    #[test]
    fn prune_snapshot_rejects_foreign_network_names() {
        let error = prune_managed_networks(&["bridge".to_string()])
            .expect_err("foreign networks must fail before deletion");

        assert_eq!(
            error.to_string(),
            "network cleanup snapshot contains a foreign network"
        );
    }

    #[test]
    fn network_subprocesses_capture_output_instead_of_inheriting_stderr() {
        let production = include_str!("network.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production source");
        let inherited_status_call = [".", "status()"].concat();
        assert!(
            !production.contains(&inherited_status_call),
            "network subprocesses must capture output so raw Apple container stderr cannot reach the desktop log"
        );
    }

    #[test]
    fn failed_network_list_is_an_error_not_a_fake_network_name() {
        let error = managed_networks_from_command(false, b"", b"raw xpc detail")
            .expect_err("failed list must fail closed");

        assert_eq!(error.to_string(), "container network list failed");
        assert!(!error.to_string().contains("xpc"));
    }

    #[test]
    fn successful_network_list_keeps_only_runhaven_owned_names() {
        let networks = managed_networks_from_command(
            true,
            b"bridge\n1\nrunhaven-codex-0123456789abcdef-provider\n",
            b"",
        )
        .expect("successful list");

        assert_eq!(networks, vec!["runhaven-codex-0123456789abcdef-provider"]);
    }
}
