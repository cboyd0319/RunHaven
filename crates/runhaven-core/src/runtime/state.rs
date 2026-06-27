use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

use anyhow::{Result, bail};

use crate::runtime::session_state::{validate_session_name, volume_matches_session};

pub fn state_list(session: Option<&str>) -> Result<i32> {
    let volumes = list_state_volumes(session)?;
    if volumes.is_empty() {
        if let Some(session) = session {
            println!("No RunHaven state volumes found for session {session}.");
        } else {
            println!("No RunHaven state volumes found.");
        }
        return Ok(0);
    }
    for volume in volumes {
        println!("{volume}");
    }
    Ok(0)
}

pub fn state_prune(confirm: bool, session: Option<&str>) -> Result<i32> {
    let volumes = list_state_volumes(session)?;
    if volumes.is_empty() {
        if let Some(session) = session {
            println!("No RunHaven state volumes found for session {session}.");
        } else {
            println!("No RunHaven state volumes found.");
        }
        return Ok(0);
    }
    if !confirm {
        for volume in volumes {
            println!("{volume}");
        }
        println!("Rerun with --yes to delete these volumes.");
        return Ok(2);
    }
    for volume in volumes {
        if matches!(delete_volume(&volume)?, VolumeDeletion::Failed) {
            return Ok(1);
        }
    }
    Ok(0)
}

/// Result of attempting to delete a RunHaven volume.
pub enum VolumeDeletion {
    /// The volume existed and was deleted.
    Deleted,
    /// The volume did not exist, so there was nothing to delete.
    Missing,
    /// The volume exists but could not be deleted (its error was printed).
    Failed,
}

/// Delete a RunHaven volume. A volume that does not exist is reported `Missing`
/// (resetting or pruning state that was never created is not an error). An
/// existing volume can stay briefly held after its container is stopped or
/// killed (Apple container does not auto-remove the container), so deletion
/// retries across a short window before reporting `Failed`.
pub fn delete_volume(volume: &str) -> Result<VolumeDeletion> {
    if !volume_exists(volume)? {
        return Ok(VolumeDeletion::Missing);
    }
    const ATTEMPTS: u32 = 6;
    const DELAY: Duration = Duration::from_secs(1);
    let mut last_stderr = String::new();
    for attempt in 0..ATTEMPTS {
        let output = Command::new("container")
            .args(["volume", "delete", volume])
            .output()?;
        if output.status.success() {
            return Ok(VolumeDeletion::Deleted);
        }
        last_stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        // It may have been released and reaped between the check and now.
        if !volume_exists(volume)? {
            return Ok(VolumeDeletion::Missing);
        }
        if attempt + 1 < ATTEMPTS {
            sleep(DELAY);
        }
    }
    if !last_stderr.is_empty() {
        eprintln!("{last_stderr}");
    }
    Ok(VolumeDeletion::Failed)
}

fn volume_exists(volume: &str) -> Result<bool> {
    let output = Command::new("container")
        .args(["volume", "list", "--quiet"])
        .output()?;
    if !output.status.success() {
        // If the list fails, do not assume the volume is gone; let delete try.
        return Ok(true);
    }
    Ok(volume_in_list(
        &String::from_utf8_lossy(&output.stdout),
        volume,
    ))
}

fn volume_in_list(stdout: &str, volume: &str) -> bool {
    stdout.lines().any(|line| line.trim() == volume)
}

pub fn list_state_volumes(session: Option<&str>) -> Result<Vec<String>> {
    if let Some(session) = session {
        validate_session_name(session)?;
    }
    let output = Command::new("container")
        .args(["volume", "list", "--quiet"])
        .output()?;
    if !output.status.success() {
        bail!("container volume list failed: {}", output.status);
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter(|line| volume_matches_session(line, session).unwrap_or(false))
        .map(str::to_string)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn volume_in_list_matches_whole_lines_only() {
        let stdout = "runhaven-shell-shared-home\nrunhaven-shell-abc-s-sess-home\n";
        assert!(volume_in_list(stdout, "runhaven-shell-shared-home"));
        assert!(volume_in_list(stdout, "runhaven-shell-abc-s-sess-home"));
        // A prefix or substring must not count as present.
        assert!(!volume_in_list(stdout, "runhaven-shell-abc"));
        assert!(!volume_in_list(stdout, "shared-home"));
        assert!(!volume_in_list("", "runhaven-shell-shared-home"));
    }
}
