use std::path::Path;

use anyhow::{Result, bail};
use serde_json::{Value, json};

use crate::provider::egress::{EgressPolicy, is_ip_literal, normalize_host};
use crate::provider::endpoints::{ProviderEndpoint, match_provider_endpoints};
use crate::runtime::plans::{
    NetworkMode, SUPPORTED_NETWORK_MODES, WorkspaceScope, apply_workspace_scope, validate_workspace,
};
use crate::runtime::profiles::{get_profile, profiles};
use crate::runtime::session_state::SESSION_DEFAULT;

// Shared, read-only "why is this safe" explanation builders. Each returns the
// same JSON the CLI `why ... --json` path prints and the desktop maps into a
// secret-free typed contract. No builder reads a credential store or prints a
// secret. The workspace builder does resolve a path on disk (the CLI does the
// same), and it also emits path-free `verdict`/`decision` fields so the desktop
// can show a boundary verdict without exposing a raw host path.

const DNS_SAFETY: &str = "checked at runtime before the proxy opens the connection.";

/// Explain a network mode boundary. Errors on an unknown mode, matching the CLI.
pub fn why_network_payload(mode: &str) -> Result<Value> {
    let network_mode = match NetworkMode::try_from(mode) {
        Ok(mode) => mode,
        Err(_) => bail!(
            "invalid network mode: {mode:?}. Expected one of: {}",
            SUPPORTED_NETWORK_MODES.join(", ")
        ),
    };
    Ok(match network_mode {
        NetworkMode::Internet => json!({
            "mode": network_mode.as_str(),
            "behavior": "uses Apple container's default internet networking.",
            "boundary": "provider domain allowlisting is not enforced.",
            "use_when": "the task needs normal package installs or broad internet access.",
        }),
        NetworkMode::Internal => json!({
            "mode": network_mode.as_str(),
            "behavior": "creates a managed internal Apple container network.",
            "boundary": "internet egress is disabled for the agent container.",
            "use_when": "the task can run from local files and existing dependencies.",
        }),
        NetworkMode::Provider => json!({
            "mode": network_mode.as_str(),
            "behavior": "creates a managed internal network and routes egress through RunHaven's allowlist proxy.",
            "boundary": "only bundled provider hosts and explicit --provider-host entries are eligible.",
            "dns_safety": "IP literals, single-label names, and non-public resolved addresses are denied.",
            "inspect": "runhaven why host HOST --agent AGENT, then runhaven egress log.",
        }),
    })
}

/// Explain the state-volume isolation for an agent. Errors on an unknown agent.
pub fn why_state_payload(agent: &str) -> Result<Value> {
    let profile = get_profile(agent)?;
    Ok(json!({
        "profile": profile.name,
        "default_session": SESSION_DEFAULT,
        "home_mount": "/home/agent",
        "state_volume_pattern": format!("runhaven-{}-<project-id>-home", profile.name),
        "named_session_pattern": format!(
            "runhaven-{}-<project-id>-s-<session>-<digest>-home",
            profile.name
        ),
        "boundary": "workspace files are mounted separately at /workspace.",
        "project_id": "derived from the resolved workspace path; the path is not embedded in the volume name.",
        "manage": "runhaven state list, runhaven state reset AGENT --workspace PATH, runhaven state prune.",
    }))
}

/// Explain provider-host allowlist behavior for a host/port, optionally scoped to
/// one agent profile. Errors on an unresolvable host or a zero port, matching the
/// CLI. Never returns a secret; the host is caller-supplied and echoed back.
pub fn why_host_payload(host: &str, port: u16, agent: Option<&str>) -> Result<Value> {
    if port == 0 {
        bail!("--port must be between 1 and 65535");
    }
    let normalized = normalize_host(host)?;
    if is_ip_literal(&normalized) {
        return Ok(host_denied(
            &normalized,
            port,
            "IP literal targets cannot be allowed in provider mode.",
            "use a reviewed fully qualified provider hostname instead.",
        ));
    }
    if !normalized.contains('.') {
        return Ok(host_denied(
            &normalized,
            port,
            "provider hosts must be fully qualified, not single-label names.",
            "use a specific hostname such as api.example.com.",
        ));
    }
    match agent {
        Some(agent) => why_host_for_agent_payload(&normalized, port, agent),
        None => Ok(why_host_across_profiles_payload(&normalized, port)),
    }
}

fn host_denied(host: &str, port: u16, reason: &str, next_action: &str) -> Value {
    json!({
        "host": host,
        "port": port,
        "decision": "denied",
        "reason": reason,
        "next_action": next_action,
    })
}

fn why_host_for_agent_payload(normalized: &str, port: u16, agent: &str) -> Result<Value> {
    let profile = get_profile(agent)?;
    if profile.provider_hosts.is_empty() {
        return Ok(json!({
            "host": normalized,
            "port": port,
            "agent": agent,
            "profile": profile.name,
            "decision": "not-allowed",
            "reason": "no bundled provider hosts are defined for this profile.",
            "next_action": "use --provider-host only after reviewing a fully qualified host.",
        }));
    }
    let policy = policy_for_hosts(profile.provider_hosts)?;
    if let Some(rule) = policy.match_rule(normalized, port) {
        return Ok(json!({
            "host": normalized,
            "port": port,
            "agent": agent,
            "profile": profile.name,
            "decision": "allowed",
            "matched_rules": [rule],
            "dns_safety": DNS_SAFETY,
        }));
    }
    Ok(json!({
        "host": normalized,
        "port": port,
        "agent": agent,
        "profile": profile.name,
        "decision": "not-allowed",
        "bundled_hosts": profile.provider_hosts,
        "endpoint_records": endpoint_records(&match_provider_endpoints(normalized, Some(profile.name))),
        "next_action": format!("review before rerunning with --provider-host {normalized}."),
    }))
}

