from __future__ import annotations

import http.client
import ipaddress
import socketserver
import threading
from collections.abc import Callable, Mapping, Sequence
from dataclasses import asdict, dataclass
from http.server import BaseHTTPRequestHandler
from types import MappingProxyType
from typing import Any, Literal, cast
from urllib.parse import urlsplit

DESIGN_ONLY_AUTH_BROKER_STATUS = "design-only"
CODEX_API_KEY_BROKER_STATUS = "api-key-prototype"
AUTH_BROKER_STATUS = "codex-api-key-prototype"
AUTH_BROKER_RUNTIME = "macOS 26+ with Apple container only"
CODEX_BROKER_PROVIDER_ID = "runhaven_openai"
CODEX_BROKER_PLACEHOLDER_ENV = "RUNHAVEN_CODEX_BROKER_TOKEN"
CODEX_BROKER_PLACEHOLDER_VALUE = "runhaven-broker-placeholder"
CODEX_BROKER_UPSTREAM_HOST = "api.openai.com"
CODEX_BROKER_RESPONSES_PATH = "/v1/responses"
CODEX_BROKER_REQUEST_TIMEOUT_SECONDS = 120.0
MAX_CODEX_BROKER_REQUEST_BYTES = 64 * 1024 * 1024
HOP_BY_HOP_REQUEST_HEADERS = frozenset(
    {
        "authorization",
        "connection",
        "content-length",
        "host",
        "keep-alive",
        "proxy-authenticate",
        "proxy-authorization",
        "proxy-connection",
        "te",
        "trailer",
        "transfer-encoding",
        "upgrade",
    }
)
HOP_BY_HOP_RESPONSE_HEADERS = frozenset(
    {
        "connection",
        "keep-alive",
        "proxy-authenticate",
        "proxy-authorization",
        "te",
        "trailer",
        "transfer-encoding",
        "upgrade",
    }
)

CodexBrokerUpstream = Callable[[str, str, dict[str, str], bytes], "BrokerUpstreamResponse"]


@dataclass(frozen=True)
class AuthBrokerProfile:
    name: str
    status: str
    supported_auth: tuple[str, ...]
    host_keeps: tuple[str, ...]
    guest_receives: tuple[str, ...]
    current_safe_path: str
    notes: tuple[str, ...] = ()

    def to_json(self) -> dict[str, Any]:
        return asdict(self)


@dataclass(frozen=True)
class BrokerUpstreamResponse:
    status: int
    reason: str
    headers: tuple[tuple[str, str], ...]
    body: bytes


@dataclass(frozen=True)
class BrokerDecision:
    method: str
    path: str
    decision: Literal["allowed", "denied"]
    reason: str
    upstream_status: int | None = None
    count: int = 1


class OpenAIResponsesUpstream:
    def __init__(
        self,
        *,
        host: str = CODEX_BROKER_UPSTREAM_HOST,
        port: int = 443,
        timeout: float = CODEX_BROKER_REQUEST_TIMEOUT_SECONDS,
    ) -> None:
        self.host = host
        self.port = port
        self.timeout = timeout

    def __call__(
        self,
        method: str,
        path: str,
        headers: dict[str, str],
        body: bytes,
    ) -> BrokerUpstreamResponse:
        connection = http.client.HTTPSConnection(
            self.host,
            self.port,
            timeout=self.timeout,
        )
        try:
            connection.request(method, path, body=body, headers=headers)
            response = connection.getresponse()
            response_body = response.read()
            response_headers = tuple(
                (name, value)
                for name, value in response.getheaders()
                if name.lower() not in HOP_BY_HOP_RESPONSE_HEADERS
            )
            return BrokerUpstreamResponse(
                status=response.status,
                reason=response.reason,
                headers=response_headers,
                body=response_body,
            )
        finally:
            connection.close()


