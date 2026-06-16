use std::collections::BTreeSet;
use std::process::Command;

use anyhow::{Result, bail};
use serde_json::Value;

use crate::active::read_active_run_records;
use crate::images::{RUNHAVEN_SOURCE_DIGEST_LABEL, image_source_digest};
use crate::profiles::{AgentProfile, get_profile, profiles};
use crate::session_state::is_runhaven_state_volume;

#[derive(Clone, Debug)]
struct LocalImage {
    names: BTreeSet<String>,
    source_digest: Option<String>,
}

pub fn image_doctor(agent: Option<&str>) -> Result<i32> {
    let local_images = list_local_images()?;
    let state_volumes = list_state_volume_names()?;
    let active_state_volumes = active_run_state_volumes();
    let selected = selected_profiles(agent)?;
    let mut ok = true;

    println!("Image doctor");
    for profile in &selected {
        let source_digest = image_source_digest(profile)?;
        let local = find_local_image(profile.image, &local_images);
        let stale = local
            .and_then(|image| image.source_digest.as_deref())
            .is_some_and(|digest| digest != source_digest);
        let status = if local.is_none() {
            ok = false;
            "missing"
        } else if stale {
            ok = false;
            "stale"
        } else {
            "ok"
        };
        println!("{status} {}: {}", profile.name, profile.image);
        if local.is_none() || stale {
            if stale {
                println!("reason: bundled source digest differs from local image metadata");
            }
            println!("fix: runhaven image rebuild {}", profile.name);
        }
    }
    print_state_volume_review(&selected, &state_volumes, &active_state_volumes, agent);
    print_preflight_recovery(agent);
    Ok(if ok { 0 } else { 1 })
}

fn selected_profiles(agent: Option<&str>) -> Result<Vec<AgentProfile>> {
    if let Some(agent) = agent {
        return Ok(vec![get_profile(agent)?]);
    }
    Ok(profiles())
}

fn list_local_images() -> Result<Vec<LocalImage>> {
    let output = Command::new("container")
        .args(["image", "list", "--format", "json"])
        .output()?;
    if !output.status.success() {
        bail!("container image list failed: {}", output.status);
    }
    let payload: Value = serde_json::from_slice(&output.stdout)?;
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
        for value in annotations.values().filter_map(Value::as_str) {
            names.insert(value.to_string());
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
            if let Some(labels) = variant.pointer("/config/Labels").and_then(Value::as_object) {
                mappings.push(labels);
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
