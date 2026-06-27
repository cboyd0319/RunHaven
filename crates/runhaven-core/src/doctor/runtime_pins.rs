use std::process::Command;

use serde_json::Value as JsonValue;
use toml::Value as TomlValue;

use super::{Check, parse_container_version};

pub(super) fn container_runtime_pin_checks() -> Vec<Check> {
    let pins = match load_apple_container_pins() {
        Ok(pins) => pins,
        Err(error) => {
            return vec![Check::new(
                "Apple container runtime pins",
                false,
                error,
                "Repair pins.toml before checking runtime helper pins.",
            )];
        }
    };
    let mut checks = Vec::new();
    checks.push(container_system_version_check(&pins));
    checks.extend(container_system_property_checks(&pins));
    checks
}

#[derive(Clone, Debug)]
struct AppleContainerPins {
    version: String,
    commit: String,
    kata_release: String,
    kernel: String,
    builder_image: String,
    vminit_image: String,
}

fn load_apple_container_pins() -> Result<AppleContainerPins, String> {
    let pins = toml::from_str::<TomlValue>(include_str!("../../../../pins.toml"))
        .map_err(|error| format!("pins.toml is invalid: {error}"))?;
    Ok(AppleContainerPins {
        version: required_pin(&pins, &["apple_container", "version"])?,
        commit: required_pin(&pins, &["apple_container", "commit"])?,
        kata_release: required_pin(&pins, &["apple_container_runtime", "kata_release"])?,
        kernel: required_pin(&pins, &["apple_container_runtime", "kernel"])?,
        builder_image: required_pin(&pins, &["apple_container_runtime", "builder_image"])?,
        vminit_image: required_pin(&pins, &["apple_container_runtime", "vminit_image"])?,
    })
}

fn required_pin(root: &TomlValue, path: &[&str]) -> Result<String, String> {
    toml_path(root, path)
        .and_then(TomlValue::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("pins.toml missing {}", path.join(".")))
}

fn toml_path<'a>(root: &'a TomlValue, path: &[&str]) -> Option<&'a TomlValue> {
    let mut value = root;
    for key in path {
        value = value.get(*key)?;
    }
    Some(value)
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct SystemVersionSummary {
    container_version: Option<String>,
    container_commit: Option<String>,
    apiserver_version: Option<String>,
    apiserver_commit: Option<String>,
}

fn container_system_version_check(pins: &AppleContainerPins) -> Check {
    match Command::new("container")
        .args(["system", "version", "--format", "json"])
        .output()
    {
        Ok(output) => {
            let text = if output.stdout.is_empty() {
                &output.stderr
            } else {
                &output.stdout
            };
            system_version_check_from_text(
                output.status.success(),
                &String::from_utf8_lossy(text),
                pins,
            )
        }
        Err(error) => Check::new(
            "Apple container runtime commit",
            false,
            error.to_string(),
            "Check `container system version --format json`.",
        ),
    }
}

fn system_version_check_from_text(
    command_ok: bool,
    text: &str,
    pins: &AppleContainerPins,
) -> Check {
    if !command_ok {
        return Check::new(
            "Apple container runtime commit",
            false,
            text.trim(),
            "Check `container system version --format json`.",
        );
    }
    let summary = match parse_system_version(text) {
        Ok(summary) => summary,
        Err(error) => {
            return Check::new(
                "Apple container runtime commit",
                false,
                error,
                "Check `container system version --format json`.",
            );
        }
    };
    let ok = summary.container_version.as_deref() == Some(pins.version.as_str())
        && summary.container_commit.as_deref() == Some(pins.commit.as_str())
        && summary.apiserver_version.as_deref() == Some(pins.version.as_str())
        && summary.apiserver_commit.as_deref() == Some(pins.commit.as_str());
    Check::new(
        "Apple container runtime commit",
        ok,
        format!(
            "container={} {} apiserver={} {}; expected {} {}",
            summary.container_version.as_deref().unwrap_or("-"),
            summary.container_commit.as_deref().unwrap_or("-"),
            summary.apiserver_version.as_deref().unwrap_or("-"),
            summary.apiserver_commit.as_deref().unwrap_or("-"),
            pins.version,
            pins.commit
        ),
        "Install the reviewed Apple container release and run `container system start`.",
    )
}

