//! GUI launch-plan auth-source selection (design C, F1). Rust decides the Claude
//! auth source from the REAL, non-secret Keychain presence, never from a
//! WebView-supplied value. Presence is injected here; no real Keychain is
//! touched, and the token is never read during plan preparation.

use super::*;
use runhaven_core::runtime::plans::ClaudeAuthSource;

#[test]
fn gui_launch_plan_selects_keychain_oauth_from_signed_in_presence() {
    let (_cache, _cache_home) = isolated_cache();
    let workspace = tempfile::tempdir().expect("workspace");
    let mut plan_request = request(workspace.path().to_path_buf());
    plan_request.agent = "claude".to_string();

    // Signed-in presence, injected. The plan selects KeychainOAuth from presence,
    // NOT from any WebView field.
    let plan = build_agent_run_plan_with_presence(
        &plan_request,
        Some("gui-claude-oauth".to_string()),
        || Ok(true),
    )
    .expect("claude launch plan");

    assert_eq!(
        plan.claude_auth_source,
        Some(ClaudeAuthSource::KeychainOAuth)
    );
}

#[test]
fn gui_launch_plan_without_token_selects_no_broker() {
    let (_cache, _cache_home) = isolated_cache();
    let workspace = tempfile::tempdir().expect("workspace");
    let mut plan_request = request(workspace.path().to_path_buf());
    plan_request.agent = "claude".to_string();

    let plan = build_agent_run_plan_with_presence(
        &plan_request,
        Some("gui-claude-none".to_string()),
        || Ok(false),
    )
    .expect("claude launch plan");

    assert_eq!(plan.claude_auth_source, None);
}

#[test]
fn gui_launch_plan_explicit_env_broker_beats_a_present_token() {
    let (_cache, _cache_home) = isolated_cache();
    let workspace = tempfile::tempdir().expect("workspace");
    let mut plan_request = request(workspace.path().to_path_buf());
    plan_request.agent = "claude".to_string();
    plan_request.api_key_broker_env = Some("ANTHROPIC_API_KEY".to_string());

    let plan = build_agent_run_plan_with_presence(
        &plan_request,
        Some("gui-claude-env".to_string()),
        || Ok(true),
    )
    .expect("claude launch plan");

    assert_eq!(
        plan.claude_auth_source,
        Some(ClaudeAuthSource::ApiKeyBrokerEnv(
            "ANTHROPIC_API_KEY".to_string()
        ))
    );
}

#[test]
fn launch_confirm_names_the_saved_token_for_keychain_oauth() {
    let (_cache, _cache_home) = isolated_cache();
    let workspace = tempfile::tempdir().expect("workspace");
    let mut plan_request = request(workspace.path().to_path_buf());
    plan_request.agent = "claude".to_string();

    // Signed-in presence selects KeychainOAuth; the confirmation must say the run
    // will use the saved Claude token and prompt, not the misleading "none".
    let keychain_plan = build_agent_run_plan_with_presence(
        &plan_request,
        Some("gui-claude-confirm".to_string()),
        || Ok(true),
    )
    .expect("claude keychain plan");
    assert_eq!(
        keychain_plan.claude_auth_source,
        Some(ClaudeAuthSource::KeychainOAuth)
    );

    let message = trusted_launch_message_for_test(&keychain_plan, &[], false);
    assert!(
        message.contains("Claude auth: your saved Claude token"),
        "confirm must name the saved-token use: {message}"
    );
    assert!(
        message.contains("Touch ID or your Mac password"),
        "confirm must warn about the approval prompt: {message}"
    );
    assert!(
        !message.contains("API key broker: none"),
        "the KeychainOAuth run must not read as no broker: {message}"
    );
    // No token or token prefix ever reaches the confirmation surface.
    assert!(
        !message.contains("sk-ant-"),
        "no token in confirm: {message}"
    );

    // The env-broker case is unchanged: it still names the host variable.
    plan_request.api_key_broker_env = Some("ANTHROPIC_API_KEY".to_string());
    let env_plan = build_agent_run_plan_with_presence(
        &plan_request,
        Some("gui-claude-confirm-env".to_string()),
        || Ok(true),
    )
    .expect("claude env plan");
    let env_message = trusted_launch_message_for_test(&env_plan, &[], false);
    assert!(
        env_message.contains("API key broker: ANTHROPIC_API_KEY"),
        "env broker line unchanged: {env_message}"
    );
    assert!(!env_message.contains("Claude auth: your saved Claude token"));

    // The no-broker case is unchanged: it still reads "none".
    let mut none_request = request(workspace.path().to_path_buf());
    none_request.agent = "claude".to_string();
    let none_plan = build_agent_run_plan_with_presence(
        &none_request,
        Some("gui-claude-confirm-none".to_string()),
        || Ok(false),
    )
    .expect("claude no-broker plan");
    let none_message = trusted_launch_message_for_test(&none_plan, &[], false);
    assert!(
        none_message.contains("API key broker: none"),
        "no-broker line unchanged: {none_message}"
    );
}

#[test]
fn gui_launch_plan_non_claude_agent_ignores_a_present_token() {
    let (_cache, _cache_home) = isolated_cache();
    let workspace = tempfile::tempdir().expect("workspace");
    // Default request agent is codex; a present Keychain token must not select
    // the claude-only OAuth path for it.
    let plan_request = request(workspace.path().to_path_buf());

    let plan = build_agent_run_plan_with_presence(
        &plan_request,
        Some("gui-codex-none".to_string()),
        || Ok(true),
    )
    .expect("codex launch plan");

    assert_eq!(plan.claude_auth_source, None);
}
