use runhaven_core::diagnostics::{
    why_host_payload, why_network_payload, why_state_payload, why_workspace_payload,
};
use serde_json::Value;

use super::diagnostics::{
    bool_field, optional_string_field, string_field, string_list_field, u32_field,
};
use super::validation::{
    MAX_AGENT_NAME_LEN, MAX_PROVIDER_HOST_LEN, MAX_SHORT_FIELD_LEN, MAX_WORKSPACE_PATH_LEN,
    validate_text_len,
};
use crate::contracts::{
    HostExplanationResponse, NetworkExplanationResponse, StateExplanationResponse, WhyHostRequest,
    WhyWorkspaceRequest, WorkspaceExplanationResponse,
};

// Read-only desktop bindings for the CLI `why` boundary explanations. Each
// command validates its untrusted WebView input for length before calling the
// shared core builder, then maps only secret-free, path-free fields into a typed
// contract. All four are read-only: they mutate nothing and expose no path.

#[tauri::command]
pub(crate) fn get_why_network(mode: String) -> Result<NetworkExplanationResponse, String> {
    validate_text_len("network mode", &mode, MAX_SHORT_FIELD_LEN)?;
    let payload = why_network_payload(&mode).map_err(|error| error.to_string())?;
    Ok(network_response(&payload))
}

#[tauri::command]
pub(crate) fn get_why_state(agent: String) -> Result<StateExplanationResponse, String> {
    validate_text_len("agent", &agent, MAX_AGENT_NAME_LEN)?;
    let payload = why_state_payload(&agent).map_err(|error| error.to_string())?;
    Ok(state_response(&payload))
}

#[tauri::command]
pub(crate) fn get_why_host(request: WhyHostRequest) -> Result<HostExplanationResponse, String> {
    validate_text_len("host", &request.host, MAX_PROVIDER_HOST_LEN)?;
    if let Some(agent) = request.agent.as_deref() {
        validate_text_len("agent", agent, MAX_AGENT_NAME_LEN)?;
    }
    let payload = why_host_payload(&request.host, request.port, request.agent.as_deref())
        .map_err(|error| error.to_string())?;
    Ok(host_response(&payload))
}

#[tauri::command]
pub(crate) fn get_why_workspace(
    request: WhyWorkspaceRequest,
) -> Result<WorkspaceExplanationResponse, String> {
    validate_text_len("workspace path", &request.path, MAX_WORKSPACE_PATH_LEN)?;
    validate_text_len(
        "workspace scope",
        &request.workspace_scope,
        MAX_SHORT_FIELD_LEN,
    )?;
    let payload = why_workspace_payload(
        std::path::Path::new(&request.path),
        &request.workspace_scope,
        request.allow_sensitive_workspace,
    )
    .map_err(|error| error.to_string())?;
    Ok(workspace_response(&payload))
}

fn network_response(payload: &Value) -> NetworkExplanationResponse {
    NetworkExplanationResponse {
        mode: string_field(payload, "mode"),
        behavior: string_field(payload, "behavior"),
        boundary: string_field(payload, "boundary"),
        use_when: optional_string_field(payload, "use_when"),
        dns_safety: optional_string_field(payload, "dns_safety"),
        inspect: optional_string_field(payload, "inspect"),
    }
}

fn state_response(payload: &Value) -> StateExplanationResponse {
    StateExplanationResponse {
        profile: string_field(payload, "profile"),
        default_session: string_field(payload, "default_session"),
        home_mount: string_field(payload, "home_mount"),
        state_volume_pattern: string_field(payload, "state_volume_pattern"),
        named_session_pattern: string_field(payload, "named_session_pattern"),
        boundary: string_field(payload, "boundary"),
        project_id: string_field(payload, "project_id"),
        manage: string_field(payload, "manage"),
    }
}

fn host_response(payload: &Value) -> HostExplanationResponse {
    HostExplanationResponse {
        host: string_field(payload, "host"),
        port: u32_field(payload, "port"),
        decision: string_field(payload, "decision"),
        reason: optional_string_field(payload, "reason"),
        next_action: optional_string_field(payload, "next_action"),
        agent: optional_string_field(payload, "agent"),
        profile: optional_string_field(payload, "profile"),
        matched_rules: string_list_field(payload, "matched_rules"),
        bundled_hosts: string_list_field(payload, "bundled_hosts"),
        endpoint_notes: endpoint_notes(payload),
        dns_safety: optional_string_field(payload, "dns_safety"),
    }
}

// Flatten the structured endpoint records into short, path-free note lines. The
// record fields are static provider-endpoint table data, never host state.
fn endpoint_notes(payload: &Value) -> Vec<String> {
    payload
        .get("endpoint_records")
        .and_then(Value::as_array)
        .map(|records| {
            records
                .iter()
                .map(|record| {
                    let mut note = format!(
                        "{}: {}; {}",
                        string_field(record, "profile"),
                        string_field(record, "status"),
                        string_field(record, "purpose"),
                    );
                    let extra = optional_string_field(record, "note");
                    if !extra.is_empty() {
                        note.push_str(&format!(" ({extra})"));
                    }
                    note
                })
                .collect()
        })
        .unwrap_or_default()
}

