use crate::egress::{is_ip_literal, normalize_host};

#[derive(Clone, Copy, Debug)]
pub struct ProviderEndpoint {
    pub profile: &'static str,
    pub host: &'static str,
    pub status: &'static str,
    pub purpose: &'static str,
    pub evidence: &'static [&'static str],
    pub note: &'static str,
}

pub fn bundled_provider_hosts(profile: &str) -> &'static [&'static str] {
    match profile {
        "claude" => &["api.anthropic.com", "claude.ai", "platform.claude.com"],
        // auth.openai.com is the Codex device-login + token-refresh host.
        "codex" => &["api.openai.com", "chatgpt.com", "auth.openai.com"],
        "gemini" => &["generativelanguage.googleapis.com"],
        "copilot" => &[
            "api.githubcopilot.com",
            "individual.githubcopilot.com",
            "business.githubcopilot.com",
            "enterprise.githubcopilot.com",
            "githubcopilot.com",
            "copilot-proxy.githubusercontent.com",
            "origin-tracker.githubusercontent.com",
            // github.com is the device-login host and api.github.com is the
            // Copilot token-exchange host. github.com is broad (a deliberate
            // egress widening for the copilot profile).
            "github.com",
            "api.github.com",
        ],
        _ => &[],
    }
}

pub fn match_provider_endpoints(host: &str, profile: Option<&str>) -> Vec<ProviderEndpoint> {
    let Ok(normalized) = normalize_host(host) else {
        return Vec::new();
    };
    if is_ip_literal(&normalized) {
        return Vec::new();
    }
    let mut exact = Vec::new();
    let mut suffix = Vec::new();
    for endpoint in PROVIDER_ENDPOINTS {
        if profile.is_some_and(|profile| endpoint.profile != profile) {
            continue;
        }
        let endpoint_host =
            normalize_host(endpoint.host).unwrap_or_else(|_| endpoint.host.to_string());
        if normalized == endpoint_host {
            exact.push(*endpoint);
        } else if normalized.ends_with(&format!(".{endpoint_host}")) {
            suffix.push(*endpoint);
        }
    }
    if exact.is_empty() { suffix } else { exact }
}

