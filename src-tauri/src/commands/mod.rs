use std::path::PathBuf;

use runhaven::active::read_active_run_records;
use runhaven::doctor::collect_checks;
use runhaven::plans::{
    NetworkMode, RunOptions, WorkspaceScope, build_run_plan, normalize_provider_hosts,
};
use runhaven::profiles::{get_profile, profiles};
use runhaven::records::read_run_records;
use serde_json::Value;

use crate::contracts::{
    AgentProfileSummary, CheckStatus, DashboardStatus, PlanWarning, RunPlanRequest,
    RunPlanResponse, RunSummary, SetupStatus,
};

#[tauri::command]
pub(crate) fn get_setup_status() -> SetupStatus {
    collect_setup_status()
}

#[tauri::command]
pub(crate) fn list_agents() -> Vec<AgentProfileSummary> {
    agent_summaries()
}

#[tauri::command]
pub(crate) fn get_dashboard_status() -> DashboardStatus {
    collect_dashboard_status()
}

#[tauri::command]
pub(crate) fn plan_run(request: RunPlanRequest) -> Result<RunPlanResponse, String> {
    build_plan_response(request)
}

fn collect_setup_status() -> SetupStatus {
    let checks = collect_checks()
        .into_iter()
        .map(|check| CheckStatus {
            name: check.name,
            ok: check.ok,
            detail: check.detail,
            remedy: check.remedy,
        })
        .collect::<Vec<_>>();
    let blocker_count = checks.iter().filter(|check| !check.ok).count();
    SetupStatus {
        ok: blocker_count == 0,
        checks,
        blocker_count,
        ssh_available: false,
    }
}

fn agent_summaries() -> Vec<AgentProfileSummary> {
    profiles()
        .into_iter()
        .map(|profile| AgentProfileSummary {
            name: profile.name.to_string(),
            description: profile.description.to_string(),
            image: profile.image.to_string(),
            default_command: profile
                .command
                .iter()
                .map(|arg| (*arg).to_string())
                .collect(),
            provider_hosts: profile
                .provider_hosts
                .iter()
                .map(|host| (*host).to_string())
                .collect(),
        })
        .collect()
}

fn collect_dashboard_status() -> DashboardStatus {
    let mut warnings = Vec::new();
    let recent_runs = match read_run_records(10) {
        Ok(records) => records.iter().map(run_summary).collect(),
        Err(error) => {
            warnings.push(format!("Run history is unavailable: {error}"));
            Vec::new()
        }
    };
    DashboardStatus {
        setup: collect_setup_status(),
        agents: agent_summaries(),
        active_runs: read_active_run_records().iter().map(run_summary).collect(),
        recent_runs,
        warnings,
    }
}

fn build_plan_response(request: RunPlanRequest) -> Result<RunPlanResponse, String> {
    let profile = get_profile(&request.agent).map_err(|error| error.to_string())?;
    let network =
        NetworkMode::try_from(request.network_mode.as_str()).map_err(|error| error.to_string())?;
    let workspace_scope = WorkspaceScope::try_from(request.workspace_scope.as_str())
        .map_err(|error| error.to_string())?;
    let provider_hosts =
        normalize_provider_hosts(&request.provider_hosts).map_err(|error| error.to_string())?;
    let plan = build_run_plan(RunOptions {
        profile,
        workspace: PathBuf::from(&request.workspace_path),
        agent_args: Vec::new(),
        image: non_empty(request.image.clone()),
        cpus: defaulted(&request.cpus, "4"),
        memory: defaulted(&request.memory, "4g"),
        network,
        workspace_scope,
        session: non_empty(request.session_name.clone()),
        read_only_workspace: request.read_only_workspace,
        ssh: false,
        env: request.env_names.clone(),
        user: defaulted(&request.user, "agent"),
        interactive: false,
        tty: false,
        allow_sensitive_workspace: request.allow_sensitive_workspace,
        allow_root_user: request.allow_root_user,
        provider_hosts,
        codex_api_key_broker_env: None,
        worktree: None,
        run_id: None,
    })
    .map_err(|error| error.to_string())?;
    Ok(RunPlanResponse {
        profile: plan.profile_name,
        workspace: plan.workspace.display().to_string(),
        workspace_scope: plan.workspace_scope.as_str().to_string(),
        workspace_scope_note: plan.workspace_scope_note,
        state_volume: plan.state_volume,
        session: plan.session,
        container_name: plan.container_name,
        network_mode: plan.network_mode.as_str().to_string(),
        network_name: plan.network_name,
        egress_summary: plan.egress_summary,
        image: plan.image,
        provider_allowed_hosts: plan.provider_allowed_hosts,
        preflight_count: plan.preflight.len(),
        warnings: plan_warnings(&request),
    })
}

