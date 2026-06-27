use std::path::Path;

use anyhow::{Result, bail};
use serde_json::{Value, json};

use runhaven_core::diagnostics::{
    auth_status_payload, read_auth_broker_log, read_egress_policy_log,
};
use runhaven_core::provider::auth_profiles::{
    AUTH_BROKER_RUNTIME, AUTH_BROKER_STATUS, auth_broker_profiles, get_auth_broker_profile,
};
use runhaven_core::provider::egress::{EgressPolicy, is_ip_literal, normalize_host};
use runhaven_core::provider::endpoints::{ProviderEndpoint, match_provider_endpoints};
use runhaven_core::runtime::plans::{
    NetworkMode, SUPPORTED_NETWORK_MODES, WorkspaceScope, apply_workspace_scope, validate_workspace,
};
use runhaven_core::runtime::profiles::{AgentProfile, get_profile, profiles};
use runhaven_core::runtime::session_state::SESSION_DEFAULT;

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

pub fn why_host(host: &str, port: u16, agent: Option<&str>) -> Result<i32> {
    if port == 0 {
        bail!("--port must be between 1 and 65535");
    }
    let normalized = normalize_host(host)?;
    println!("Host: {normalized}");
    println!("Port: {port}");
    if is_ip_literal(&normalized) {
        println!("Provider mode: denied");
        println!("Reason: IP literal targets cannot be allowed in provider mode.");
        println!("Next action: use a reviewed fully qualified provider hostname instead.");
        return Ok(0);
    }
    if !normalized.contains('.') {
        println!("Provider mode: denied");
        println!("Reason: provider hosts must be fully qualified, not single-label names.");
        println!("Next action: use a specific hostname such as api.example.com.");
        return Ok(0);
    }
    if let Some(agent) = agent {
        let profile = get_profile(agent)?;
        println!("Provider profile: {}", profile.name);
        if profile.provider_hosts.is_empty() {
            println!("Provider mode: no bundled provider hosts are defined for this profile.");
            println!(
                "Next action: use --provider-host only after reviewing a fully qualified host."
            );
            return Ok(0);
        }
        let policy = EgressPolicy::new(
            &profile
                .provider_hosts
                .iter()
                .map(|host| (*host).to_string())
                .collect::<Vec<_>>(),
        )?;
        if let Some(rule) = policy.match_rule(&normalized, port) {
            println!("Provider mode: allowed by bundled provider profile");
            println!("Matched rule: {rule}");
            println!("DNS safety: checked at runtime before the proxy opens the connection.");
            return Ok(0);
        }
        println!("Provider mode: not allowed by bundled provider profile");
        println!("Bundled hosts: {}", profile.provider_hosts.join(", "));
        print_endpoint_matches(&match_provider_endpoints(&normalized, Some(profile.name)));
        println!("Next action: review before rerunning with --provider-host {normalized}.");
        return Ok(0);
    }
    let mut matches = Vec::new();
    for profile in profiles() {
        if profile.provider_hosts.is_empty() {
            continue;
        }
        let hosts = profile
            .provider_hosts
            .iter()
            .map(|host| (*host).to_string())
            .collect::<Vec<_>>();
        let policy = EgressPolicy::new(&hosts)?;
        if let Some(rule) = policy.match_rule(&normalized, port) {
            matches.push(format!("{} ({rule})", profile.name));
        }
    }
    if matches.is_empty() {
        println!("Provider mode: not allowed by any bundled provider profile");
        print_endpoint_matches(&match_provider_endpoints(&normalized, None));
        println!("Next action: review before rerunning with --provider-host {normalized}.");
    } else {
        println!("Provider mode: allowed by bundled profile(s)");
        println!("Matches: {}", matches.join(", "));
    }
    println!("DNS safety: checked at runtime before the proxy opens the connection.");
    Ok(0)
}

