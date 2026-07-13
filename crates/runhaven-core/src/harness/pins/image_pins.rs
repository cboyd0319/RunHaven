use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use toml::Value;

use super::toml_path;

mod npm;

use npm::{check_image_packages, check_linux_arm64_packages};
const BUNDLED_IMAGES: [&str; 5] = ["base", "claude", "codex", "antigravity", "copilot"];

pub(super) fn check_image_pins(root: &Path, pins: &Value) -> Vec<String> {
    let mut failures = check_image_inventory(root);
    failures.extend(check_pin_inventory(pins));
    failures.extend(check_required_pins(pins));
    failures.extend(check_containerfiles(root, pins));
    failures.extend(check_runtime_volume_prep_image(root, pins));
    failures.extend(check_debian_snapshot(root, pins));
    failures.extend(check_debian_packages(root, pins));
    failures.extend(check_image_packages(root, pins));
    failures.extend(check_linux_arm64_packages(root, pins));
    failures.extend(check_antigravity(root, pins));
    failures
}

fn check_image_inventory(root: &Path) -> Vec<String> {
    let expected = BUNDLED_IMAGES
        .into_iter()
        .chain(["common"])
        .collect::<BTreeSet<_>>();
    let Ok(entries) = fs::read_dir(root.join("images")) else {
        return vec!["images: unreadable".to_string()];
    };
    entries
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| !expected.contains(name.as_str()))
        .map(|name| format!("unexpected bundled image directory {name}"))
        .collect()
}

fn check_pin_inventory(pins: &Value) -> Vec<String> {
    let expected = [
        (
            "agent_cli",
            [
                "antigravity_cli",
                "claude_code",
                "codex",
                "copilot_cli",
                "npm",
            ]
            .as_slice(),
        ),
        (
            "agent_cli_integrity",
            ["antigravity_cli", "claude_code", "codex", "copilot_cli"].as_slice(),
        ),
        (
            "agent_cli_linux_arm64",
            ["claude", "codex", "copilot"].as_slice(),
        ),
    ];
    let mut failures = Vec::new();
    for (table_name, expected_keys) in expected {
        let expected_keys = expected_keys.iter().copied().collect::<BTreeSet<_>>();
        let Some(table) = pins.get(table_name).and_then(Value::as_table) else {
            continue;
        };
        failures.extend(
            table
                .keys()
                .filter(|key| !expected_keys.contains(key.as_str()))
                .map(|key| format!("unexpected {table_name} pin {key}")),
        );
    }
    failures
}

fn check_required_pins(pins: &Value) -> Vec<String> {
    let paths: &[&[&str]] = &[
        &["container_images", "debian_trixie_slim", "tag"],
        &["container_images", "debian_trixie_slim", "digest"],
        &["container_images", "node_26_trixie_slim", "tag"],
        &["container_images", "node_26_trixie_slim", "digest"],
        &["node_runtime", "npm"],
        &["debian_snapshot", "timestamp"],
        &["debian_snapshot", "debian_uri"],
        &["debian_snapshot", "security_uri"],
        &["agent_cli", "claude_code"],
        &["agent_cli", "codex"],
        &["agent_cli", "antigravity_cli"],
        &["agent_cli", "copilot_cli"],
        &["agent_cli", "npm"],
        &["agent_cli_integrity", "claude_code"],
        &["agent_cli_integrity", "codex"],
        &["agent_cli_integrity", "antigravity_cli"],
        &["agent_cli_integrity", "copilot_cli"],
        &["agent_cli_source", "antigravity_url"],
        &["agent_cli_linux_arm64", "claude", "package"],
        &["agent_cli_linux_arm64", "claude", "version"],
        &["agent_cli_linux_arm64", "claude", "integrity"],
        &["agent_cli_linux_arm64", "codex", "package"],
        &["agent_cli_linux_arm64", "codex", "version"],
        &["agent_cli_linux_arm64", "codex", "integrity"],
        &["agent_cli_linux_arm64", "copilot", "package"],
        &["agent_cli_linux_arm64", "copilot", "version"],
        &["agent_cli_linux_arm64", "copilot", "integrity"],
    ];
    paths
        .iter()
        .filter(|path| pin_str(pins, path).is_empty())
        .map(|path| format!("pins.toml: missing {}", path.join(".")))
        .collect()
}