fn run_summary(record: &Value) -> RunSummary {
    RunSummary {
        run_id: field(record, "run_id"),
        profile: field(record, "profile"),
        workspace: field(record, "workspace"),
        network: field(record, "network"),
        status: field(record, "status"),
        timestamp: field(record, "timestamp"),
        state_volume: field(record, "state_volume"),
        session: field(record, "session"),
    }
}

fn field(record: &Value, name: &str) -> String {
    record
        .get(name)
        .and_then(Value::as_str)
        .unwrap_or("-")
        .to_string()
}

fn defaulted(value: &str, fallback: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        trimmed.to_string()
    }
}

fn non_empty(value: Option<String>) -> Option<String> {
    value.and_then(|text| {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn plan_warnings(request: &RunPlanRequest) -> Vec<PlanWarning> {
    let mut warnings = Vec::new();
    if request.network_mode == "internet" {
        warnings.push(warning(
            "full-internet",
            "Full internet lets the agent reach unrestricted network destinations from inside the container.",
        ));
    }
    if request.allow_sensitive_workspace {
        warnings.push(warning(
            "sensitive-workspace",
            "The selected folder may contain private files. The agent can read files inside that folder.",
        ));
    }
    if request.allow_root_user || matches!(request.user.as_str(), "root" | "0") {
        warnings.push(warning(
            "root-user",
            "The agent will run as root inside the container, weakening normal container guardrails.",
        ));
    }
    if !request.env_names.is_empty() {
        warnings.push(warning(
            "environment",
            "Environment variable names are passed into the run. Values are never shown in the UI.",
        ));
    }
    if request
        .image
        .as_deref()
        .is_some_and(|image| !image.trim().is_empty())
    {
        warnings.push(warning(
            "custom-image",
            "Custom images are outside the bundled RunHaven image set.",
        ));
    }
    if !request.provider_hosts.is_empty() {
        warnings.push(warning(
            "provider-host",
            "Additional provider hosts allow that host and its subdomains in provider-only mode.",
        ));
    }
    warnings
}

fn warning(code: &str, message: &str) -> PlanWarning {
    PlanWarning {
        code: code.to_string(),
        message: message.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(workspace: PathBuf) -> RunPlanRequest {
        RunPlanRequest {
            agent: "codex".to_string(),
            workspace_path: workspace.display().to_string(),
            network_mode: "provider".to_string(),
            workspace_scope: "current".to_string(),
            session_name: None,
            read_only_workspace: false,
            cpus: "4".to_string(),
            memory: "4g".to_string(),
            provider_hosts: Vec::new(),
            env_names: Vec::new(),
            image: None,
            allow_sensitive_workspace: false,
            allow_root_user: false,
            user: "agent".to_string(),
        }
    }

    #[test]
    fn lists_agent_profiles_without_secrets() {
        let agents = agent_summaries();
        assert!(agents.iter().any(|agent| agent.name == "codex"));
        assert!(agents.iter().all(|agent| !agent.image.is_empty()));
    }

    #[test]
    fn build_plan_response_uses_existing_runhaven_planner() {
        let workspace = tempfile::tempdir().expect("workspace");
        let response = build_plan_response(request(workspace.path().to_path_buf())).expect("plan");
        assert_eq!(response.profile, "codex");
        assert_eq!(response.network_mode, "provider");
        assert!(response.egress_summary.contains("provider allowlist"));
        assert_eq!(response.warnings.len(), 0);
    }

    #[test]
    fn build_plan_response_warns_for_supported_advanced_choices() {
        let workspace = tempfile::tempdir().expect("workspace");
        let mut request = request(workspace.path().to_path_buf());
        request.network_mode = "internet".to_string();
        request.allow_sensitive_workspace = true;
        request.env_names = vec!["OPENAI_API_KEY".to_string()];
        request.image = Some("example/custom:1.0.0".to_string());
        let response = build_plan_response(request).expect("plan");
        let codes = response
            .warnings
            .into_iter()
            .map(|warning| warning.code)
            .collect::<Vec<_>>();
        assert_eq!(
            codes,
            vec![
                "full-internet",
                "sensitive-workspace",
                "environment",
                "custom-image"
            ]
        );
    }

    #[test]
    fn build_plan_response_rejects_invalid_workspace() {
        let mut request = request(PathBuf::from("/definitely/not/a/runhaven/workspace"));
        request.network_mode = "internal".to_string();
        assert!(build_plan_response(request).is_err());
    }
}