class CodexApiKeyBrokerProxy(socketserver.ThreadingTCPServer):
    allow_reuse_address = True
    daemon_threads = True

    def __init__(
        self,
        server_address: tuple[str, int],
        *,
        api_key: str,
        allowed_client_subnets: Sequence[str] = (),
        upstream_host: str = CODEX_BROKER_UPSTREAM_HOST,
        upstream: CodexBrokerUpstream | None = None,
    ) -> None:
        if not api_key.strip():
            raise ValueError("Codex API key broker requires a host API key")
        self.api_key = api_key
        self.upstream_host = upstream_host
        self.upstream = upstream or OpenAIResponsesUpstream(host=upstream_host)
        self.allowed_client_networks = tuple(
            ipaddress.ip_network(subnet, strict=False) for subnet in allowed_client_subnets
        )
        self._broker_decisions: dict[tuple[str, str, str, str, int | None], int] = {}
        self._broker_decisions_lock = threading.Lock()
        super().__init__(server_address, CodexApiKeyBrokerHandler)

    def allows_client(self, address: str) -> bool:
        if not self.allowed_client_networks:
            return True
        try:
            client_address = ipaddress.ip_address(address)
        except ValueError:
            return False
        return any(client_address in network for network in self.allowed_client_networks)

    def record_broker_decision(
        self,
        method: str,
        path: str,
        *,
        decision: Literal["allowed", "denied"],
        reason: str,
        upstream_status: int | None = None,
    ) -> None:
        key = (method, path, decision, reason, upstream_status)
        with self._broker_decisions_lock:
            self._broker_decisions[key] = self._broker_decisions.get(key, 0) + 1

    def broker_decisions(self) -> tuple[BrokerDecision, ...]:
        with self._broker_decisions_lock:
            return tuple(
                BrokerDecision(
                    method=method,
                    path=path,
                    decision=cast(Literal["allowed", "denied"], decision),
                    reason=reason,
                    upstream_status=upstream_status,
                    count=count,
                )
                for (method, path, decision, reason, upstream_status), count in (
                    self._broker_decisions.items()
                )
            )


