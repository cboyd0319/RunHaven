use std::path::PathBuf;

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

#[test]
fn launch_run_requires_explicit_confirmation() {
    let workspace = tempfile::tempdir().expect("workspace");
    let request = LaunchRunRequest {
        plan: request(workspace.path().to_path_buf()),
        confirm_launch: false,
        confirmed_warnings: Vec::new(),
    };

    let error = validate_launch_confirmation(&request).expect_err("confirmation required");

    assert!(error.contains("Confirm the launch"));
}

#[test]
fn launch_run_requires_each_warning_confirmation() {
    let workspace = tempfile::tempdir().expect("workspace");
    let mut plan = request(workspace.path().to_path_buf());
    plan.network_mode = "internet".to_string();
    let request = LaunchRunRequest {
        plan,
        confirm_launch: true,
        confirmed_warnings: Vec::new(),
    };

    let error = validate_launch_confirmation(&request).expect_err("warning required");

    assert!(error.contains("full-internet"));
}

#[test]
fn launch_run_response_uses_reserved_run_id_without_starting() {
    let workspace = tempfile::tempdir().expect("workspace");
    let response = build_launch_response(
        request(workspace.path().to_path_buf()),
        "runhaven-test-run".to_string(),
    )
    .expect("launch response");

    assert_eq!(response.run_id, "runhaven-test-run");
    assert_eq!(response.status, "started");
    assert_eq!(response.profile, "codex");
}

#[test]
fn launch_run_blocks_missing_bundled_image() {
    let workspace = tempfile::tempdir().expect("workspace");
    let request = request(workspace.path().to_path_buf());

    let error = image_readiness_error(
        &request,
        false,
        "missing",
        &request.agent,
        &request.agent,
        Some("runhaven image rebuild codex"),
    )
    .expect("image readiness error");

    assert!(error.contains("Image for codex is missing"));
    assert!(error.contains("runhaven image rebuild codex"));
}

#[test]
fn launch_run_allows_custom_image_without_bundled_image_check() {
    let workspace = tempfile::tempdir().expect("workspace");
    let mut request = request(workspace.path().to_path_buf());
    request.image = Some("example/custom:1.0.0".to_string());

    assert!(image_readiness_error(&request, false, "missing", "codex", "codex", None).is_none());
}
