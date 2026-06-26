use anyhow::{Result, bail};
use sha2::{Digest, Sha256};

pub const SESSION_DEFAULT: &str = "default";
pub const SESSION_MARKER: &str = "-s-";

pub fn validate_session_name(name: &str) -> Result<String> {
    let mut chars = name.chars();
    let first = chars.next();
    let valid = name.len() <= 63
        && matches!(first, Some(c) if c.is_ascii_lowercase() || c.is_ascii_digit())
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '_' | '.' | '-'));
    if !valid {
        bail!("invalid session name: use lowercase letters, numbers, dots, underscores, or dashes");
    }
    if name == SESSION_DEFAULT {
        bail!("invalid session name: 'default' is reserved");
    }
    Ok(name.to_string())
}

pub fn state_volume_name(
    profile_name: &str,
    project_id: &str,
    session: Option<&str>,
) -> Result<String> {
    let Some(session) = session else {
        return Ok(format!("runhaven-{profile_name}-{project_id}-home"));
    };
    let session = validate_session_name(session)?;
    let digest = session_digest(&session);
    let prefix = format!("runhaven-{profile_name}-{project_id}{SESSION_MARKER}");
    let suffix = format!("-{digest}-home");
    let budget = 63usize.saturating_sub(prefix.len() + suffix.len());
    if budget < 1 {
        bail!("session state volume name would be too long");
    }
    let visible = session.chars().take(budget).collect::<String>();
    Ok(format!("{prefix}{visible}{suffix}"))
}

/// Per-agent state volume shared across every workspace, so an OAuth login is
/// done once and reused everywhere (`AuthScope::Agent`). The literal `shared`
/// segment never collides with the hex `project_id` used by per-workspace
/// volumes.
pub fn shared_state_volume_name(profile_name: &str) -> String {
    format!("runhaven-{profile_name}-shared-home")
}

pub fn session_digest(session: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(session.as_bytes());
    hex_prefix(&digest.finalize(), 8)
}

pub fn volume_matches_session(volume: &str, session: Option<&str>) -> Result<bool> {
    if !is_runhaven_state_volume(volume) {
        return Ok(false);
    }
    let Some(session) = session else {
        return Ok(true);
    };
    let session = validate_session_name(session)?;
    Ok(volume.contains(&format!("{SESSION_MARKER}{session}-"))
        || volume.ends_with(&format!("-{}-home", session_digest(&session))))
}

pub fn is_runhaven_state_volume(volume: &str) -> bool {
    volume.starts_with("runhaven-") && volume.ends_with("-home")
}

fn hex_prefix(bytes: &[u8], len: usize) -> String {
    bytes
        .iter()
        .flat_map(|byte| [byte >> 4, byte & 0x0f])
        .take(len)
        .map(|n| char::from_digit(n as u32, 16).expect("hex digit"))
        .collect()
}
