//! Provider broker profiles.
//!
//! Each profile describes how the host-side API-key broker talks to one
//! provider: which upstream host it pins, which request paths it allows, how it
//! injects the real credential, and how the guest agent is pointed at the broker
//! without ever holding the credential. Codex is the original prototype; Claude
//! and Gemini reuse the same proxy core through this profile.

/// How the broker injects the real credential into an upstream request. The
/// guest never sends a usable credential; the broker strips any placeholder and
/// applies this strategy host-side.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CredentialInjection {
    /// `Authorization: Bearer <key>` (OpenAI/Codex).
    BearerAuth,
    /// A single API-key header plus any fixed extra headers the provider
    /// requires (Anthropic: `x-api-key` + `anthropic-version`; Gemini:
    /// `x-goog-api-key`).
    ApiKeyHeader {
        name: &'static str,
        extra: &'static [(&'static str, &'static str)],
    },
}

impl CredentialInjection {
    /// Header names this strategy sets, so the broker can strip any guest-sent
    /// copy (placeholder or otherwise) before injecting the real value.
    pub fn injected_header_names(&self) -> Vec<&'static str> {
        match self {
            CredentialInjection::BearerAuth => vec!["authorization"],
            CredentialInjection::ApiKeyHeader { name, extra } => {
                let mut names = vec![*name];
                names.extend(extra.iter().map(|(k, _)| *k));
                names
            }
        }
    }
}

/// Which upstream request paths the broker forwards. Anything else is rejected,
/// so a compromised guest cannot turn the broker into an open proxy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PathRule {
    /// Exactly one path (Codex Responses API).
    Exact(&'static str),
    /// Any path under this prefix (Anthropic `/v1/`, Gemini `/v1beta/`).
    Prefix(&'static str),
}

impl PathRule {
    pub fn allows(&self, path: &str) -> bool {
        // Compare only the path component, never the query string.
        let path = path.split(['?', '#']).next().unwrap_or(path);
        match self {
            PathRule::Exact(expected) => path == *expected,
            PathRule::Prefix(prefix) => path.starts_with(prefix),
        }
    }
}

/// How the guest agent CLI is redirected at the broker.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GuestRedirect {
    /// Codex: inject `-c model_providers.<id>.*` config flags into the `codex`
    /// subcommand, pointing at the broker base URL with the placeholder env key.
    CodexCustomProvider {
        provider_id: &'static str,
        wire_api: &'static str,
    },
    /// Claude/Gemini: set a base-URL env var to the broker plus the placeholder
    /// key env var. No command-argument rewriting.
    EnvRedirect { base_url_env: &'static str },
}

/// A single provider's broker configuration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProviderBrokerProfile {
    /// Agent profile id this broker serves (`codex`, `claude`, `gemini`).
    pub agent: &'static str,
    /// Human label used in config and decisions.
    pub label: &'static str,
    /// Pinned upstream host the broker forwards to.
    pub upstream_host: &'static str,
    /// Allowed upstream request paths.
    pub path_rule: PathRule,
    /// Credential injection strategy.
    pub injection: CredentialInjection,
    /// Guest env var holding the placeholder key value.
    pub placeholder_env: &'static str,
    /// Placeholder value the guest receives in place of the real key.
    pub placeholder_value: &'static str,
    /// How the guest CLI is redirected at the broker.
    pub guest_redirect: GuestRedirect,
}

pub const BROKER_PLACEHOLDER_VALUE: &str = "runhaven-broker-placeholder";

/// Codex broker: the original prototype, preserved exactly.
pub const CODEX_BROKER: ProviderBrokerProfile = ProviderBrokerProfile {
    agent: "codex",
    label: "RunHaven OpenAI API-key broker",
    upstream_host: "api.openai.com",
    path_rule: PathRule::Exact("/v1/responses"),
    injection: CredentialInjection::BearerAuth,
    placeholder_env: "RUNHAVEN_CODEX_BROKER_TOKEN",
    placeholder_value: BROKER_PLACEHOLDER_VALUE,
    guest_redirect: GuestRedirect::CodexCustomProvider {
        provider_id: "runhaven_openai",
        wire_api: "responses",
    },
};

