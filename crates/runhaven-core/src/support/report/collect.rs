use std::path::Path;
use std::time::Duration;

use serde_json::Value;
use time::format_description::well_known::Rfc3339;

use crate::diagnostics::{read_auth_broker_log_tail_bounded, read_egress_policy_log_tail_bounded};
use crate::doctor::{PINNED_APPLE_CONTAINER_VERSION, parse_container_cli_version};
use crate::image::doctor::profile_image_statuses_from_list_json;
use crate::records::read_run_records_tail_bounded;
use crate::runtime::profiles::agent_profiles;

use super::app_log::read_app_events;
use super::process::run_bounded;
use super::{
    DecisionSummary, ImageInvariant, Invariant, SessionSummary, SupportReportData,
    SupportReportRequest,
};

const RECORD_LIMIT: usize = 10;
const DECISION_LIMIT: usize = 20;
const JSONL_TAIL_BYTES: u64 = 128 * 1024;
const PROCESS_TIMEOUT: Duration = Duration::from_secs(3);
const PROCESS_OUTPUT_BYTES: usize = 256 * 1024;

pub(super) fn collect_report_data(request: &SupportReportRequest<'_>) -> SupportReportData {
    let sessions = read_run_records_tail_bounded(RECORD_LIMIT, JSONL_TAIL_BYTES)
        .unwrap_or_default()
        .iter()
        .filter_map(project_run_record)
        .collect();
    let egress = read_egress_policy_log_tail_bounded(DECISION_LIMIT, JSONL_TAIL_BYTES)
        .unwrap_or_default()
        .iter()
        .filter_map(project_egress_record)
        .collect();
    let auth = read_auth_broker_log_tail_bounded(DECISION_LIMIT, JSONL_TAIL_BYTES)
        .unwrap_or_default()
        .iter()
        .filter_map(project_auth_record)
        .collect();
    let (app_events, omitted_app_log_lines) = request
        .app_log
        .as_ref()
        .map(read_app_events)
        .unwrap_or_default();
    let (system, images) = collect_live_invariants();
    SupportReportData {
        app_version: safe_app_version(request.app_version),
        created_at: request
            .created_at
            .format(&Rfc3339)
            .unwrap_or_else(|_| "unavailable".to_string()),
        system,
        images,
        sessions,
        egress,
        auth,
        app_events,
        omitted_app_log_lines,
    }
}

fn collect_live_invariants() -> (Vec<Invariant>, Vec<ImageInvariant>) {
    let mut system = vec![
        Invariant {
            name: "operating system".to_string(),
            status: if std::env::consts::OS == "macos" {
                "ready"
            } else {
                "not ready"
            }
            .to_string(),
            value: if std::env::consts::OS == "macos" {
                "macOS"
            } else {
                "unsupported"
            }
            .to_string(),
        },
        Invariant {
            name: "architecture".to_string(),
            status: if matches!(std::env::consts::ARCH, "aarch64" | "arm64") {
                "ready"
            } else {
                "not ready"
            }
            .to_string(),
            value: allowed_architecture(std::env::consts::ARCH),
        },
    ];
    system.push(macos_version_invariant());
    let Some(program) = crate::apple_container::resolve_program() else {
        system.push(Invariant {
            name: "Apple container CLI".to_string(),
            status: "not ready".to_string(),
            value: "missing or unsafe".to_string(),
        });
        system.push(unavailable_invariant("Apple container service"));
        return (system, unavailable_images());
    };
    system.push(Invariant {
        name: "Apple container CLI".to_string(),
        status: "ready".to_string(),
        value: "installed".to_string(),
    });
    system.push(container_version_invariant(&program));
    system.push(container_service_invariant(&program));
    (system, collect_images(&program))
}

fn macos_version_invariant() -> Invariant {
    let output = run_bounded(
        Path::new("/usr/bin/sw_vers"),
        &["-productVersion"],
        PROCESS_TIMEOUT,
        128,
    );
    let version = output
        .ok()
        .filter(|output| output.success && !output.timed_out)
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|value| value.trim().to_string())
        .filter(|value| valid_version(value));
    Invariant {
        name: "macOS version".to_string(),
        status: version
            .as_ref()
            .map_or("unavailable", |_| "ready")
            .to_string(),
        value: version.unwrap_or_else(|| "unavailable".to_string()),
    }
}

fn container_version_invariant(program: &Path) -> Invariant {
    let version = run_bounded(program, &["--version"], PROCESS_TIMEOUT, 1024)
        .ok()
        .filter(|output| output.success && !output.timed_out)
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .and_then(|value| parse_container_cli_version(value.trim()));
    let status = match version.as_deref() {
        Some(PINNED_APPLE_CONTAINER_VERSION) => "ready",
        Some(_) => "not ready",
        None => "unavailable",
    };
    Invariant {
        name: "Apple container version".to_string(),
        status: status.to_string(),
        value: version.unwrap_or_else(|| "unavailable".to_string()),
    }
}

fn container_service_invariant(program: &Path) -> Invariant {
    let output = run_bounded(program, &["system", "status"], PROCESS_TIMEOUT, 1024);
    let status = match output {
        Ok(output) if output.timed_out => "unavailable",
        Ok(output) if output.success => "ready",
        Ok(_) => "not ready",
        Err(_) => "unavailable",
    };
    Invariant {
        name: "Apple container service".to_string(),
        status: status.to_string(),
        value: status.to_string(),
    }
}