// Maps only the path-free verdict fields. `input_path`, `resolved_path`,
// `mounted_path`, `reason`, `note`, and `override_reason` are intentionally not
// read here: they may carry an absolute host path and must not reach the WebView.
fn workspace_response(payload: &Value) -> WorkspaceExplanationResponse {
    WorkspaceExplanationResponse {
        scope: string_field(payload, "scope"),
        decision: string_field(payload, "decision"),
        verdict: string_field(payload, "verdict"),
        next_action: optional_string_field(payload, "next_action"),
        boundary: optional_string_field(payload, "boundary"),
        sensitive_override_used: bool_field(payload, "sensitive_override_used"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_why_network_reports_provider_mode() {
        let response = get_why_network("provider".to_string()).expect("provider mode");
        assert_eq!(response.mode, "provider");
        assert!(!response.behavior.is_empty());
        assert!(!response.dns_safety.is_empty());
    }

    #[test]
    fn get_why_network_rejects_unknown_and_overlong_mode() {
        assert!(get_why_network("bogus".to_string()).is_err());
        let error = get_why_network("a".repeat(MAX_SHORT_FIELD_LEN + 1))
            .expect_err("overlong mode must fail closed");
        assert!(error.contains("too long"));
    }

    #[test]
    fn get_why_state_reports_known_agent() {
        let response = get_why_state("claude".to_string()).expect("known agent");
        assert_eq!(response.profile, "claude");
        assert!(response.state_volume_pattern.contains("<project-id>"));
    }

    #[test]
    fn get_why_state_rejects_unknown_and_overlong_agent() {
        assert!(get_why_state("not-a-real-agent".to_string()).is_err());
        let error = get_why_state("a".repeat(MAX_AGENT_NAME_LEN + 1))
            .expect_err("overlong agent must fail closed");
        assert!(error.contains("too long"));
    }

    fn host_request(host: &str, agent: Option<&str>) -> WhyHostRequest {
        WhyHostRequest {
            host: host.to_string(),
            port: 443,
            agent: agent.map(str::to_string),
        }
    }

    fn workspace_request(path: &str, scope: &str) -> WhyWorkspaceRequest {
        WhyWorkspaceRequest {
            path: path.to_string(),
            workspace_scope: scope.to_string(),
            allow_sensitive_workspace: false,
        }
    }

    #[test]
    fn get_why_host_allows_bundled_host_and_rejects_overlong() {
        let response =
            get_why_host(host_request("api.openai.com", Some("codex"))).expect("codex host");
        assert_eq!(response.decision, "allowed");
        assert_eq!(response.profile, "codex");
        assert!(!response.matched_rules.is_empty());

        let error = get_why_host(host_request(&"a".repeat(MAX_PROVIDER_HOST_LEN + 1), None))
            .expect_err("overlong host must fail closed");
        assert!(error.contains("too long"));
    }

    #[test]
    fn get_why_host_maps_unmatched_records_without_paths() {
        let response =
            get_why_host(host_request("api.example.com", Some("codex"))).expect("unmatched host");
        assert_eq!(response.decision, "not-allowed");
        assert!(!response.bundled_hosts.is_empty());
    }

    #[test]
    fn get_why_workspace_reports_verdict_without_paths() {
        let dir = tempfile::tempdir().expect("dir");
        let response = get_why_workspace(workspace_request(
            &dir.path().display().to_string(),
            "current",
        ))
        .expect("existing dir");
        assert_eq!(response.decision, "allowed");
        // The response never carries the resolved absolute path.
        let serialized = serde_json::to_string(&response).expect("serialize");
        assert!(!serialized.contains(&dir.path().display().to_string()));
    }

    #[test]
    fn get_why_workspace_rejects_overlong_scope() {
        let error = get_why_workspace(workspace_request(".", &"a".repeat(MAX_SHORT_FIELD_LEN + 1)))
            .expect_err("overlong scope must fail closed");
        assert!(error.contains("too long"));
    }

    #[test]
    fn workspace_response_never_maps_raw_host_paths() {
        // Seed a payload that carries fake host paths and a path-bearing reason,
        // exactly as the core builder would for a denied or allowed decision.
        let payload = serde_json::json!({
            "scope": "current",
            "decision": "denied",
            "verdict": "RunHaven will not mount this folder by default.",
            "reason": "sensitive workspace requires --allow-sensitive-workspace: /Users/secret/.ssh",
            "next_action": "choose a project subdirectory or pass --allow-sensitive-workspace intentionally.",
            "note": "selected workspace is inside git repository root /Users/secret/project",
            "input_path": "/Users/secret/project/.ssh",
            "resolved_path": "/Users/secret/project/.ssh",
            "mounted_path": "/Users/secret/project/.ssh",
            "override_reason": "sensitive workspace requires --allow-sensitive-workspace: /Users/secret/.ssh",
            "sensitive_override_used": false,
        });
        let response = workspace_response(&payload);
        let serialized = serde_json::to_string(&response).expect("serialize");
        assert!(!serialized.contains("/Users/secret"));
        assert!(!serialized.contains(".ssh"));
        assert!(!serialized.contains("git repository root"));
        assert_eq!(response.decision, "denied");
        assert!(!response.next_action.is_empty());
    }

    #[test]
    fn host_response_flattens_records_without_leaking_extra_fields() {
        let payload = serde_json::json!({
            "host": "api.example.com",
            "port": 443,
            "decision": "not-allowed",
            "endpoint_records": [
                {"profile": "codex", "status": "active", "purpose": "model calls", "note": "live"}
            ],
            "next_action": "review before rerunning with --provider-host api.example.com.",
            // A stray secret-looking field must never be mapped onto the contract.
            "secret_token": "sk-should-never-map",
        });
        let response = host_response(&payload);
        assert_eq!(response.endpoint_notes.len(), 1);
        assert!(response.endpoint_notes[0].contains("codex"));
        let serialized = serde_json::to_string(&response).expect("serialize");
        assert!(!serialized.contains("sk-should-never-map"));
    }
}
