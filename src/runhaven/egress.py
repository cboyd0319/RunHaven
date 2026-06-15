from __future__ import annotations

import ipaddress
import select
import socket
import socketserver
import threading
from collections.abc import Callable, Sequence
from dataclasses import dataclass
from typing import Literal, Protocol, cast

MAX_HEADER_BYTES = 64 * 1024
RELAY_BUFFER_BYTES = 64 * 1024
MAX_DENIED_CONNECT_TARGETS = 50

SocketAddress = tuple[object, ...]
AddressInfo = tuple[socket.AddressFamily, socket.SocketKind, int, str, SocketAddress]


class Resolver(Protocol):
    def __call__(
        self,
        host: str,
        port: int,
        *,
        type: socket.SocketKind = socket.SOCK_STREAM,
    ) -> Sequence[AddressInfo]: ...


Connector = Callable[[Sequence[AddressInfo], float], socket.socket]


class UnsafeResolvedAddress(ValueError):
    def __init__(self, address: str) -> None:
        super().__init__(f"unsafe resolved address: {address}")
        self.address = address


@dataclass(frozen=True)
class ProxyDecision:
    host: str
    port: int
    decision: Literal["allowed", "denied"]
    reason: str
    matched_rule: str
    count: int = 1


@dataclass(frozen=True)
class EgressPolicy:
    allowed_hosts: Sequence[str]
    allowed_ports: Sequence[int] = (443,)

    def __post_init__(self) -> None:
        hosts = tuple(normalize_host(host) for host in self.allowed_hosts)
        ports = tuple(int(port) for port in self.allowed_ports)
        if not hosts:
            raise ValueError("provider egress policy needs at least one allowed host")
        if any(port < 1 or port > 65535 for port in ports):
            raise ValueError("provider egress policy ports must be between 1 and 65535")
        object.__setattr__(self, "allowed_hosts", hosts)
        object.__setattr__(self, "allowed_ports", ports)

    def allows(self, host: str, port: int) -> bool:
        return self.match_rule(host, port) is not None

    def match_rule(self, host: str, port: int) -> str | None:
        try:
            normalized = normalize_host(host)
        except ValueError:
            return None
        if port not in self.allowed_ports:
            return None
        if is_ip_literal(normalized):
            return None
        for allowed in self.allowed_hosts:
            if normalized == allowed or normalized.endswith(f".{allowed}"):
                return allowed
        return None

    def denial_reason(self, host: str, port: int) -> str:
        try:
            normalized = normalize_host(host)
        except ValueError:
            return "invalid-host"
        if port not in self.allowed_ports:
            return "port-not-allowed"
        if is_ip_literal(normalized):
            return "ip-literal"
        return "not-in-allowlist"


class ThreadedAllowlistProxy(socketserver.ThreadingTCPServer):
    allow_reuse_address = True
    daemon_threads = True

    def __init__(
        self,
        server_address: tuple[str, int],
        policy: EgressPolicy,
        *,
        connect_timeout: float = 10.0,
        relay_timeout: float = 30.0,
        allowed_client_subnets: Sequence[str] = (),
        resolver: Resolver | None = None,
        connector: Connector | None = None,
    ) -> None:
        self.policy = policy
        self.connect_timeout = connect_timeout
        self.relay_timeout = relay_timeout
        self.resolver = resolver or default_resolver
        self.connector = connector or default_connector
        self.allowed_client_networks = tuple(
            ipaddress.ip_network(subnet, strict=False) for subnet in allowed_client_subnets
        )
        self._denied_connect_targets: list[tuple[str, int]] = []
        self._denied_connect_target_set: set[tuple[str, int]] = set()
        self._denied_connect_targets_lock = threading.Lock()
        self._policy_decisions: dict[tuple[str, int, str, str, str], int] = {}
        self._policy_decisions_lock = threading.Lock()
        super().__init__(server_address, AllowlistProxyHandler)

    def allows_client(self, address: str) -> bool:
        if not self.allowed_client_networks:
            return True
        try:
            client_address = ipaddress.ip_address(address)
        except ValueError:
            return False
        return any(client_address in network for network in self.allowed_client_networks)

    def record_denied_connect_target(self, host: str, port: int) -> None:
        target = (host, port)
        with self._denied_connect_targets_lock:
            if target not in self._denied_connect_target_set:
                if len(self._denied_connect_targets) >= MAX_DENIED_CONNECT_TARGETS:
                    return
                self._denied_connect_targets.append(target)
                self._denied_connect_target_set.add(target)

    def denied_connect_targets(self) -> tuple[tuple[str, int], ...]:
        with self._denied_connect_targets_lock:
            return tuple(self._denied_connect_targets)

    def record_policy_decision(
        self,
        host: str,
        port: int,
        *,
        decision: Literal["allowed", "denied"],
        reason: str,
        matched_rule: str = "",
    ) -> None:
        key = (host, port, decision, reason, matched_rule)
        with self._policy_decisions_lock:
            self._policy_decisions[key] = self._policy_decisions.get(key, 0) + 1

    def policy_decisions(self) -> tuple[ProxyDecision, ...]:
        with self._policy_decisions_lock:
            return tuple(
                ProxyDecision(
                    host=host,
                    port=port,
                    decision=cast(Literal["allowed", "denied"], decision),
                    reason=reason,
                    matched_rule=matched_rule,
                    count=count,
                )
                for (host, port, decision, reason, matched_rule), count in (
                    self._policy_decisions.items()
                )
            )


