use std::fs;
use std::path::Path;

use serde_json::Value as JsonValue;
use toml::Value;

use super::is_exact_npm_version;
use super::toml_path;

pub(super) fn check_frontend_package(root: &Path, pins: &Value) -> Vec<String> {
    let package_json = root.join("ui/package.json");
    if !package_json.exists() {
        return Vec::new();
    }
    let mut failures = Vec::new();
    if !root.join("package-lock.json").is_file() {
        failures.push("package-lock.json: missing".to_string());
    }
    let Ok(text) = fs::read_to_string(&package_json) else {
        return vec!["ui/package.json: unreadable".to_string()];
    };
    let Ok(json) = serde_json::from_str::<JsonValue>(&text) else {
        return vec!["ui/package.json: invalid JSON".to_string()];
    };
    let release_version = toml_path(pins, &["runhaven", "version"])
        .and_then(Value::as_str)
        .unwrap_or("");
    if json.get("version").and_then(JsonValue::as_str) != Some(release_version) {
        failures.push("ui/package.json: package version does not match pins.toml".to_string());
    }
    match fs::read_to_string(root.join("package-lock.json"))
        .ok()
        .and_then(|text| serde_json::from_str::<JsonValue>(&text).ok())
    {
        Some(lock)
            if lock.get("version").and_then(JsonValue::as_str) == Some(release_version)
                && lock
                    .pointer("/packages/ui")
                    .and_then(|package| package.get("version"))
                    .and_then(JsonValue::as_str)
                    == Some(release_version) => {}
        Some(_) => failures
            .push("package-lock.json: UI package version does not match pins.toml".to_string()),
        None => failures.push("package-lock.json: invalid JSON".to_string()),
    }
    let Some(frontend_pins) = pins.get("frontend").and_then(Value::as_table) else {
        return vec!["pins.toml: missing [frontend] dependency pins".to_string()];
    };
    let mut seen = Vec::new();
    for section in ["dependencies", "devDependencies", "optionalDependencies"] {
        let Some(object) = json.get(section).and_then(JsonValue::as_object) else {
            continue;
        };
        for (name, version) in object {
            seen.push(name.as_str());
            let Some(version) = version.as_str() else {
                failures.push(format!("ui/package.json: {section}.{name} is not a string"));
                continue;
            };
            if !is_exact_npm_version(version) {
                failures.push(format!(
                    "ui/package.json: {section}.{name} is not exact-pinned"
                ));
            }
            if frontend_pins.get(name).and_then(Value::as_str) != Some(version) {
                failures.push(format!(
                    "ui/package.json: {section}.{name} does not match pins.toml"
                ));
            }
        }
    }
    for name in frontend_pins.keys() {
        if !seen.contains(&name.as_str()) {
            failures.push(format!("ui/package.json: missing pinned dependency {name}"));
        }
    }
    failures
}
