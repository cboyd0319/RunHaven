//! Host-side API-key broker-secret resolution.
//!
//! Plans carry only the name of an explicitly configured host environment
//! variable. The value is read at launch, wrapped as a secret immediately, and
//! never written to a plan, record, marker, terminal stream, or log.

use anyhow::{Result, bail};
use secrecy::{ExposeSecret, SecretString};

use crate::provider::auth_broker::{ProviderBrokerProfile, broker_profile_for_agent};
use crate::runtime::plans::{AgentRunPlan, ClaudeAuthSource, NetworkMode};

pub fn select_api_key_broker_source_for_network(
    env_broker: Option<String>,
    network: NetworkMode,
) -> Result<Option<ClaudeAuthSource>> {
    let source = env_broker
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(ClaudeAuthSource::ApiKeyBrokerEnv);
    if network != NetworkMode::Provider && source.is_none() {
        return Ok(None);
    }
    Ok(source)
}

pub fn require_api_key_broker_secret(
    plan: &AgentRunPlan,
) -> Result<Option<(ProviderBrokerProfile, SecretString)>> {
    require_api_key_broker_secret_with_env(plan, |name| std::env::var(name).ok())
}

pub(crate) fn require_api_key_broker_secret_with_env(
    plan: &AgentRunPlan,
    env_value: impl Fn(&str) -> Option<String>,
) -> Result<Option<(ProviderBrokerProfile, SecretString)>> {
    let Some(ClaudeAuthSource::ApiKeyBrokerEnv(name)) = &plan.claude_auth_source else {
        return Ok(None);
    };
    Ok(Some(resolve_env_broker(plan, name, &env_value)?))
}

fn resolve_env_broker(
    plan: &AgentRunPlan,
    name: &str,
    env_value: &impl Fn(&str) -> Option<String>,
) -> Result<(ProviderBrokerProfile, SecretString)> {
    let Some(profile) = broker_profile_for_agent(&plan.profile_name) else {
        bail!(
            "the {} profile has no API-key broker; remove --api-key-broker-env",
            plan.profile_name
        );
    };
    if name != profile.host_env {
        bail!(
            "{} API key broker only accepts host environment variable {}",
            plan.profile_name,
            profile.host_env
        );
    }
    let value = SecretString::from(env_value(name).unwrap_or_default());
    if value.expose_secret().trim().is_empty() {
        bail!("{name} is not set on the host; export it before using --api-key-broker-env");
    }
    Ok((profile, value))
}

pub fn validate_runtime_auth_broker_environment(plan: &AgentRunPlan) -> Result<()> {
    validate_runtime_auth_broker_environment_with_env(plan, |name| std::env::var(name).ok())
}

pub(crate) fn validate_runtime_auth_broker_environment_with_env(
    plan: &AgentRunPlan,
    env_value: impl Fn(&str) -> Option<String>,
) -> Result<()> {
    let Some(ClaudeAuthSource::ApiKeyBrokerEnv(name)) = &plan.claude_auth_source else {
        return Ok(());
    };
    resolve_env_broker(plan, name, &env_value).map(|_| ())
}