class AllowlistProxyHandler(socketserver.StreamRequestHandler):
    def handle(self) -> None:
        server = cast(ThreadedAllowlistProxy, self.server)
        self.connection.settimeout(server.connect_timeout)
        if not server.allows_client(self.client_address[0]):
            self.send_response(403, "Forbidden")
            return

        request_line = self.rfile.readline(MAX_HEADER_BYTES + 1)
        if not request_line or len(request_line) > MAX_HEADER_BYTES:
            self.send_response(400, "Bad Request")
            return
        try:
            method, target, _version = request_line.decode("ascii").strip().split(maxsplit=2)
        except ValueError:
            self.send_response(400, "Bad Request")
            return

        if not self.discard_headers():
            self.send_response(400, "Bad Request")
            return
        if method.upper() != "CONNECT":
            self.send_response(405, "Method Not Allowed")
            return

        try:
            host, port = parse_connect_target(target)
        except ValueError:
            self.send_response(400, "Bad Request")
            return
        matched_rule = server.policy.match_rule(host, port)
        if matched_rule is None:
            reason = server.policy.denial_reason(host, port)
            server.record_policy_decision(host, port, decision="denied", reason=reason)
            server.record_denied_connect_target(host, port)
            self.send_response(403, "Forbidden")
            return

        try:
            addrinfos = safe_addrinfos(host, port, server.resolver)
        except UnsafeResolvedAddress:
            server.record_policy_decision(
                host,
                port,
                decision="denied",
                reason="unsafe-resolved-address",
                matched_rule=matched_rule,
            )
            self.send_response(403, "Forbidden")
            return
        except OSError:
            server.record_policy_decision(
                host,
                port,
                decision="denied",
                reason="dns-resolution-failed",
                matched_rule=matched_rule,
            )
            self.send_response(502, "Bad Gateway")
            return

        try:
            upstream = server.connector(addrinfos, server.connect_timeout)
        except OSError:
            server.record_policy_decision(
                host,
                port,
                decision="allowed",
                reason="allowed",
                matched_rule=matched_rule,
            )
            self.send_response(502, "Bad Gateway")
            return

        server.record_policy_decision(
            host,
            port,
            decision="allowed",
            reason="allowed",
            matched_rule=matched_rule,
        )
        with upstream:
            self.send_raw_response(200, "Connection Established")
            relay(self.connection, upstream, timeout=server.relay_timeout)

    def discard_headers(self) -> bool:
        consumed = 0
        while True:
            line = self.rfile.readline(MAX_HEADER_BYTES + 1)
            consumed += len(line)
            if consumed > MAX_HEADER_BYTES:
                return False
            if line in (b"\r\n", b"\n", b""):
                return True

    def send_response(self, status: int, reason: str) -> None:
        body = f"{status} {reason}\n".encode("ascii")
        self.wfile.write(
            (
                f"HTTP/1.1 {status} {reason}\r\n"
                "Connection: close\r\n"
                "Content-Type: text/plain\r\n"
                f"Content-Length: {len(body)}\r\n"
                "\r\n"
            ).encode("ascii")
        )
        self.wfile.write(body)

    def send_raw_response(self, status: int, reason: str) -> None:
        self.wfile.write(f"HTTP/1.1 {status} {reason}\r\n\r\n".encode("ascii"))
        self.wfile.flush()