pub static PROVIDER_ENDPOINTS: &[ProviderEndpoint] = &[
    ProviderEndpoint {
        profile: "claude",
        host: "api.anthropic.com",
        status: "bundled",
        purpose: "Claude API requests and WebFetch domain safety checks.",
        evidence: &["https://code.claude.com/docs/en/corporate-proxy"],
        note: "",
    },
    ProviderEndpoint {
        profile: "claude",
        host: "claude.ai",
        status: "bundled",
        purpose: "Claude account authentication for Claude Pro, Max, and web-backed auth flows.",
        evidence: &["https://code.claude.com/docs/en/corporate-proxy"],
        note: "",
    },
    ProviderEndpoint {
        profile: "claude",
        host: "platform.claude.com",
        status: "bundled",
        purpose: "Anthropic Console account authentication.",
        evidence: &["https://code.claude.com/docs/en/corporate-proxy"],
        note: "",
    },
    ProviderEndpoint {
        profile: "claude",
        host: "downloads.claude.ai",
        status: "optional",
        purpose: "Claude Code plugin executable downloads, native installer, and native updater.",
        evidence: &["https://code.claude.com/docs/en/corporate-proxy"],
        note: "RunHaven images install pinned npm packages, so native updater hosts are not bundled.",
    },
    ProviderEndpoint {
        profile: "claude",
        host: "raw.githubusercontent.com",
        status: "optional",
        purpose: "Claude Code changelog feed, release notes, and plugin marketplace install counts.",
        evidence: &["https://code.claude.com/docs/en/corporate-proxy"],
        note: "Path-specific GitHub access is not bundled until RunHaven has path-aware policy.",
    },
    ProviderEndpoint {
        profile: "codex",
        host: "api.openai.com",
        status: "bundled",
        purpose: "OpenAI API traffic and Codex network-policy examples.",
        evidence: &[
            "https://developers.openai.com/codex/agent-approvals-security",
            "https://developers.openai.com/codex/permissions",
        ],
        note: "",
    },
    ProviderEndpoint {
        profile: "codex",
        host: "chatgpt.com",
        status: "bundled",
        purpose: "ChatGPT sign-in, Codex web surface, and standalone installer host.",
        evidence: &[
            "https://developers.openai.com/codex/auth",
            "https://developers.openai.com/codex/cli",
        ],
        note: "",
    },
    ProviderEndpoint {
        profile: "codex",
        host: "auth.openai.com",
        status: "bundled",
        purpose: "Codex device-code login and OAuth token refresh.",
        evidence: &["https://developers.openai.com/codex/auth"],
        note: "`runhaven login codex` runs `codex login --device-auth`; the account must allow device-code login.",
    },
    ProviderEndpoint {
        profile: "gemini",
        host: "generativelanguage.googleapis.com",
        status: "bundled",
        purpose: "Gemini API key model traffic.",
        evidence: &[
            "https://google-gemini.github.io/gemini-cli/docs/get-started/authentication.html",
        ],
        note: "",
    },
    ProviderEndpoint {
        profile: "gemini",
        host: "accounts.google.com",
        status: "candidate",
        purpose: "Browser-based Google account sign-in for Gemini CLI.",
        evidence: &[
            "https://google-gemini.github.io/gemini-cli/docs/get-started/authentication.html",
        ],
        note: "The documented flow uses a browser and localhost callback; live container smoke is needed.",
    },
    ProviderEndpoint {
        profile: "antigravity",
        host: "storage.googleapis.com",
        status: "build",
        purpose: "Pinned Antigravity CLI archive download during image build.",
        evidence: &["images/antigravity/Containerfile"],
        note: "No source-backed minimal runtime host set has been identified for Antigravity CLI.",
    },
    ProviderEndpoint {
        profile: "copilot",
        host: "api.githubcopilot.com",
        status: "bundled",
        purpose: "Copilot API service for suggestions.",
        evidence: &["https://docs.github.com/en/copilot/reference/copilot-allowlist-reference"],
        note: "",
    },
    ProviderEndpoint {
        profile: "copilot",
        host: "githubcopilot.com",
        status: "bundled",
        purpose: "Copilot API service wildcard family.",
        evidence: &["https://docs.github.com/en/copilot/reference/copilot-allowlist-reference"],
        note: "",
    },
    ProviderEndpoint {
        profile: "copilot",
        host: "individual.githubcopilot.com",
        status: "bundled",
        purpose: "Copilot individual subscription routing.",
        evidence: &["https://docs.github.com/en/copilot/reference/copilot-allowlist-reference"],
        note: "",
    },
    ProviderEndpoint {
        profile: "copilot",
        host: "business.githubcopilot.com",
        status: "bundled",
        purpose: "Copilot Business subscription routing.",
        evidence: &["https://docs.github.com/en/copilot/reference/copilot-allowlist-reference"],
        note: "",
    },
    ProviderEndpoint {
        profile: "copilot",
        host: "enterprise.githubcopilot.com",
        status: "bundled",
        purpose: "Copilot Enterprise subscription routing.",
        evidence: &["https://docs.github.com/en/copilot/reference/copilot-allowlist-reference"],
        note: "",
    },
    ProviderEndpoint {
        profile: "copilot",
        host: "copilot-proxy.githubusercontent.com",
        status: "bundled",
        purpose: "Copilot API service for suggestions.",
        evidence: &["https://docs.github.com/en/copilot/reference/copilot-allowlist-reference"],
        note: "",
    },
    ProviderEndpoint {
        profile: "copilot",
        host: "origin-tracker.githubusercontent.com",
        status: "bundled",
        purpose: "Copilot API service for suggestions.",
        evidence: &["https://docs.github.com/en/copilot/reference/copilot-allowlist-reference"],
        note: "",
    },
    ProviderEndpoint {
        profile: "copilot",
        host: "github.com",
        status: "bundled",
        purpose: "GitHub device-flow login (github.com/login/device) for `runhaven login copilot`.",
        evidence: &["https://docs.github.com/en/copilot/reference/copilot-allowlist-reference"],
        note: "Deliberate egress widening: RunHaven cannot yet restrict this host to /login, so the whole host is allowed for the Copilot login flow.",
    },
    ProviderEndpoint {
        profile: "copilot",
        host: "api.github.com",
        status: "bundled",
        purpose: "Copilot token exchange (api.github.com/user, /copilot_internal).",
        evidence: &["https://docs.github.com/en/copilot/reference/copilot-allowlist-reference"],
        note: "Deliberate egress widening: RunHaven cannot yet restrict this host to specific API paths.",
    },
];
