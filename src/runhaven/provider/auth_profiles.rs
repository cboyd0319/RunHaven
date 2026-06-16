use anyhow::{Result, bail};
use serde::Serialize;

pub const DESIGN_ONLY_AUTH_BROKER_STATUS: &str = "design-only";
pub const CODEX_API_KEY_BROKER_STATUS: &str = "api-key-prototype";
pub const AUTH_BROKER_STATUS: &str = "codex-api-key-prototype";
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

pub fn auth_broker_profiles() -> Vec<AuthBrokerProfile> {
    profile_names()
        .iter()
        .map(|name| get_auth_broker_profile(name).expect("known auth profile"))
        .collect()
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
            status: DESIGN_ONLY_AUTH_BROKER_STATUS,
            supported_auth: &[
                "Claude.ai browser login",
                "Anthropic API key",
                "Claude Code apiKeyHelper script",
                "Bedrock, Vertex, Azure, or Foundry provider auth",
            ],
            host_keeps: &[
                "future broker-owned Claude credential material",
                "future broker helper output cache, if a rotating helper is implemented",
            ],
            guest_receives: &[
                "nothing brokered by RunHaven today",
                "current runs expose credentials only through isolated agent state or explicit --env NAME",
            ],
            current_safe_path: "Authenticate inside the isolated Claude state volume when interactive, or pass ANTHROPIC_API_KEY by name only for a deliberate headless run.",
            notes: &[],
        },
        "codex" => AuthBrokerProfile {
            name: "codex",
            status: CODEX_API_KEY_BROKER_STATUS,
            supported_auth: &[
                "OpenAI API key through --codex-api-key-broker-env NAME",
                "ChatGPT browser sign-in",
                "OpenAI API key sign-in",
                "Codex access token from a trusted environment",
            ],
            host_keeps: &[
                "host environment variable value named by --codex-api-key-broker-env",
                "the API key is injected only into brokered host requests to api.openai.com",
            ],
            guest_receives: &[
                "RUNHAVEN_CODEX_BROKER_TOKEN placeholder token value",
                "temporary Codex custom provider config pointing at the broker on the RunHaven provider network",
            ],
            current_safe_path: "Use --network provider --codex-api-key-broker-env OPENAI_API_KEY for a headless API-key run, or authenticate inside isolated Codex state when using browser login.",
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
                "GitHub and API hosts remain path-sensitive and explicit until a broker can avoid broad host access.",
            ],
        },
        "gemini" => AuthBrokerProfile {
            name: "gemini",
            status: DESIGN_ONLY_AUTH_BROKER_STATUS,
            supported_auth: &[
                "Google account login",
                "Gemini API key",
                "Vertex AI Application Default Credentials",
                "Vertex AI service account JSON key",
                "Vertex AI Google Cloud API key",
            ],
            host_keeps: &[
                "future broker-owned Gemini API key or Vertex credential material",
                "no service account JSON is copied into the guest by default",
            ],
            guest_receives: &[
                "nothing brokered by RunHaven today",
                "current runs expose credentials only through isolated agent state or explicit --env NAME",
            ],
            current_safe_path: "Use an isolated Gemini login or pass GEMINI_API_KEY by name only; do not mount Google Cloud ADC or service-account files into the guest by default.",
            notes: &[],
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
