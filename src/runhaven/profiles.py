from __future__ import annotations

from collections.abc import Mapping
from dataclasses import dataclass, field
from types import MappingProxyType


@dataclass(frozen=True)
class AgentProfile:
    name: str
    description: str
    image: str
    command: tuple[str, ...]
    home_env: Mapping[str, str] = field(default_factory=dict)
    image_context: str | None = None

    def env(self) -> Mapping[str, str]:
        return MappingProxyType(dict(self.home_env))


PROFILES: Mapping[str, AgentProfile] = MappingProxyType(
    {
        "claude": AgentProfile(
            name="claude",
            description="Claude Code with state isolated under /home/agent/.claude.",
            image="runhaven/claude:0.1.0",
            command=("claude",),
            home_env={"CLAUDE_CONFIG_DIR": "/home/agent/.claude"},
            image_context="claude",
        ),
        "codex": AgentProfile(
            name="codex",
            description="OpenAI Codex CLI with workspace-write sandboxing inside the container.",
            image="runhaven/codex:0.1.0",
            command=("codex", "--sandbox", "workspace-write", "--ask-for-approval", "on-request"),
            home_env={"CODEX_HOME": "/home/agent/.codex"},
            image_context="codex",
        ),
        "gemini": AgentProfile(
            name="gemini",
            description="Gemini CLI with project-scoped home state.",
            image="runhaven/gemini:0.1.0",
            command=("gemini",),
            image_context="gemini",
        ),
        "antigravity": AgentProfile(
            name="antigravity",
            description="Google Antigravity CLI with project-scoped home state.",
            image="runhaven/antigravity:0.1.0",
            command=("agy",),
            image_context="antigravity",
        ),
        "copilot": AgentProfile(
            name="copilot",
            description="GitHub Copilot CLI with COPILOT_HOME isolated per project.",
            image="runhaven/copilot:0.1.0",
            command=("copilot",),
            home_env={"COPILOT_HOME": "/home/agent/.copilot"},
            image_context="copilot",
        ),
        "shell": AgentProfile(
            name="shell",
            description="Generic shell profile for custom agent images.",
            image="runhaven/base:0.1.0",
            command=("/bin/bash",),
            image_context="base",
        ),
    }
)


def get_profile(name: str) -> AgentProfile:
    try:
        return PROFILES[name]
    except KeyError as exc:
        known = ", ".join(sorted(PROFILES))
        raise ValueError(f"unknown agent {name!r}; known agents: {known}") from exc
