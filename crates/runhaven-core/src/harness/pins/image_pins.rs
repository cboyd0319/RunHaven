use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value as JsonValue;
use toml::Value;

use super::{is_exact_npm_version, toml_path};

pub(super) fn check_image_pins(root: &Path, pins: &Value) -> Vec<String> {
    let mut failures = Vec::new();
    failures.extend(check_containerfiles(root, pins));
    failures.extend(check_debian_packages(root));
    failures.extend(check_image_package_json(root));
    failures
}

fn check_containerfiles(root: &Path, pins: &Value) -> Vec<String> {
    let mut failures = Vec::new();
    for path in image_files(root, "Containerfile") {
        let relative = relative_path(root, &path);
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        check_base_image_digest(&mut failures, &relative, &text);
        check_expected_base_digest(&mut failures, pins, &relative, &text);
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
    if uses_node_base(relative) {
        if !text.contains(node_digest) {
            failures.push(format!(
                "{relative}: node base image digest does not match pins.toml"
            ));
        }
    } else if !text.contains(debian_digest) {
        failures.push(format!(
            "{relative}: Debian base image digest does not match pins.toml"
        ));
    }
}

fn uses_node_base(relative: &str) -> bool {
    relative.contains("/claude/")
        || relative.contains("/codex/")
        || relative.contains("/gemini/")
        || relative.contains("/copilot/")
}

fn check_debian_packages(root: &Path) -> Vec<String> {
    let mut failures = Vec::new();
    if let Ok(text) = fs::read_to_string(root.join("images/common/debian-packages.txt")) {
        for (index, line) in text.lines().enumerate() {
            let value = line.trim();
            if !value.is_empty() && !value.contains('=') {
                failures.push(format!(
                    "images/common/debian-packages.txt:{}: unpinned apt package",
                    index + 1
                ));
            }
        }
    }
    failures
}

fn check_image_package_json(root: &Path) -> Vec<String> {
    let mut failures = Vec::new();
    for package_json in image_files(root, "package.json") {
        let relative = relative_path(root, &package_json);
        let Ok(text) = fs::read_to_string(&package_json) else {
            continue;
        };
        let Ok(json) = serde_json::from_str::<JsonValue>(&text) else {
            failures.push(format!("{relative}: invalid JSON"));
            continue;
        };
        for section in ["dependencies", "devDependencies", "optionalDependencies"] {
            check_image_package_section(&mut failures, &relative, &json, section);
        }
    }
    failures
}

fn check_image_package_section(
    failures: &mut Vec<String>,
    relative: &str,
    json: &JsonValue,
    section: &str,
) {
    let Some(object) = json.get(section).and_then(JsonValue::as_object) else {
        return;
    };
    for (name, version) in object {
        let Some(version) = version.as_str() else {
            failures.push(format!("{relative}: {section}.{name} is not a string"));
            continue;
        };
        if !is_exact_npm_version(version) {
            failures.push(format!("{relative}: {section}.{name} is not exact-pinned"));
        }
    }
}

fn image_files(root: &Path, name: &str) -> Vec<PathBuf> {
    let images = root.join("images");
    let Ok(entries) = fs::read_dir(images) else {
        return Vec::new();
    };
    entries
        .flatten()
        .map(|entry| entry.path().join(name))
        .filter(|path| path.is_file())
        .collect()
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}
