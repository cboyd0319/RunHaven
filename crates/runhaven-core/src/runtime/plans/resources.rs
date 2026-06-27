use std::path::Path;

use sha2::{Digest, Sha256};

use crate::runtime::profiles::AgentProfile;
use crate::support::shell;

pub fn bind_mount(source: &Path, target: &str, read_only: bool) -> String {
    let mut parts = vec![
        "type=bind".to_string(),
        format!("source={}", source.display()),
        format!("target={target}"),
    ];
    if read_only {
        parts.push("readonly".to_string());
    }
    parts.join(",")
}

pub fn volume_mount(source: &str, target: &str) -> String {
    format!("type=volume,source={source},target={target}")
}

pub fn home_setup_command(profile: &AgentProfile) -> String {
    let mut commands = vec![
        "chown 1000:1000 /home/agent".to_string(),
        "chmod 700 /home/agent".to_string(),
    ];
    for (_, value) in profile.env() {
        if value.strip_prefix("/home/agent/").is_some() {
            let quoted = shell::quote(value);
            commands.push(format!("mkdir -p {quoted}"));
            commands.push(format!("chown -R 1000:1000 {quoted}"));
        }
    }
    commands.join(" && ")
}

pub fn project_identifier(workspace: &Path) -> String {
    let mut digest = Sha256::new();
    digest.update(workspace.display().to_string().as_bytes());
    digest
        .finalize()
        .iter()
        .flat_map(|byte| [byte >> 4, byte & 0x0f])
        .take(16)
        .map(|n| char::from_digit(n as u32, 16).expect("hex digit"))
        .collect()
}

pub fn safe_resource_name(value: &str) -> String {
    let mut normalized = String::new();
    let mut last_dash = false;
    for c in value.to_ascii_lowercase().chars() {
        if c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '_' | '.' | '-') {
            normalized.push(c);
            last_dash = false;
        } else if !last_dash {
            normalized.push('-');
            last_dash = true;
        }
    }
    let trimmed = normalized.trim_matches('-');
    let value = if trimmed.is_empty() {
        "runhaven"
    } else {
        trimmed
    };
    value.chars().take(63).collect()
}

pub fn strip_remainder_separator(args: &[String]) -> Vec<String> {
    if args.first().map(String::as_str) == Some("--") {
        args[1..].to_vec()
    } else {
        args.to_vec()
    }
}
