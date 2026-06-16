use std::process::Command;

use anyhow::{Result, bail};

use crate::session_state::{validate_session_name, volume_matches_session};

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
        let status = Command::new("container")
            .args(["volume", "delete", &volume])
            .status()?;
        if !status.success() {
            return Ok(status.code().unwrap_or(1));
        }
    }
    Ok(0)
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