fn check_runtime_volume_prep_image(root: &Path, pins: &Value) -> Vec<String> {
    let tag = pin_str(pins, &["container_images", "debian_trixie_slim", "tag"]);
    let digest = pin_str(pins, &["container_images", "debian_trixie_slim", "digest"]);
    let expected = format!("debian:{tag}@{digest}");
    let path = root.join("crates/runhaven-core/src/runtime/plans/types.rs");
    match fs::read_to_string(path) {
        Ok(text)
            if text
                .split("pub const VOLUME_PREP_IMAGE")
                .nth(1)
                .and_then(|declaration| declaration.split(';').next())
                .is_some_and(|declaration| declaration.contains(&format!("\"{expected}\""))) =>
        {
            Vec::new()
        }
        Ok(_) => vec!["runtime volume-prep image does not match pins.toml".to_string()],
        Err(_) => vec!["runtime volume-prep image definition is unreadable".to_string()],
    }
}

fn check_containerfiles(root: &Path, pins: &Value) -> Vec<String> {
    let mut failures = Vec::new();
    for image in BUNDLED_IMAGES {
        let relative = format!("images/{image}/Containerfile");
        let path = root.join(&relative);
        let Ok(text) = fs::read_to_string(&path) else {
            failures.push(format!("{relative}: unreadable"));
            continue;
        };
        check_base_image_digest(&mut failures, &relative, &text);
        check_expected_base_digest(&mut failures, pins, &relative, &text);
        for required in [
            "COPY common/debian.sources ",
            "COPY common/debian-packages.txt ",
            "apt-get update",
            "xargs -a /tmp/debian-packages.txt apt-get install",
            "USER agent",
        ] {
            if !text.contains(required) {
                failures.push(format!(
                    "{relative}: missing required build step {required}"
                ));
            }
        }
        if uses_node_base(&relative) {
            let npm = pin_str(pins, &["node_runtime", "npm"]);
            if !text.contains(&format!("npm@{npm}")) {
                failures.push(format!("{relative}: npm version does not match pins.toml"));
            }
            for required in [
                format!("COPY {image}/package.json {image}/package-lock.json "),
                "npm ci ".to_string(),
                "--strict-allow-scripts=true".to_string(),
            ] {
                if !text.contains(&required) {
                    failures.push(format!(
                        "{relative}: missing required build step {required}"
                    ));
                }
            }
        }
    }
    let agent_npm = pin_str(pins, &["agent_cli", "npm"]);
    let runtime_npm = pin_str(pins, &["node_runtime", "npm"]);
    if agent_npm != runtime_npm {
        failures.push("pins.toml: agent_cli.npm does not match node_runtime.npm".to_string());
    }
    failures
}

fn check_base_image_digest(failures: &mut Vec<String>, relative: &str, text: &str) {
    for (index, line) in text.lines().enumerate() {
        let value = line.trim();
        if value.starts_with("FROM ") && !value.contains("@sha256:") {
            failures.push(format!(
                "{relative}:{}: base image is not digest-pinned",
                index + 1
            ));
        }
    }
}

fn check_expected_base_digest(
    failures: &mut Vec<String>,
    pins: &Value,
    relative: &str,
    text: &str,
) {
    let node_digest = toml_path(pins, &["container_images", "node_26_trixie_slim", "digest"])
        .and_then(Value::as_str)
        .unwrap_or("");
    let debian_digest = toml_path(pins, &["container_images", "debian_trixie_slim", "digest"])
        .and_then(Value::as_str)
        .unwrap_or("");
    let from: Vec<_> = text
        .lines()
        .filter_map(|line| line.trim().strip_prefix("FROM "))
        .collect();
    let expected = if uses_node_base(relative) {
        let tag = pin_str(pins, &["container_images", "node_26_trixie_slim", "tag"]);
        format!("node:{tag}@{node_digest}")
    } else {
        let tag = pin_str(pins, &["container_images", "debian_trixie_slim", "tag"]);
        format!("debian:{tag}@{debian_digest}")
    };
    if from != [expected.as_str()] {
        failures.push(format!(
            "{relative}: base image tag or digest does not match pins.toml"
        ));
    }
}

