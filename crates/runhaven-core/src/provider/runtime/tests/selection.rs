//! Explicit API-key broker selection tests. Ordinary OAuth uses no host broker.

use super::super::broker_secret::{
    require_api_key_broker_secret_with_env, select_api_key_broker_source_for_network,
    validate_runtime_auth_broker_environment_with_env,
};
use crate::runtime::plans::{ClaudeAuthSource, NetworkMode};
use secrecy::ExposeSecret;

use super::broker_plan;

#[test]
fn no_explicit_broker_selects_native_container_auth() {
    for network in [
        NetworkMode::Internal,
        NetworkMode::Provider,
        NetworkMode::Internet,
    ] {
        assert_eq!(
            select_api_key_broker_source_for_network(None, network).expect("selection"),
            None,
        );
    }
}

#[test]
fn explicit_environment_broker_is_trimmed_and_preserved_for_validation() {
    let source = select_api_key_broker_source_for_network(
        Some("  OPENAI_API_KEY  ".to_string()),
        NetworkMode::Provider,
    )
    .expect("selection");
    assert_eq!(
        source,
        Some(ClaudeAuthSource::ApiKeyBrokerEnv(
            "OPENAI_API_KEY".to_string(),
        )),
    );
}

#[test]
fn explicit_environment_broker_is_read_only_at_validation_and_launch() {
    let mut plan = broker_plan("codex", "codex");
    plan.claude_auth_source = Some(ClaudeAuthSource::ApiKeyBrokerEnv(
        "OPENAI_API_KEY".to_string(),
    ));

    validate_runtime_auth_broker_environment_with_env(&plan, |name| {
        (name == "OPENAI_API_KEY").then(|| "test-key".to_string())
    })
    .expect("valid environment");

    let (_, secret) = require_api_key_broker_secret_with_env(&plan, |name| {
        (name == "OPENAI_API_KEY").then(|| "test-key".to_string())
    })
    .expect("resolve")
    .expect("broker");
    assert_eq!(secret.expose_secret(), "test-key");
}
