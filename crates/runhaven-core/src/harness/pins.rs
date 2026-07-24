use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use toml::Value;

mod frontend;
mod image_pins;
mod script_descriptions;

use frontend::check_frontend_package;
use image_pins::check_image_pins;
use script_descriptions::check_script_descriptions;

pub fn check_pins() -> Result<()> {
    let root = repo_root();
    let pins = load_pins(&root)?;
    let mut failures = Vec::new();
    failures.extend(check_rust_toolchain(&root, &pins));
    failures.extend(check_apple_container_constants(&pins));
    failures.extend(check_cargo_against_ledger(&root, &pins));
    failures.extend(check_tauri_cargo_against_ledger(&root, &pins));
    failures.extend(check_frontend_package(&root, &pins));
    failures.extend(check_text_policy(&root));
    failures.extend(check_image_pins(&root, &pins));
    failures.extend(check_script_descriptions(&root));
    if failures.is_empty() {
        println!("Pin policy passed");
        return Ok(());
    }
    println!("Pin policy failures:");
    for failure in failures {
        println!("  {failure}");
    }
    bail!("pin policy failed");
}

fn check_apple_container_constants(pins: &Value) -> Vec<String> {
    let mut failures = Vec::new();
    if toml_path(pins, &["apple_container", "version"]).and_then(Value::as_str)
        != Some(crate::doctor::PINNED_APPLE_CONTAINER_VERSION)
    {
        failures.push("doctor Apple container version does not match pins.toml".to_string());
    }
    if toml_path(pins, &["apple_container", "installer_url"]).and_then(Value::as_str)
        != Some(crate::apple_container::INSTALLER_URL)
    {
        failures.push("Apple container installer URL does not match pins.toml".to_string());
    }
    failures
}

fn check_rust_toolchain(root: &Path, pins: &Value) -> Vec<String> {
    let path = "rust-toolchain.toml";
    let expected = toml_path(pins, &["rust", "toolchain"])
        .and_then(Value::as_str)
        .unwrap_or("");
    let Some(toolchain) = fs::read_to_string(root.join(path))
        .ok()
        .and_then(|text| toml::from_str::<Value>(&text).ok())
    else {
        return vec![format!("{path}: missing or invalid TOML")];
    };
    if toml_path(&toolchain, &["toolchain", "channel"]).and_then(Value::as_str) == Some(expected) {
        Vec::new()
    } else {
        vec![format!("{path}: channel does not match pins.toml")]
    }
}

fn repo_root() -> PathBuf {
    let mut current = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    loop {
        if current.join("pins.toml").is_file() {
            return current;
        }
        if !current.pop() {
            return PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        }
    }
}

fn load_pins(root: &Path) -> Result<Value> {
    Ok(toml::from_str::<Value>(&fs::read_to_string(
        root.join("pins.toml"),
    )?)?)
}

fn check_cargo_against_ledger(root: &Path, pins: &Value) -> Vec<String> {
    let mut failures = Vec::new();
    let Ok(text) = fs::read_to_string(root.join("Cargo.toml")) else {
        return vec!["Cargo.toml: missing".to_string()];
    };
    let Ok(cargo) = toml::from_str::<Value>(&text) else {
        return vec!["Cargo.toml: invalid TOML".to_string()];
    };
    let version = toml_path(pins, &["runhaven", "version"])
        .and_then(Value::as_str)
        .unwrap_or("");
    if toml_path(&cargo, &["workspace", "package", "version"]).and_then(Value::as_str)
        != Some(version)
    {
        failures.push("Cargo.toml: workspace package version does not match pins.toml".to_string());
    }
    let Some(deps) = toml_path(&cargo, &["workspace", "dependencies"]).and_then(Value::as_table)
    else {
        failures.push("Cargo.toml: missing workspace dependencies".to_string());
        return failures;
    };
    let Some(rust_pins) = pins.get("rust").and_then(Value::as_table) else {
        failures.push("pins.toml: missing [rust] dependency pins".to_string());
        return failures;
    };
    for (name, pinned) in rust_pins {
        if name == "toolchain" || name == "edition" {
            continue;
        }
        let expected = pinned.as_str().unwrap_or_default();
        let actual = deps.get(name).and_then(dependency_version);
        if actual != Some(format!("={expected}")) {
            failures.push(format!(
                "Cargo.toml: workspace dependency {name} must be pinned as ={expected}"
            ));
        }
    }
    failures
}