/// Claude broker: Anthropic Messages API via `ANTHROPIC_BASE_URL` redirect.
/// `x-api-key` + `anthropic-version` auth. The `/v1/` prefix matcher avoids
/// enumerating which `/v1/*` endpoints the CLI version hits.
pub const CLAUDE_BROKER: ProviderBrokerProfile = ProviderBrokerProfile {
    agent: "claude",
    label: "RunHaven Anthropic API-key broker",
    upstream_host: "api.anthropic.com",
    path_rule: PathRule::Prefix("/v1/"),
    injection: CredentialInjection::ApiKeyHeader {
        name: "x-api-key",
        extra: &[("anthropic-version", "2023-06-01")],
    },
    placeholder_env: "ANTHROPIC_API_KEY",
    placeholder_value: BROKER_PLACEHOLDER_VALUE,
    guest_redirect: GuestRedirect::EnvRedirect {
        base_url_env: "ANTHROPIC_BASE_URL",
    },
};

/// Gemini broker: Generative Language API via the genai base-URL env.
/// `x-goog-api-key` auth. The base-URL env is currently undocumented and
/// version-fragile; gate it behind a per-version smoke.
pub const GEMINI_BROKER: ProviderBrokerProfile = ProviderBrokerProfile {
    agent: "gemini",
    label: "RunHaven Gemini API-key broker",
    upstream_host: "generativelanguage.googleapis.com",
    path_rule: PathRule::Prefix("/v1beta/"),
    injection: CredentialInjection::ApiKeyHeader {
        name: "x-goog-api-key",
        extra: &[],
    },
    placeholder_env: "GEMINI_API_KEY",
    placeholder_value: BROKER_PLACEHOLDER_VALUE,
    guest_redirect: GuestRedirect::EnvRedirect {
        base_url_env: "GOOGLE_GEMINI_BASE_URL",
    },
};

/// The broker profile for an agent, if one exists. Providers without a clean
/// host-side broker (e.g. Copilot, whose token exchange and dynamic API host
/// cannot be brokered without TLS interception) return `None` and stay
/// design-only.
pub fn broker_profile_for_agent(agent: &str) -> Option<ProviderBrokerProfile> {
    match agent {
        "codex" => Some(CODEX_BROKER),
        "claude" => Some(CLAUDE_BROKER),
        "gemini" => Some(GEMINI_BROKER),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_rule_ignores_query_string() {
        assert!(PathRule::Exact("/v1/responses").allows("/v1/responses"));
        assert!(PathRule::Exact("/v1/responses").allows("/v1/responses?x=1"));
        assert!(!PathRule::Exact("/v1/responses").allows("/v1/models"));
        assert!(PathRule::Prefix("/v1/").allows("/v1/messages"));
        assert!(PathRule::Prefix("/v1/").allows("/v1/messages/count_tokens"));
        assert!(!PathRule::Prefix("/v1/").allows("/v2/messages"));
        // A traversal-style escape out of the prefix must not match.
        assert!(!PathRule::Prefix("/v1beta/").allows("/v2/models"));
    }

    #[test]
    fn injected_header_names_cover_placeholders_to_strip() {
        assert_eq!(
            CredentialInjection::BearerAuth.injected_header_names(),
            vec!["authorization"]
        );
        assert_eq!(
            CLAUDE_BROKER.injection.injected_header_names(),
            vec!["x-api-key", "anthropic-version"]
        );
        assert_eq!(
            GEMINI_BROKER.injection.injected_header_names(),
            vec!["x-goog-api-key"]
        );
    }

    #[test]
    fn only_brokerable_providers_have_profiles() {
        assert!(broker_profile_for_agent("codex").is_some());
        assert!(broker_profile_for_agent("claude").is_some());
        assert!(broker_profile_for_agent("gemini").is_some());
        // Copilot stays design-only: token exchange + dynamic API host.
        assert!(broker_profile_for_agent("copilot").is_none());
        assert!(broker_profile_for_agent("shell").is_none());
    }
}