pub fn why_workspace(
    workspace: &Path,
    workspace_scope: &str,
    allow_sensitive_workspace: bool,
) -> Result<i32> {
    let scope = WorkspaceScope::try_from(workspace_scope)?;
    println!("Workspace input: {}", workspace.display());
    println!("Workspace scope: {}", scope.as_str());
    let Ok(resolved) = workspace.canonicalize() else {
        println!("Mount decision: denied");
        println!("Reason: workspace path does not exist or cannot be resolved.");
        println!("Next action: pass an existing project directory.");
        return Ok(0);
    };
    println!("Resolved path: {}", resolved.display());
    if !resolved.is_dir() {
        println!("Mount decision: denied");
        println!("Reason: workspace path is not a directory.");
        println!("Next action: pass a project directory, not a file.");
        return Ok(0);
    }
    let (mounted, note) = match apply_workspace_scope(&resolved, scope) {
        Ok(value) => value,
        Err(error) => {
            println!("Mount decision: denied");
            println!("Reason: {error}");
            println!("Next action: use --workspace-scope current or run from a git worktree.");
            return Ok(0);
        }
    };
    println!("Mounted path: {}", mounted.display());
    if let Some(note) = note {
        println!("Scope note: {note}");
    }
    match validate_workspace(&mounted, allow_sensitive_workspace) {
        Ok(()) => {
            println!("Mount decision: allowed");
            if allow_sensitive_workspace && let Err(error) = validate_workspace(&mounted, false) {
                println!("Default decision: denied");
                println!("Override reason: {error}");
            }
            println!(
                "Boundary: only the mounted path is exposed at /workspace; agent home stays in a RunHaven state volume."
            );
        }
        Err(error) => {
            println!("Mount decision: denied");
            println!("Reason: {error}");
            println!(
                "Next action: choose a project subdirectory or pass --allow-sensitive-workspace intentionally."
            );
        }
    }
    Ok(0)
}

pub fn why_network(mode: &str) -> Result<i32> {
    let network_mode = match NetworkMode::try_from(mode) {
        Ok(mode) => mode,
        Err(_) => bail!(
            "invalid network mode: {mode:?}. Expected one of: {}",
            SUPPORTED_NETWORK_MODES.join(", ")
        ),
    };
    println!("Network mode: {}", network_mode.as_str());
    match network_mode {
        NetworkMode::Internet => {
            println!("Behavior: uses Apple container's default internet networking.");
            println!("Boundary: provider domain allowlisting is not enforced.");
            println!("Use when: the task needs normal package installs or broad internet access.");
        }
        NetworkMode::Internal => {
            println!("Behavior: creates a managed internal Apple container network.");
            println!("Boundary: internet egress is disabled for the agent container.");
            println!("Use when: the task can run from local files and existing dependencies.");
        }
        NetworkMode::Provider => {
            println!(
                "Behavior: creates a managed internal network and routes egress through RunHaven's allowlist proxy."
            );
            println!(
                "Boundary: only bundled provider hosts and explicit --provider-host entries are eligible."
            );
            println!(
                "DNS safety: IP literals, single-label names, and non-public resolved addresses are denied."
            );
            println!("Inspect: runhaven why host HOST --agent AGENT, then runhaven egress log.");
        }
    }
    Ok(0)
}

pub fn why_state(agent: &str) -> Result<i32> {
    let profile = get_profile(agent)?;
    print_state_explanation(profile);
    Ok(0)
}

fn print_state_explanation(profile: AgentProfile) {
    println!("Profile: {}", profile.name);
    println!("Default session: {SESSION_DEFAULT}");
    println!("Home mount: /home/agent");
    println!(
        "State volume pattern: runhaven-{}-<project-id>-home",
        profile.name
    );
    println!(
        "Named session pattern: runhaven-{}-<project-id>-s-<session>-<digest>-home",
        profile.name
    );
    println!("Boundary: workspace files are mounted separately at /workspace.");
    println!(
        "Project id: derived from the resolved workspace path; the path is not embedded in the volume name."
    );
    println!(
        "Manage: runhaven state list, runhaven state reset AGENT --workspace PATH, runhaven state prune."
    );
}

fn print_endpoint_matches(matches: &[ProviderEndpoint]) {
    if matches.is_empty() {
        return;
    }
    println!("Known endpoint record(s):");
    for endpoint in matches {
        println!(
            "  - {}: {}; {}",
            endpoint.profile, endpoint.status, endpoint.purpose
        );
        if !endpoint.note.is_empty() {
            println!("    Note: {}", endpoint.note);
        }
    }
}
