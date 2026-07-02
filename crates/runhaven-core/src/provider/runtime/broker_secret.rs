//! Host-side broker-secret resolution: turn a run's non-secret
//! [`ClaudeAuthSource`] selector into the broker profile plus the real secret
//! the broker injects. This is the trust seam where the Claude OAuth token
//! enters the process, so it is owned by one small module (design C Section 3).
//!
//! The `ClaudeAuthSource` variant is the ONLY selector: it cannot reach the
//! removed `api_key_broker_env` plan field, so an OAuth token can never be read
//! from a persisted field (design C F1). The token flows from the Keychain read
//! straight into the returned secret string (held in `BrokerState` for the run),
//! never onto a plan, marker, record, or log.

use anyhow::{Result, anyhow, bail};
use zeroize::Zeroizing;

use crate::provider::auth_broker::{
    CLAUDE_OAUTH_BROKER, ProviderBrokerProfile, broker_profile_for_agent,
};
use crate::runtime::plans::{AgentRunPlan, ClaudeAuthSource};

/// The one agent profile the Keychain OAuth broker serves. Kept in one place so
/// selection, presence validation, and the secret read all pin the same profile.
const CLAUDE_PROFILE: &str = "claude";

/// Choose the run's Claude auth source from the request inputs and the real,
/// NON-SECRET Keychain presence (design C, Section 3, F1). This is the single
/// home of the selection precedence, so the CLI and GUI do not re-implement it:
///
/// 1. An explicitly requested env-var broker NAME wins (unchanged pre-design-C
///    behavior).
/// 2. Else, for the `claude` profile only, a PRESENT Keychain OAuth token drives
///    the payload-free `KeychainOAuth` path.
/// 3. Else, no broker.
///
/// The caller supplies at most the env-var broker NAME; it can NEVER force or
/// forge `KeychainOAuth`, because that variant is selected only from a real
/// signed-in Keychain token that RunHaven checks host-side.
///
/// `keychain_present` MUST be the non-prompting, attributes-only presence check
/// ([`claude_oauth_token_present`]) in production; it never reads the token, so it
/// is safe during plan preparation. The token itself is read once, later, at
/// broker startup.
pub fn select_claude_auth_source_with_presence(
    env_broker: Option<String>,
    agent: &str,
    keychain_present: impl Fn() -> Result<bool>,
) -> Result<Option<ClaudeAuthSource>> {
    if let Some(name) = env_broker
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        return Ok(Some(ClaudeAuthSource::ApiKeyBrokerEnv(name)));
    }
    if agent == CLAUDE_PROFILE && keychain_present()? {
        return Ok(Some(ClaudeAuthSource::KeychainOAuth));
    }
    Ok(None)
}

/// Non-prompting presence of RunHaven's Claude OAuth Keychain token. On macOS
/// this is the attributes-only Keychain query (no `kSecReturnData`, no secret
/// read, no user-presence prompt); on other targets there is no Keychain broker,
/// so it reports absent. Safe to call during plan preparation and status
/// polling; it NEVER reads the token and NEVER prompts.
#[cfg(target_os = "macos")]
pub fn claude_oauth_token_present() -> Result<bool> {
    crate::provider::claude_oauth_keychain::token_present()
}

#[cfg(not(target_os = "macos"))]
pub fn claude_oauth_token_present() -> Result<bool> {
    Ok(false)
}

pub fn require_api_key_broker_secret(
    plan: &AgentRunPlan,
) -> Result<Option<(ProviderBrokerProfile, Zeroizing<String>)>> {
    require_api_key_broker_secret_with_sources(
        plan,
        |name| std::env::var(name).ok(),
        read_keychain_oauth_token,
    )
}

/// Default Keychain OAuth-token reader. On macOS this delegates to the custody
/// module, whose read triggers the one-per-run user-presence prompt (Touch ID or
/// device password). On other targets the Keychain broker does not exist, so it
/// fails closed. It sits behind a seam so tests inject a fake reader and never
/// touch the real Keychain.
#[cfg(target_os = "macos")]
fn read_keychain_oauth_token() -> Result<Option<Zeroizing<Vec<u8>>>> {
    crate::provider::claude_oauth_keychain::read_token()
}

#[cfg(not(target_os = "macos"))]
fn read_keychain_oauth_token() -> Result<Option<Zeroizing<Vec<u8>>>> {
    bail!("the Claude Keychain OAuth broker is only available on macOS");
}

/// Resolve the host-side broker secret for a run. `env_value` and
/// `keychain_token` are seams so tests exercise each path without a real env var
/// or the real Keychain.
pub(crate) fn require_api_key_broker_secret_with_sources(
    plan: &AgentRunPlan,
    env_value: impl Fn(&str) -> Option<String>,
    keychain_token: impl Fn() -> Result<Option<Zeroizing<Vec<u8>>>>,
) -> Result<Option<(ProviderBrokerProfile, Zeroizing<String>)>> {
    let Some(source) = &plan.claude_auth_source else {
        return Ok(None);
    };
    match source {
        ClaudeAuthSource::ApiKeyBrokerEnv(name) => {
            Ok(Some(resolve_env_broker(plan, name, &env_value)?))
        }
        ClaudeAuthSource::KeychainOAuth => {
            // Claude-only broker (design C). Read the token from the Keychain
            // exactly once per run launch (the user-presence prompt fires here)
            // and hand it straight to the broker's in-memory state as the
            // `CLAUDE_OAUTH_BROKER` secret. This is the ONLY prompting read; plan
            // preparation and pre-confirm validation use presence only (F3). Fail
            // closed on a missing item, a read/cancel error, or a non-text token;
            // every error is fixed and path-free and never echoes the token bytes.
            if plan.profile_name != CLAUDE_PROFILE {
                bail!("the Claude Keychain OAuth broker is only available for the claude profile");
            }
            let Some(secret) = keychain_token()? else {
                bail!("no Claude token is saved; sign in to Claude before starting the run");
            };
            // Stay in `Zeroizing` from the Keychain read to `BrokerState.api_key`:
            // the trimmed copy is wrapped the moment it is allocated, so every
            // heap copy of the token on this path (and on the error paths below)
            // is overwritten on drop. No plain `String` ever holds the token.
            let token = Zeroizing::new(
                std::str::from_utf8(&secret)
                    .map_err(|_| {
                        anyhow!("the saved Claude token is not valid text; sign in to Claude again")
                    })?
                    .trim()
                    .to_string(),
            );
            if token.is_empty() {
                bail!("the saved Claude token is empty; sign in to Claude again");
            }
            Ok(Some((CLAUDE_OAUTH_BROKER, token)))
        }
    }
}