fn parse_system_version(text: &str) -> Result<SystemVersionSummary, String> {
    let entries = serde_json::from_str::<Vec<JsonValue>>(text).map_err(|error| {
        format!("could not parse `container system version --format json`: {error}")
    })?;
    let mut summary = SystemVersionSummary::default();
    for entry in entries {
        match json_string(&entry, "appName") {
            Some("container") => {
                summary.container_version = json_string(&entry, "version").map(str::to_string);
                summary.container_commit = json_string(&entry, "commit").map(str::to_string);
            }
            Some("container-apiserver") => {
                summary.apiserver_version =
                    json_string(&entry, "version").and_then(parse_container_version);
                summary.apiserver_commit = json_string(&entry, "commit").map(str::to_string);
            }
            _ => {}
        }
    }
    if summary.container_version.is_none() || summary.container_commit.is_none() {
        return Err("`container system version --format json` missing container entry".to_string());
    }
    if summary.apiserver_version.is_none() || summary.apiserver_commit.is_none() {
        return Err(
            "`container system version --format json` missing container-apiserver entry"
                .to_string(),
        );
    }
    Ok(summary)
}

fn container_system_property_checks(pins: &AppleContainerPins) -> Vec<Check> {
    match Command::new("container")
        .args(["system", "property", "list", "--format", "json"])
        .output()
    {
        Ok(output) => {
            let text = if output.stdout.is_empty() {
                &output.stderr
            } else {
                &output.stdout
            };
            system_property_checks_from_text(
                output.status.success(),
                &String::from_utf8_lossy(text),
                pins,
            )
        }
        Err(error) => runtime_property_command_error_checks(error.to_string()),
    }
}

fn system_property_checks_from_text(
    command_ok: bool,
    text: &str,
    pins: &AppleContainerPins,
) -> Vec<Check> {
    if !command_ok {
        return runtime_property_command_error_checks(text.trim().to_string());
    }
    let properties = match serde_json::from_str::<JsonValue>(text) {
        Ok(properties) => properties,
        Err(error) => {
            return vec![Check::new(
                "Apple container runtime properties",
                false,
                format!("could not parse `container system property list --format json`: {error}"),
                "Check `container system property list --format json`.",
            )];
        }
    };
    let builder = json_path_string(&properties, &["build", "image"]).unwrap_or("-");
    let vminit = json_path_string(&properties, &["vminit", "image"]).unwrap_or("-");
    let kernel = json_path_string(&properties, &["kernel", "binaryPath"]).unwrap_or("-");
    let kernel_url = json_path_string(&properties, &["kernel", "url"]).unwrap_or("-");
    let kernel_name = kernel.rsplit('/').next().unwrap_or(kernel);
    let kata_release = kata_release_from_url(kernel_url).unwrap_or("-");
    vec![
        Check::new(
            "Apple container builder image",
            builder == pins.builder_image,
            format!("{builder}; expected {}", pins.builder_image),
            "Update Apple container runtime properties or repo pins together.",
        ),
        Check::new(
            "Apple container vminit image",
            vminit == pins.vminit_image,
            format!("{vminit}; expected {}", pins.vminit_image),
            "Update Apple container runtime properties or repo pins together.",
        ),
        Check::new(
            "Apple container kernel",
            kernel_name == pins.kernel && kata_release == pins.kata_release,
            format!(
                "binaryPath={kernel} url={kernel_url}; expected {} from Kata {}",
                pins.kernel, pins.kata_release
            ),
            "Update Apple container runtime properties or repo pins together.",
        ),
    ]
}

fn runtime_property_command_error_checks(detail: String) -> Vec<Check> {
    ["builder image", "vminit image", "kernel"]
        .into_iter()
        .map(|name| {
            Check::new(
                &format!("Apple container {name}"),
                false,
                detail.clone(),
                "Check `container system property list --format json`.",
            )
        })
        .collect()
}

fn json_string<'a>(value: &'a JsonValue, key: &str) -> Option<&'a str> {
    value.get(key).and_then(JsonValue::as_str)
}

fn json_path_string<'a>(root: &'a JsonValue, path: &[&str]) -> Option<&'a str> {
    let mut value = root;
    for key in path {
        value = value.get(*key)?;
    }
    value.as_str()
}

