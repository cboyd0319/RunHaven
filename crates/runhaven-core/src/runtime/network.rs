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
        for network in networks {
            println!("{network}");
        }
        println!("Rerun with --yes to delete these networks.");
        return Ok(2);
    }
    for network in networks {
        let status = Command::new("container")
            .args(["network", "delete", &network])
            .status()?;
        if !status.success() {
            return Ok(status.code().unwrap_or(1));
        }
    }
    Ok(0)
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