class CodexApiKeyBrokerHandler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def do_POST(self) -> None:
        server = cast(CodexApiKeyBrokerProxy, self.server)
        if not server.allows_client(self.client_address[0]):
            server.record_broker_decision(
                "POST",
                "<client-denied>",
                decision="denied",
                reason="client-not-allowed",
            )
            self.send_broker_error(403, "Forbidden")
            return

        try:
            upstream_path = codex_broker_upstream_path(self.path)
        except ValueError:
            server.record_broker_decision(
                "POST",
                "<unsupported>",
                decision="denied",
                reason="unsupported-path",
            )
            self.send_broker_error(403, "Forbidden")
            return

        try:
            length = parse_content_length(self.headers.get("Content-Length"))
        except ValueError:
            server.record_broker_decision(
                "POST",
                CODEX_BROKER_RESPONSES_PATH,
                decision="denied",
                reason="bad-content-length",
            )
            self.send_broker_error(400, "Bad Request")
            return
        if length is None:
            server.record_broker_decision(
                "POST",
                CODEX_BROKER_RESPONSES_PATH,
                decision="denied",
                reason="length-required",
            )
            self.send_broker_error(411, "Length Required")
            return
        if length > MAX_CODEX_BROKER_REQUEST_BYTES:
            server.record_broker_decision(
                "POST",
                CODEX_BROKER_RESPONSES_PATH,
                decision="denied",
                reason="payload-too-large",
            )
            self.send_broker_error(413, "Payload Too Large")
            return

        body = self.rfile.read(length)
        headers = broker_request_headers(
            self.headers.items(),
            upstream_host=server.upstream_host,
            api_key=server.api_key,
            body_length=len(body),
        )
        try:
            response = server.upstream("POST", upstream_path, headers, body)
        except OSError:
            server.record_broker_decision(
                "POST",
                CODEX_BROKER_RESPONSES_PATH,
                decision="allowed",
                reason="upstream-error",
            )
            self.send_broker_error(502, "Bad Gateway")
            return

        server.record_broker_decision(
            "POST",
            CODEX_BROKER_RESPONSES_PATH,
            decision="allowed",
            reason="upstream-response",
            upstream_status=response.status,
        )
        self.send_response(response.status, response.reason)
        has_content_length = False
        for name, value in response.headers:
            if name.lower() in HOP_BY_HOP_RESPONSE_HEADERS:
                continue
            if name.lower() == "content-length":
                has_content_length = True
            self.send_header(name, value)
        if not has_content_length:
            self.send_header("Content-Length", str(len(response.body)))
        self.end_headers()
        self.wfile.write(response.body)

    def do_GET(self) -> None:
        self.send_method_not_allowed("GET")

    def do_PUT(self) -> None:
        self.send_method_not_allowed("PUT")

    def do_PATCH(self) -> None:
        self.send_method_not_allowed("PATCH")

    def do_DELETE(self) -> None:
        self.send_method_not_allowed("DELETE")

    def send_method_not_allowed(self, method: str) -> None:
        server = cast(CodexApiKeyBrokerProxy, self.server)
        server.record_broker_decision(
            method,
            "<unsupported>",
            decision="denied",
            reason="method-not-allowed",
        )
        self.send_broker_error(405, "Method Not Allowed")

    def log_message(self, format: str, *args: object) -> None:
        return

    def send_broker_error(self, status: int, reason: str) -> None:
        body = f"{status} {reason}\n".encode("ascii")
        self.send_response(status, reason)
        self.send_header("Connection", "close")
        self.send_header("Content-Type", "text/plain; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)


def codex_broker_upstream_path(target: str) -> str:
    parsed = urlsplit(target)
    if parsed.path != CODEX_BROKER_RESPONSES_PATH:
        raise ValueError("Codex API key broker only supports the Responses create path")
    if parsed.query:
        return f"{parsed.path}?{parsed.query}"
    return parsed.path


def parse_content_length(value: str | None) -> int | None:
    if value is None:
        return None
    try:
        length = int(value)
    except ValueError as exc:
        raise ValueError("invalid Content-Length") from exc
    if length < 0:
        raise ValueError("invalid Content-Length")
    return length


def broker_request_headers(
    headers: Sequence[tuple[str, str]],
    *,
    upstream_host: str,
    api_key: str,
    body_length: int,
) -> dict[str, str]:
    forwarded = {
        name: value
        for name, value in headers
        if name.lower() not in HOP_BY_HOP_REQUEST_HEADERS
    }
    forwarded["Host"] = upstream_host
    forwarded["Authorization"] = f"Bearer {api_key}"
    forwarded["Content-Length"] = str(body_length)
    return forwarded


AUTH_BROKER_PROFILES: Mapping[str, AuthBrokerProfile] = MappingProxyType(
    {
        "antigravity": AuthBrokerProfile(
            name="antigravity",
            status=DESIGN_ONLY_AUTH_BROKER_STATUS,
            supported_auth=(
                "runtime auth sources are incomplete",
                "no bundled credential broker is planned until official auth sources are reviewed",
            ),
            host_keeps=(
                "no Antigravity credential is read by RunHaven",
                "no host browser, Keychain, or cloud credential state is imported",
            ),
            guest_receives=(
                "nothing brokered by RunHaven today",
                "only explicitly named --env values or isolated agent state can be visible",
            ),
            current_safe_path=(
                "Use isolated agent state or explicit --env NAME only after reviewing the "
                "provider's current auth requirements."
            ),
            notes=("Antigravity has no bundled provider hosts yet.",),
        ),
        "claude": AuthBrokerProfile(
            name="claude",
            status=DESIGN_ONLY_AUTH_BROKER_STATUS,
            supported_auth=(
                "Claude.ai browser login",
                "Anthropic API key",
                "Claude Code apiKeyHelper script",
                "Bedrock, Vertex, Azure, or Foundry provider auth",
            ),
            host_keeps=(
                "future broker-owned Claude credential material",
                "future broker helper output cache, if a rotating helper is implemented",
            ),
            guest_receives=(
                "nothing brokered by RunHaven today",
                "current runs expose credentials only through isolated agent state or "
                "explicit --env NAME",
            ),
            current_safe_path=(
                "Authenticate inside the isolated Claude state volume when interactive, or pass "
                "ANTHROPIC_API_KEY by name only for a deliberate headless run."
            ),
        ),
        "codex": AuthBrokerProfile(
            name="codex",
            status=CODEX_API_KEY_BROKER_STATUS,
            supported_auth=(
                "OpenAI API key through --codex-api-key-broker-env NAME",
                "ChatGPT browser sign-in",
                "OpenAI API key sign-in",
                "Codex access token from a trusted environment",
            ),
            host_keeps=(
                "host environment variable value named by --codex-api-key-broker-env",
                "the API key is injected only into brokered host requests to api.openai.com",
            ),
            guest_receives=(
                f"{CODEX_BROKER_PLACEHOLDER_ENV} placeholder token value",
                "temporary Codex custom provider config pointing at the broker on the "
                "RunHaven provider network",
            ),
            current_safe_path=(
                "Use --network provider --codex-api-key-broker-env OPENAI_API_KEY for a "
                "headless API-key run, or authenticate inside isolated Codex state when "
                "using browser login."
            ),
            notes=(
                "The prototype supports Codex Responses API requests only.",
                "The raw host API key is never placed in the container command or guest "
                "environment.",
            ),
        ),
        "copilot": AuthBrokerProfile(
            name="copilot",
            status=DESIGN_ONLY_AUTH_BROKER_STATUS,
            supported_auth=(
                "GitHub OAuth device flow",
                "GitHub CLI fallback token",
                "COPILOT_GITHUB_TOKEN, GH_TOKEN, or GITHUB_TOKEN for headless use",
                "BYOK provider environment variables",
            ),
            host_keeps=(
                "future broker-owned Copilot or GitHub token material",
                "future provider-specific BYOK credentials when explicitly configured",
            ),
            guest_receives=(
                "nothing brokered by RunHaven today",
                "current runs expose credentials only through isolated agent state or "
                "explicit --env NAME",
            ),
            current_safe_path=(
                "Use Copilot's login inside isolated state when interactive, or pass "
                "COPILOT_GITHUB_TOKEN by name only after choosing the narrowest token scope."
            ),
            notes=(
                "GitHub and API hosts remain path-sensitive and explicit until a broker can "
                "avoid broad host access.",
            ),
        ),
        "gemini": AuthBrokerProfile(
            name="gemini",
            status=DESIGN_ONLY_AUTH_BROKER_STATUS,
            supported_auth=(
                "Google account login",
                "Gemini API key",
                "Vertex AI Application Default Credentials",
                "Vertex AI service account JSON key",
                "Vertex AI Google Cloud API key",
            ),
            host_keeps=(
                "future broker-owned Gemini API key or Vertex credential material",
                "no service account JSON is copied into the guest by default",
            ),
            guest_receives=(
                "nothing brokered by RunHaven today",
                "current runs expose credentials only through isolated agent state or "
                "explicit --env NAME",
            ),
            current_safe_path=(
                "Use an isolated Gemini login or pass GEMINI_API_KEY by name only; do not mount "
                "Google Cloud ADC or service-account files into the guest by default."
            ),
        ),
        "shell": AuthBrokerProfile(
            name="shell",
            status=DESIGN_ONLY_AUTH_BROKER_STATUS,
            supported_auth=("custom image or command decides its own auth requirements",),
            host_keeps=("no custom-image credential is read by RunHaven",),
            guest_receives=(
                "nothing brokered by RunHaven today",
                "current runs expose credentials only through isolated state or "
                "explicit --env NAME",
            ),
            current_safe_path=(
                "Prefer no credentials; when required, pass the narrowest single variable by "
                "name with --env NAME after reviewing the custom image."
            ),
        ),
    }
)


def auth_broker_profiles() -> tuple[AuthBrokerProfile, ...]:
    return tuple(AUTH_BROKER_PROFILES[name] for name in sorted(AUTH_BROKER_PROFILES))


def get_auth_broker_profile(name: str) -> AuthBrokerProfile:
    try:
        return AUTH_BROKER_PROFILES[name]
    except KeyError as exc:
        known = ", ".join(sorted(AUTH_BROKER_PROFILES))
        raise ValueError(f"unknown auth profile {name!r}; known profiles: {known}") from exc
