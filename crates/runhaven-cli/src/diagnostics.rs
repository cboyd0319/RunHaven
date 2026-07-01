use anyhow::Result;
use serde_json::{Value, json};

use runhaven_core::diagnostics::{
    auth_status_payload, read_auth_broker_log, read_egress_policy_log,
};
use runhaven_core::provider::auth_profiles::{
    AUTH_BROKER_RUNTIME, AUTH_BROKER_STATUS, auth_broker_profiles, get_auth_broker_profile,
};
use runhaven_core::runtime::profiles::get_profile;

mod diagnostics_why;

pub use diagnostics_why::{why_host, why_network, why_state, why_workspace};

pub fn egress_log(limit: usize, json_output: bool) -> Result<i32> {
    let entries = read_egress_policy_log(limit)?;
    if json_output {
        println!("{}", serde_json::to_string_pretty(&entries)?);
        return Ok(0);
    }
    if entries.is_empty() {
        println!("No RunHaven provider egress policy log entries found.");
        return Ok(0);
    }
    for entry in entries {
        println!(
            "{}  {}  {}  {}:{}  count={}  reason={}  rule={}  run={}",
            entry
                .get("timestamp")
                .and_then(Value::as_str)
                .unwrap_or("<unknown>"),
            entry
                .get("profile")
                .and_then(Value::as_str)
                .unwrap_or("unknown"),
            entry
                .get("decision")
                .and_then(Value::as_str)
                .unwrap_or("unknown"),
            entry
                .get("host")
                .and_then(Value::as_str)
                .unwrap_or("<unknown>"),
            entry
                .get("port")
                .map(Value::to_string)
                .unwrap_or_else(|| "?".to_string()),
            entry
                .get("count")
                .map(Value::to_string)
                .unwrap_or_else(|| "1".to_string()),
            entry
                .get("reason")
                .and_then(Value::as_str)
                .unwrap_or("unknown"),
            entry
                .get("matched_rule")
                .and_then(Value::as_str)
                .unwrap_or("-"),
            entry.get("run_id").and_then(Value::as_str).unwrap_or("-"),
        );
    }
    Ok(0)
}

pub fn auth_log(limit: usize, json_output: bool) -> Result<i32> {
    let entries = read_auth_broker_log(limit)?;
    if json_output {
        println!("{}", serde_json::to_string_pretty(&entries)?);
        return Ok(0);
    }
    if entries.is_empty() {
        println!("No RunHaven auth broker log entries found.");
        return Ok(0);
    }
    for entry in entries {
        println!(
            "{}  {}  {}  {}  {} {}  status={}  count={}  reason={}  run={}",
            entry
                .get("timestamp")
                .and_then(Value::as_str)
                .unwrap_or("<unknown>"),
            entry
                .get("profile")
                .and_then(Value::as_str)
                .unwrap_or("unknown"),
            entry
                .get("broker")
                .and_then(Value::as_str)
                .unwrap_or("unknown"),
            entry
                .get("decision")
                .and_then(Value::as_str)
                .unwrap_or("unknown"),
            entry.get("method").and_then(Value::as_str).unwrap_or("-"),
            entry.get("path").and_then(Value::as_str).unwrap_or("-"),
            entry
                .get("upstream_status")
                .map(Value::to_string)
                .unwrap_or_else(|| "-".to_string()),
            entry
                .get("count")
                .map(Value::to_string)
                .unwrap_or_else(|| "1".to_string()),
            entry
                .get("reason")
                .and_then(Value::as_str)
                .unwrap_or("unknown"),
            entry.get("run_id").and_then(Value::as_str).unwrap_or("-"),
        );
    }
    Ok(0)
}

pub fn auth_status(json_output: bool) -> Result<i32> {
    if json_output {
        println!("{}", serde_json::to_string_pretty(&auth_status_payload())?);
        return Ok(0);
    }
    let profiles = auth_broker_profiles();
    println!("Auth broker: {AUTH_BROKER_STATUS}");
    println!("Runtime: {AUTH_BROKER_RUNTIME}");
    println!("Credential stores inspected: no");
    println!("Environment values inspected: no");
    println!("Secrets printed: no");
    println!("Profiles:");
    let width = profiles
        .iter()
        .map(|profile| profile.name.len())
        .max()
        .unwrap_or(0);
    for profile in profiles {
        println!(
            "  {:width$}  {}",
            profile.name,
            profile.status,
            width = width
        );
    }
    println!("Current safe paths:");
    println!("  - authenticate inside the isolated agent state volume when interactive");
    println!("  - use the Codex API-key broker for headless Codex API-key runs");
    println!("  - pass one token with --env NAME only when explicitly needed");
    println!("  - use --network provider to constrain provider egress separately");
    Ok(0)
}

pub fn auth_explain(agent: &str, json_output: bool) -> Result<i32> {
    let profile = get_profile(agent)?;
    let auth_profile = get_auth_broker_profile(profile.name)?;
    let payload = json!({
        "name": auth_profile.name,
        "status": auth_profile.status,
        "supported_auth": auth_profile.supported_auth,
        "host_keeps": auth_profile.host_keeps,
        "guest_receives": auth_profile.guest_receives,
        "current_safe_path": auth_profile.current_safe_path,
        "notes": auth_profile.notes,
        "runtime": AUTH_BROKER_RUNTIME,
        "credential_stores_inspected": false,
        "environment_values_inspected": false,
        "secrets_printed": false,
        "provider_hosts": profile.provider_hosts,
    });
    if json_output {
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(0);
    }
    println!("Profile: {}", profile.name);
    println!("Auth broker: {}", auth_profile.status);
    println!("Runtime: {AUTH_BROKER_RUNTIME}");
    println!("Credential stores inspected: no");
    println!("Environment values inspected: no");
    println!("Secrets printed: no");
    println!("Supported auth surfaces:");
    for item in auth_profile.supported_auth {
        println!("  - {item}");
    }
    println!("Host keeps:");
    for item in auth_profile.host_keeps {
        println!("  - {item}");
    }
    println!("Guest receives:");
    for item in auth_profile.guest_receives {
        println!("  - {item}");
    }
    if profile.provider_hosts.is_empty() {
        println!("Provider hosts: none bundled");
    } else {
        println!("Provider hosts: {}", profile.provider_hosts.join(", "));
    }
    println!("Current safe path: {}", auth_profile.current_safe_path);
    if !auth_profile.notes.is_empty() {
        println!("Notes:");
        for note in auth_profile.notes {
            println!("  - {note}");
        }
    }
    Ok(0)
}
