from __future__ import annotations

import argparse
import platform
import subprocess
import sys
import threading
import uuid
from collections.abc import Sequence
from dataclasses import dataclass
from urllib.parse import urlparse

from runhaven.egress import EgressPolicy, ThreadedAllowlistProxy

DEFAULT_IMAGE = "runhaven/base:0.1.0"
DEFAULT_ALLOWED_HOST = "example.com"
DEFAULT_DENIED_HOST = "iana.org"
DEFAULT_DIRECT_IP_URL = "http://1.1.1.1/"


class SmokeFailure(RuntimeError):
    pass


@dataclass(frozen=True)
class CommandResult:
    args: tuple[str, ...]
    returncode: int
    stdout: str
    stderr: str


def main(argv: Sequence[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    try:
        run_smoke(args)
    except SmokeFailure as exc:
        print(f"FAIL {exc}", file=sys.stderr)
        return 1
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description=(
            "Prove the RunHaven provider-egress proxy pattern with Apple container. "
            "Requires a local image with curl and python3."
        )
    )
    parser.add_argument(
        "--image",
        default=DEFAULT_IMAGE,
        help="container image with curl and python3",
    )
    parser.add_argument("--allowed-host", default=DEFAULT_ALLOWED_HOST)
    parser.add_argument("--denied-host", default=DEFAULT_DENIED_HOST)
    parser.add_argument("--allowed-url", help="HTTPS URL on the allowed host")
    parser.add_argument("--direct-ip-url", default=DEFAULT_DIRECT_IP_URL)
    parser.add_argument("--timeout", type=int, default=8)
    parser.add_argument("--network-name", help="override the temporary internal network name")
    return parser


def run_smoke(args: argparse.Namespace) -> None:
    if platform.system() != "Darwin":
        raise SmokeFailure("provider egress smoke requires macOS")

    network_name = args.network_name or f"runhaven-egress-smoke-{uuid.uuid4().hex[:12]}"
    allowed_url = args.allowed_url or f"https://{args.allowed_host}/"
    allowed_host = url_host(allowed_url)
    if allowed_host != args.allowed_host:
        raise SmokeFailure("--allowed-url host must match --allowed-host")

    run_checked(("container", "network", "create", "--internal", network_name), args.timeout)
    try:
        gateway = discover_gateway(network_name, args.image, args.timeout)
        print(f"PASS discovered internal-network gateway {gateway}")

        policy = EgressPolicy(allowed_hosts=(args.allowed_host,))
        proxy, bind_note = create_proxy(gateway, policy, args.timeout)
        thread = threading.Thread(target=proxy.serve_forever, daemon=True)
        thread.start()
        try:
            proxy_url = f"http://{gateway}:{proxy.server_address[1]}"
            print(f"PASS started allowlist proxy on {proxy_url} ({bind_note})")
            assert_allowed_proxy_path(
                network_name,
                args.image,
                proxy_url,
                allowed_url,
                args.timeout,
            )
            assert_blocked_proxy_path(
                network_name,
                args.image,
                proxy_url,
                f"https://{args.denied_host}/",
                "denied host",
                args.timeout,
            )
            assert_blocked_proxy_path(
                network_name,
                args.image,
                proxy_url,
                "https://1.1.1.1/",
                "proxied IP literal",
                args.timeout,
            )
            assert_direct_path_blocked(
                network_name,
                args.image,
                allowed_url,
                "direct DNS path",
                args.timeout,
            )
            assert_direct_path_blocked(
                network_name,
                args.image,
                args.direct_ip_url,
                "direct IP path",
                args.timeout,
            )
        finally:
            proxy.shutdown()
            proxy.server_close()
            thread.join(timeout=args.timeout)
    finally:
        run_command(("container", "network", "delete", network_name), args.timeout)


def create_proxy(
    gateway: str,
    policy: EgressPolicy,
    timeout: int,
) -> tuple[ThreadedAllowlistProxy, str]:
    try:
        return (
            ThreadedAllowlistProxy(
                (gateway, 0),
                policy,
                connect_timeout=float(timeout),
                relay_timeout=float(timeout),
            ),
            "bound to gateway",
        )
    except OSError:
        return (
            ThreadedAllowlistProxy(
                ("0.0.0.0", 0),
                policy,
                connect_timeout=float(timeout),
                relay_timeout=float(timeout),
                allowed_client_subnets=(f"{gateway}/24",),
            ),
            f"bound to all interfaces, restricted to {gateway}/24 clients",
        )


def discover_gateway(network_name: str, image: str, timeout: int) -> str:
    code = (
        "from pathlib import Path\n"
        "for line in Path('/proc/net/route').read_text().splitlines()[1:]:\n"
        "    fields = line.split()\n"
        "    if fields[1] == '00000000':\n"
        "        raw = bytes.fromhex(fields[2])\n"
        "        print('.'.join(str(value) for value in raw[::-1]))\n"
        "        raise SystemExit(0)\n"
        "raise SystemExit('missing default route')\n"
    )
    result = run_checked(
        (
            "container",
            "run",
            "--rm",
            "--network",
            network_name,
            image,
            "python3",
            "-c",
            code,
        ),
        timeout,
    )
    gateway = result.stdout.strip().splitlines()[-1]
    if not gateway:
        raise SmokeFailure("could not discover internal-network gateway")
    return gateway


def assert_allowed_proxy_path(
    network_name: str,
    image: str,
    proxy_url: str,
    url: str,
    timeout: int,
) -> None:
    result = curl(network_name, image, url, timeout, proxy_url=proxy_url)
    if result.returncode != 0:
        raise SmokeFailure(f"allowed proxied path failed: {summarize(result)}")
    print(f"PASS allowed proxied HTTPS path reached {url_host(url)}")


def assert_blocked_proxy_path(
    network_name: str,
    image: str,
    proxy_url: str,
    url: str,
    label: str,
    timeout: int,
) -> None:
    result = curl(network_name, image, url, timeout, proxy_url=proxy_url)
    if result.returncode == 0:
        raise SmokeFailure(f"{label} unexpectedly succeeded through proxy")
    print(f"PASS blocked {label} through proxy")


def assert_direct_path_blocked(
    network_name: str,
    image: str,
    url: str,
    label: str,
    timeout: int,
) -> None:
    result = curl(network_name, image, url, timeout)
    if result.returncode == 0:
        raise SmokeFailure(f"{label} unexpectedly succeeded without proxy")
    print(f"PASS blocked {label} without proxy")


def curl(
    network_name: str,
    image: str,
    url: str,
    timeout: int,
    *,
    proxy_url: str | None = None,
) -> CommandResult:
    command = ["container", "run", "--rm", "--network", network_name]
    if proxy_url is not None:
        command.extend(
            (
                "--env",
                f"HTTPS_PROXY={proxy_url}",
                "--env",
                f"HTTP_PROXY={proxy_url}",
                "--env",
                f"ALL_PROXY={proxy_url}",
            )
        )
    command.extend(
        (
            image,
            "curl",
            "-sS",
            "-o",
            "/dev/null",
            "--connect-timeout",
            str(timeout),
            "--max-time",
            str(timeout),
            url,
        )
    )
    return run_command(tuple(command), timeout + 15)


def run_checked(command: tuple[str, ...], timeout: int) -> CommandResult:
    result = run_command(command, timeout)
    if result.returncode != 0:
        raise SmokeFailure(f"command failed: {summarize(result)}")
    return result


def run_command(command: tuple[str, ...], timeout: int) -> CommandResult:
    completed = subprocess.run(
        command,
        check=False,
        capture_output=True,
        text=True,
        timeout=timeout,
    )
    return CommandResult(
        args=command,
        returncode=completed.returncode,
        stdout=completed.stdout,
        stderr=completed.stderr,
    )


def summarize(result: CommandResult) -> str:
    output = "\n".join(part.strip() for part in (result.stdout, result.stderr) if part.strip())
    if len(output) > 500:
        output = f"{output[:500]}..."
    return f"exit {result.returncode} from {result.args[0]!r}: {output}"


def url_host(url: str) -> str:
    parsed = urlparse(url)
    if parsed.scheme != "https":
        raise SmokeFailure("smoke URLs must use https except --direct-ip-url")
    if not parsed.hostname:
        raise SmokeFailure(f"URL has no host: {url}")
    return parsed.hostname.lower()


if __name__ == "__main__":
    raise SystemExit(main())
