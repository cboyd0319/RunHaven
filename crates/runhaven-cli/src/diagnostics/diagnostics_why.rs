use std::path::Path;

use anyhow::{Result, bail};

use runhaven_core::diagnostics::{
    why_host_payload, why_network_payload, why_state_payload, why_workspace_payload,
};
use runhaven_core::provider::egress::{EgressPolicy, is_ip_literal, normalize_host};
use runhaven_core::provider::endpoints::{ProviderEndpoint, match_provider_endpoints};
use runhaven_core::runtime::plans::{
    NetworkMode, SUPPORTED_NETWORK_MODES, WorkspaceScope, apply_workspace_scope, validate_workspace,
};
use runhaven_core::runtime::profiles::{AgentProfile, get_profile, profiles};
use runhaven_core::runtime::session_state::SESSION_DEFAULT;

pub fn why_host(host: &str, port: u16, agent: Option<&str>, json_output: bool) -> Result<i32> {
    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&why_host_payload(host, port, agent)?)?
        );
        return Ok(0);
    }
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
        return why_host_for_agent(&normalized, port, agent);
    }
    why_host_across_profiles(&normalized, port)
}

fn why_host_for_agent(normalized: &str, port: u16, agent: &str) -> Result<i32> {
    let profile = get_profile(agent)?;
    println!("Provider profile: {}", profile.name);
    if profile.provider_hosts.is_empty() {
        println!("Provider mode: no bundled provider hosts are defined for this profile.");
        println!("Next action: use --provider-host only after reviewing a fully qualified host.");
        return Ok(0);
    }
    let policy = policy_for_hosts(profile.provider_hosts)?;
    if let Some(rule) = policy.match_rule(normalized, port) {
        println!("Provider mode: allowed by bundled provider profile");
        println!("Matched rule: {rule}");
        println!("DNS safety: checked at runtime before the proxy opens the connection.");
        return Ok(0);
    }
    println!("Provider mode: not allowed by bundled provider profile");
    println!("Bundled hosts: {}", profile.provider_hosts.join(", "));
    print_endpoint_matches(&match_provider_endpoints(normalized, Some(profile.name)));
    println!("Next action: review before rerunning with --provider-host {normalized}.");
    Ok(0)
}

fn why_host_across_profiles(normalized: &str, port: u16) -> Result<i32> {
    let mut matches = Vec::new();
    for profile in profiles() {
        if profile.provider_hosts.is_empty() {
            continue;
        }
        let policy = policy_for_hosts(profile.provider_hosts)?;
        if let Some(rule) = policy.match_rule(normalized, port) {
            matches.push(format!("{} ({rule})", profile.name));
        }
    }
    if matches.is_empty() {
        println!("Provider mode: not allowed by any bundled provider profile");
        print_endpoint_matches(&match_provider_endpoints(normalized, None));
        println!("Next action: review before rerunning with --provider-host {normalized}.");
    } else {
        println!("Provider mode: allowed by bundled profile(s)");
        println!("Matches: {}", matches.join(", "));
    }
    println!("DNS safety: checked at runtime before the proxy opens the connection.");
    Ok(0)
}

fn policy_for_hosts(hosts: &[&str]) -> Result<EgressPolicy> {
    EgressPolicy::new(
        &hosts
            .iter()
            .map(|host| (*host).to_string())
            .collect::<Vec<_>>(),
    )
}

pub fn why_workspace(
    workspace: &Path,
    workspace_scope: &str,
    allow_sensitive_workspace: bool,
    json_output: bool,
) -> Result<i32> {
    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&why_workspace_payload(
                workspace,
                workspace_scope,
                allow_sensitive_workspace
            )?)?
        );
        return Ok(0);
    }
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
    print_workspace_decision(&resolved, scope, allow_sensitive_workspace)
}

fn print_workspace_decision(
    resolved: &Path,
    scope: WorkspaceScope,
    allow_sensitive_workspace: bool,
) -> Result<i32> {
    let (mounted, note) = match apply_workspace_scope(resolved, scope) {
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
        Ok(()) => print_workspace_allowed(&mounted, allow_sensitive_workspace),
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

fn print_workspace_allowed(mounted: &Path, allow_sensitive_workspace: bool) {
    println!("Mount decision: allowed");
    if allow_sensitive_workspace && let Err(error) = validate_workspace(mounted, false) {
        println!("Default decision: denied");
        println!("Override reason: {error}");
    }
    println!(
        "Boundary: only the mounted path is exposed at /workspace; agent home stays in a RunHaven state volume."
    );
}

pub fn why_network(mode: &str, json_output: bool) -> Result<i32> {
    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&why_network_payload(mode)?)?
        );
        return Ok(0);
    }
    let network_mode = match NetworkMode::try_from(mode) {
        Ok(mode) => mode,
        Err(_) => bail!(
            "invalid network mode: {mode:?}. Expected one of: {}",
            SUPPORTED_NETWORK_MODES.join(", ")
        ),
    };
    println!("Network mode: {}", network_mode.as_str());
    match network_mode {
        NetworkMode::Internet => print_internet_mode(),
        NetworkMode::Internal => print_internal_mode(),
        NetworkMode::Provider => print_provider_mode(),
    }
    Ok(0)
}

fn print_internet_mode() {
    println!("Behavior: uses Apple container's default internet networking.");
    println!("Boundary: provider domain allowlisting is not enforced.");
    println!("Use when: the task needs normal package installs or broad internet access.");
}

fn print_internal_mode() {
    println!("Behavior: creates a managed internal Apple container network.");
    println!("Boundary: internet egress is disabled for the agent container.");
    println!("Use when: the task can run from local files and existing dependencies.");
}

fn print_provider_mode() {
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

pub fn why_state(agent: &str, json_output: bool) -> Result<i32> {
    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&why_state_payload(agent)?)?
        );
        return Ok(0);
    }
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
