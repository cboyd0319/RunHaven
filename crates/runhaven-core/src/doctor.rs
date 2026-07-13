use std::process::Command;

use anyhow::{Result, bail};

use crate::apple_container;

mod runtime_pins;

pub const PINNED_APPLE_CONTAINER_VERSION: &str = "1.1.0";

#[derive(Clone, Debug)]
pub struct Check {
    pub name: String,
    pub ok: bool,
    pub detail: String,
    pub remedy: String,
}

impl Check {
    fn new(name: &str, ok: bool, detail: impl Into<String>, remedy: &str) -> Self {
        Self {
            name: name.to_string(),
            ok,
            detail: detail.into(),
            remedy: remedy.to_string(),
        }
    }
}

pub fn collect_checks() -> Vec<Check> {
    let mut checks = Vec::new();
    checks.push(Check::new(
        "rust",
        true,
        rust_version(),
        "Use the pinned Rust 1.97.0 toolchain.",
    ));
    let os = run_capture("uname", &["-s"]).unwrap_or_else(|| "unknown".to_string());
    checks.push(Check::new(
        "operating system",
        os.trim() == "Darwin",
        os.trim(),
        "RunHaven only supports macOS 26+ on Apple silicon.",
    ));
    let mac_version =
        run_capture("sw_vers", &["-productVersion"]).unwrap_or_else(|| "unknown".to_string());
    checks.push(Check::new(
        "macOS",
        parse_major_version(mac_version.trim()).is_some_and(|major| major >= 26),
        mac_version.trim(),
        "Use a macOS 26+ host.",
    ));
    let machine = run_capture("uname", &["-m"]).unwrap_or_else(|| "unknown".to_string());
    checks.push(Check::new(
        "architecture",
        matches!(machine.trim(), "arm64" | "aarch64"),
        machine.trim(),
        "Use an Apple silicon Mac.",
    ));
    let container_path = apple_container::resolve_program().map(|path| path.display().to_string());
    checks.push(Check::new(
        "Apple container CLI",
        container_path.is_some(),
        container_path
            .as_deref()
            .unwrap_or("not found on PATH")
            .to_string(),
        "In the desktop app, choose Download Apple container. For the CLI, install Apple's signed 1.1.0 package, then run `container system start`.",
    ));
    if container_path.is_some() {
        checks.push(container_version_check());
        checks.push(container_status_check());
        checks.extend(runtime_pins::container_runtime_pin_checks());
    }
    checks
}

/// Recheck the supported host and pinned Apple container runtime immediately
/// before a real launch or image mutation. This is read-only and deliberately
/// does not cache safety-critical runtime state across operations.
pub fn require_supported_runtime() -> Result<()> {
    supported_runtime_result(&collect_checks())
}

fn supported_runtime_result(checks: &[Check]) -> Result<()> {
    let Some(failed) = checks.iter().find(|check| !check.ok) else {
        return Ok(());
    };
    bail!(
        "RunHaven cannot start this operation because the supported runtime check failed: {}. {} Run `runhaven doctor` for all checks.",
        failed.name,
        failed.remedy
    )
}

pub fn parse_major_version(value: &str) -> Option<u32> {
    value.split('.').next()?.parse().ok()
}

pub fn container_version_check() -> Check {
    match apple_container::command()
        .and_then(|mut command| Ok(command.arg("--version").output()?))
    {
        Ok(output) => {
            let text = if output.stdout.is_empty() {
                &output.stderr
            } else {
                &output.stdout
            };
            let detail = String::from_utf8_lossy(text).trim().to_string();
            let version = parse_container_cli_version(&detail);
            Check::new(
                "Apple container version",
                output.status.success()
                    && version.as_deref() == Some(PINNED_APPLE_CONTAINER_VERSION),
                format!(
                    "{}; expected {PINNED_APPLE_CONTAINER_VERSION}",
                    if detail.is_empty() {
                        output.status.to_string()
                    } else {
                        detail
                    }
                ),
                "Install Apple's signed Apple container 1.1.0 package, then check again.",
            )
        }
        Err(error) => Check::new(
            "Apple container version",
            false,
            error.to_string(),
            "Check `container --version`.",
        ),
    }
}

pub fn parse_container_version(value: &str) -> Option<String> {
    let re = regex::Regex::new(r"\b(\d+\.\d+\.\d+)\b").ok()?;
    re.captures(value)
        .and_then(|captures| captures.get(1))
        .map(|m| m.as_str().to_string())
}

pub fn parse_container_cli_version(value: &str) -> Option<String> {
    let re =
        regex::Regex::new(r"^container CLI version (\d+\.\d+\.\d+)(?: \([^\r\n]*\))?$").ok()?;
    re.captures(value.trim())
        .and_then(|captures| captures.get(1))
        .map(|version| version.as_str().to_string())
}

pub fn container_status_check() -> Check {
    match apple_container::command()
        .and_then(|mut command| Ok(command.args(["system", "status"]).output()?))
    {
        Ok(output) => {
            let text = if output.stdout.is_empty() {
                &output.stderr
            } else {
                &output.stdout
            };
            let detail = String::from_utf8_lossy(text).trim().to_string();
            Check::new(
                "container system",
                output.status.success(),
                if detail.is_empty() {
                    output.status.to_string()
                } else {
                    detail
                },
                "Run `container system start`.",
            )
        }
        Err(error) => Check::new(
            "container system",
            false,
            error.to_string(),
            "Run `container system start`.",
        ),
    }
}

fn rust_version() -> String {
    run_capture("rustc", &["--version"]).unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string())
}

fn run_capture(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program).args(args).output().ok()?;
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn find_on_path(program: &str) -> Option<String> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(program);
        if candidate.is_file() {
            return Some(candidate.display().to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_runtime_result_is_plain_and_actionable() {
        let checks = [Check::new(
            "Apple container runtime commit",
            false,
            "raw host path and subprocess detail",
            "Install the reviewed Apple container release.",
        )];

        let error = supported_runtime_result(&checks).expect_err("failed runtime check");
        let message = error.to_string();

        assert!(message.contains("Apple container runtime commit"));
        assert!(message.contains("Install the reviewed Apple container release."));
        assert!(message.contains("runhaven doctor"));
        assert!(message.contains("cannot start this operation"));
        assert!(!message.contains("raw host path"));
    }

    #[test]
    fn supported_runtime_result_accepts_all_green_checks() {
        let checks = [Check::new("container system", true, "running", "start it")];

        supported_runtime_result(&checks).expect("supported runtime");
    }

    #[test]
    fn cli_version_parser_accepts_only_apples_version_line() {
        assert_eq!(
            parse_container_cli_version(
                "container CLI version 1.0.0 (build: release, commit: ee848e3)"
            )
            .as_deref(),
            Some("1.0.0")
        );
        assert_eq!(
            parse_container_cli_version("warning 9.9.9; container CLI version 1.0.0"),
            None
        );
        assert_eq!(parse_container_cli_version("container 1.0.0"), None);
    }
}
