use anyhow::{Result, bail};
use serde::Serialize;

pub const DESIGN_ONLY_AUTH_BROKER_STATUS: &str = "design-only";
pub const CODEX_API_KEY_BROKER_STATUS: &str = "api-key-broker";
pub const AUTH_BROKER_STATUS: &str = "api-key-broker (codex, claude, gemini)";
pub const AUTH_BROKER_RUNTIME: &str = "macOS 26+ with Apple container only";
pub const CODEX_BROKER_PLACEHOLDER_ENV: &str = "RUNHAVEN_CODEX_BROKER_TOKEN";

#[derive(Clone, Copy, Debug, Serialize)]
pub struct AuthBrokerProfile {
    pub name: &'static str,
    pub status: &'static str,
    pub supported_auth: &'static [&'static str],
    pub host_keeps: &'static [&'static str],
    pub guest_receives: &'static [&'static str],
    pub current_safe_path: &'static str,
    pub notes: &'static [&'static str],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentSignIn {
    RunHavenLogin,
    InSandbox,
    NotApplicable,
}

impl AgentSignIn {
    pub fn label(self) -> &'static str {
        match self {
            AgentSignIn::RunHavenLogin => "runhaven login",
            AgentSignIn::InSandbox => "in-sandbox",
            AgentSignIn::NotApplicable => "n/a",
        }
    }

    pub fn supports_login(self) -> bool {
        matches!(self, AgentSignIn::RunHavenLogin)
    }
}

pub fn auth_broker_profiles() -> Vec<AuthBrokerProfile> {
    profile_names()
        .iter()
        .map(|name| get_auth_broker_profile(name).expect("known auth profile"))
        .collect()
}

/// How a user signs this agent in: a `runhaven login` command, an in-sandbox
/// login at first run, or not applicable for the generic shell profile.
pub fn agent_sign_in_mode(name: &str) -> AgentSignIn {
    match name {
        "antigravity" | "claude" | "codex" | "copilot" => AgentSignIn::RunHavenLogin,
        "shell" => AgentSignIn::NotApplicable,
        _ => AgentSignIn::InSandbox,
    }
}

pub fn agent_sign_in(name: &str) -> &'static str {
    agent_sign_in_mode(name).label()
}

/// Whether the host-side API-key broker covers this agent.
pub fn agent_broker(name: &str) -> &'static str {
    if name == "shell" {
        "n/a"
    } else if is_brokered(name) {
        "yes"
    } else {
        "no"
    }
}

/// Whether RunHaven has a host-side API-key broker for this agent. Codex,
/// Claude, and Gemini are brokered today; Copilot and Antigravity are
/// design-only. Unknown agents are not brokered.
pub fn is_brokered(name: &str) -> bool {
    get_auth_broker_profile(name)
        .map(|profile| profile.status == CODEX_API_KEY_BROKER_STATUS)
        .unwrap_or(false)
}

