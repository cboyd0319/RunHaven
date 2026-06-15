from __future__ import annotations

import ipaddress
import select
import socket
import socketserver
import threading
from collections.abc import Sequence
from dataclasses import dataclass
from typing import cast

MAX_HEADER_BYTES = 64 * 1024
RELAY_BUFFER_BYTES = 64 * 1024
MAX_DENIED_CONNECT_TARGETS = 50


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
        try:
            normalized = normalize_host(host)
        except ValueError:
            return False
        if port not in self.allowed_ports:
            return False
        if is_ip_literal(normalized):
            return False
        return any(
            normalized == allowed or normalized.endswith(f".{allowed}")
            for allowed in self.allowed_hosts
        )


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
    ) -> None:
        self.policy = policy
        self.connect_timeout = connect_timeout
        self.relay_timeout = relay_timeout
        self.allowed_client_networks = tuple(
            ipaddress.ip_network(subnet, strict=False) for subnet in allowed_client_subnets
        )
        self._denied_connect_targets: list[tuple[str, int]] = []
        self._denied_connect_target_set: set[tuple[str, int]] = set()
        self._denied_connect_targets_lock = threading.Lock()
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
        if not server.policy.allows(host, port):
            server.record_denied_connect_target(host, port)
            self.send_response(403, "Forbidden")
            return

        try:
            upstream = socket.create_connection((host, port), timeout=server.connect_timeout)
        except OSError:
            self.send_response(502, "Bad Gateway")
            return

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