/// Resolve and validate the env-var broker path (shared by the pre-confirm
/// validation and the broker-startup secret read). Non-prompting: it only reads
/// a host env var, so it is safe before the trusted launch confirmation.
fn resolve_env_broker(
    plan: &AgentRunPlan,
    name: &str,
    env_value: &impl Fn(&str) -> Option<String>,
) -> Result<(ProviderBrokerProfile, Zeroizing<String>)> {
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
    // Wrap the host env value in `Zeroizing` immediately, for symmetry with the
    // Keychain path: this key is also a secret and the broker's copy is already
    // `Zeroizing`, so it stays zeroize-on-drop end to end.
    let value = Zeroizing::new(env_value(name).unwrap_or_default());
    if value.trim().is_empty() {
        bail!("{name} is not set on the host; export it before using --api-key-broker-env");
    }
    Ok((profile, value))
}

/// Pre-confirm broker validation. Runs BEFORE the trusted launch confirmation
/// (GUI plan preparation, CLI worktree preflight) and again at the top of the
/// launch path, so it MUST NOT read the Keychain token or prompt (F3). The env
/// path resolves its host env var (non-prompting); the KeychainOAuth path checks
/// PRESENCE only (attributes-only, no secret read, no prompt) and fails closed
/// when no token is saved. The prompting token read is deferred to broker
/// startup in [`require_api_key_broker_secret`].
pub fn validate_runtime_auth_broker_environment(plan: &AgentRunPlan) -> Result<()> {
    validate_runtime_auth_broker_environment_with_sources(
        plan,
        |name| std::env::var(name).ok(),
        claude_oauth_token_present,
    )
}

pub(crate) fn validate_runtime_auth_broker_environment_with_sources(
    plan: &AgentRunPlan,
    env_value: impl Fn(&str) -> Option<String>,
    keychain_present: impl Fn() -> Result<bool>,
) -> Result<()> {
    let Some(source) = &plan.claude_auth_source else {
        return Ok(());
    };
    match source {
        ClaudeAuthSource::ApiKeyBrokerEnv(name) => {
            resolve_env_broker(plan, name, &env_value).map(|_| ())
        }
        ClaudeAuthSource::KeychainOAuth => {
            if plan.profile_name != CLAUDE_PROFILE {
                bail!("the Claude Keychain OAuth broker is only available for the claude profile");
            }
            // Presence only: the attributes-only query never decrypts the secret
            // and never prompts, so this is safe pre-confirm. The token is read
            // once later, at broker startup, after the launch is confirmed.
            if !keychain_present()? {
                bail!("no Claude token is saved; sign in to Claude before starting the run");
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    //! No-leak source tripwire for the broker-secret seam. This is the module
    //! where the Claude OAuth token first enters process memory (from the
    //! Keychain read). Its only legitimate destination is the returned secret
    //! string, which the caller hands to the in-memory `BrokerState`. It must
    //! NEVER write the token to a file, a log, an emitted event, or a run
    //! record/marker. Forbidden strings are assembled at runtime so this test's
    //! own source cannot satisfy the scan.

    fn forbidden_sinks() -> Vec<String> {
        let mut markers = vec![
            format!("{}{}", "fs::", "write"),
            format!("{}{}", "fs::", "File"),
            format!("{}{}", "File::", "create"),
            format!("{}{}", "write_", "all"),
            format!("{}{}", "trac", "ing"),
            format!("{}{}", "eprint", "ln!"),
            format!("{}{}", "print", "ln!"),
            format!("{}{}", ".em", "it("),
            format!("{}{}", "run_", "history"),
            format!("{}{}", "write_active_", "run"),
            format!("{}{}", "markers", "::"),
        ];
        for level in ["error", "warn", "info", "debug", "trace"] {
            markers.push(format!("{}{}{}", "log", "::", level));
        }
        markers
    }

    /// Production text only: strip everything from the first `#[cfg(test)]`.
    fn production(source: &str) -> &str {
        let cfg_test = format!("#[{}]", "cfg(test)");
        match source.split_once(&cfg_test) {
            Some((before, _)) => before,
            None => source,
        }
    }

    #[test]
    fn broker_secret_seam_never_persists_or_logs_the_token() {
        let source = production(include_str!("broker_secret.rs"));
        // Prove the scan reaches the path that reads the token.
        assert!(source.contains("keychain_token()"));
        let hit = forbidden_sinks()
            .into_iter()
            .find(|marker| source.contains(marker.as_str()));
        assert!(
            hit.is_none(),
            "the broker-secret seam must not sink the token; found `{}`",
            hit.unwrap_or_default()
        );
    }
}