fn why_host_across_profiles_payload(normalized: &str, port: u16) -> Value {
    let mut matches = Vec::new();
    for profile in profiles() {
        if profile.provider_hosts.is_empty() {
            continue;
        }
        let Ok(policy) = policy_for_hosts(profile.provider_hosts) else {
            continue;
        };
        if let Some(rule) = policy.match_rule(normalized, port) {
            matches.push(format!("{} ({rule})", profile.name));
        }
    }
    if matches.is_empty() {
        json!({
            "host": normalized,
            "port": port,
            "decision": "not-allowed",
            "endpoint_records": endpoint_records(&match_provider_endpoints(normalized, None)),
            "next_action": format!("review before rerunning with --provider-host {normalized}."),
            "dns_safety": DNS_SAFETY,
        })
    } else {
        json!({
            "host": normalized,
            "port": port,
            "decision": "allowed",
            "matched_rules": matches,
            "dns_safety": DNS_SAFETY,
        })
    }
}

fn policy_for_hosts(hosts: &[&str]) -> Result<EgressPolicy> {
    EgressPolicy::new(
        &hosts
            .iter()
            .map(|host| (*host).to_string())
            .collect::<Vec<_>>(),
    )
}

fn endpoint_records(matches: &[ProviderEndpoint]) -> Value {
    Value::Array(
        matches
            .iter()
            .map(|endpoint| {
                json!({
                    "profile": endpoint.profile,
                    "status": endpoint.status,
                    "purpose": endpoint.purpose,
                    "note": endpoint.note,
                })
            })
            .collect(),
    )
}

/// Explain workspace mount validation for a path. Errors on an unknown scope,
/// matching the CLI. The payload carries raw host paths for CLI parity, plus
/// path-free `decision`/`verdict`/`next_action`/`boundary` fields so the desktop
/// can show a verdict without ever surfacing an absolute path.
pub fn why_workspace_payload(
    workspace: &Path,
    workspace_scope: &str,
    allow_sensitive_workspace: bool,
) -> Result<Value> {
    let scope = WorkspaceScope::try_from(workspace_scope)?;
    let input_path = workspace.display().to_string();
    let Ok(resolved) = workspace.canonicalize() else {
        return Ok(workspace_denied(
            scope,
            &input_path,
            None,
            None,
            "workspace path does not exist or cannot be resolved.",
            "RunHaven cannot find this folder.",
            "pass an existing project directory.",
        ));
    };
    let resolved_path = resolved.display().to_string();
    if !resolved.is_dir() {
        return Ok(workspace_denied(
            scope,
            &input_path,
            Some(&resolved_path),
            None,
            "workspace path is not a directory.",
            "This is a file, not a folder.",
            "pass a project directory, not a file.",
        ));
    }
    let (mounted, note) = match apply_workspace_scope(&resolved, scope) {
        Ok(value) => value,
        Err(error) => {
            return Ok(workspace_denied(
                scope,
                &input_path,
                Some(&resolved_path),
                None,
                &error.to_string(),
                "This folder is not inside a git project.",
                "use --workspace-scope current or run from a git worktree.",
            ));
        }
    };
    let mounted_path = mounted.display().to_string();
    match validate_workspace(&mounted, allow_sensitive_workspace) {
        Ok(()) => Ok(workspace_allowed(
            scope,
            &input_path,
            &resolved_path,
            &mounted_path,
            note.as_deref(),
            allow_sensitive_workspace,
            validate_workspace(&mounted, false),
        )),
        Err(error) => Ok(workspace_denied(
            scope,
            &input_path,
            Some(&resolved_path),
            note.as_deref(),
            &error.to_string(),
            "RunHaven will not mount this folder by default.",
            "choose a project subdirectory or pass --allow-sensitive-workspace intentionally.",
        )),
    }
}

fn workspace_denied(
    scope: WorkspaceScope,
    input_path: &str,
    resolved_path: Option<&str>,
    note: Option<&str>,
    reason: &str,
    verdict: &str,
    next_action: &str,
) -> Value {
    let mut payload = json!({
        "scope": scope.as_str(),
        "decision": "denied",
        "verdict": verdict,
        "reason": reason,
        "next_action": next_action,
        "input_path": input_path,
        "sensitive_override_used": false,
    });
    insert_optional(&mut payload, "resolved_path", resolved_path);
    insert_optional(&mut payload, "note", note);
    payload
}

fn workspace_allowed(
    scope: WorkspaceScope,
    input_path: &str,
    resolved_path: &str,
    mounted_path: &str,
    note: Option<&str>,
    allow_sensitive_workspace: bool,
    default_check: Result<()>,
) -> Value {
    let mut payload = json!({
        "scope": scope.as_str(),
        "decision": "allowed",
        "verdict": "RunHaven can mount this folder safely.",
        "boundary": "only the mounted path is exposed at /workspace; agent home stays in a RunHaven state volume.",
        "input_path": input_path,
        "resolved_path": resolved_path,
        "mounted_path": mounted_path,
        "sensitive_override_used": allow_sensitive_workspace,
    });
    insert_optional(&mut payload, "note", note);
    if allow_sensitive_workspace
        && let Err(error) = default_check
        && let Some(object) = payload.as_object_mut()
    {
        object.insert(
            "default_decision".to_string(),
            Value::String("denied".to_string()),
        );
        object.insert(
            "override_reason".to_string(),
            Value::String(error.to_string()),
        );
    }
    payload
}

fn insert_optional(payload: &mut Value, key: &str, value: Option<&str>) {
    if let (Some(object), Some(value)) = (payload.as_object_mut(), value) {
        object.insert(key.to_string(), Value::String(value.to_string()));
    }
}

#[cfg(test)]
mod tests;
