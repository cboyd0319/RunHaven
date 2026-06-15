from __future__ import annotations

import unittest

from scripts.provider_egress_smoke import (
    DEFAULT_ALLOWED_HOST,
    SmokeFailure,
    allowed_hosts_for_args,
    allowed_urls_for_args,
    build_parser,
)


class ProviderEgressSmokeTests(unittest.TestCase):
    def test_default_allowed_host_is_used_without_profile(self) -> None:
        args = build_parser().parse_args([])

        hosts = allowed_hosts_for_args(args)
        urls = allowed_urls_for_args(args, hosts)

        self.assertEqual(hosts, (DEFAULT_ALLOWED_HOST,))
        self.assertEqual(urls, (f"https://{DEFAULT_ALLOWED_HOST}/",))

    def test_agent_profile_uses_bundled_provider_hosts(self) -> None:
        args = build_parser().parse_args(["--agent", "codex"])

        hosts = allowed_hosts_for_args(args)
        urls = allowed_urls_for_args(args, hosts)

        self.assertIn("api.openai.com", hosts)
        self.assertIn("chatgpt.com", hosts)
        self.assertIn("https://api.openai.com/", urls)
        self.assertIn("https://chatgpt.com/", urls)

    def test_profile_without_provider_hosts_fails_closed(self) -> None:
        args = build_parser().parse_args(["--agent", "antigravity"])

        with self.assertRaisesRegex(SmokeFailure, "no bundled provider hosts"):
            allowed_hosts_for_args(args)

    def test_allowed_url_must_match_allowed_host_count(self) -> None:
        args = build_parser().parse_args(
            ["--allowed-host", "api.example.com", "--allowed-host", "auth.example.com"]
        )
        hosts = allowed_hosts_for_args(args)
        args.allowed_url = ["https://api.example.com/"]

        with self.assertRaisesRegex(SmokeFailure, "count must match"):
            allowed_urls_for_args(args, hosts)

    def test_allowed_url_host_must_be_allowed(self) -> None:
        args = build_parser().parse_args(
            ["--allowed-host", "api.example.com", "--allowed-url", "https://other.example.com/"]
        )
        hosts = allowed_hosts_for_args(args)

        with self.assertRaisesRegex(SmokeFailure, "host must match"):
            allowed_urls_for_args(args, hosts)


if __name__ == "__main__":
    unittest.main()
