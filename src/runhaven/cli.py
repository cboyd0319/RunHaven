from __future__ import annotations

import argparse
import fcntl
import json
import os
import shutil
import subprocess
import sys
import threading
from collections.abc import Iterator, Sequence
from contextlib import contextmanager
from dataclasses import dataclass
from pathlib import Path
from typing import TextIO

from .doctor import collect_checks
from .egress import EgressPolicy, ThreadedAllowlistProxy, is_ip_literal
from .images import build_image_plan
from .plans import SUPPORTED_NETWORK_MODES, AgentRunPlan, RunOptions, build_run_plan
from .profiles import PROFILES, get_profile


def main(argv: Sequence[str] | None = None) -> int:
    raw_args = list(sys.argv[1:] if argv is None else argv)
    parse_args, agent_args = split_agent_args(raw_args)
    parser = build_parser()
    args = parser.parse_args(parse_args)
    args.agent_args = agent_args

    try:
        if args.command == "agents":
            return list_agents()
        if args.command == "doctor":
            return doctor()
        if args.command == "plan":
            return plan_run(args)
        if args.command == "run":
            return run_agent(args)
        if args.command == "image":
            return image_command(args)
        if args.command == "state":
            return state_command(args)
    except ValueError as exc:
        parser.exit(2, f"runhaven: {exc}\n")
    except KeyboardInterrupt:
        parser.exit(130, "runhaven: interrupted\n")

    parser.print_help()
    return 2


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="runhaven",
        description="Run AI coding agents inside Apple container on macOS.",
    )
    subcommands = parser.add_subparsers(dest="command")

    subcommands.add_parser("agents", help="list bundled agent profiles")
    subcommands.add_parser("doctor", help="check local runtime prerequisites")

    agent_args_epilog = (
        "Use -- before flags meant for the agent, for example:\n"
        "  runhaven run claude -- --version"
    )
    plan_parser = subcommands.add_parser(
        "plan",
        help="print the Apple container run plan",
        epilog=agent_args_epilog,
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    add_run_arguments(plan_parser)

    run_parser = subcommands.add_parser(
        "run",
        help="run an agent through Apple container",
        epilog=agent_args_epilog,
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    add_run_arguments(run_parser)
    run_parser.add_argument(
        "--dry-run",
        action="store_true",
        help="print the plan instead of running",
    )

    image_parser = subcommands.add_parser("image", help="manage local agent images")
    image_subcommands = image_parser.add_subparsers(dest="image_command", required=True)
    build_parser_ = image_subcommands.add_parser("build", help="build a bundled agent image")
    build_parser_.add_argument(
        "agent",
        choices=sorted(PROFILES),
        help="agent image template to build",
    )
    build_parser_.add_argument("--tag", help="override the image tag")
    build_parser_.add_argument("--dry-run", action="store_true", help="print the build command")

    state_parser = subcommands.add_parser("state", help="inspect or remove RunHaven state volumes")
    state_subcommands = state_parser.add_subparsers(dest="state_command", required=True)
    state_subcommands.add_parser("list", help="list RunHaven agent home volumes")
    prune_parser = state_subcommands.add_parser("prune", help="remove RunHaven agent home volumes")
    prune_parser.add_argument("--yes", action="store_true", help="delete listed volumes")

    return parser


def add_run_arguments(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("agent", choices=sorted(PROFILES), help="agent profile to run")
    parser.add_argument(
        "--workspace",
        type=Path,
        default=Path("."),
        help="host project directory to mount at /workspace",
    )
    parser.add_argument("--image", help="override the profile image")
    parser.add_argument("--cpus", default="4", help="virtual CPUs for the container")
    parser.add_argument("--memory", default="4g", help="memory limit for the container")
    parser.add_argument(
        "--tty",
        choices=("auto", "always", "never"),
        default="auto",
        help="allocate a container TTY; auto follows the current terminal",
    )
    parser.add_argument(
        "--no-interactive",
        action="store_true",
        help="do not keep container standard input open",
    )
    parser.add_argument(
        "--network",
        choices=SUPPORTED_NETWORK_MODES,
        default="internet",
        help=(
            "internet uses unrestricted default networking; internal creates a host-only "
            "network; provider routes through a runtime allowlist proxy"
        ),
    )
    parser.add_argument(
        "--provider-host",
        action="append",
        default=[],
        metavar="HOST",
        help="additional fully qualified HTTPS host allowed by --network provider",
    )
    parser.add_argument(
        "--read-only-workspace",
        action="store_true",
        help="mount the workspace read-only so the agent can inspect but not edit it",
    )
    parser.add_argument(
        "--ssh",
        action="store_true",
        help="forward the host SSH agent socket without mounting raw SSH keys",
    )
    parser.add_argument(
        "--env",
        action="append",
        default=[],
        metavar="NAME",
        help="inherit a single host environment variable by name",
    )
    parser.add_argument(
        "--user",
        default="agent",
        help="container user to run as; bundled images provide the non-root agent user",
    )
    parser.add_argument(
        "--allow-sensitive-workspace",
        action="store_true",
        help="allow mounting broad or credential-bearing host paths",
    )
    parser.add_argument(
        "--allow-root-user",
        action="store_true",
        help="allow running the agent process as root inside the container",
    )


def list_agents() -> int:
    width = max(len(name) for name in PROFILES)
    for name, profile in sorted(PROFILES.items()):
        print(f"{name:<{width}}  {profile.description}")
    return 0


def doctor() -> int:
    checks = collect_checks()
    for check in checks:
        status = "ok" if check.ok else "fail"
        print(f"{status:4} {check.name}: {check.detail}")
        if not check.ok and check.remedy:
            print(f"     fix: {check.remedy}")
    return 0 if all(check.ok for check in checks) else 1


def plan_run(args: argparse.Namespace) -> int:
    plan = make_run_plan(args)
    print_run_plan(plan)
    return 0


def run_agent(args: argparse.Namespace) -> int:
    plan = make_run_plan(args)
    if args.dry_run:
        print_run_plan(plan)
        return 0

    require_container_cli()
    with acquire_state_lock(plan.state_volume):
        if plan.network_mode == "provider":
            return run_provider_agent(plan)
        for command in plan.preflight:
            run_preflight(command)
        return subprocess.call(plan.command)


def image_command(args: argparse.Namespace) -> int:
    if args.image_command != "build":
        raise ValueError(f"unknown image command: {args.image_command}")

    profile = get_profile(args.agent)
    plan = build_image_plan(profile, tag=args.tag)
    if args.dry_run:
        print(plan.shell_command())
        return 0

    require_container_cli()
    return subprocess.call(plan.command)


def state_command(args: argparse.Namespace) -> int:
    if args.state_command == "list":
        return state_list()
    if args.state_command == "prune":
        return state_prune(confirm=args.yes)
    raise ValueError(f"unknown state command: {args.state_command}")


def make_run_plan(args: argparse.Namespace) -> AgentRunPlan:
    profile = get_profile(args.agent)
    tty = args.tty == "always" or (
        args.tty == "auto" and sys.stdin.isatty() and sys.stdout.isatty()
    )
    return build_run_plan(
        RunOptions(
            profile=profile,
            workspace=args.workspace,
            agent_args=tuple(args.agent_args),
            image=args.image,
            cpus=args.cpus,
            memory=args.memory,
            network=args.network,
            read_only_workspace=args.read_only_workspace,
            ssh=args.ssh,
            env=tuple(args.env),
            user=args.user,
            interactive=not args.no_interactive,
            tty=tty,
            allow_sensitive_workspace=args.allow_sensitive_workspace,
            allow_root_user=args.allow_root_user,
            provider_hosts=tuple(args.provider_host),
        )
    )


def print_run_plan(plan: AgentRunPlan) -> None:
    print(f"Workspace: {plan.workspace}")
    print(f"State volume: {plan.state_volume}")
    print(f"Network: {plan.network_name or 'default internet network'}")
    print(f"Egress: {plan.egress_summary}")
    if plan.network_mode == "provider":
        print(f"Provider hosts: {', '.join(plan.provider_allowed_hosts)}")
        print("Provider proxy: RunHaven injects proxy environment variables at runtime.")
    if plan.preflight:
        print("Preflight:")
        for command in plan.shell_preflight():
            print(f"  {command}")
    print("Run:")
    print(f"  {plan.shell_command()}")


def run_preflight(command: tuple[str, ...]) -> None:
    if command[:4] == ("container", "network", "create", "--internal"):
        ensure_internal_network(command[-1])
        return

    result = subprocess.run(command, check=False)
    if result.returncode != 0:
        raise SystemExit(result.returncode)


@dataclass(frozen=True)
class InternalNetworkInfo:
    ipv4_gateway: str
    ipv4_subnet: str


def run_provider_agent(plan: AgentRunPlan) -> int:
    if not plan.network_name:
        raise ValueError("provider network plan is missing an internal network")
    if not plan.provider_allowed_hosts:
        raise ValueError("provider network plan is missing provider hosts")

    provider_network_created = False
    proxy: ThreadedAllowlistProxy | None = None
    thread: threading.Thread | None = None
    try:
        for command in plan.preflight:
            run_preflight(command)
            if command[:4] == ("container", "network", "create", "--internal"):
                provider_network_created = (
                    provider_network_created or command[-1] == plan.network_name
                )

        network_info = inspect_internal_network(plan.network_name)
        policy = EgressPolicy(plan.provider_allowed_hosts)
        proxy = create_provider_proxy(policy, network_info)
        worker = threading.Thread(target=proxy.serve_forever, daemon=True)
        worker.start()
        thread = worker
        proxy_url = f"http://{network_info.ipv4_gateway}:{proxy.server_address[1]}"
        return_code = subprocess.call(with_provider_proxy_environment(plan, proxy_url))
        print_provider_denials(proxy.denied_connect_targets())
        return return_code
    finally:
        if proxy is not None:
            if thread is not None:
                proxy.shutdown()
            proxy.server_close()
        if thread is not None:
            thread.join(timeout=5)
        if provider_network_created:
            delete_container_network(plan.network_name)


def with_provider_proxy_environment(plan: AgentRunPlan, proxy_url: str) -> tuple[str, ...]:
    image_index = plan.command.index(plan.image)
    proxy_environment = (
        ("HTTPS_PROXY", proxy_url),
        ("HTTP_PROXY", proxy_url),
        ("ALL_PROXY", proxy_url),
        ("https_proxy", proxy_url),
        ("http_proxy", proxy_url),
        ("all_proxy", proxy_url),
        ("NO_PROXY", "localhost,127.0.0.1,::1"),
        ("no_proxy", "localhost,127.0.0.1,::1"),
    )
    injected: list[str] = []
    for name, value in proxy_environment:
        injected.extend(("--env", f"{name}={value}"))
    return (*plan.command[:image_index], *injected, *plan.command[image_index:])


def print_provider_denials(denied_targets: tuple[tuple[str, int], ...]) -> None:
    if not denied_targets:
        return
    count = len(denied_targets)
    plural = "target" if count == 1 else "targets"
    print(f"RunHaven provider proxy blocked {count} CONNECT {plural}:", file=sys.stderr)
    for host, port in denied_targets:
        target = f"{host}:{port}"
        if is_ip_literal(host):
            note = "IP literal targets cannot be allowed"
        else:
            note = f"review before rerunning with --provider-host {host}"
        print(f"  - {target} ({note})", file=sys.stderr)


def create_provider_proxy(
    policy: EgressPolicy,
    network_info: InternalNetworkInfo,
) -> ThreadedAllowlistProxy:
    try:
        return ThreadedAllowlistProxy(
            (network_info.ipv4_gateway, 0),
            policy,
            allowed_client_subnets=(network_info.ipv4_subnet,),
        )
    except OSError:
        return ThreadedAllowlistProxy(
            ("0.0.0.0", 0),
            policy,
            allowed_client_subnets=(network_info.ipv4_subnet,),
        )


def inspect_internal_network(name: str) -> InternalNetworkInfo:
    result = subprocess.run(
        ("container", "network", "inspect", name),
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        raise SystemExit(result.returncode)
    try:
        payload = json.loads(result.stdout)
    except json.JSONDecodeError as exc:
        raise ValueError(f"could not inspect provider network {name!r}") from exc
    if not isinstance(payload, list) or not payload or not isinstance(payload[0], dict):
        raise ValueError(f"could not inspect provider network {name!r}")

    item = payload[0]
    configuration = item.get("configuration")
    if not isinstance(configuration, dict) or configuration.get("mode") != "hostOnly":
        raise ValueError(f"provider network {name!r} is not host-only")
    status = item.get("status")
    if not isinstance(status, dict):
        raise ValueError(f"provider network {name!r} is missing status")
    gateway = status.get("ipv4Gateway")
    subnet = status.get("ipv4Subnet")
    if not isinstance(gateway, str) or not isinstance(subnet, str):
        raise ValueError(f"provider network {name!r} is missing IPv4 gateway or subnet")
    return InternalNetworkInfo(ipv4_gateway=gateway, ipv4_subnet=subnet)


def delete_container_network(name: str) -> None:
    subprocess.run(("container", "network", "delete", name), check=False)


def ensure_internal_network(name: str) -> None:
    existing = subprocess.run(
        ("container", "network", "inspect", name),
        check=False,
        capture_output=True,
        text=True,
    )
    if existing.returncode == 0:
        mode = inspect_network_mode(existing.stdout)
        if mode == "hostOnly":
            return
        raise ValueError(
            f"existing container network {name!r} is {mode or 'unknown'}, not host-only"
        )

    created = subprocess.run(("container", "network", "create", "--internal", name), check=False)
    if created.returncode != 0:
        raise SystemExit(created.returncode)


def inspect_network_mode(output: str) -> str | None:
    try:
        payload = json.loads(output)
    except json.JSONDecodeError:
        return None
    if not isinstance(payload, list) or not payload:
        return None
    first = payload[0]
    if not isinstance(first, dict):
        return None
    configuration = first.get("configuration")
    if not isinstance(configuration, dict):
        return None
    mode = configuration.get("mode")
    return mode if isinstance(mode, str) else None


def state_list() -> int:
    require_container_cli()
    volumes = list_state_volumes()
    if not volumes:
        print("No RunHaven state volumes found.")
        return 0
    for volume in volumes:
        print(volume)
    return 0


def state_prune(*, confirm: bool) -> int:
    require_container_cli()
    volumes = list_state_volumes()
    if not volumes:
        print("No RunHaven state volumes found.")
        return 0
    if not confirm:
        for volume in volumes:
            print(volume)
        print("Rerun with --yes to delete these volumes.")
        return 2
    for volume in volumes:
        result = subprocess.run(("container", "volume", "delete", volume), check=False)
        if result.returncode != 0:
            return result.returncode
    return 0


def list_state_volumes() -> tuple[str, ...]:
    result = subprocess.run(
        ("container", "volume", "list", "--quiet"),
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        raise SystemExit(result.returncode)
    return tuple(
        line.strip()
        for line in result.stdout.splitlines()
        if line.strip().startswith("runhaven-") and line.strip().endswith("-home")
    )


@contextmanager
def acquire_state_lock(state_volume: str) -> Iterator[None]:
    path = state_lock_path(state_volume)
    path.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
    path.touch(mode=0o600, exist_ok=True)
    path.chmod(0o600)
    with path.open("r+", encoding="utf-8") as lock_file:
        try:
            lock_state_file(lock_file)
        except BlockingIOError as exc:
            raise ValueError(
                "agent state for this workspace is already in use. "
                "Wait for the other run to finish, or use a different workspace/profile."
            ) from exc
        lock_file.seek(0)
        lock_file.truncate()
        lock_file.write(f"{os.getpid()}\n")
        lock_file.flush()
        try:
            yield
        finally:
            unlock_state_file(lock_file)


def lock_state_file(lock_file: TextIO) -> None:
    fcntl.flock(lock_file, fcntl.LOCK_EX | fcntl.LOCK_NB)


def unlock_state_file(lock_file: TextIO) -> None:
    fcntl.flock(lock_file, fcntl.LOCK_UN)


def state_lock_path(state_volume: str) -> Path:
    override = os.environ.get("RUNHAVEN_CACHE_HOME")
    if override:
        cache_root = Path(override)
    else:
        cache_root = Path.home() / "Library" / "Caches" / "runhaven"
    return cache_root / "locks" / f"{state_volume}.lock"


def require_container_cli() -> None:
    if shutil.which("container") is None:
        raise ValueError(
            "Apple container CLI was not found. Install it from "
            "https://github.com/apple/container/releases and run `container system start`."
        )


def split_agent_args(argv: Sequence[str]) -> tuple[list[str], tuple[str, ...]]:
    if "--" not in argv:
        return list(argv), ()
    separator = list(argv).index("--")
    return list(argv[:separator]), tuple(argv[separator + 1 :])


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
