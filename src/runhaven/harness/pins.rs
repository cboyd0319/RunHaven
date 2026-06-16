use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use serde_json::Value as JsonValue;
use toml::Value;

pub fn check_pins() -> Result<()> {
    let root = repo_root();
    let pins = load_pins(&root)?;
    let mut failures = Vec::new();
    failures.extend(check_cargo_against_ledger(&root, &pins));
    failures.extend(check_ci_against_ledger(&root, &pins));
    failures.extend(check_text_policy(&root));
    failures.extend(check_image_pins(&root, &pins));
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

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
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
    if toml_path(&cargo, &["package", "version"]).and_then(Value::as_str) != Some(version) {
        failures.push("Cargo.toml: package version does not match pins.toml".to_string());
    }
    let Some(deps) = cargo.get("dependencies").and_then(Value::as_table) else {
        failures.push("Cargo.toml: missing dependencies".to_string());
        return failures;
    };
    let Some(rust_pins) = pins.get("rust").and_then(Value::as_table) else {
        failures.push("pins.toml: missing [rust] dependency pins".to_string());
        return failures;
    };
    for (name, pinned) in rust_pins {
        if matches!(name.as_str(), "toolchain" | "edition" | "tempfile") {
            continue;
        }
        let expected = pinned.as_str().unwrap_or_default();
        let actual = deps.get(name).and_then(dependency_version);
        if actual != Some(format!("={expected}")) {
            failures.push(format!(
                "Cargo.toml: dependency {name} must be pinned as ={expected}"
            ));
        }
    }
    let Some(dev_deps) = cargo.get("dev-dependencies").and_then(Value::as_table) else {
        return failures;
    };
    if let Some(expected) = rust_pins.get("tempfile").and_then(Value::as_str)
        && dev_deps.get("tempfile").and_then(dependency_version) != Some(format!("={expected}"))
    {
        failures.push(format!(
            "Cargo.toml: dev dependency tempfile must be pinned as ={expected}"
        ));
    }
    failures
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

fn check_ci_against_ledger(root: &Path, pins: &Value) -> Vec<String> {
    let mut failures = Vec::new();
    let workflow_files = workflow_files(root);
    if workflow_files.is_empty() {
        return failures;
    }
    let macos = toml_path(pins, &["github_runners", "macos"])
        .and_then(Value::as_str)
        .unwrap_or("");
    let toolchain = toml_path(pins, &["rust", "toolchain"])
        .and_then(Value::as_str)
        .unwrap_or("");
    let sha = regex::Regex::new(r"^[0-9a-f]{40}$").unwrap();
    let action_ref = regex::Regex::new(r"uses:\s*[\w./-]+@([^\s#]+)").unwrap();
    for path in workflow_files {
        let relative = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .display()
            .to_string();
        let Ok(text) = fs::read_to_string(&path) else {
            failures.push(format!("{relative}: unreadable workflow file"));
            continue;
        };
        if macos.is_empty() {
            failures.push(format!(
                "{relative}: active workflow requires pins.toml [github_runners].macos"
            ));
        } else if !text.contains(macos) {
            failures.push(format!("{relative}: macOS runner does not match pins.toml"));
        }
        let lower = text.to_ascii_lowercase();
        if lower.contains("ubuntu") || lower.contains("windows") {
            failures.push(format!("{relative}: CI must run only on macOS 26+"));
        }
        if toolchain.is_empty() || !text.contains(toolchain) {
            failures.push(format!(
                "{relative}: Rust toolchain does not match pins.toml"
            ));
        }
        for capture in action_ref.captures_iter(&text) {
            if !sha.is_match(&capture[1]) {
                failures.push(format!(
                    "{relative}: GitHub Action ref is not an immutable SHA"
                ));
            }
        }
    }
    failures
}

fn workflow_files(root: &Path) -> Vec<PathBuf> {
    let workflows = root.join(".github/workflows");
    let Ok(entries) = fs::read_dir(workflows) else {
        return Vec::new();
    };
    entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| matches!(extension, "yml" | "yaml"))
        })
        .collect()
}

fn check_text_policy(root: &Path) -> Vec<String> {
    let mut failures = Vec::new();
    let mut files = [
        root.join("Cargo.toml"),
        root.join("images/common/debian-packages.txt"),
        root.join("images/common/debian.sources"),
    ]
    .into_iter()
    .collect::<Vec<_>>();
    files.extend(workflow_files(root));
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

fn check_image_pins(root: &Path, pins: &Value) -> Vec<String> {
    let mut failures = Vec::new();
    for path in image_files(root, "Containerfile") {
        let relative = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .display()
            .to_string();
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        for (index, line) in text.lines().enumerate() {
            let value = line.trim();
            if value.starts_with("FROM ") && !value.contains("@sha256:") {
                failures.push(format!(
                    "{relative}:{}: base image is not digest-pinned",
                    index + 1
                ));
            }
        }
        let node_digest = toml_path(pins, &["container_images", "node_26_trixie_slim", "digest"])
            .and_then(Value::as_str)
            .unwrap_or("");
        let debian_digest = toml_path(pins, &["container_images", "debian_trixie_slim", "digest"])
            .and_then(Value::as_str)
            .unwrap_or("");
        if relative.contains("/claude/")
            || relative.contains("/codex/")
            || relative.contains("/gemini/")
            || relative.contains("/copilot/")
        {
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
    for package_json in image_files(root, "package.json") {
        let relative = package_json
            .strip_prefix(root)
            .unwrap_or(&package_json)
            .display()
            .to_string();
        let Ok(text) = fs::read_to_string(&package_json) else {
            continue;
        };
        let Ok(json) = serde_json::from_str::<JsonValue>(&text) else {
            failures.push(format!("{relative}: invalid JSON"));
            continue;
        };
        for section in ["dependencies", "devDependencies", "optionalDependencies"] {
            if let Some(object) = json.get(section).and_then(JsonValue::as_object) {
                for (name, version) in object {
                    let Some(version) = version.as_str() else {
                        failures.push(format!("{relative}: {section}.{name} is not a string"));
                        continue;
                    };
                    if version.starts_with('^')
                        || version.starts_with('~')
                        || version.contains('*')
                        || version.contains(">=")
                    {
                        failures.push(format!("{relative}: {section}.{name} is not exact-pinned"));
                    }
                }
            }
        }
    }
    failures
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

fn toml_path<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    Some(current)
}
