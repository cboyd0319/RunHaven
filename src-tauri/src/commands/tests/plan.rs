use super::*;

#[test]
fn lists_agent_profiles_without_secrets() {
    let agents = agent_summaries();
    let names = agents
        .iter()
        .map(|agent| agent.name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(names, ["antigravity", "claude", "codex", "copilot"]);
    assert!(agents.iter().all(|agent| !agent.image.is_empty()));
    let codex = agents
        .iter()
        .find(|agent| agent.name == "codex")
        .expect("codex profile");
    assert_eq!(codex.sign_in, "runhaven login");
    assert_eq!(codex.broker, "yes");
}

#[test]
fn build_plan_response_uses_existing_runhaven_planner() {
    let (_cache, _cache_home) = isolated_cache();
    let workspace = tempfile::tempdir().expect("workspace");
    let response = build_plan_response(request(workspace.path().to_path_buf())).expect("plan");
    assert_eq!(response.profile, "codex");
    assert_eq!(response.auth_scope, "agent");
    assert_eq!(response.network_mode, "provider");
    assert!(response.egress_summary.contains("provider allowlist"));
    assert_eq!(response.access.label, "Shared agent sign-in");
    assert!(
        response
            .access
            .detail
            .contains("across projects on this Mac")
    );
    assert!(!response.access.blocking);
    assert_eq!(response.warnings.len(), 1);
    assert_eq!(response.warnings[0].code, "direct-workspace");
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
            "custom-image",
            "direct-workspace"
        ]
    );
}

#[test]
fn build_plan_response_accepts_allowed_api_key_broker_env_name() {
    let (_cache, _cache_home) = isolated_cache();
    let workspace = tempfile::tempdir().expect("workspace");
    let mut request = request(workspace.path().to_path_buf());
    request.api_key_broker_env = Some("OPENAI_API_KEY".to_string());

    let response = build_plan_response(request).expect("broker plan");

    assert!(response.egress_summary.contains("API key broker enabled"));
    assert_eq!(
        response.access.api_key_broker_env,
        Some("OPENAI_API_KEY".to_string())
    );
    assert!(response.access.api_key_broker_env_present.is_some());
    assert_eq!(response.warnings.len(), 1);
    assert_eq!(response.warnings[0].code, "direct-workspace");
}

#[test]
fn build_plan_response_rejects_unrelated_api_key_broker_env() {
    let (_cache, _cache_home) = isolated_cache();
    let workspace = tempfile::tempdir().expect("workspace");
    let mut request = request(workspace.path().to_path_buf());
    request.api_key_broker_env = Some("PATH".to_string());

    let error = build_plan_response(request).expect_err("broker env allowlist");

    assert!(error.contains("OPENAI_API_KEY"));
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
    assert_eq!(response.auth_scope, "agent");
    assert!(!response.container_name.is_empty());
}

#[test]
fn launch_run_requires_trusted_approval_before_spawning() {
    let (_cache, _cache_home) = isolated_cache();
    let workspace = tempfile::tempdir().expect("workspace");
    let run_id = "runhaven-trusted-deny".to_string();
    let mut plan_request =
        shell_request(workspace.path().to_path_buf(), "trusted-deny".to_string());
    plan_request.image = Some("runhaven/test-shell:local".to_string());
    let launch_plan =
        build_agent_run_plan(&plan_request, Some(run_id.clone())).expect("launch plan");
    let response = build_launch_response(plan_request, run_id.clone()).expect("launch response");

    let outcome = launch_prepared_with_approval_for_test(
        run_id,
        launch_plan,
        response,
        Vec::new(),
        SessionInputRegistry::default(),
        SessionOutputRegistry::default(),
        |launch_plan, warnings| {
            assert_eq!(launch_plan.profile_name, "shell");
            assert!(warnings.is_empty());
            Ok(false)
        },
    )
    .expect("trusted approval cancellation is not a launch failure");

    assert!(outcome.is_none());
    assert!(read_active_run_records().is_empty());
}

#[test]
fn trusted_launch_prompt_contains_reviewable_launch_facts() {
    let (_cache, _cache_home) = isolated_cache();
    let workspace = tempfile::tempdir().expect("workspace");
    let mut plan_request = request(workspace.path().to_path_buf());
    plan_request.api_key_broker_env = Some("OPENAI_API_KEY".to_string());
    plan_request.env_names = vec![
        "OPENAI_API_KEY".to_string(),
        "RUNHAVEN_TEST_TOKEN".to_string(),
    ];
    let plan = build_agent_run_plan(&plan_request, Some("prompt-review-run".to_string()))
        .expect("launch plan");

    let message = trusted_launch_message_for_test(&plan, &[], false, &plan_request.env_names);

    assert!(message.contains("Agent: codex"));
    assert!(message.contains(&format!("Workspace: {}", plan.workspace.display())));
    // COPY-1: the consent dialog shows the plain shared labels, never the raw
    // enum codes a non-technical user cannot read.
    assert!(message.contains("Sign-in: Shared agent sign-in"));
    assert!(message.contains("Image: runhaven/codex"));
    assert!(message.contains("Network: Provider only"));
    assert!(!message.contains("Auth: agent"));
    assert!(!message.contains("Network: provider"));
    assert!(message.contains("Provider hosts: "));
    assert!(message.contains("API key broker: OPENAI_API_KEY"));
    assert!(
        message
            .lines()
            .any(|line| { line == "Extra host variables: OPENAI_API_KEY, RUNHAVEN_TEST_TOKEN" })
    );
    assert!(!message.contains("OPENAI_API_KEY="));
    assert!(!message.contains("RUNHAVEN_TEST_TOKEN="));
    assert!(message.contains("Worktree: no"));

    let worktree_message =
        trusted_launch_message_for_test(&plan, &[], true, &plan_request.env_names);
    assert!(worktree_message.contains("Worktree: yes"));
    assert!(worktree_message.contains("isolated git branch"));
}