def normalize_host(host: str) -> str:
    value = host.strip().lower().rstrip(".")
    if value.startswith("[") and value.endswith("]"):
        value = value[1:-1]
    if not value or any(character.isspace() or character in "/\\" for character in value):
        raise ValueError(f"invalid host: {host!r}")
    try:
        return str(ipaddress.ip_address(value))
    except ValueError:
        pass
    if ":" in value:
        raise ValueError(f"invalid host: {host!r}")
    try:
        ascii_host = value.encode("idna").decode("ascii")
    except UnicodeError as exc:
        raise ValueError(f"invalid host: {host!r}") from exc
    if not ascii_host or ascii_host.startswith("-") or ascii_host.endswith("-"):
        raise ValueError(f"invalid host: {host!r}")
    return ascii_host


def is_ip_literal(host: str) -> bool:
    try:
        ipaddress.ip_address(host)
    except ValueError:
        return False
    return True


def default_resolver(
    host: str,
    port: int,
    *,
    type: socket.SocketKind = socket.SOCK_STREAM,
) -> Sequence[AddressInfo]:
    return cast(
        Sequence[AddressInfo],
        socket.getaddrinfo(host, port, type=type),
    )


def safe_addrinfos(host: str, port: int, resolver: Resolver) -> tuple[AddressInfo, ...]:
    addrinfos = tuple(resolver(host, port, type=socket.SOCK_STREAM))
    if not addrinfos:
        raise OSError(f"no addresses resolved for {host}:{port}")
    for _family, _socktype, _proto, _canonname, sockaddr in addrinfos:
        address = str(sockaddr[0])
        if not is_safe_upstream_address(address):
            raise UnsafeResolvedAddress(address)
    return addrinfos


def is_safe_upstream_address(address: str) -> bool:
    try:
        parsed = ipaddress.ip_address(address)
    except ValueError:
        return False
    if isinstance(parsed, ipaddress.IPv6Address) and parsed.ipv4_mapped is not None:
        parsed = parsed.ipv4_mapped
    return parsed.is_global


def default_connector(addrinfos: Sequence[AddressInfo], timeout: float) -> socket.socket:
    last_error: OSError | None = None
    for family, socktype, proto, _canonname, sockaddr in addrinfos:
        upstream = socket.socket(family, socktype, proto)
        try:
            upstream.settimeout(timeout)
            upstream.connect(sockaddr)
            return upstream
        except OSError as exc:
            upstream.close()
            last_error = exc
    if last_error is not None:
        raise last_error
    raise OSError("no upstream addresses to connect")


def parse_connect_target(target: str) -> tuple[str, int]:
    if target.startswith("["):
        host, separator, port_text = target.rpartition("]:")
        if not separator:
            raise ValueError(f"invalid CONNECT target: {target!r}")
        host = f"{host}]"
    else:
        host, separator, port_text = target.rpartition(":")
        if not separator:
            raise ValueError(f"invalid CONNECT target: {target!r}")
    try:
        port = int(port_text)
    except ValueError as exc:
        raise ValueError(f"invalid CONNECT target port: {target!r}") from exc
    if port < 1 or port > 65535:
        raise ValueError(f"invalid CONNECT target port: {target!r}")
    return normalize_host(host), port


def relay(client: socket.socket, upstream: socket.socket, *, timeout: float) -> None:
    sockets = [client, upstream]
    for item in sockets:
        item.setblocking(False)
    while sockets:
        readable, _, _ = select.select(sockets, (), (), timeout)
        if not readable:
            return
        for source in readable:
            try:
                data = source.recv(RELAY_BUFFER_BYTES)
            except OSError:
                return
            if not data:
                return
            destination = upstream if source is client else client
            try:
                destination.sendall(data)
            except OSError:
                return
