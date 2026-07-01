use super::live_support::{
    LiveTauriRunCleanup, assert_live_response_excludes, stop_live_session_if_active,
    wait_for_live_launch_status, wait_for_live_managed_session_status,
    wait_for_live_monitor_session, wait_for_live_session_inactive,
    wait_for_live_session_output_available,
};
use super::*;

#[test]
#[ignore = "requires RUNHAVEN_GUI_LIVE_REAL_AGENT=1, Apple container, and a built brokered real-agent image"]
fn managed_session_live_smoke_launches_real_agent_profile() {
    let Some(config) = live_agent_smoke_config() else {
        eprintln!(
            "skipping real-agent GUI smoke: set RUNHAVEN_GUI_LIVE_REAL_AGENT=1, \
RUNHAVEN_GUI_LIVE_AGENT=<codex|claude|gemini>, RUNHAVEN_GUI_LIVE_AGENT_API_KEY_ENV=<fixed broker env>, \
and export that broker env on the host to opt in"
        );
        return;
    };
    let agent = config.agent.as_str();
    let expected_env =
        expected_broker_env(agent).expect("real-agent smoke requires a brokered agent");
    assert_eq!(
        config.broker_env, expected_env,
        "real-agent GUI smoke must use the fixed broker env for the selected agent"
    );
    get_profile(agent).expect("known real-agent profile");

    let workspace = tempfile::tempdir().expect("workspace");
    std::fs::write(
        workspace.path().join("README.md"),
        "runhaven real-agent live smoke\n",
    )
    .expect("workspace fixture");
    let session_name = format!("gui-agent-{agent}-{}", std::process::id());
    let mut plan_request = real_agent_request(agent, workspace.path().to_path_buf(), session_name);
    plan_request.api_key_broker_env = Some(config.broker_env.clone());

    let plan = build_plan_response(plan_request.clone()).expect("real-agent live plan");
    assert_eq!(plan.profile, agent);
    assert_eq!(plan.auth_scope, "project");
    assert_eq!(plan.network_mode, "provider");
    assert!(!plan.provider_allowed_hosts.is_empty());
    assert_eq!(
        plan.access.api_key_broker_env.as_deref(),
        Some(config.broker_env.as_str())
    );
    assert_eq!(plan.access.api_key_broker_env_present, Some(true));
    let forbidden_fragments = vec![
        ("workspace path", workspace.path().display().to_string()),
        ("planned workspace path", plan.workspace.clone()),
        ("state volume", plan.state_volume.clone()),
        ("broker secret", config.broker_secret.clone()),
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
    let launch_status = LaunchStatusRegistry::default();
    let input_registry = SessionInputRegistry::default();
    let output_registry = SessionOutputRegistry::default();

    let launched = crate::commands::launch_run_with_registries_status_and_approval(
        LaunchRunRequest {
            plan: plan_request,
            confirm_launch: true,
            confirmed_warnings,
        },
        input_registry.clone(),
        output_registry.clone(),
        launch_status.clone(),
        |_, _| Ok(true),
    )
    .expect("launch real agent through Tauri runner");
    cleanup.run_id = Some(launched.run_id.clone());
    assert_eq!(launched.profile, agent);
    assert_eq!(launched.auth_scope, "project");
    assert_eq!(launched.network_mode, "provider");

    let starting =
        wait_for_live_launch_status(&launched.run_id, &launch_status, &["starting", "running"]);
    assert_eq!(starting.run_id, launched.run_id);
    assert!(matches!(starting.state.as_str(), "starting" | "running"));
    assert_live_response_excludes("launch status", &starting, &forbidden_fragments);

    let session = wait_for_live_monitor_session(&launched.run_id);
    assert_eq!(session.agent, agent);
    assert_eq!(session.run_id, launched.run_id);
    assert_eq!(session.auth_scope, "project");
    assert_eq!(session.network, "provider");
    assert_eq!(session.next_action.kind, "monitor");
    assert_live_response_excludes("managed session", &session, &forbidden_fragments);

    let running = wait_for_live_launch_status(&launched.run_id, &launch_status, &["running"]);
    assert_eq!(running.state, "running");
    assert_eq!(running.label, "Running");
    assert_live_response_excludes("running launch status", &running, &forbidden_fragments);

    let runtime = wait_for_live_managed_session_status(&session.id, &input_registry);
    assert_eq!(runtime.session.id, session.id);
    assert_eq!(runtime.session.run_id, launched.run_id);
    assert!(runtime.session.input.attached);
    assert_eq!(runtime.status_unavailable, None);
    assert!(runtime.recovery.is_none());
    let container = runtime
        .container
        .as_ref()
        .expect("running container status");
    assert_eq!(container.state, "running");
    assert_live_response_excludes("managed session status", &runtime, &forbidden_fragments);

    let log = wait_for_live_session_output_available(&session.id, &output_registry);
    assert_eq!(log.session.id, session.id);
    assert_eq!(log.session.run_id, launched.run_id);
    assert_live_response_excludes("managed session output", &log, &forbidden_fragments);

    stop_live_session_if_active(&session);
    let finished = wait_for_live_session_inactive(&launched.run_id);
    assert!(matches!(
        finished.lifecycle.as_str(),
        "stopped" | "failed" | "finished"
    ));
    let terminal =
        wait_for_live_launch_status(&launched.run_id, &launch_status, &["finished", "failed"]);
    assert_live_response_excludes("terminal launch status", &terminal, &forbidden_fragments);
}

struct LiveAgentSmokeConfig {
    agent: String,
    broker_env: String,
    broker_secret: String,
}

fn live_agent_smoke_config() -> Option<LiveAgentSmokeConfig> {
    if !env_truthy("RUNHAVEN_GUI_LIVE_REAL_AGENT") {
        return None;
    }
    let agent = std::env::var("RUNHAVEN_GUI_LIVE_AGENT").unwrap_or_else(|_| {
        panic!("RUNHAVEN_GUI_LIVE_AGENT is required when RUNHAVEN_GUI_LIVE_REAL_AGENT=1")
    });
    let broker_env = std::env::var("RUNHAVEN_GUI_LIVE_AGENT_API_KEY_ENV").unwrap_or_else(|_| {
        panic!(
            "RUNHAVEN_GUI_LIVE_AGENT_API_KEY_ENV is required when RUNHAVEN_GUI_LIVE_REAL_AGENT=1"
        )
    });
    let broker_secret = std::env::var(&broker_env)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| panic!("{broker_env} must be exported for real-agent GUI smoke"));
    Some(LiveAgentSmokeConfig {
        agent,
        broker_env,
        broker_secret,
    })
}

fn env_truthy(name: &str) -> bool {
    std::env::var(name).ok().is_some_and(|value| {
        matches!(
            value.trim(),
            "1" | "true" | "TRUE" | "yes" | "YES" | "on" | "ON"
        )
    })
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
