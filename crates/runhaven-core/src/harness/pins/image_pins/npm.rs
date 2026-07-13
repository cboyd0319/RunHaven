use std::fs;
use std::path::Path;

use serde_json::Value as JsonValue;
use toml::Value;

use crate::harness::pins::is_exact_npm_version;

use super::pin_str;

const NPM_IMAGES: [(&str, &str, &str); 3] = [
    ("claude", "@anthropic-ai/claude-code", "claude_code"),
    ("codex", "@openai/codex", "codex"),
    ("copilot", "@github/copilot", "copilot_cli"),
];

pub(super) fn check_image_packages(root: &Path, pins: &Value) -> Vec<String> {
    let mut failures = Vec::new();
    for (image, package, pin) in NPM_IMAGES {
        let package_path = format!("images/{image}/package.json");
        let lock_path = format!("images/{image}/package-lock.json");
        let expected_version = pin_str(pins, &["agent_cli", pin]);
        let expected_integrity = pin_str(pins, &["agent_cli_integrity", pin]);
        if !is_exact_npm_version(expected_version) {
            failures.push(format!("pins.toml: agent_cli.{pin} is not exact-pinned"));
        }
        let Some(package_json) = read_json(root, &package_path, &mut failures) else {
            continue;
        };
        let dependencies = package_json
            .get("dependencies")
            .and_then(JsonValue::as_object);
        if dependencies.is_none_or(|items| {
            items.len() != 1
                || items.get(package).and_then(JsonValue::as_str) != Some(expected_version)
        }) {
            failures.push(format!(
                "{package_path}: dependencies do not match pins.toml"
            ));
        }
        for section in ["devDependencies", "optionalDependencies"] {
            if package_json
                .get(section)
                .and_then(JsonValue::as_object)
                .is_some_and(|items| !items.is_empty())
            {
                failures.push(format!("{package_path}: unexpected {section}"));
            }
        }
        check_allow_scripts(
            &package_path,
            &package_json,
            root,
            &lock_path,
            &mut failures,
        );

        let Some(lock) = read_json(root, &lock_path, &mut failures) else {
            continue;
        };
        if lock.get("lockfileVersion").and_then(JsonValue::as_u64) != Some(3) {
            failures.push(format!("{lock_path}: lockfileVersion must be 3"));
        }
        let root_dependencies = lock
            .get("packages")
            .and_then(JsonValue::as_object)
            .and_then(|packages| packages.get(""))
            .and_then(|root| root.get("dependencies"))
            .and_then(JsonValue::as_object);
        if root_dependencies.is_none_or(|items| {
            items.len() != 1
                || items.get(package).and_then(JsonValue::as_str) != Some(expected_version)
        }) {
            failures.push(format!(
                "{lock_path}: root {package} dependency does not match pins.toml"
            ));
        }
        let package_entry = lock
            .get("packages")
            .and_then(JsonValue::as_object)
            .and_then(|packages| packages.get(&format!("node_modules/{package}")));
        if package_entry
            .and_then(|entry| entry.get("version"))
            .and_then(JsonValue::as_str)
            != Some(expected_version)
        {
            failures.push(format!(
                "{lock_path}: {package} version does not match pins.toml"
            ));
        }
        if package_entry
            .and_then(|entry| entry.get("integrity"))
            .and_then(JsonValue::as_str)
            != Some(expected_integrity)
        {
            failures.push(format!(
                "{lock_path}: {package} integrity does not match pins.toml"
            ));
        }
    }
    failures
}

fn check_allow_scripts(
    package_path: &str,
    package_json: &JsonValue,
    root: &Path,
    lock_path: &str,
    failures: &mut Vec<String>,
) {
    let Some(allow_scripts) = package_json
        .get("allowScripts")
        .and_then(JsonValue::as_object)
    else {
        return;
    };
    let Some(lock) = read_json(root, lock_path, failures) else {
        return;
    };
    let packages = lock.get("packages").and_then(JsonValue::as_object);
    for entry in allow_scripts.keys() {
        let Some((name, version)) = entry.rsplit_once('@') else {
            failures.push(format!("{package_path}: invalid allowScripts key {entry}"));
            continue;
        };
        let locked = packages
            .and_then(|items| items.get(&format!("node_modules/{name}")))
            .and_then(|item| item.get("version"))
            .and_then(JsonValue::as_str);
        if locked != Some(version) {
            failures.push(format!(
                "{package_path}: allowScripts entry {entry} does not match package-lock.json"
            ));
        }
    }
}

pub(super) fn check_linux_arm64_packages(root: &Path, pins: &Value) -> Vec<String> {
    let mut failures = Vec::new();
    for image in ["claude", "codex", "copilot"] {
        let lock_path = format!("images/{image}/package-lock.json");
        let Some(lock) = read_json(root, &lock_path, &mut failures) else {
            continue;
        };
        let package = pin_str(pins, &["agent_cli_linux_arm64", image, "package"]);
        let expected_version = pin_str(pins, &["agent_cli_linux_arm64", image, "version"]);
        let expected_integrity = pin_str(pins, &["agent_cli_linux_arm64", image, "integrity"]);
        let entry = lock
            .get("packages")
            .and_then(JsonValue::as_object)
            .and_then(|packages| packages.get(&format!("node_modules/{package}")));
        if entry
            .and_then(|item| item.get("version"))
            .and_then(JsonValue::as_str)
            != Some(expected_version)
            || entry
                .and_then(|item| item.get("integrity"))
                .and_then(JsonValue::as_str)
                != Some(expected_integrity)
        {
            failures.push(format!(
                "{lock_path}: Linux arm64 package does not match pins.toml"
            ));
        }
    }
    failures
}

fn read_json(root: &Path, relative: &str, failures: &mut Vec<String>) -> Option<JsonValue> {
    match fs::read_to_string(root.join(relative))
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
    {
        Some(json) => Some(json),
        None => {
            failures.push(format!("{relative}: missing or invalid JSON"));
            None
        }
    }
}
