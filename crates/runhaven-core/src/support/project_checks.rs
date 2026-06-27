use std::fs;
use std::path::Path;

use serde::Serialize;
use serde_json::Value;

use crate::support::shell;

#[derive(Clone, Debug, Serialize)]
pub struct SuggestedCheck {
    pub label: String,
    pub command: String,
    pub argv: Vec<String>,
    pub reason: String,
}

pub fn suggest_project_checks(workspace: &Path) -> Vec<SuggestedCheck> {
    let mut suggestions = Vec::new();
    let scripts = package_json_scripts(&workspace.join("package.json"));
    if script_is_real(scripts.get("test").map(String::as_str)) {
        suggestions.push(runhaven_shell_check(
            workspace,
            "Node tests",
            &["npm", "test"],
            "package.json defines scripts.test",
        ));
    }
    if script_is_real(scripts.get("lint").map(String::as_str)) {
        suggestions.push(runhaven_shell_check(
            workspace,
            "Node lint",
            &["npm", "run", "lint"],
            "package.json defines scripts.lint",
        ));
    }
    if workspace.join("tests").is_dir() {
        suggestions.push(runhaven_shell_check(
            workspace,
            "Python tests",
            &["python", "-m", "unittest", "discover", "-s", "tests"],
            "tests directory exists",
        ));
    }
    if ruff_config_exists(workspace) {
        suggestions.push(runhaven_shell_check(
            workspace,
            "Python lint",
            &["python", "-m", "ruff", "check", "."],
            "Ruff configuration detected",
        ));
    }
    suggestions
}

pub fn package_json_scripts(path: &Path) -> std::collections::BTreeMap<String, String> {
    let Ok(text) = fs::read_to_string(path) else {
        return Default::default();
    };
    let Ok(Value::Object(root)) = serde_json::from_str::<Value>(&text) else {
        return Default::default();
    };
    let Some(Value::Object(scripts)) = root.get("scripts") else {
        return Default::default();
    };
    scripts
        .iter()
        .filter_map(|(key, value)| value.as_str().map(|value| (key.clone(), value.to_string())))
        .collect()
}

pub fn script_is_real(script: Option<&str>) -> bool {
    let Some(script) = script else {
        return false;
    };
    let lowered = script.trim().to_ascii_lowercase();
    !lowered.is_empty() && !lowered.contains("no test specified")
}

pub fn ruff_config_exists(workspace: &Path) -> bool {
    if workspace.join("ruff.toml").is_file() || workspace.join(".ruff.toml").is_file() {
        return true;
    }
    let pyproject = workspace.join("pyproject.toml");
    fs::read_to_string(pyproject).is_ok_and(|text| text.contains("[tool.ruff"))
}

pub fn runhaven_shell_check(
    workspace: &Path,
    label: &str,
    tool_args: &[&str],
    reason: &str,
) -> SuggestedCheck {
    let mut argv = vec![
        "runhaven".to_string(),
        "run".to_string(),
        "shell".to_string(),
        "--workspace".to_string(),
        workspace.display().to_string(),
        "--network".to_string(),
        "internal".to_string(),
        "--".to_string(),
    ];
    argv.extend(tool_args.iter().map(|value| (*value).to_string()));
    SuggestedCheck {
        label: label.to_string(),
        command: shell::join(&argv),
        argv,
        reason: reason.to_string(),
    }
}
