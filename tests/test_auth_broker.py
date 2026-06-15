from __future__ import annotations

import json
import threading
import unittest
import urllib.error
import urllib.request

from runhaven.auth_broker import BrokerUpstreamResponse, CodexApiKeyBrokerProxy


class CodexApiKeyBrokerTests(unittest.TestCase):
    def test_broker_injects_host_key_and_strips_guest_authorization(self) -> None:
        calls: list[tuple[str, str, dict[str, str], bytes]] = []

        def upstream(
            method: str, path: str, headers: dict[str, str], body: bytes
        ) -> BrokerUpstreamResponse:
            calls.append((method, path, headers, body))
            return BrokerUpstreamResponse(
                status=200,
                reason="OK",
                headers=(("Content-Type", "application/json"),),
                body=b'{"id":"resp_test"}',
            )

        broker = CodexApiKeyBrokerProxy(
            ("127.0.0.1", 0),
            api_key="host-api-key-value",
            upstream=upstream,
        )
        thread = threading.Thread(target=broker.serve_forever, daemon=True)
        thread.start()
        try:
            request = urllib.request.Request(
                f"http://127.0.0.1:{broker.server_address[1]}/v1/responses",
                data=json.dumps({"model": "gpt-test", "input": "hello"}).encode(),
                headers={
                    "Authorization": "Bearer guest-token",
                    "Content-Type": "application/json",
                },
                method="POST",
            )

            with urllib.request.urlopen(request, timeout=5) as response:
                payload = response.read()
        finally:
            broker.shutdown()
            broker.server_close()
            thread.join(timeout=5)

        self.assertEqual(payload, b'{"id":"resp_test"}')
        self.assertEqual(len(calls), 1)
        method, path, headers, body = calls[0]
        self.assertEqual(method, "POST")
        self.assertEqual(path, "/v1/responses")
        self.assertEqual(headers["Authorization"], "Bearer host-api-key-value")
        self.assertEqual(headers["Host"], "api.openai.com")
        self.assertNotIn("guest-token", "\n".join(headers.values()))
        self.assertIn(b'"input": "hello"', body)
        decisions = broker.broker_decisions()
        self.assertEqual(len(decisions), 1)
        self.assertEqual(decisions[0].method, "POST")
        self.assertEqual(decisions[0].path, "/v1/responses")
        self.assertEqual(decisions[0].decision, "allowed")
        self.assertEqual(decisions[0].reason, "upstream-response")
        self.assertEqual(decisions[0].upstream_status, 200)
        self.assertEqual(decisions[0].count, 1)

    def test_broker_blocks_unsupported_paths(self) -> None:
        calls: list[object] = []

        def upstream(
            method: str, path: str, headers: dict[str, str], body: bytes
        ) -> BrokerUpstreamResponse:
            calls.append((method, path, headers, body))
            return BrokerUpstreamResponse(status=200, reason="OK", headers=(), body=b"{}")

        broker = CodexApiKeyBrokerProxy(
            ("127.0.0.1", 0),
            api_key="host-api-key-value",
            upstream=upstream,
        )
        thread = threading.Thread(target=broker.serve_forever, daemon=True)
        thread.start()
        try:
            request = urllib.request.Request(
                f"http://127.0.0.1:{broker.server_address[1]}/v1/models",
                data=b"{}",
                method="POST",
            )
            with self.assertRaises(urllib.error.HTTPError) as error:
                urllib.request.urlopen(request, timeout=5)
        finally:
            broker.shutdown()
            broker.server_close()
            thread.join(timeout=5)

        self.assertEqual(error.exception.code, 403)
        error.exception.close()
        self.assertEqual(calls, [])
        decisions = broker.broker_decisions()
        self.assertEqual(len(decisions), 1)
        self.assertEqual(decisions[0].method, "POST")
        self.assertEqual(decisions[0].path, "<unsupported>")
        self.assertEqual(decisions[0].decision, "denied")
        self.assertEqual(decisions[0].reason, "unsupported-path")
        self.assertIsNone(decisions[0].upstream_status)

    def test_broker_restricts_client_subnets(self) -> None:
        broker = CodexApiKeyBrokerProxy(
            ("127.0.0.1", 0),
            api_key="host-api-key-value",
            allowed_client_subnets=("192.0.2.0/24",),
        )
        thread = threading.Thread(target=broker.serve_forever, daemon=True)
        thread.start()
        try:
            request = urllib.request.Request(
                f"http://127.0.0.1:{broker.server_address[1]}/v1/responses",
                data=b"{}",
                method="POST",
            )
            with self.assertRaises(urllib.error.HTTPError) as error:
                urllib.request.urlopen(request, timeout=5)
        finally:
            broker.shutdown()
            broker.server_close()
            thread.join(timeout=5)

        self.assertEqual(error.exception.code, 403)
        error.exception.close()
        decisions = broker.broker_decisions()
        self.assertEqual(len(decisions), 1)
        self.assertEqual(decisions[0].decision, "denied")
        self.assertEqual(decisions[0].reason, "client-not-allowed")


if __name__ == "__main__":
    unittest.main()
