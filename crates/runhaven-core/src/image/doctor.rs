use std::collections::BTreeSet;
use std::process::Command;

use anyhow::{Result, bail};
use serde_json::Value;

mod builder;

use crate::image::build::{RUNHAVEN_SOURCE_DIGEST_LABEL, image_source_digest};
use crate::runtime::active::read_active_run_records;
use crate::runtime::profiles::{AgentProfile, get_profile, profiles};
use crate::runtime::session_state::is_runhaven_state_volume;

#[derive(Clone, Debug)]
struct LocalImage {
    names: BTreeSet<String>,
    source_digest: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProfileImageStatus {
    pub agent: String,
    pub image: String,
    pub status: String,
    pub ready: bool,
    pub expected_source_digest: String,
    pub local_source_digest: Option<String>,
    pub fix_command: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuilderStatusSummary {
    pub status: String,
    pub detail: String,
    pub image: Option<String>,
    pub cpus: Option<String>,
    pub memory: Option<String>,
    pub rosetta: Option<bool>,
    pub started_date: Option<String>,
    pub ipv4_address: Option<String>,
    pub warning: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImageStatusReport {
    pub agent: String,
    pub image: ProfileImageStatus,
    pub builder: BuilderStatusSummary,
}

pub fn collect_image_status(agent: &str) -> Result<ImageStatusReport> {
    let profile = get_profile(agent)?;
    let local_images = list_local_images()?;
    let builder_diagnostics = builder::read_builder_diagnostics();
    Ok(ImageStatusReport {
        agent: profile.name.to_string(),
        image: profile_image_status(&profile, &local_images)?,
        builder: builder::summarize_builder_diagnostics(&builder_diagnostics),
    })
}

pub fn image_doctor(agent: Option<&str>) -> Result<i32> {
    let local_images = list_local_images()?;
    let builder_diagnostics = builder::read_builder_diagnostics();
    let state_volumes = list_state_volume_names()?;
    let active_state_volumes = active_run_state_volumes();
    let selected = selected_profiles(agent)?;
    let mut ok = true;

    println!("Image doctor");
    for profile in &selected {
        let status = profile_image_status(profile, &local_images)?;
        println!("{} {}: {}", status.status, profile.name, profile.image);
        if !status.ready {
            ok = false;
            if status.status == "stale" {
                println!("reason: bundled source digest differs from local image metadata");
            }
            println!("fix: runhaven image rebuild {}", profile.name);
        }
    }
    builder::print_builder_status(&builder_diagnostics);
    print_state_volume_review(&selected, &state_volumes, &active_state_volumes, agent);
    print_preflight_recovery(agent);
    Ok(if ok { 0 } else { 1 })
}

fn profile_image_status(
    profile: &AgentProfile,
    local_images: &[LocalImage],
) -> Result<ProfileImageStatus> {
    let expected_source_digest = image_source_digest(profile)?;
    let local = find_local_image(profile.image, local_images);
    let stale = local
        .and_then(|image| image.source_digest.as_deref())
        .is_some_and(|digest| digest != expected_source_digest);
    let status = if local.is_none() {
        "missing"
    } else if stale {
        "stale"
    } else {
        "ok"
    };
    let ready = status == "ok";
    Ok(ProfileImageStatus {
        agent: profile.name.to_string(),
        image: profile.image.to_string(),
        status: status.to_string(),
        ready,
        expected_source_digest,
        local_source_digest: local.and_then(|image| image.source_digest.clone()),
        fix_command: (!ready).then(|| format!("runhaven image rebuild {}", profile.name)),
    })
}

fn selected_profiles(agent: Option<&str>) -> Result<Vec<AgentProfile>> {
    if let Some(agent) = agent {
        return Ok(vec![get_profile(agent)?]);
    }
    Ok(profiles())
}

/// Whether `image` (for example `runhaven/codex:0.1.0`) is present in the local
/// Apple container image store. Used to fail a run with a clear "build it first"
/// message instead of letting `container run` try to pull a RunHaven image from
/// a registry and return a confusing 401.
pub fn image_is_built(image: &str) -> Result<bool> {
    Ok(find_local_image(image, &list_local_images()?).is_some())
}

fn list_local_images() -> Result<Vec<LocalImage>> {
    let output = Command::new("container")
        .args(["image", "list", "--format", "json"])
        .output()?;
    if !output.status.success() {
        bail!("container image list failed: {}", output.status);
    }
    parse_local_images(&output.stdout)
}

fn parse_local_images(stdout: &[u8]) -> Result<Vec<LocalImage>> {
    let payload: Value = serde_json::from_slice(stdout)?;
    let Some(items) = payload.as_array() else {
        bail!("could not parse Apple container image list JSON");
    };
    Ok(items
        .iter()
        .filter_map(|item| {
            let names = image_names(item);
            if names.is_empty() {
                None
            } else {
                Some(LocalImage {
                    names,
                    source_digest: image_source_digest_label(item),
                })
            }
        })
        .collect())
}

fn image_names(item: &Value) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    if let Some(name) = item.pointer("/configuration/name").and_then(Value::as_str) {
        names.insert(name.to_string());
    }
    if let Some(annotations) = item
        .pointer("/configuration/descriptor/annotations")
        .and_then(Value::as_object)
    {
        for key in [
            "com.apple.containerization.image.name",
            "io.containerd.image.name",
        ] {
            if let Some(value) = annotations.get(key).and_then(Value::as_str) {
                names.insert(value.to_string());
            }
        }
    }
    names
}

fn image_source_digest_label(item: &Value) -> Option<String> {
    for labels in image_label_mappings(item) {
        if let Some(value) = labels
            .get(RUNHAVEN_SOURCE_DIGEST_LABEL)
            .and_then(Value::as_str)
        {
            return Some(value.to_string());
        }
    }
    None
}

fn image_label_mappings(item: &Value) -> Vec<&serde_json::Map<String, Value>> {
    let mut mappings = Vec::new();
    for pointer in [
        "/configuration/labels",
        "/configuration/descriptor/annotations",
        "/status/labels",
    ] {
        if let Some(labels) = item.pointer(pointer).and_then(Value::as_object) {
            mappings.push(labels);
        }
    }
    if let Some(variants) = item.get("variants").and_then(Value::as_array) {
        for variant in variants {
            for pointer in ["/config/Labels", "/config/config/Labels"] {
                if let Some(labels) = variant.pointer(pointer).and_then(Value::as_object) {
                    mappings.push(labels);
                }
            }
        }
    }
    mappings
}

fn find_local_image<'a>(image: &str, local_images: &'a [LocalImage]) -> Option<&'a LocalImage> {
    let docker_name = format!("docker.io/{image}");
    local_images
        .iter()
        .find(|local| local.names.contains(image) || local.names.contains(&docker_name))
}