fn collect_images(program: &Path) -> Vec<ImageInvariant> {
    let output = run_bounded(
        program,
        &["image", "list", "--format", "json"],
        PROCESS_TIMEOUT,
        PROCESS_OUTPUT_BYTES,
    );
    let Ok(output) = output else {
        return unavailable_images();
    };
    if !output.success || output.timed_out {
        return unavailable_images();
    }
    profile_image_statuses_from_list_json(&output.stdout)
        .map(|statuses| {
            statuses
                .into_iter()
                .map(|status| ImageInvariant {
                    agent: status.agent,
                    image: status.image,
                    status: allowed_image_status(&status.status),
                })
                .collect()
        })
        .unwrap_or_else(|_| unavailable_images())
}

fn unavailable_images() -> Vec<ImageInvariant> {
    agent_profiles()
        .into_iter()
        .map(|profile| ImageInvariant {
            agent: profile.name.to_string(),
            image: profile.image.to_string(),
            status: "unavailable".to_string(),
        })
        .collect()
}

fn unavailable_invariant(name: &str) -> Invariant {
    Invariant {
        name: name.to_string(),
        status: "unavailable".to_string(),
        value: "unavailable".to_string(),
    }
}

pub(super) fn project_run_record(record: &Value) -> Option<SessionSummary> {
    Some(SessionSummary {
        timestamp: safe_timestamp(record.get("timestamp")?.as_str()?),
        profile: allowed_profile(record.get("profile")?.as_str()?),
        network: allowed_network(record.get("network").and_then(Value::as_str).unwrap_or("")),
        auth_scope: allowed_auth_scope(
            record
                .get("auth_scope")
                .and_then(Value::as_str)
                .unwrap_or(""),
        ),
        status: allowed_run_status(record.get("status")?.as_str()?),
        return_code: record
            .get("return_code")
            .and_then(Value::as_i64)
            .unwrap_or(-1),
        provider_allowed: nested_u64(record, "provider_policy", "allowed"),
        provider_denied: nested_u64(record, "provider_policy", "denied"),
        auth_allowed: nested_u64(record, "auth_broker", "allowed"),
        auth_denied: nested_u64(record, "auth_broker", "denied"),
        cleanup: cleanup_status(record.get("cleanup")),
    })
}

pub(super) fn project_egress_record(record: &Value) -> Option<DecisionSummary> {
    project_decision(record, "network-policy")
}

pub(super) fn project_auth_record(record: &Value) -> Option<DecisionSummary> {
    project_decision(record, "sign-in-broker")
}

fn project_decision(record: &Value, category: &str) -> Option<DecisionSummary> {
    Some(DecisionSummary {
        timestamp: safe_timestamp(record.get("timestamp")?.as_str()?),
        profile: allowed_profile(record.get("profile")?.as_str()?),
        decision: allowed_decision(record.get("decision")?.as_str()?),
        count: record.get("count").and_then(Value::as_u64).unwrap_or(1),
        category: category.to_string(),
    })
}

fn nested_u64(record: &Value, parent: &str, field: &str) -> u64 {
    record
        .pointer(&format!("/{parent}/{field}"))
        .and_then(Value::as_u64)
        .unwrap_or(0)
}

fn cleanup_status(cleanup: Option<&Value>) -> String {
    let Some(object) = cleanup.and_then(Value::as_object) else {
        return "unavailable".to_string();
    };
    let provider_network = object.get("provider_network").and_then(Value::as_str);
    let run_container = object.get("run_container").and_then(Value::as_str);
    if provider_network
        .is_some_and(|status| matches!(status, "cleanup-error" | "delete-failed" | "launch-failed"))
        || run_container.is_some_and(|status| status == "remove-failed")
    {
        "failed".to_string()
    } else if provider_network.is_none() && run_container.is_none() {
        "unavailable".to_string()
    } else if provider_network.is_none_or(|status| {
        matches!(
            status,
            "deleted" | "not-applicable" | "not-created" | "removed"
        )
    }) && run_container.is_none_or(|status| matches!(status, "already-gone" | "removed"))
    {
        "complete".to_string()
    } else {
        "unavailable".to_string()
    }
}

fn safe_timestamp(value: &str) -> String {
    time::OffsetDateTime::parse(value, &Rfc3339)
        .ok()
        .and_then(|timestamp| timestamp.format(&Rfc3339).ok())
        .unwrap_or_else(|| "unavailable".to_string())
}

fn safe_app_version(value: &str) -> String {
    let filtered = value
        .chars()
        .take(96)
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_' | '+' | ' ')
            {
                character
            } else {
                '_'
            }
        })
        .collect::<String>();
    if filtered.trim().is_empty() {
        "unavailable".to_string()
    } else {
        filtered
    }
}

fn valid_version(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 32
        && value
            .chars()
            .all(|character| character.is_ascii_digit() || character == '.')
}

fn allowed_architecture(value: &str) -> String {
    match value {
        "aarch64" | "arm64" => "arm64".to_string(),
        _ => "unsupported".to_string(),
    }
}

fn allowed_profile(value: &str) -> String {
    match value {
        "antigravity" | "claude" | "codex" | "copilot" => value.to_string(),
        _ => "unknown".to_string(),
    }
}

fn allowed_network(value: &str) -> String {
    match value {
        "provider" | "internal" | "internet" => value.to_string(),
        _ => "unknown".to_string(),
    }
}

fn allowed_auth_scope(value: &str) -> String {
    match value {
        "agent" | "project" => value.to_string(),
        _ => "unknown".to_string(),
    }
}

fn allowed_run_status(value: &str) -> String {
    match value {
        "ended" | "failed" | "killed" | "running" | "stopped" | "succeeded" => value.to_string(),
        _ => "unknown".to_string(),
    }
}

fn allowed_decision(value: &str) -> String {
    match value {
        "allowed" | "denied" => value.to_string(),
        _ => "unknown".to_string(),
    }
}

fn allowed_image_status(value: &str) -> String {
    match value {
        "ok" | "missing" | "stale" => value.to_string(),
        _ => "unavailable".to_string(),
    }
}
