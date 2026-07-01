use super::*;

#[test]
fn lists_agent_profiles_without_secrets() {
    let agents = agent_summaries();
    assert!(agents.iter().any(|agent| agent.name == "codex"));
    assert!(agents.iter().all(|agent| !agent.image.is_empty()));
    let codex = agents
        .iter()
        .find(|agent| agent.name == "codex")
        .expect("codex profile");
    assert_eq!(codex.sign_in, "runhaven login");
    assert_eq!(codex.broker, "yes");
    let shell = agents
        .iter()
        .find(|agent| agent.name == "shell")
        .expect("shell profile");
    assert_eq!(shell.sign_in, "n/a");
    assert_eq!(shell.broker, "n/a");
}

#[test]
fn build_plan_response_uses_existing_runhaven_planner() {
    let (_cache, _cache_home) = isolated_cache();
    let workspace = tempfile::tempdir().expect("workspace");
    let response = build_plan_response(request(workspace.path().to_path_buf())).expect("plan");
    assert_eq!(response.profile, "codex");
    assert_eq!(response.auth_scope, "project");
    assert_eq!(response.network_mode, "provider");
    assert!(response.egress_summary.contains("provider allowlist"));
    assert_eq!(response.warnings.len(), 0);
}

#[test]
fn build_plan_response_warns_for_supported_advanced_choices() {
    let (_cache, _cache_home) = isolated_cache();
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
fn build_plan_response_accepts_api_key_broker_env_name() {
    let (_cache, _cache_home) = isolated_cache();
    let workspace = tempfile::tempdir().expect("workspace");
    let mut request = request(workspace.path().to_path_buf());
    request.api_key_broker_env = Some("OPENAI_API_KEY".to_string());

    let response = build_plan_response(request).expect("broker plan");

    assert!(response.egress_summary.contains("API key broker enabled"));
    assert_eq!(response.warnings.len(), 0);
}

#[test]
fn build_plan_response_warns_for_active_runs_and_material_memory() {
    let (_cache, _cache_home) = isolated_cache();
    write_active_run("active-warning-run");
    let workspace = tempfile::tempdir().expect("workspace");

    let response = build_plan_response(request(workspace.path().to_path_buf())).expect("plan");
    let codes = response
        .warnings
        .into_iter()
        .map(|warning| warning.code)
        .collect::<Vec<_>>();

    assert!(codes.contains(&"active-runs".to_string()));
    assert!(codes.contains(&"resource-memory".to_string()));
}

#[test]
fn build_plan_response_rejects_invalid_workspace() {
    let mut request = request(PathBuf::from("/definitely/not/a/runhaven/workspace"));
    request.network_mode = "internal".to_string();
    assert!(build_plan_response(request).is_err());
}

#[test]
fn build_plan_response_rejects_oversized_ipc_fields() {
    let (_cache, _cache_home) = isolated_cache();
    let workspace = tempfile::tempdir().expect("workspace");
    let mut plan_request = request(workspace.path().to_path_buf());
    plan_request.workspace_path = "x".repeat(4097);

    let error = build_plan_response(plan_request).expect_err("workspace path cap");

    assert!(error.contains("workspace path"));

    let mut plan_request = request(workspace.path().to_path_buf());
    plan_request.provider_hosts = vec!["api.openai.com".to_string(); 51];

    let error = build_plan_response(plan_request).expect_err("provider host count cap");

    assert!(error.contains("provider hosts"));

    let mut plan_request = request(workspace.path().to_path_buf());
    plan_request.env_names = vec!["OPENAI_API_KEY".to_string(); 51];

    let error = build_plan_response(plan_request).expect_err("env name count cap");

    assert!(error.contains("environment variable names"));

    let mut plan_request = request(workspace.path().to_path_buf());
    plan_request.api_key_broker_env = Some("A".repeat(129));

    let error = build_plan_response(plan_request).expect_err("broker env cap");

    assert!(error.contains("API key broker environment variable"));
}

#[test]
fn launch_run_rejects_oversized_warning_confirmations() {
    let workspace = tempfile::tempdir().expect("workspace");
    let request = LaunchRunRequest {
        plan: request(workspace.path().to_path_buf()),
        confirm_launch: true,
        confirmed_warnings: vec!["full-internet".to_string(); 17],
    };

    let error = validate_launch_confirmation(&request).expect_err("warning count cap");

    assert!(error.contains("confirmed warning codes"));
}

#[test]
fn launch_run_requires_explicit_confirmation() {
    let (_cache, _cache_home) = isolated_cache();
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
    let (_cache, _cache_home) = isolated_cache();
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
fn launch_run_requires_active_run_warning_confirmation() {
    let (_cache, _cache_home) = isolated_cache();
    write_active_run("active-launch-run");
    let workspace = tempfile::tempdir().expect("workspace");
    let request = LaunchRunRequest {
        plan: request(workspace.path().to_path_buf()),
        confirm_launch: true,
        confirmed_warnings: Vec::new(),
    };

    let error = validate_launch_confirmation(&request).expect_err("active warning required");

    assert!(error.contains("active-runs"));
}

#[test]
fn launch_run_response_uses_reserved_run_id_without_starting() {
    let (_cache, _cache_home) = isolated_cache();
    let workspace = tempfile::tempdir().expect("workspace");
    let response = build_launch_response(
        request(workspace.path().to_path_buf()),
        "runhaven-test-run".to_string(),
    )
    .expect("launch response");

    assert_eq!(response.run_id, "runhaven-test-run");
    assert_eq!(response.status, "started");
    assert_eq!(response.profile, "codex");
    assert_eq!(response.auth_scope, "project");
    assert_eq!(response.snapshot.run_id, "runhaven-test-run");
    assert_eq!(response.snapshot.status, "started");
    assert_eq!(response.snapshot.auth_scope, "project");
    assert!(!response.snapshot.container_name.is_empty());
}

#[test]
fn launch_plan_for_desktop_keeps_piped_stdin_open_without_tty() {
    let (_cache, _cache_home) = isolated_cache();
    let workspace = tempfile::tempdir().expect("workspace");

    let plan = build_agent_run_plan(
        &request(workspace.path().to_path_buf()),
        Some("desktop-stdin-plan".to_string()),
    )
    .expect("desktop launch plan");

    assert!(plan.command.iter().any(|arg| arg == "--interactive"));
    assert!(!plan.command.iter().any(|arg| arg == "--tty"));
}

#[test]
fn launch_plan_for_desktop_uses_session_scoped_state() {
    let (_cache, _cache_home) = isolated_cache();
    let workspace = tempfile::tempdir().expect("workspace");
    let alpha = build_agent_run_plan(
        &shell_request(workspace.path().to_path_buf(), "alpha".to_string()),
        Some("desktop-alpha-plan".to_string()),
    )
    .expect("alpha launch plan");
    let beta = build_agent_run_plan(
        &shell_request(workspace.path().to_path_buf(), "beta".to_string()),
        Some("desktop-beta-plan".to_string()),
    )
    .expect("beta launch plan");

    assert_eq!(alpha.auth_scope, AuthScope::Project);
    assert_eq!(beta.auth_scope, AuthScope::Project);
    assert_ne!(alpha.state_volume, beta.state_volume);
    assert_ne!(alpha.container_name, beta.container_name);
}
