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
    failures.extend(check_tauri_cargo_against_ledger(&root, &pins));
    failures.extend(check_frontend_package_against_ledger(&root, &pins));
    failures.extend(check_ci_against_ledger(&root, &pins));
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

fn check_frontend_package_against_ledger(root: &Path, pins: &Value) -> Vec<String> {
    let package_json = root.join("ui/package.json");
    if !package_json.exists() {
        return Vec::new();
    }
    let mut failures = Vec::new();
    if !root.join("ui/package-lock.json").is_file() {
        failures.push("ui/package-lock.json: missing".to_string());
    }
    let Ok(text) = fs::read_to_string(&package_json) else {
        return vec!["ui/package.json: unreadable".to_string()];
    };
    let Ok(json) = serde_json::from_str::<JsonValue>(&text) else {
        return vec!["ui/package.json: invalid JSON".to_string()];
    };
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
        root.join("src-tauri/Cargo.toml"),
        root.join("ui/package.json"),
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
                    if !is_exact_npm_version(version) {
                        failures.push(format!("{relative}: {section}.{name} is not exact-pinned"));
                    }
                }
            }
        }
    }
    failures
}

fn is_exact_npm_version(version: &str) -> bool {
    !(version.starts_with('^')
        || version.starts_with('~')
        || version.contains('*')
        || version.contains(">=")
        || version.contains("<=")
        || version.contains(">")
        || version.contains("<")
        || version == "latest")
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

fn check_script_descriptions(root: &Path) -> Vec<String> {
    let mut failures = Vec::new();
    for path in maintained_script_files(root) {
        let relative = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .display()
            .to_string();
        let Ok(text) = fs::read_to_string(&path) else {
            failures.push(format!("{relative}: unreadable maintained script"));
            continue;
        };
        if !has_top_description(&text, path.file_name().and_then(|name| name.to_str())) {
            failures.push(format!(
                "{relative}: add two top comment lines describing what this file is and what it does"
            ));
        }
    }
    failures
}

fn maintained_script_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let init = root.join("init.sh");
    if init.is_file() {
        files.push(init);
    }
    files.extend(files_in(root.join("scripts"), |path| {
        path.extension().and_then(|value| value.to_str()) == Some("sh")
    }));
    files.extend(files_in(root.join("images"), |path| {
        path.file_name().and_then(|value| value.to_str()) == Some("Containerfile")
            || path.extension().and_then(|value| value.to_str()) == Some("sh")
    }));
    files.sort();
    files.dedup();
    files
}

fn files_in(dir: PathBuf, keep: impl Fn(&Path) -> bool + Copy) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            files.extend(files_in(path, keep));
        } else if path.is_file() && keep(&path) {
            files.push(path);
        }
    }
    files
}

fn has_top_description(text: &str, file_name: Option<&str>) -> bool {
    let mut lines = text.lines();
    if file_name != Some("Containerfile") && lines.next().is_none_or(|line| !line.starts_with("#!"))
    {
        return false;
    }
    lines
        .take(3)
        .filter(|line| is_description_line(line))
        .count()
        >= 2
}

fn is_description_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("# ") && trimmed.trim().len() > 10
}

fn toml_path<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    Some(current)
}
