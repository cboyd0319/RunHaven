use anyhow::{Result, bail};

use crate::provider::endpoints::bundled_provider_hosts;

#[derive(Clone, Debug)]
pub struct AgentProfile {
    pub name: &'static str,
    pub description: &'static str,
    pub image: &'static str,
    pub command: &'static [&'static str],
    pub home_env: &'static [(&'static str, &'static str)],
    pub image_context: Option<&'static str>,
    pub provider_hosts: &'static [&'static str],
}

impl AgentProfile {
    pub fn env(&self) -> &'static [(&'static str, &'static str)] {
        self.home_env
    }
}

pub fn profiles() -> Vec<AgentProfile> {
    profile_names()
        .iter()
        .map(|name| get_profile(name).expect("known profile"))
        .collect()
}

/// Product agents shown in the desktop app. The generic `shell` profile remains
/// a CLI/custom-image utility and is not presented as a fifth agent platform.
pub fn agent_profiles() -> Vec<AgentProfile> {
    profiles()
        .into_iter()
        .filter(|profile| profile.name != "shell")
        .collect()
}

pub fn profile_names() -> &'static [&'static str] {
    &["antigravity", "claude", "codex", "copilot", "shell"]
}

pub fn get_profile(name: &str) -> Result<AgentProfile> {
    let profile = match name {
        "claude" => AgentProfile {
            name: "claude",
            description: "Claude Code with state isolated under /home/agent/.claude.",
            image: "runhaven/claude:0.7.0",
            command: &["claude"],
            // Disable Claude Code's non-essential traffic (DataDog telemetry,
            // error reporting, auto-update checks) so the sandboxed agent never
            // attempts non-provider egress. Provider-only egress blocks those
            // hosts anyway; turning them off keeps the egress log clean and the
            // agent from phoning home. Verified env var in the claude binary.
            home_env: &[
                ("CLAUDE_CONFIG_DIR", "/home/agent/.claude"),
                ("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC", "1"),
            ],
            image_context: Some("claude"),
            provider_hosts: bundled_provider_hosts("claude"),
        },
        "codex" => AgentProfile {
            name: "codex",
            description: "OpenAI Codex CLI with workspace-write sandboxing inside the container.",
            image: "runhaven/codex:0.7.0",
            command: &[
                "codex",
                "--sandbox",
                "workspace-write",
                "--ask-for-approval",
                "on-request",
            ],
            home_env: &[],
            image_context: Some("codex"),
            provider_hosts: bundled_provider_hosts("codex"),
        },
        "antigravity" => AgentProfile {
            name: "antigravity",
            description: "Google Antigravity CLI with isolated home state.",
            image: "runhaven/antigravity:0.7.0",
            command: &["agy"],
            home_env: &[],
            image_context: Some("antigravity"),
            provider_hosts: bundled_provider_hosts("antigravity"),
        },
        "copilot" => AgentProfile {
            name: "copilot",
            description: "GitHub Copilot CLI with isolated COPILOT_HOME state.",
            image: "runhaven/copilot:0.7.0",
            command: &["copilot"],
            home_env: &[("COPILOT_HOME", "/home/agent/.copilot")],
            image_context: Some("copilot"),
            provider_hosts: bundled_provider_hosts("copilot"),
        },
        "shell" => AgentProfile {
            name: "shell",
            description: "Generic shell profile for custom agent images.",
            image: "runhaven/base:0.7.0",
            command: &["/bin/bash"],
            home_env: &[],
            image_context: Some("base"),
            provider_hosts: bundled_provider_hosts("shell"),
        },
        _ => {
            let known = profile_names().join(", ");
            bail!("unknown agent {name:?}; known agents: {known}");
        }
    };
    Ok(profile)
}

#[cfg(test)]
mod tests {
    use super::{agent_profiles, get_profile};

    #[test]
    fn bundled_agent_platforms_match_the_supported_product_set() {
        let platform_names = agent_profiles()
            .iter()
            .map(|profile| profile.name)
            .collect::<Vec<_>>();

        assert_eq!(
            platform_names,
            ["antigravity", "claude", "codex", "copilot"]
        );
        assert!(get_profile("gemini").is_err());
    }

    #[test]
    fn codex_uses_its_home_default_without_a_fresh_state_warning() {
        let profile = get_profile("codex").expect("codex profile");
        assert!(
            profile
                .home_env
                .iter()
                .all(|(name, _)| *name != "CODEX_HOME")
        );
    }
}
