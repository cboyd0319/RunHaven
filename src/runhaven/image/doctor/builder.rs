use std::process::Command;

use anyhow::{Result, bail};
use serde_json::Value;

use super::BuilderStatusSummary;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum BuilderDiagnostics {
    Live(BuilderStatus),
    NotRunning {
        defaults: std::result::Result<BuilderDefaults, String>,
    },
    Unavailable {
        detail: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct BuilderStatus {
    id: Option<String>,
    image: Option<String>,
    state: Option<String>,
    cpus: Option<u64>,
    memory_bytes: Option<u64>,
    rosetta: Option<bool>,
    started_date: Option<String>,
    ipv4_address: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct BuilderDefaults {
    image: Option<String>,
    cpus: Option<u64>,
    memory: Option<String>,
    rosetta: Option<bool>,
}

pub(super) fn read_builder_diagnostics() -> BuilderDiagnostics {
    match read_builder_status() {
        Ok(Some(status)) => BuilderDiagnostics::Live(status),
        Ok(None) => BuilderDiagnostics::NotRunning {
            defaults: read_builder_defaults().map_err(|error| error.to_string()),
        },
        Err(error) => BuilderDiagnostics::Unavailable {
            detail: error.to_string(),
        },
    }
}

pub(super) fn summarize_builder_diagnostics(
    diagnostics: &BuilderDiagnostics,
) -> BuilderStatusSummary {
    match diagnostics {
        BuilderDiagnostics::Live(status) => BuilderStatusSummary {
            status: status
                .state
                .clone()
                .unwrap_or_else(|| "unknown".to_string()),
            detail: format!(
                "{}: {}",
                status.id.as_deref().unwrap_or("buildkit"),
                status.image.as_deref().unwrap_or("unknown image")
            ),
            image: status.image.clone(),
            cpus: status.cpus.map(|value| value.to_string()),
            memory: status.memory_bytes.map(format_memory),
            rosetta: status.rosetta,
            started_date: status.started_date.clone(),
            ipv4_address: status.ipv4_address.clone(),
            warning: (status.state.as_deref() != Some("running")).then(|| {
                "Builder is not running; the next build may need to start it.".to_string()
            }),
        },
        BuilderDiagnostics::NotRunning { defaults } => match defaults {
            Ok(defaults) => BuilderStatusSummary {
                status: "not-running".to_string(),
                detail: "No BuildKit builder container is running.".to_string(),
                image: defaults.image.clone(),
                cpus: defaults.cpus.map(|value| value.to_string()),
                memory: defaults.memory.clone(),
                rosetta: defaults.rosetta,
                started_date: None,
                ipv4_address: None,
                warning: Some(
                    "The next image build can start the builder automatically.".to_string(),
                ),
            },
            Err(error) => BuilderStatusSummary {
                status: "not-running".to_string(),
                detail: format!(
                    "No BuildKit builder container is running. Configured defaults unavailable: {error}"
                ),
                image: None,
                cpus: None,
                memory: None,
                rosetta: None,
                started_date: None,
                ipv4_address: None,
                warning: Some(
                    "The next image build can start the builder automatically.".to_string(),
                ),
            },
        },
        BuilderDiagnostics::Unavailable { detail } => BuilderStatusSummary {
            status: "unavailable".to_string(),
            detail: detail.clone(),
            image: None,
            cpus: None,
            memory: None,
            rosetta: None,
            started_date: None,
            ipv4_address: None,
            warning: Some("Run setup checks before relying on builder status.".to_string()),
        },
    }
}

fn read_builder_status() -> Result<Option<BuilderStatus>> {
    let output = Command::new("container")
        .args(["builder", "status", "--format", "json"])
        .output()?;
    if !output.status.success() {
        let detail = if output.stderr.is_empty() {
            output.status.to_string()
        } else {
            String::from_utf8_lossy(&output.stderr).trim().to_string()
        };
        bail!("command `container builder status --format json` failed: {detail}");
    }
    parse_builder_status(&output.stdout)
}

fn read_builder_defaults() -> Result<BuilderDefaults> {
    let output = Command::new("container")
        .args(["system", "property", "list", "--format", "json"])
        .output()?;
    if !output.status.success() {
        let detail = if output.stderr.is_empty() {
            output.status.to_string()
        } else {
            String::from_utf8_lossy(&output.stderr).trim().to_string()
        };
        bail!("command `container system property list --format json` failed: {detail}");
    }
    parse_builder_defaults(&output.stdout)
}

fn parse_builder_status(stdout: &[u8]) -> Result<Option<BuilderStatus>> {
    let payload: Value = serde_json::from_slice(stdout)?;
    let Some(items) = payload.as_array() else {
        bail!("could not parse Apple container builder status JSON");
    };
    let Some(item) = items.first() else {
        return Ok(None);
    };
    if !item.is_object() {
        bail!("could not parse Apple container builder status JSON");
    }
    Ok(Some(BuilderStatus {
        id: json_path_string(item, &["id"]),
        image: json_path_string(item, &["configuration", "image", "reference"]),
        state: json_path_string(item, &["status", "state"]),
        cpus: json_path_u64(item, &["configuration", "resources", "cpus"]),
        memory_bytes: json_path_u64(item, &["configuration", "resources", "memoryInBytes"]),
        rosetta: json_path_bool(item, &["configuration", "rosetta"]),
        started_date: json_path_string(item, &["status", "startedDate"]),
        ipv4_address: item
            .pointer("/status/networks/0/ipv4Address")
            .and_then(Value::as_str)
            .map(str::to_string),
    }))
}

fn parse_builder_defaults(stdout: &[u8]) -> Result<BuilderDefaults> {
    let payload: Value = serde_json::from_slice(stdout)?;
    if !payload.is_object() {
        bail!("could not parse Apple container system property JSON");
    }
    Ok(BuilderDefaults {
        image: json_path_string(&payload, &["build", "image"]),
        cpus: json_path_u64(&payload, &["build", "cpus"]),
        memory: json_path_string(&payload, &["build", "memory"]),
        rosetta: json_path_bool(&payload, &["build", "rosetta"]),
    })
}

fn json_path_string(root: &Value, path: &[&str]) -> Option<String> {
    let mut value = root;
    for key in path {
        value = value.get(*key)?;
    }
    value.as_str().map(str::to_string)
}

fn json_path_u64(root: &Value, path: &[&str]) -> Option<u64> {
    let mut value = root;
    for key in path {
        value = value.get(*key)?;
    }
    value.as_u64()
}

fn json_path_bool(root: &Value, path: &[&str]) -> Option<bool> {
    let mut value = root;
    for key in path {
        value = value.get(*key)?;
    }
    value.as_bool()
}

pub(super) fn print_builder_status(diagnostics: &BuilderDiagnostics) {
    println!("Builder status");
    match diagnostics {
        BuilderDiagnostics::Live(status) => print_live_builder_status(status),
        BuilderDiagnostics::NotRunning { defaults } => {
            println!("not running: no BuildKit builder container found");
            match defaults {
                Ok(defaults) => print_builder_defaults(defaults),
                Err(error) => println!("configured defaults unavailable: {error}"),
            }
            println!("next build: container build can start the builder automatically");
            print_builder_guidance();
        }
        BuilderDiagnostics::Unavailable { detail } => {
            println!("diagnostics unavailable: {detail}");
            println!("check: runhaven doctor");
            println!("check: container system status");
            println!("manual: container builder status --format json");
            print_builder_guidance();
        }
    }
}

fn print_live_builder_status(status: &BuilderStatus) {
    println!(
        "{} {}: {}",
        status.state.as_deref().unwrap_or("unknown"),
        status.id.as_deref().unwrap_or("buildkit"),
        status.image.as_deref().unwrap_or("unknown image")
    );
    let cpus = status
        .cpus
        .map(|value| value.to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let memory = status
        .memory_bytes
        .map(format_memory)
        .unwrap_or_else(|| "unknown memory".to_string());
    println!("resources: {cpus} CPUs, {memory}");
    if let Some(rosetta) = status.rosetta {
        println!("rosetta: {}", if rosetta { "enabled" } else { "disabled" });
    }
    if let Some(started) = &status.started_date {
        println!("started: {started}");
    }
    if let Some(ipv4) = &status.ipv4_address {
        println!("network: {ipv4}");
    }
    if status.state.as_deref() != Some("running") {
        println!("attention: builder is not running; the next build may need to start it");
    }
    print_builder_guidance();
}

fn print_builder_defaults(defaults: &BuilderDefaults) {
    println!(
        "configured defaults: {}",
        defaults.image.as_deref().unwrap_or("unknown builder image")
    );
    let cpus = defaults
        .cpus
        .map(|value| value.to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let memory = defaults.memory.as_deref().unwrap_or("unknown memory");
    println!("configured resources: {cpus} CPUs, {memory}");
    if let Some(rosetta) = defaults.rosetta {
        println!(
            "configured rosetta: {}",
            if rosetta { "enabled" } else { "disabled" }
        );
    }
}

fn print_builder_guidance() {
    println!("large build resources: container builder start --cpus N --memory SIZE");
    println!("resize existing builder: container builder stop");
    println!("resize existing builder: container builder delete");
    println!("resize existing builder: container builder start --cpus N --memory SIZE");
}

fn format_memory(bytes: u64) -> String {
    const MIB: u64 = 1024 * 1024;
    if bytes.is_multiple_of(MIB) {
        format!("{} MiB", bytes / MIB)
    } else {
        format!("{bytes} bytes")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BUILDER_STATUS_CURRENT: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/apple_container/builder-status-current.json"
    ));
    const SYSTEM_PROPERTY_CURRENT: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/apple_container/system-property-current.json"
    ));

    #[test]
    fn parses_current_apple_builder_status_shape() {
        let status = parse_builder_status(BUILDER_STATUS_CURRENT)
            .expect("builder status")
            .expect("builder entry");

        assert_eq!(status.id.as_deref(), Some("buildkit"));
        assert_eq!(
            status.image.as_deref(),
            Some("ghcr.io/apple/container-builder-shim/builder:0.12.0")
        );
        assert_eq!(status.state.as_deref(), Some("running"));
        assert_eq!(status.cpus, Some(2));
        assert_eq!(status.memory_bytes, Some(2_147_483_648));
        assert_eq!(status.rosetta, Some(true));
        assert_eq!(status.started_date.as_deref(), Some("2026-06-14T02:50:37Z"));
        assert_eq!(status.ipv4_address.as_deref(), Some("192.168.64.2/24"));
    }

    #[test]
    fn parses_empty_builder_status_as_not_running() {
        let status = parse_builder_status(br#"[]"#).expect("builder status");

        assert_eq!(status, None);
    }

    #[test]
    fn rejects_invalid_builder_status_shape() {
        let error = parse_builder_status(br#"{"configuration":{}}"#).expect_err("invalid shape");

        assert!(
            error
                .to_string()
                .contains("could not parse Apple container builder status JSON")
        );
    }

    #[test]
    fn parses_current_builder_defaults_shape() {
        let defaults = parse_builder_defaults(SYSTEM_PROPERTY_CURRENT).expect("defaults");

        assert_eq!(
            defaults.image.as_deref(),
            Some("ghcr.io/apple/container-builder-shim/builder:0.12.0")
        );
        assert_eq!(defaults.cpus, Some(2));
        assert_eq!(defaults.memory.as_deref(), Some("2048mb"));
        assert_eq!(defaults.rosetta, Some(true));
    }

    #[test]
    fn rejects_invalid_builder_defaults_shape() {
        let error = parse_builder_defaults(br#"[]"#).expect_err("invalid shape");

        assert!(
            error
                .to_string()
                .contains("could not parse Apple container system property JSON")
        );
    }

    #[test]
    fn formats_builder_memory_as_mib() {
        assert_eq!(format_memory(2_147_483_648), "2048 MiB");
    }
}