fn list_state_volume_names() -> Result<Vec<String>> {
    let output = Command::new("container")
        .args(["volume", "list", "--quiet"])
        .output()?;
    if !output.status.success() {
        bail!("container volume list failed: {}", output.status);
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| is_runhaven_state_volume(line))
        .map(str::to_string)
        .collect())
}

fn active_run_state_volumes() -> BTreeSet<String> {
    read_active_run_records()
        .into_iter()
        .filter_map(|record| {
            record
                .get("state_volume")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .filter(|volume| is_runhaven_state_volume(volume))
        .collect()
}

fn print_state_volume_review(
    profiles: &[AgentProfile],
    state_volumes: &[String],
    active_state_volumes: &BTreeSet<String>,
    agent: Option<&str>,
) {
    println!("State volume review");
    let inactive = state_volumes
        .iter()
        .filter(|volume| {
            profiles
                .iter()
                .any(|profile| volume.starts_with(&format!("runhaven-{}-", profile.name)))
        })
        .filter(|volume| !active_state_volumes.contains(*volume))
        .collect::<Vec<_>>();
    if inactive.is_empty() {
        println!("No inactive RunHaven state volumes found.");
        return;
    }
    println!("Inactive RunHaven state volumes found:");
    for volume in inactive {
        println!("- {volume}");
    }
    let agent = agent.unwrap_or("AGENT");
    println!(
        "These can be normal reusable session state. Reset only when you want to discard that agent home state."
    );
    println!("reset: runhaven state reset {agent} --workspace PATH --yes");
}

fn print_preflight_recovery(agent: Option<&str>) {
    let agent = agent.unwrap_or("AGENT");
    println!("Preflight recovery");
    println!("- Rebuild a missing or stale bundled image: runhaven image rebuild {agent}");
    println!("- Inspect RunHaven-managed networks: runhaven network list");
    println!("- Remove stale managed networks after review: runhaven network prune --yes");
    println!(
        "- Reset interrupted isolated home state only when you want to discard it: runhaven state reset {agent} --workspace PATH --yes"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    const IMAGE_LIST_CURRENT: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/apple_container/image-list-current.json"
    ));

    #[test]
    fn parses_current_apple_image_list_shape() {
        let images = parse_local_images(IMAGE_LIST_CURRENT).expect("image list");

        assert_eq!(images.len(), 1);
        assert!(images[0].names.contains("runhaven/base:0.1.0"));
        assert!(images[0].names.contains("docker.io/runhaven/base:0.1.0"));
        assert!(!images[0].names.contains("0.1.0"));
        assert!(!images[0].names.contains("2026-06-14T03:26:29Z"));
        assert_eq!(
            images[0].source_digest.as_deref(),
            Some("fixture-source-digest")
        );
    }

    #[test]
    fn rejects_non_array_image_list_json() {
        let error = parse_local_images(br#"{"configuration":{}}"#).expect_err("invalid shape");

        assert!(
            error
                .to_string()
                .contains("could not parse Apple container image list JSON")
        );
    }

    #[test]
    fn profile_image_status_marks_missing_bundled_image_not_ready() {
        let profile = get_profile("shell").expect("profile");
        let status = profile_image_status(&profile, &[]).expect("image status");

        assert_eq!(status.status, "missing");
        assert!(!status.ready);
        assert_eq!(
            status.fix_command.as_deref(),
            Some("runhaven image rebuild shell")
        );
    }

    #[test]
    fn profile_image_status_marks_stale_bundled_image_not_ready() {
        let profile = get_profile("shell").expect("profile");
        let local = LocalImage {
            names: BTreeSet::from([profile.image.to_string()]),
            source_digest: Some("old-source".to_string()),
        };
        let status = profile_image_status(&profile, &[local]).expect("image status");

        assert_eq!(status.status, "stale");
        assert!(!status.ready);
        assert_eq!(status.local_source_digest.as_deref(), Some("old-source"));
    }

    #[test]
    fn profile_image_status_marks_current_bundled_image_ready() {
        let profile = get_profile("shell").expect("profile");
        let source_digest = image_source_digest(&profile).expect("digest");
        let local = LocalImage {
            names: BTreeSet::from([profile.image.to_string()]),
            source_digest: Some(source_digest),
        };
        let status = profile_image_status(&profile, &[local]).expect("image status");

        assert_eq!(status.status, "ok");
        assert!(status.ready);
        assert_eq!(status.fix_command, None);
    }
}
