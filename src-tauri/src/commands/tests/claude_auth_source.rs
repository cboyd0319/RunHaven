//! GUI launch-plan auth selection. Ordinary OAuth stays inside the agent
//! container and therefore selects no host broker. The optional advanced
//! environment-variable API-key broker remains explicit.

use super::*;
use runhaven_core::runtime::plans::ClaudeAuthSource;

#[test]
fn gui_claude_launch_defaults_to_native_container_auth() {
    let (_cache, _cache_home) = isolated_cache();
    let workspace = tempfile::tempdir().expect("workspace");
    let mut request = request(workspace.path().to_path_buf());
    request.agent = "claude".to_string();

    let plan = build_agent_run_plan(&request, Some("gui-claude-native".to_string()))
        .expect("claude launch plan");

    assert_eq!(plan.claude_auth_source, None);
    let message = trusted_launch_message_for_test(&plan, &[], false, &[]);
    assert!(message.contains("API key broker: none"));
    assert!(!message.contains("Keychain"));
    assert!(!message.contains("saved Claude token"));
}

#[test]
fn gui_explicit_api_key_broker_stays_available() {
    let (_cache, _cache_home) = isolated_cache();
    let workspace = tempfile::tempdir().expect("workspace");
    let mut request = request(workspace.path().to_path_buf());
    request.agent = "claude".to_string();
    request.api_key_broker_env = Some("ANTHROPIC_API_KEY".to_string());

    let plan = build_agent_run_plan(&request, Some("gui-claude-api-key".to_string()))
        .expect("explicit API-key plan");

    assert_eq!(
        plan.claude_auth_source,
        Some(ClaudeAuthSource::ApiKeyBrokerEnv(
            "ANTHROPIC_API_KEY".to_string(),
        )),
    );
    let message = trusted_launch_message_for_test(&plan, &[], false, &[]);
    assert!(message.contains("API key broker: ANTHROPIC_API_KEY"));
}

#[test]
fn gui_non_provider_explicit_api_key_broker_is_rejected() {
    let (_cache, _cache_home) = isolated_cache();
    let workspace = tempfile::tempdir().expect("workspace");

    for network in ["internal", "internet"] {
        let mut request = request(workspace.path().to_path_buf());
        request.agent = "claude".to_string();
        request.network_mode = network.to_string();
        request.api_key_broker_env = Some("ANTHROPIC_API_KEY".to_string());

        let error = build_agent_run_plan(&request, Some(format!("gui-claude-{network}-api-key")))
            .expect_err("host API-key broker requires provider mode");
        assert!(error.contains("--network provider"));
    }
}