#[test]
fn launch_run_response_round_trips_project_auth_scope_without_starting() {
    let (_cache, _cache_home) = isolated_cache();
    let workspace = tempfile::tempdir().expect("workspace");
    let mut plan_request = request(workspace.path().to_path_buf());
    plan_request.auth_scope = "project".to_string();

    let response = build_launch_response(plan_request, "runhaven-project-auth-test".to_string())
        .expect("launch response");

    assert_eq!(response.auth_scope, "project");
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
    let mut alpha_request = shell_request(workspace.path().to_path_buf(), "alpha".to_string());
    let mut beta_request = shell_request(workspace.path().to_path_buf(), "beta".to_string());
    alpha_request.auth_scope = "project".to_string();
    beta_request.auth_scope = "project".to_string();
    let alpha = build_agent_run_plan(&alpha_request, Some("desktop-alpha-plan".to_string()))
        .expect("alpha launch plan");
    let beta = build_agent_run_plan(&beta_request, Some("desktop-beta-plan".to_string()))
        .expect("beta launch plan");

    assert_eq!(alpha.auth_scope, AuthScope::Project);
    assert_eq!(beta.auth_scope, AuthScope::Project);
    assert_ne!(alpha.state_volume, beta.state_volume);
    assert_ne!(alpha.container_name, beta.container_name);
}

#[test]
fn launch_plan_for_desktop_defaults_to_shared_agent_login_state() {
    let (_cache, _cache_home) = isolated_cache();
    let workspace = tempfile::tempdir().expect("workspace");

    let plan = build_agent_run_plan(
        &request(workspace.path().to_path_buf()),
        Some("desktop-agent-auth-plan".to_string()),
    )
    .expect("desktop launch plan");

    assert_eq!(plan.auth_scope, AuthScope::Agent);
    assert_eq!(plan.state_volume, "runhaven-codex-shared-home");
}

#[test]
fn build_plan_response_rejects_invalid_auth_scope() {
    let (_cache, _cache_home) = isolated_cache();
    let workspace = tempfile::tempdir().expect("workspace");
    let mut plan_request = request(workspace.path().to_path_buf());
    plan_request.auth_scope = "host-home".to_string();

    let error = build_plan_response(plan_request).expect_err("invalid auth scope");

    assert!(error.contains("invalid auth scope"));
}

#[test]
fn launch_request_with_omitted_confirm_flag_fails_closed_with_friendly_error() {
    let workspace = tempfile::tempdir().expect("workspace");
    // No `confirmLaunch` key at all: the flag must default to false instead of
    // failing deserialization with a raw serde error.
    let launch_request: LaunchRunRequest = serde_json::from_value(json!({
        "plan": {
            "agent": "codex",
            "workspacePath": workspace.path().display().to_string(),
            "networkMode": "provider",
            "workspaceScope": "current",
            "sessionName": null,
            "readOnlyWorkspace": false,
            "cpus": "4",
            "memory": "4g",
            "providerHosts": [],
            "envNames": [],
            "apiKeyBrokerEnv": null,
            "image": null,
            "allowSensitiveWorkspace": false,
            "allowRootUser": false,
            "user": "agent"
        }
    }))
    .expect("deserialize launch request without confirm flag");

    assert!(!launch_request.confirm_launch);

    let error =
        validate_launch_confirmation(&launch_request).expect_err("unconfirmed launch rejected");

    assert_eq!(error, "Confirm the launch before starting a run.");
}

#[test]
fn run_plan_request_defaults_missing_auth_scope_to_agent() {
    let workspace = tempfile::tempdir().expect("workspace");
    let request: RunPlanRequest = serde_json::from_value(json!({
        "agent": "codex",
        "workspacePath": workspace.path().display().to_string(),
        "networkMode": "provider",
        "workspaceScope": "current",
        "sessionName": null,
        "readOnlyWorkspace": false,
        "cpus": "4",
        "memory": "4g",
        "providerHosts": [],
        "envNames": [],
        "apiKeyBrokerEnv": null,
        "image": null,
        "allowSensitiveWorkspace": false,
        "allowRootUser": false,
        "user": "agent"
    }))
    .expect("deserialize request");

    assert_eq!(request.auth_scope, "agent");
}