fn check_tauri_cargo_against_ledger(root: &Path, pins: &Value) -> Vec<String> {
    let path = root.join("src-tauri/Cargo.toml");
    if !path.exists() {
        return Vec::new();
    }
    let relative = "src-tauri/Cargo.toml";
    let Ok(text) = fs::read_to_string(&path) else {
        return vec![format!("{relative}: unreadable")];
    };
    let Ok(cargo) = toml::from_str::<Value>(&text) else {
        return vec![format!("{relative}: invalid TOML")];
    };
    let Ok(root_text) = fs::read_to_string(root.join("Cargo.toml")) else {
        return vec!["Cargo.toml: unreadable".to_string()];
    };
    let Ok(root_cargo) = toml::from_str::<Value>(&root_text) else {
        return vec!["Cargo.toml: invalid TOML".to_string()];
    };
    let tauri_pins = pins.get("tauri").and_then(Value::as_table);
    let rust_pins = pins.get("rust").and_then(Value::as_table);
    let mut failures = Vec::new();
    for (section, name, source) in [
        ("dependencies", "tauri", tauri_pins),
        ("dependencies", "tauri-plugin-dialog", tauri_pins),
        ("dependencies", "tauri-plugin-window-state", tauri_pins),
        ("dependencies", "tauri-plugin-log", tauri_pins),
        ("dependencies", "log", rust_pins),
        ("dependencies", "serde", rust_pins),
        ("dependencies", "serde_json", rust_pins),
        ("build-dependencies", "tauri-build", tauri_pins),
        ("dev-dependencies", "tempfile", rust_pins),
    ] {
        let expected = source
            .and_then(|table| table.get(name))
            .and_then(Value::as_str)
            .unwrap_or("");
        let actual = dependency_version_from_member(&cargo, &root_cargo, section, name);
        if actual != Some(format!("={expected}")) {
            failures.push(format!(
                "{relative}: {section}.{name} must be pinned as ={expected}"
            ));
        }
    }
    failures
}

fn dependency_version_from_member(
    member: &Value,
    root: &Value,
    section: &str,
    name: &str,
) -> Option<String> {
    let dependency = member
        .get(section)
        .and_then(Value::as_table)
        .and_then(|deps| deps.get(name))?;
    if dependency
        .as_table()
        .and_then(|table| table.get("workspace"))
        .and_then(Value::as_bool)
        == Some(true)
    {
        return toml_path(root, &["workspace", "dependencies"])
            .and_then(Value::as_table)
            .and_then(|deps| deps.get(name))
            .and_then(dependency_version);
    }
    dependency_version(dependency)
}

fn dependency_version(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => Some(value.clone()),
        Value::Table(table) => table
            .get("version")
            .and_then(Value::as_str)
            .map(str::to_string),
        _ => None,
    }
}

fn check_text_policy(root: &Path) -> Vec<String> {
    let mut failures = Vec::new();
    let files = [
        root.join("Cargo.toml"),
        root.join("src-tauri/Cargo.toml"),
        root.join("ui/package.json"),
        root.join("images/common/debian-packages.txt"),
        root.join("images/common/debian.sources"),
    ]
    .into_iter()
    .collect::<Vec<_>>();
    for path in files {
        let relative = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .display()
            .to_string();
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        for (index, line) in text.lines().enumerate() {
            if line.contains("latest") {
                failures.push(format!("{relative}:{}: mutable latest tag", index + 1));
            }
            if line.contains("npm install") && !line.contains('@') {
                failures.push(format!("{relative}:{}: unpinned npm install", index + 1));
            }
        }
    }
    failures
}

pub(super) fn is_exact_npm_version(version: &str) -> bool {
    !(version.starts_with('^')
        || version.starts_with('~')
        || version.contains('*')
        || version.contains(">=")
        || version.contains("<=")
        || version.contains(">")
        || version.contains("<")
        || version == "latest")
}

pub(super) fn toml_path<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    Some(current)
}
