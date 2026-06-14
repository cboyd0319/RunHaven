from __future__ import annotations

import socket
import threading
import unittest
from contextlib import closing

from runhaven.egress import EgressPolicy, ThreadedAllowlistProxy


class EgressPolicyTests(unittest.TestCase):
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

    def test_proxy_relays_allowed_connect_tunnel(self) -> None:
        with running_echo_server() as upstream:
            policy = EgressPolicy(allowed_hosts=("localhost",), allowed_ports=(upstream.port,))
            with running_proxy(policy) as proxy:
                with socket.create_connection(proxy.server_address, timeout=3) as client:
                    request = (
                        f"CONNECT localhost:{upstream.port} HTTP/1.1\r\n"
                        f"Host: localhost:{upstream.port}\r\n\r\n"
                    )
                    client.sendall(request.encode("ascii"))
                    self.assertIn(b"200 Connection Established", client.recv(4096))
                    client.sendall(b"ping")
                    self.assertEqual(client.recv(4), b"pong")


class running_proxy:
    def __init__(
        self,
        policy: EgressPolicy,
        *,
        allowed_client_subnets: tuple[str, ...] = (),
    ) -> None:
        self.server = ThreadedAllowlistProxy(
            ("127.0.0.1", 0),
            policy,
            allowed_client_subnets=allowed_client_subnets,
        )
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)

    def __enter__(self) -> ThreadedAllowlistProxy:
        self.thread.start()
        return self.server

    def __exit__(self, *args: object) -> None:
        self.server.shutdown()
        self.server.server_close()
        self.thread.join(timeout=3)


class running_echo_server:
    def __init__(self) -> None:
        self.listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.listener.bind(("127.0.0.1", 0))
        self.listener.listen(1)
        self.port = self.listener.getsockname()[1]
        self.thread = threading.Thread(target=self._serve, daemon=True)

    def __enter__(self) -> running_echo_server:
        self.thread.start()
        return self

    def __exit__(self, *args: object) -> None:
        with closing(socket.socket(socket.AF_INET, socket.SOCK_STREAM)) as stopper:
            stopper.settimeout(1)
            try:
                stopper.connect(("127.0.0.1", self.port))
            except OSError:
                pass
        self.listener.close()
        self.thread.join(timeout=3)

    def _serve(self) -> None:
        try:
            connection, _ = self.listener.accept()
        except OSError:
            return
        with connection:
            data = connection.recv(4)
            if data == b"ping":
                connection.sendall(b"pong")


def proxy_request(proxy: ThreadedAllowlistProxy, request: bytes) -> bytes:
    with socket.create_connection(proxy.server_address, timeout=3) as client:
        client.sendall(request)
        return client.recv(4096)


if __name__ == "__main__":
    unittest.main()
