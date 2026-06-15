from __future__ import annotations

from collections.abc import Mapping
from dataclasses import dataclass
from types import MappingProxyType
from typing import Literal

from .egress import is_ip_literal, normalize_host

EndpointStatus = Literal["bundled", "candidate", "optional", "build"]


@dataclass(frozen=True)
class ProviderEndpoint:
    profile: str
    host: str
    status: EndpointStatus
    purpose: str
    evidence: tuple[str, ...]
    note: str = ""


BUNDLED_PROVIDER_HOSTS: Mapping[str, tuple[str, ...]] = MappingProxyType(
    {
        "claude": (
            "api.anthropic.com",
            "claude.ai",
            "platform.claude.com",
        ),
        "codex": (
            "api.openai.com",
            "chatgpt.com",
        ),
        "gemini": (
            "generativelanguage.googleapis.com",
        ),
        "antigravity": (),
        "copilot": (
            "api.githubcopilot.com",
            "individual.githubcopilot.com",
            "business.githubcopilot.com",
            "enterprise.githubcopilot.com",
            "githubcopilot.com",
            "copilot-proxy.githubusercontent.com",
            "origin-tracker.githubusercontent.com",
        ),
        "shell": (),
    }
)


PROVIDER_ENDPOINTS: tuple[ProviderEndpoint, ...] = (
    ProviderEndpoint(
        profile="claude",
        host="api.anthropic.com",
        status="bundled",
        purpose="Claude API requests and WebFetch domain safety checks.",
        evidence=("https://code.claude.com/docs/en/corporate-proxy",),
    ),
    ProviderEndpoint(
        profile="claude",
        host="claude.ai",
        status="bundled",
        purpose="Claude account authentication for Claude Pro, Max, and web-backed auth flows.",
        evidence=("https://code.claude.com/docs/en/corporate-proxy",),
    ),
    ProviderEndpoint(
        profile="claude",
        host="platform.claude.com",
        status="bundled",
        purpose="Anthropic Console account authentication.",
        evidence=("https://code.claude.com/docs/en/corporate-proxy",),
    ),
    ProviderEndpoint(
        profile="claude",
        host="downloads.claude.ai",
        status="optional",
        purpose="Claude Code plugin executable downloads, native installer, and native updater.",
        evidence=("https://code.claude.com/docs/en/corporate-proxy",),
        note=(
            "RunHaven images install pinned npm packages, so native updater hosts are not "
            "bundled."
        ),
    ),
    ProviderEndpoint(
        profile="claude",
        host="raw.githubusercontent.com",
        status="optional",
        purpose="Claude Code changelog feed, release notes, and plugin marketplace install counts.",
        evidence=("https://code.claude.com/docs/en/corporate-proxy",),
        note="Path-specific GitHub access is not bundled until RunHaven has path-aware policy.",
    ),
    ProviderEndpoint(
        profile="claude",
        host="bridge.claudeusercontent.com",
        status="optional",
        purpose="Claude in Chrome extension WebSocket bridge.",
        evidence=("https://code.claude.com/docs/en/corporate-proxy",),
    ),
    ProviderEndpoint(
        profile="codex",
        host="api.openai.com",
        status="bundled",
        purpose="OpenAI API traffic and Codex network-policy examples.",
        evidence=(
            "https://developers.openai.com/codex/agent-approvals-security",
            "https://developers.openai.com/codex/permissions",
        ),
    ),
    ProviderEndpoint(
        profile="codex",
        host="chatgpt.com",
        status="bundled",
        purpose="ChatGPT sign-in, Codex web surface, and standalone installer host.",
        evidence=(
            "https://developers.openai.com/codex/auth",
            "https://developers.openai.com/codex/cli",
        ),
    ),
    ProviderEndpoint(
        profile="gemini",
        host="generativelanguage.googleapis.com",
        status="bundled",
        purpose="Gemini API key model traffic.",
        evidence=("https://google-gemini.github.io/gemini-cli/docs/get-started/authentication.html",),
    ),
    ProviderEndpoint(
        profile="gemini",
        host="accounts.google.com",
        status="candidate",
        purpose="Browser-based Google account sign-in for Gemini CLI.",
        evidence=("https://google-gemini.github.io/gemini-cli/docs/get-started/authentication.html",),
        note=(
            "The documented flow uses a browser and localhost callback; live container smoke "
            "is needed."
        ),
    ),
    ProviderEndpoint(
        profile="gemini",
        host="aiplatform.googleapis.com",
        status="candidate",
        purpose=(
            "Vertex AI mode when GOOGLE_GENAI_USE_VERTEXAI and project/location are "
            "configured."
        ),
        evidence=("https://google-gemini.github.io/gemini-cli/docs/get-started/authentication.html",),
        note="Keep explicit because Vertex projects may have organization-specific controls.",
    ),
    ProviderEndpoint(
        profile="gemini",
        host="cloudcode-pa.googleapis.com",
        status="candidate",
        purpose="Gemini Code Assist path observed in public Gemini CLI error reports.",
        evidence=("https://github.com/google-gemini/gemini-cli/issues/7544",),
        note="Not bundled because this is issue evidence, not a stable vendor allowlist.",
    ),
    ProviderEndpoint(
        profile="antigravity",
        host="storage.googleapis.com",
        status="build",
        purpose="Pinned Antigravity CLI archive download during image build.",
        evidence=("src/runhaven/images/antigravity/Containerfile",),
        note="No source-backed minimal runtime host set has been identified for Antigravity CLI.",
    ),
    ProviderEndpoint(
        profile="copilot",
        host="api.githubcopilot.com",
        status="bundled",
        purpose="Copilot API service for suggestions.",
        evidence=("https://docs.github.com/en/copilot/reference/copilot-allowlist-reference",),
    ),
    ProviderEndpoint(
        profile="copilot",
        host="githubcopilot.com",
        status="bundled",
        purpose="Copilot API service wildcard family.",
        evidence=("https://docs.github.com/en/copilot/reference/copilot-allowlist-reference",),
    ),
    ProviderEndpoint(
        profile="copilot",
        host="individual.githubcopilot.com",
        status="bundled",
        purpose="Copilot individual subscription routing.",
        evidence=(
            "https://docs.github.com/en/copilot/reference/copilot-allowlist-reference",
            "https://docs.github.com/en/copilot/how-tos/administer-copilot/manage-for-organization/manage-access/manage-network-access",
        ),
    ),
    ProviderEndpoint(
        profile="copilot",
        host="business.githubcopilot.com",
        status="bundled",
        purpose="Copilot Business subscription routing.",
        evidence=(
            "https://docs.github.com/en/copilot/reference/copilot-allowlist-reference",
            "https://docs.github.com/en/copilot/how-tos/administer-copilot/manage-for-organization/manage-access/manage-network-access",
        ),
    ),
    ProviderEndpoint(
        profile="copilot",
        host="enterprise.githubcopilot.com",
        status="bundled",
        purpose="Copilot Enterprise subscription routing.",
        evidence=(
            "https://docs.github.com/en/copilot/reference/copilot-allowlist-reference",
            "https://docs.github.com/en/copilot/how-tos/administer-copilot/manage-for-organization/manage-access/manage-network-access",
        ),
    ),
    ProviderEndpoint(
        profile="copilot",
        host="copilot-proxy.githubusercontent.com",
        status="bundled",
        purpose="Copilot API service for suggestions.",
        evidence=("https://docs.github.com/en/copilot/reference/copilot-allowlist-reference",),
    ),
    ProviderEndpoint(
        profile="copilot",
        host="origin-tracker.githubusercontent.com",
        status="bundled",
        purpose="Copilot API service for suggestions.",
        evidence=("https://docs.github.com/en/copilot/reference/copilot-allowlist-reference",),
    ),
    ProviderEndpoint(
        profile="copilot",
        host="github.com",
        status="candidate",
        purpose="GitHub login and Copilot web paths.",
        evidence=("https://docs.github.com/en/copilot/reference/copilot-allowlist-reference",),
        note=(
            "Not bundled because RunHaven currently cannot restrict this host to /login or "
            "/copilot."
        ),
    ),
    ProviderEndpoint(
        profile="copilot",
        host="api.github.com",
        status="candidate",
        purpose="GitHub user and Copilot user-management API paths.",
        evidence=("https://docs.github.com/en/copilot/reference/copilot-allowlist-reference",),
        note=(
            "Not bundled because RunHaven currently cannot restrict this host to specific "
            "API paths."
        ),
    ),
    ProviderEndpoint(
        profile="copilot",
        host="collector.github.com",
        status="optional",
        purpose="GitHub analytics telemetry.",
        evidence=("https://docs.github.com/en/copilot/reference/copilot-allowlist-reference",),
    ),
    ProviderEndpoint(
        profile="copilot",
        host="copilot-telemetry.githubusercontent.com",
        status="optional",
        purpose="Copilot client telemetry.",
        evidence=("https://docs.github.com/en/copilot/reference/copilot-allowlist-reference",),
    ),
    ProviderEndpoint(
        profile="copilot",
        host="default.exp-tas.com",
        status="optional",
        purpose="Copilot client experimentation.",
        evidence=("https://docs.github.com/en/copilot/reference/copilot-allowlist-reference",),
    ),
)


def bundled_provider_hosts(profile: str) -> tuple[str, ...]:
    return BUNDLED_PROVIDER_HOSTS.get(profile, ())


def match_provider_endpoints(
    host: str,
    *,
    profile: str | None = None,
) -> tuple[ProviderEndpoint, ...]:
    try:
        normalized = normalize_host(host)
    except ValueError:
        return ()
    if is_ip_literal(normalized):
        return ()
    exact_matches: list[ProviderEndpoint] = []
    suffix_matches: list[ProviderEndpoint] = []
    for endpoint in PROVIDER_ENDPOINTS:
        if profile is not None and endpoint.profile != profile:
            continue
        endpoint_host = normalize_host(endpoint.host)
        if normalized == endpoint_host:
            exact_matches.append(endpoint)
        elif normalized.endswith(f".{endpoint_host}"):
            suffix_matches.append(endpoint)
    if exact_matches:
        return tuple(exact_matches)
    return tuple(suffix_matches)
