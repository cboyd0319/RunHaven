from __future__ import annotations

import socket
import threading
import unittest

from runhaven.egress import (
    MAX_DENIED_CONNECT_TARGETS,
    EgressPolicy,
    ProxyDecision,
    ThreadedAllowlistProxy,
)


class EgressPolicyTests(unittest.TestCase):
    def test_policy_rejects_empty_allowlist(self) -> None:
        with self.assertRaisesRegex(ValueError, "at least one allowed host"):
            EgressPolicy(allowed_hosts=())

    def test_policy_allows_exact_hosts_and_subdomains(self) -> None:
        policy = EgressPolicy(allowed_hosts=("api.example.com",))

        self.assertTrue(policy.allows("api.example.com", 443))
        self.assertTrue(policy.allows("uploads.api.example.com", 443))
        self.assertFalse(policy.allows("example.com", 443))

    def test_policy_rejects_ip_literals_by_default(self) -> None:
        policy = EgressPolicy(allowed_hosts=("example.com",))

        self.assertFalse(policy.allows("93.184.216.34", 443))
        self.assertFalse(policy.allows("2606:2800:220:1:248:1893:25c8:1946", 443))

    def test_policy_restricts_ports(self) -> None:
        policy = EgressPolicy(allowed_hosts=("api.example.com",), allowed_ports=(443,))

        self.assertTrue(policy.allows("api.example.com", 443))
        self.assertFalse(policy.allows("api.example.com", 80))


class AllowlistProxyTests(unittest.TestCase):
    def test_proxy_rejects_disallowed_connect_host(self) -> None:
        with running_proxy(EgressPolicy(allowed_hosts=("allowed.test",))) as proxy:
            response = proxy_request(
                proxy,
                b"CONNECT denied.test:443 HTTP/1.1\r\nHost: denied.test:443\r\n\r\n",
            )

        self.assertIn(b"403 Forbidden", response)

    def test_proxy_records_disallowed_connect_targets(self) -> None:
        with running_proxy(EgressPolicy(allowed_hosts=("allowed.test",))) as proxy:
            proxy_request(
                proxy,
                b"CONNECT denied.test:443 HTTP/1.1\r\nHost: denied.test:443\r\n\r\n",
            )
            proxy_request(
                proxy,
                b"CONNECT denied.test:443 HTTP/1.1\r\nHost: denied.test:443\r\n\r\n",
            )
            proxy_request(
                proxy,
                b"CONNECT other.test:443 HTTP/1.1\r\nHost: other.test:443\r\n\r\n",
            )

            denied_targets = proxy.denied_connect_targets()

        self.assertEqual(denied_targets, (("denied.test", 443), ("other.test", 443)))

    def test_proxy_caps_recorded_disallowed_connect_targets(self) -> None:
        with running_proxy(EgressPolicy(allowed_hosts=("allowed.test",))) as proxy:
            for index in range(MAX_DENIED_CONNECT_TARGETS + 2):
                proxy.record_denied_connect_target(f"denied-{index}.test", 443)

            denied_targets = proxy.denied_connect_targets()

        self.assertEqual(len(denied_targets), MAX_DENIED_CONNECT_TARGETS)
        self.assertEqual(denied_targets[-1], (f"denied-{MAX_DENIED_CONNECT_TARGETS - 1}.test", 443))

    def test_proxy_rejects_plain_http_requests(self) -> None:
        with running_proxy(EgressPolicy(allowed_hosts=("allowed.test",))) as proxy:
            response = proxy_request(
                proxy,
                b"GET http://allowed.test/ HTTP/1.1\r\nHost: allowed.test\r\n\r\n",
            )

        self.assertIn(b"405 Method Not Allowed", response)

    def test_proxy_rejects_clients_outside_allowed_subnets(self) -> None:
        with running_proxy(
            EgressPolicy(allowed_hosts=("allowed.test",)),
            allowed_client_subnets=("192.0.2.0/24",),
        ) as proxy:
            response = proxy_request(
                proxy,
                b"CONNECT allowed.test:443 HTTP/1.1\r\nHost: allowed.test:443\r\n\r\n",
            )

        self.assertIn(b"403 Forbidden", response)

    def test_proxy_rejects_allowed_host_resolving_to_private_address(self) -> None:
        resolver = fake_resolver(("127.0.0.1",))
        with running_proxy(
            EgressPolicy(allowed_hosts=("allowed.test",)),
            resolver=resolver,
        ) as proxy:
            response = proxy_request(
                proxy,
                b"CONNECT allowed.test:443 HTTP/1.1\r\nHost: allowed.test:443\r\n\r\n",
            )

            decisions = proxy.policy_decisions()

        self.assertIn(b"403 Forbidden", response)
        self.assertEqual(
            decisions,
            (
                ProxyDecision(
                    host="allowed.test",
                    port=443,
                    decision="denied",
                    reason="unsafe-resolved-address",
                    matched_rule="allowed.test",
                    count=1,
                ),
            ),
        )

    def test_proxy_records_allowed_and_denied_policy_decisions(self) -> None:
        connector = fake_echo_connector()
        with running_proxy(
            EgressPolicy(allowed_hosts=("allowed.test",)),
            resolver=fake_resolver(("93.184.216.34",)),
            connector=connector,
        ) as proxy:
            with socket.create_connection(proxy.server_address, timeout=3) as client:
                client.sendall(
                    b"CONNECT allowed.test:443 HTTP/1.1\r\nHost: allowed.test:443\r\n\r\n"
                )
                self.assertIn(b"200 Connection Established", client.recv(4096))
                client.sendall(b"ping")
                self.assertEqual(client.recv(4), b"pong")

            proxy_request(
                proxy,
                b"CONNECT denied.test:443 HTTP/1.1\r\nHost: denied.test:443\r\n\r\n",
            )
            proxy_request(
                proxy,
                b"CONNECT denied.test:443 HTTP/1.1\r\nHost: denied.test:443\r\n\r\n",
            )

            decisions = proxy.policy_decisions()

        self.assertEqual(
            decisions,
            (
                ProxyDecision(
                    host="allowed.test",
                    port=443,
                    decision="allowed",
                    reason="allowed",
                    matched_rule="allowed.test",
                    count=1,
                ),
                ProxyDecision(
                    host="denied.test",
                    port=443,
                    decision="denied",
                    reason="not-in-allowlist",
                    matched_rule="",
                    count=2,
                ),
            ),
        )


