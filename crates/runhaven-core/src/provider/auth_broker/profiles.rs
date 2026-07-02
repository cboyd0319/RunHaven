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
    /// Host environment variable RunHaven may read for this broker.
    pub host_env: &'static str,
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
    host_env: "OPENAI_API_KEY",
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
    host_env: "ANTHROPIC_API_KEY",
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

/// Claude OAuth-token broker (design C): the subscription OAuth-token path,
/// sibling of `CLAUDE_BROKER`. Same pinned upstream host and `/v1/` prefix, so
/// egress is identical to the API-key Claude broker (no widening). The token is
/// a Claude subscription OAuth token read from the macOS Keychain at
/// broker-spawn (never from a host env var) and injected as
/// `Authorization: Bearer` by `headers.rs`, which also strips any guest-sent
/// `authorization`. The guest holds only a placeholder `CLAUDE_CODE_OAUTH_TOKEN`
/// plus the `ANTHROPIC_BASE_URL` redirect.
///
/// `host_env` is set to the placeholder env name for consistency but is IGNORED
/// on this path: `require_api_key_broker_secret` reads the Keychain, not an env
/// var, when the run's `ClaudeAuthSource` is `KeychainOAuth`.
///
/// F5 (`anthropic-beta`): OAuth requests to `/v1/messages` also carry
/// `anthropic-beta: ...oauth-2025-04-20...`. The guest Claude Code CLI sends
/// that header and `headers.rs` forwards it unchanged (it is neither hop-by-hop
/// nor an injected header, so it is never stripped), so Bearer-only injection is
/// correct today (empirically verified capture, design Section "Verified
/// facts"). Seam: if a future guest stops sending it, inject it host-side by
/// extending `CredentialInjection::BearerAuth` with a fixed-header list
/// mirroring `ApiKeyHeader.extra`; either way only the injected Bearer plus this
/// non-secret beta tag leave the host, so it is not a leak.
pub const CLAUDE_OAUTH_BROKER: ProviderBrokerProfile = ProviderBrokerProfile {
    agent: "claude",
    host_env: "CLAUDE_CODE_OAUTH_TOKEN",
    label: "RunHaven Claude OAuth-token broker",
    upstream_host: "api.anthropic.com",
    path_rule: PathRule::Prefix("/v1/"),
    injection: CredentialInjection::BearerAuth,
    placeholder_env: "CLAUDE_CODE_OAUTH_TOKEN",
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
    host_env: "GEMINI_API_KEY",
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
    fn claude_oauth_broker_pins_bearer_host_and_path_without_widening_egress() {
        // BearerAuth so headers.rs emits `Authorization: Bearer` and strips the
        // guest `authorization` header.
        assert_eq!(
            CLAUDE_OAUTH_BROKER.injection,
            CredentialInjection::BearerAuth
        );
        assert_eq!(
            CLAUDE_OAUTH_BROKER.injection.injected_header_names(),
            vec!["authorization"]
        );
        // Same pinned upstream host and `/v1/` prefix as the API-key Claude
        // broker: the OAuth broker adds NO new egress destination.
        assert_eq!(CLAUDE_OAUTH_BROKER.upstream_host, "api.anthropic.com");
        assert_eq!(
            CLAUDE_OAUTH_BROKER.upstream_host,
            CLAUDE_BROKER.upstream_host
        );
        assert!(CLAUDE_OAUTH_BROKER.path_rule.allows("/v1/messages"));
        assert!(
            CLAUDE_OAUTH_BROKER
                .path_rule
                .allows("/v1/messages/count_tokens")
        );
        assert!(!CLAUDE_OAUTH_BROKER.path_rule.allows("/v2/messages"));
        // The guest is pointed at the broker via ANTHROPIC_BASE_URL and holds
        // only the placeholder OAuth token.
        assert_eq!(
            CLAUDE_OAUTH_BROKER.guest_redirect,
            GuestRedirect::EnvRedirect {
                base_url_env: "ANTHROPIC_BASE_URL"
            }
        );
        assert_eq!(
            CLAUDE_OAUTH_BROKER.placeholder_env,
            "CLAUDE_CODE_OAUTH_TOKEN"
        );
        assert_eq!(
            CLAUDE_OAUTH_BROKER.placeholder_value,
            BROKER_PLACEHOLDER_VALUE
        );
    }

    #[test]
    fn claude_oauth_broker_injects_bearer_strips_guest_auth_and_forwards_beta() {
        use crate::provider::auth_broker::broker_request_headers;
        // A hostile guest sends its own Authorization (the placeholder, or a
        // stolen value) plus the non-secret anthropic-beta tag.
        let guest_headers = vec![
            (
                "authorization".to_string(),
                format!("Bearer {BROKER_PLACEHOLDER_VALUE}"),
            ),
            ("anthropic-beta".to_string(), "oauth-2025-04-20".to_string()),
            ("content-type".to_string(), "application/json".to_string()),
        ];
        let real_token = "sk-ant-oat01-REAL-TOKEN-000000000000";
        let forwarded = broker_request_headers(
            &guest_headers,
            CLAUDE_OAUTH_BROKER.upstream_host,
            real_token,
            &CLAUDE_OAUTH_BROKER.injection,
            2,
        );
        // Exactly one Authorization header, and it carries the real token.
        let auth: Vec<&(String, String)> = forwarded
            .iter()
            .filter(|(name, _)| name.eq_ignore_ascii_case("authorization"))
            .collect();
        assert_eq!(auth.len(), 1, "exactly one Authorization header survives");
        assert_eq!(auth[0].1, format!("Bearer {real_token}"));
        // The guest placeholder Authorization was stripped before injection.
        assert!(
            !forwarded
                .iter()
                .any(|(_, value)| value.contains(BROKER_PLACEHOLDER_VALUE)),
            "the guest placeholder must never reach the upstream request"
        );
        // F5: the guest-supplied anthropic-beta tag is forwarded unchanged (not
        // stripped), so Bearer-only injection is sufficient today.
        assert!(
            forwarded
                .iter()
                .any(|(name, value)| name.eq_ignore_ascii_case("anthropic-beta")
                    && value == "oauth-2025-04-20"),
            "the non-secret anthropic-beta tag must be forwarded"
        );
        // The upstream Host is pinned to Anthropic.
        assert!(
            forwarded
                .iter()
                .any(|(name, value)| name == "Host" && value == "api.anthropic.com")
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