fn uses_node_base(relative: &str) -> bool {
    relative.contains("/claude/") || relative.contains("/codex/") || relative.contains("/copilot/")
}

fn check_debian_snapshot(root: &Path, pins: &Value) -> Vec<String> {
    let path = "images/common/debian.sources";
    let Ok(text) = fs::read_to_string(root.join(path)) else {
        return vec![format!("{path}: unreadable")];
    };
    let actual: Vec<_> = text
        .lines()
        .filter_map(|line| line.strip_prefix("URIs: "))
        .collect();
    let expected = [
        pin_str(pins, &["debian_snapshot", "debian_uri"]),
        pin_str(pins, &["debian_snapshot", "security_uri"]),
    ];
    let timestamp = pin_str(pins, &["debian_snapshot", "timestamp"]);
    if actual != expected || expected.iter().any(|uri| !uri.ends_with(timestamp)) {
        return vec![format!("{path}: snapshot URIs do not match pins.toml")];
    }
    Vec::new()
}

fn check_debian_packages(root: &Path, pins: &Value) -> Vec<String> {
    let mut failures = Vec::new();
    let path = "images/common/debian-packages.txt";
    let Some(expected) = pins.get("debian_trixie_arm64").and_then(Value::as_table) else {
        return vec!["pins.toml: missing [debian_trixie_arm64]".to_string()];
    };
    let Ok(text) = fs::read_to_string(root.join(path)) else {
        return vec![format!("{path}: unreadable")];
    };
    let mut seen = BTreeSet::new();
    for (index, line) in text.lines().enumerate() {
        let value = line.trim();
        if value.is_empty() {
            continue;
        }
        let Some((name, version)) = value.split_once('=') else {
            failures.push(format!("{path}:{}: unpinned apt package", index + 1));
            continue;
        };
        if name.is_empty() || version.is_empty() {
            failures.push(format!("{path}:{}: invalid package pin", index + 1));
            continue;
        }
        let key = debian_pin_key(name);
        if !seen.insert(key.clone()) {
            failures.push(format!("{path}:{}: duplicate package {name}", index + 1));
        }
        if expected.get(&key).and_then(Value::as_str) != Some(version) {
            failures.push(format!(
                "{path}:{}: {name} does not match pins.toml",
                index + 1
            ));
        }
    }
    for key in expected.keys() {
        if !seen.contains(key) {
            failures.push(format!("{path}: missing pinned package {key}"));
        }
    }
    failures
}

fn debian_pin_key(name: &str) -> String {
    name.replace(['-', '.'], "_")
}

fn check_antigravity(root: &Path, pins: &Value) -> Vec<String> {
    let path = "images/antigravity/Containerfile";
    let Ok(text) = fs::read_to_string(root.join(path)) else {
        return vec![format!("{path}: unreadable")];
    };
    let version = pin_str(pins, &["agent_cli", "antigravity_cli"]);
    let url = pin_str(pins, &["agent_cli_source", "antigravity_url"]);
    let integrity = pin_str(pins, &["agent_cli_integrity", "antigravity_cli"])
        .strip_prefix("sha512-")
        .unwrap_or("");
    let mut failures: Vec<_> = [
        (format!("ARG ANTIGRAVITY_CLI_VERSION={version}"), "version"),
        (format!("ARG ANTIGRAVITY_CLI_URL={url}"), "URL"),
        (format!("ARG ANTIGRAVITY_CLI_SHA512={integrity}"), "SHA-512"),
    ]
    .into_iter()
    .filter_map(|(expected, label)| {
        (!text.lines().any(|line| line == expected))
            .then(|| format!("{path}: Antigravity {label} does not match pins.toml"))
    })
    .collect();
    if !text.contains("${ANTIGRAVITY_CLI_SHA512}  /tmp/antigravity-cli.tar.gz\" | sha512sum -c -") {
        failures.push(format!(
            "{path}: Antigravity archive checksum is not enforced"
        ));
    }
    failures
}

fn pin_str<'a>(pins: &'a Value, path: &[&str]) -> &'a str {
    toml_path(pins, path).and_then(Value::as_str).unwrap_or("")
}

#[cfg(test)]
mod tests;