class running_proxy:
    def __init__(
        self,
        policy: EgressPolicy,
        *,
        allowed_client_subnets: tuple[str, ...] = (),
        resolver: object | None = None,
        connector: object | None = None,
    ) -> None:
        self.server = ThreadedAllowlistProxy(
            ("127.0.0.1", 0),
            policy,
            allowed_client_subnets=allowed_client_subnets,
            resolver=resolver,
            connector=connector,
        )
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)

    def __enter__(self) -> ThreadedAllowlistProxy:
        self.thread.start()
        return self.server

    def __exit__(self, *args: object) -> None:
        self.server.shutdown()
        self.server.server_close()
        self.thread.join(timeout=3)


def proxy_request(proxy: ThreadedAllowlistProxy, request: bytes) -> bytes:
    with socket.create_connection(proxy.server_address, timeout=3) as client:
        client.sendall(request)
        return client.recv(4096)


def fake_resolver(addresses: tuple[str, ...]) -> object:
    def resolve(
        _host: str,
        port: int,
        *,
        type: socket.SocketKind = socket.SOCK_STREAM,
    ) -> list[tuple[socket.AddressFamily, socket.SocketKind, int, str, tuple[str, int]]]:
        return [
            (socket.AF_INET, type, socket.IPPROTO_TCP, "", (address, port))
            for address in addresses
        ]

    return resolve


class fake_echo_connector:
    def __call__(
        self,
        _addrinfos: object,
        _timeout: float,
    ) -> socket.socket:
        client, server = socket.socketpair()
        thread = threading.Thread(target=self._serve, args=(server,), daemon=True)
        thread.start()
        return client

    def _serve(self, connection: socket.socket) -> None:
        with connection:
            data = connection.recv(4)
            if data == b"ping":
                connection.sendall(b"pong")


if __name__ == "__main__":
    unittest.main()
