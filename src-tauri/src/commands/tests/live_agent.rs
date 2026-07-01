use super::live_support::{
    LiveTauriRunCleanup, stop_live_session_if_active, wait_for_live_monitor_session,
    wait_for_live_session_inactive, wait_for_live_session_output_available,
};
use super::*;

#[test]
#[ignore = "requires Apple container and a built real-agent image"]
fn managed_session_live_smoke_launches_real_agent_profile() {
    let agent = std::env::var("RUNHAVEN_GUI_LIVE_AGENT")
        .expect("RUNHAVEN_GUI_LIVE_AGENT is required for real-agent GUI smoke");
    let expected_env =
        expected_broker_env(&agent).expect("real-agent smoke requires a brokered agent");
    let broker_env = std::env::var("RUNHAVEN_GUI_LIVE_AGENT_API_KEY_ENV")
        .expect("RUNHAVEN_GUI_LIVE_AGENT_API_KEY_ENV is required for real-agent GUI smoke");
    assert_eq!(
        broker_env, expected_env,
        "real-agent GUI smoke must use the fixed broker env for the selected agent"
    );
    assert!(
        std::env::var_os(&broker_env).is_some_and(|value| !value.is_empty()),
        "{broker_env} must be set for real-agent GUI smoke"
    );
    get_profile(&agent).expect("known real-agent profile");

    let workspace = tempfile::tempdir().expect("workspace");
    std::fs::write(
        workspace.path().join("README.md"),
        "runhaven real-agent live smoke\n",
    )
    .expect("workspace fixture");
    let session_name = format!("gui-agent-{agent}-{}", std::process::id());
    let mut plan_request = real_agent_request(&agent, workspace.path().to_path_buf(), session_name);
    plan_request.api_key_broker_env = Some(broker_env);

    let plan = build_plan_response(plan_request.clone()).expect("real-agent live plan");
    assert_eq!(plan.profile, agent);
    assert_eq!(plan.auth_scope, "project");
    assert_eq!(plan.network_mode, "provider");
    assert!(!plan.provider_allowed_hosts.is_empty());
    let forbidden_workspace_paths = [
        workspace.path().display().to_string(),
        plan.workspace.clone(),
    ];

    let confirmed_warnings = plan
        .warnings
        .iter()
        .map(|warning| warning.code.clone())
        .collect::<Vec<_>>();
    let mut cleanup = LiveTauriRunCleanup {
        run_id: None,
        state_volume: Some(plan.state_volume.clone()),
        network_name: plan.network_name.clone(),
    };
    let registry = SessionInputRegistry::default();
    let output_registry = SessionOutputRegistry::default();

    let launched = launch_run_with_registries(
        LaunchRunRequest {
            plan: plan_request,
            confirm_launch: true,
            confirmed_warnings,
        },
        registry,
        output_registry.clone(),
    )
    .expect("launch real agent through Tauri runner");
    cleanup.run_id = Some(launched.run_id.clone());

    let session = wait_for_live_monitor_session(&launched.run_id);
    assert_eq!(session.agent, agent);
    assert_eq!(session.run_id, launched.run_id);
    assert_eq!(session.auth_scope, "project");
    assert_eq!(session.network, "provider");

    let encoded_session = serde_json::to_string(&session).expect("serialize managed session");
    for path in &forbidden_workspace_paths {
        assert!(!encoded_session.contains(path));
    }
    assert!(!encoded_session.contains("state_volume"));
    assert!(!encoded_session.contains("write_session_input"));
    assert!(!encoded_session.contains("attach_session"));

    let log = wait_for_live_session_output_available(&session.id, &output_registry);
    assert_eq!(log.session.run_id, launched.run_id);
    let encoded_log = serde_json::to_string(&log).expect("serialize managed session output");
    for path in &forbidden_workspace_paths {
        assert!(!encoded_log.contains(path));
    }
    assert!(!encoded_log.contains("state_volume"));
    assert!(!encoded_log.contains("write_session_input"));
    assert!(!encoded_log.contains("attach_session"));

    stop_live_session_if_active(&session);
    let finished = wait_for_live_session_inactive(&launched.run_id);
    assert!(matches!(
        finished.lifecycle.as_str(),
        "stopped" | "failed" | "finished"
    ));
}

fn expected_broker_env(agent: &str) -> Option<&'static str> {
    match agent {
        "codex" => Some("OPENAI_API_KEY"),
        "claude" => Some("ANTHROPIC_API_KEY"),
        "gemini" => Some("GEMINI_API_KEY"),
        _ => None,
    }
}

fn real_agent_request(agent: &str, workspace: PathBuf, session: String) -> RunPlanRequest {
    RunPlanRequest {
        agent: agent.to_string(),
        workspace_path: workspace.display().to_string(),
        network_mode: "provider".to_string(),
        workspace_scope: "current".to_string(),
        auth_scope: "project".to_string(),
        session_name: Some(session),
        read_only_workspace: true,
        cpus: "2".to_string(),
        memory: "2g".to_string(),
        provider_hosts: Vec::new(),
        env_names: Vec::new(),
        api_key_broker_env: None,
        image: None,
        allow_sensitive_workspace: false,
        allow_root_user: false,
        user: "agent".to_string(),
    }
}