fn kata_release_from_url(url: &str) -> Option<&str> {
    url.split("/download/").nth(1)?.split('/').next()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_pins() -> AppleContainerPins {
        AppleContainerPins {
            version: "1.0.0".to_string(),
            commit: "ee848e3ebfd7c73b04dd419683be54fb450b8779".to_string(),
            kata_release: "3.28.0".to_string(),
            kernel: "vmlinux-6.18.15-186".to_string(),
            builder_image: "ghcr.io/apple/container-builder-shim/builder:0.12.0".to_string(),
            vminit_image: "ghcr.io/apple/containerization/vminit:0.33.3".to_string(),
        }
    }

    #[test]
    fn parses_system_version_json() {
        let summary = parse_system_version(
            r#"[{"appName":"container","buildType":"release","commit":"ee848e3ebfd7c73b04dd419683be54fb450b8779","version":"1.0.0"},{"appName":"container-apiserver","buildType":"release","commit":"ee848e3ebfd7c73b04dd419683be54fb450b8779","version":"container-apiserver version 1.0.0 (build: release, commit: ee848e3)"}]"#,
        )
        .expect("valid version JSON");

        assert_eq!(summary.container_version.as_deref(), Some("1.0.0"));
        assert_eq!(
            summary.container_commit.as_deref(),
            Some("ee848e3ebfd7c73b04dd419683be54fb450b8779")
        );
        assert_eq!(summary.apiserver_version.as_deref(), Some("1.0.0"));
        assert_eq!(
            summary.apiserver_commit.as_deref(),
            Some("ee848e3ebfd7c73b04dd419683be54fb450b8779")
        );
    }

    #[test]
    fn system_version_check_fails_commit_mismatch() {
        let check = system_version_check_from_text(
            true,
            r#"[{"appName":"container","buildType":"release","commit":"0000000000000000000000000000000000000000","version":"1.0.0"},{"appName":"container-apiserver","buildType":"release","commit":"ee848e3ebfd7c73b04dd419683be54fb450b8779","version":"container-apiserver version 1.0.0 (build: release, commit: ee848e3)"}]"#,
            &fixture_pins(),
        );

        assert!(!check.ok);
        assert!(check.detail.contains("expected 1.0.0 ee848e3"));
    }

    #[test]
    fn system_version_check_fails_invalid_json() {
        let check = system_version_check_from_text(true, "not json", &fixture_pins());

        assert!(!check.ok);
        assert!(check.detail.contains("could not parse"));
    }

    #[test]
    fn system_version_check_fails_missing_apiserver() {
        let check = system_version_check_from_text(
            true,
            r#"[{"appName":"container","buildType":"release","commit":"ee848e3ebfd7c73b04dd419683be54fb450b8779","version":"1.0.0"}]"#,
            &fixture_pins(),
        );

        assert!(!check.ok);
        assert!(check.detail.contains("missing container-apiserver"));
    }

    #[test]
    fn system_property_checks_match_pins() {
        let checks = system_property_checks_from_text(
            true,
            r#"{"build":{"image":"ghcr.io/apple/container-builder-shim/builder:0.12.0"},"kernel":{"binaryPath":"opt/kata/share/kata-containers/vmlinux-6.18.15-186","url":"https://github.com/kata-containers/kata-containers/releases/download/3.28.0/kata-static-3.28.0-arm64.tar.zst"},"vminit":{"image":"ghcr.io/apple/containerization/vminit:0.33.3"}}"#,
            &fixture_pins(),
        );

        assert_eq!(checks.len(), 3);
        assert!(checks.iter().all(|check| check.ok));
    }

    #[test]
    fn system_property_checks_fail_helper_mismatch() {
        let checks = system_property_checks_from_text(
            true,
            r#"{"build":{"image":"ghcr.io/apple/container-builder-shim/builder:0.13.0"},"kernel":{"binaryPath":"opt/kata/share/kata-containers/vmlinux-6.18.15-999","url":"https://github.com/kata-containers/kata-containers/releases/download/3.29.0/kata-static-3.29.0-arm64.tar.zst"},"vminit":{"image":"ghcr.io/apple/containerization/vminit:0.34.0"}}"#,
            &fixture_pins(),
        );

        assert_eq!(checks.len(), 3);
        assert!(checks.iter().all(|check| !check.ok));
    }

    #[test]
    fn system_property_checks_fail_invalid_json() {
        let checks = system_property_checks_from_text(true, "not json", &fixture_pins());

        assert_eq!(checks.len(), 1);
        assert!(!checks[0].ok);
        assert!(checks[0].detail.contains("could not parse"));
    }

    #[test]
    fn extracts_kata_release_from_url() {
        assert_eq!(
            kata_release_from_url(
                "https://github.com/kata-containers/kata-containers/releases/download/3.28.0/kata-static-3.28.0-arm64.tar.zst"
            ),
            Some("3.28.0")
        );
    }
}