pub fn get_auth_broker_profile(name: &str) -> Result<AuthBrokerProfile> {
    let profile = match name {
        "antigravity" => AuthBrokerProfile {
            name: "antigravity",
            status: DESIGN_ONLY_AUTH_BROKER_STATUS,
            supported_auth: &[
                "runtime auth sources are incomplete",
                "no bundled credential broker is planned until official auth sources are reviewed",
            ],
            host_keeps: &[
                "no Antigravity credential is read by RunHaven",
                "no host browser, Keychain, or cloud credential state is imported",
            ],
            guest_receives: &[
                "nothing brokered by RunHaven today",
                "only explicitly named --env values or isolated agent state can be visible",
            ],
            current_safe_path: "Use isolated agent state or explicit --env NAME only after reviewing the provider's current auth requirements.",
            notes: &["Antigravity has no bundled provider hosts yet."],
        },
        "claude" => AuthBrokerProfile {
            name: "claude",
            status: CODEX_API_KEY_BROKER_STATUS,
            supported_auth: &[
                "Anthropic API key through --api-key-broker-env NAME (brokered host-side)",
                "Claude.ai or subscription OAuth login (use isolated state, not the broker)",
                "Bedrock, Vertex, Azure, or Foundry provider auth (not brokered)",
            ],
            host_keeps: &[
                "the host environment variable value named by --api-key-broker-env",
                "the API key is injected as x-api-key only into brokered host requests to api.anthropic.com",
            ],
            guest_receives: &[
                "a placeholder ANTHROPIC_API_KEY value",
                "ANTHROPIC_BASE_URL pointed at the broker on the RunHaven provider network",
            ],
            current_safe_path: "Use --network provider --api-key-broker-env ANTHROPIC_API_KEY for a headless API-key run. For OAuth or subscription login, authenticate inside the isolated Claude state volume; RunHaven never reads your host ~/.claude.json or Keychain.",
            notes: &[
                "The broker covers the API-key path only; OAuth and subscription logins use isolated in-container state.",
                "The raw host API key is never placed in the container command or guest environment.",
            ],
        },
        "codex" => AuthBrokerProfile {
            name: "codex",
            status: CODEX_API_KEY_BROKER_STATUS,
            supported_auth: &[
                "OpenAI API key through --api-key-broker-env NAME",
                "ChatGPT browser sign-in",
                "OpenAI API key sign-in",
                "Codex access token from a trusted environment",
            ],
            host_keeps: &[
                "host environment variable value named by --api-key-broker-env",
                "the API key is injected only into brokered host requests to api.openai.com",
            ],
            guest_receives: &[
                "RUNHAVEN_CODEX_BROKER_TOKEN placeholder token value",
                "temporary Codex custom provider config pointing at the broker on the RunHaven provider network",
            ],
            current_safe_path: "Use --network provider --api-key-broker-env OPENAI_API_KEY for a headless API-key run, or authenticate inside isolated Codex state when using browser login.",
            notes: &[
                "The prototype supports Codex Responses API requests only.",
                "The raw host API key is never placed in the container command or guest environment.",
            ],
        },
        "copilot" => AuthBrokerProfile {
            name: "copilot",
            status: DESIGN_ONLY_AUTH_BROKER_STATUS,
            supported_auth: &[
                "GitHub OAuth device flow",
                "GitHub CLI fallback token",
                "COPILOT_GITHUB_TOKEN, GH_TOKEN, or GITHUB_TOKEN for headless use",
                "BYOK provider environment variables",
            ],
            host_keeps: &[
                "future broker-owned Copilot or GitHub token material",
                "future provider-specific BYOK credentials when explicitly configured",
            ],
            guest_receives: &[
                "nothing brokered by RunHaven today",
                "current runs expose credentials only through isolated agent state or explicit --env NAME",
            ],
            current_safe_path: "Use Copilot's login inside isolated state when interactive, or pass COPILOT_GITHUB_TOKEN by name only after choosing the narrowest token scope.",
            notes: &[
                "No host-side broker: Copilot exchanges the GitHub token for a short-lived, dynamically-routed Copilot API host, which cannot be brokered without TLS interception. Use isolated in-container login state instead.",
            ],
        },
        "gemini" => AuthBrokerProfile {
            name: "gemini",
            status: CODEX_API_KEY_BROKER_STATUS,
            supported_auth: &[
                "Gemini API key through --api-key-broker-env NAME (brokered host-side)",
                "Google account OAuth login (use isolated state, not the broker)",
                "Vertex AI ADC, service-account JSON, or Cloud API key (not brokered)",
            ],
            host_keeps: &[
                "the host environment variable value named by --api-key-broker-env",
                "the API key is injected as x-goog-api-key only into brokered host requests to generativelanguage.googleapis.com",
            ],
            guest_receives: &[
                "a placeholder GEMINI_API_KEY value",
                "GOOGLE_GEMINI_BASE_URL pointed at the broker on the RunHaven provider network",
            ],
            current_safe_path: "Use --network provider --api-key-broker-env GEMINI_API_KEY for a headless API-key run. For Google account login, authenticate inside the isolated Gemini state volume; do not mount Google Cloud ADC or service-account files into the guest.",
            notes: &[
                "The broker base-URL env (GOOGLE_GEMINI_BASE_URL) is currently undocumented upstream and version-fragile; re-verify per Gemini CLI version.",
                "The raw host API key is never placed in the container command or guest environment.",
            ],
        },
        "shell" => AuthBrokerProfile {
            name: "shell",
            status: DESIGN_ONLY_AUTH_BROKER_STATUS,
            supported_auth: &["custom image or command decides its own auth requirements"],
            host_keeps: &["no custom-image credential is read by RunHaven"],
            guest_receives: &[
                "nothing brokered by RunHaven today",
                "current runs expose credentials only through isolated state or explicit --env NAME",
            ],
            current_safe_path: "Prefer no credentials; when required, pass the narrowest single variable by name with --env NAME after reviewing the custom image.",
            notes: &[],
        },
        _ => {
            let known = profile_names().join(", ");
            bail!("unknown auth profile {name:?}; known profiles: {known}");
        }
    };
    Ok(profile)
}

fn profile_names() -> &'static [&'static str] {
    &[
        "antigravity",
        "claude",
        "codex",
        "copilot",
        "gemini",
        "shell",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_brokered_matches_the_api_key_broker_agents() {
        for agent in ["claude", "codex", "gemini"] {
            assert!(is_brokered(agent), "{agent} should be brokered");
        }
        for agent in ["copilot", "antigravity", "shell", "unknown"] {
            assert!(!is_brokered(agent), "{agent} should not be brokered");
        }
    }

    #[test]
    fn agent_sign_in_mode_matches_supported_login_agents() {
        for agent in ["antigravity", "claude", "codex", "copilot"] {
            assert_eq!(agent_sign_in_mode(agent), AgentSignIn::RunHavenLogin);
            assert_eq!(agent_sign_in(agent), "runhaven login");
        }
        assert_eq!(agent_sign_in_mode("shell"), AgentSignIn::NotApplicable);
        assert_eq!(agent_sign_in("shell"), "n/a");
        assert_eq!(agent_sign_in_mode("unknown"), AgentSignIn::InSandbox);
    }
}
