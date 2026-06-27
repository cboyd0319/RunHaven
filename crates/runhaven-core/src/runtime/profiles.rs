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

pub fn profile_names() -> &'static [&'static str] {
    &[
        "antigravity",
        "claude",
        "codex",
        "copilot",
        "gemini",
        "shell",
    ]
}

pub fn get_profile(name: &str) -> Result<AgentProfile> {
    let profile = match name {
        "claude" => AgentProfile {
            name: "claude",
            description: "Claude Code with state isolated under /home/agent/.claude.",
            image: "runhaven/claude:0.1.0",
            command: &["claude"],
            home_env: &[("CLAUDE_CONFIG_DIR", "/home/agent/.claude")],
            image_context: Some("claude"),
            provider_hosts: bundled_provider_hosts("claude"),
        },
        "codex" => AgentProfile {
            name: "codex",
            description: "OpenAI Codex CLI with workspace-write sandboxing inside the container.",
            image: "runhaven/codex:0.1.0",
            command: &[
                "codex",
                "--sandbox",
                "workspace-write",
                "--ask-for-approval",
                "on-request",
            ],
            home_env: &[("CODEX_HOME", "/home/agent/.codex")],
            image_context: Some("codex"),
            provider_hosts: bundled_provider_hosts("codex"),
        },
        "gemini" => AgentProfile {
            name: "gemini",
            description: "Gemini CLI with project-scoped home state.",
            image: "runhaven/gemini:0.1.0",
            command: &["gemini"],
            home_env: &[],
            image_context: Some("gemini"),
            provider_hosts: bundled_provider_hosts("gemini"),
        },
        "antigravity" => AgentProfile {
            name: "antigravity",
            description: "Google Antigravity CLI with project-scoped home state.",
            image: "runhaven/antigravity:0.1.0",
            command: &["agy"],
            home_env: &[],
            image_context: Some("antigravity"),
            provider_hosts: bundled_provider_hosts("antigravity"),
        },
        "copilot" => AgentProfile {
            name: "copilot",
            description: "GitHub Copilot CLI with COPILOT_HOME isolated per project.",
            image: "runhaven/copilot:0.1.0",
            command: &["copilot"],
            home_env: &[("COPILOT_HOME", "/home/agent/.copilot")],
            image_context: Some("copilot"),
            provider_hosts: bundled_provider_hosts("copilot"),
        },
        "shell" => AgentProfile {
            name: "shell",
            description: "Generic shell profile for custom agent images.",
            image: "runhaven/base:0.1.0",
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
