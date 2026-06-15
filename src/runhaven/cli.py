from __future__ import annotations

import argparse
import fcntl
import json
import os
import shutil
import subprocess
import sys
import threading
import uuid
from collections.abc import Iterator, Sequence
from contextlib import contextmanager
from dataclasses import dataclass
from datetime import UTC, datetime
from pathlib import Path
from typing import Any, TextIO

from .auth_broker import (
    AUTH_BROKER_RUNTIME,
    AUTH_BROKER_STATUS,
    CODEX_BROKER_PLACEHOLDER_ENV,
    CODEX_BROKER_PLACEHOLDER_VALUE,
    CODEX_BROKER_PROVIDER_ID,
    BrokerDecision,
    CodexApiKeyBrokerProxy,
    auth_broker_profiles,
    get_auth_broker_profile,
)
from .doctor import collect_checks
from .egress import (
    EgressPolicy,
    ProxyDecision,
    ThreadedAllowlistProxy,
    is_ip_literal,
    normalize_host,
)
from .images import build_image_plan
from .plans import SUPPORTED_NETWORK_MODES, AgentRunPlan, RunOptions, build_run_plan
from .profiles import PROFILES, get_profile
from .provider_endpoints import ProviderEndpoint, match_provider_endpoints

GIT_STATUS_PATH_LIMIT = 100


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
        if args.command == "runs":
            return runs_command(args)
        if args.command == "auth":
            return auth_command(args)
        if args.command == "egress":
            return egress_command(args)
        if args.command == "why":
            return why_command(args)
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

    runs_parser = subcommands.add_parser("runs", help="inspect RunHaven run history")
    runs_subcommands = runs_parser.add_subparsers(dest="runs_command", required=True)
    runs_list_parser = runs_subcommands.add_parser("list", help="show recent RunHaven runs")
    runs_list_parser.add_argument(
        "--limit",
        type=int,
        default=20,
        help="maximum entries to show; use 0 for all entries",
    )
    runs_list_parser.add_argument("--json", action="store_true", help="print JSON output")
    runs_show_parser = runs_subcommands.add_parser("show", help="show one RunHaven run record")
    runs_show_parser.add_argument("run_id", help="run id to show")
    runs_show_parser.add_argument("--json", action="store_true", help="print JSON output")
    runs_log_parser = runs_subcommands.add_parser(
        "log",
        help="show one run with related provider and auth events",
    )
    runs_log_parser.add_argument("run_id", help="run id to show")
    runs_log_parser.add_argument("--json", action="store_true", help="print JSON output")
    runs_diff_parser = runs_subcommands.add_parser(
        "diff",
        help="show live git diff for one RunHaven run",
    )
    runs_diff_parser.add_argument("run_id", help="run id to diff")
    runs_active_parser = runs_subcommands.add_parser(
        "active",
        help="show currently active RunHaven runs",
    )
    runs_active_parser.add_argument("--json", action="store_true", help="print JSON output")
    runs_stop_parser = runs_subcommands.add_parser(
        "stop",
        help="stop an active RunHaven run",
    )
    runs_stop_parser.add_argument("run_id", help="active run id to stop")

    egress_parser = subcommands.add_parser("egress", help="inspect provider egress policy logs")
    egress_subcommands = egress_parser.add_subparsers(dest="egress_command", required=True)
    egress_log_parser = egress_subcommands.add_parser(
        "log",
        help="show recent provider proxy policy decisions",
    )
    egress_log_parser.add_argument(
        "--limit",
        type=int,
        default=20,
        help="maximum entries to show; use 0 for all entries",
    )
    egress_log_parser.add_argument("--json", action="store_true", help="print JSON output")

    auth_parser = subcommands.add_parser("auth", help="inspect provider auth broker status")
    auth_subcommands = auth_parser.add_subparsers(dest="auth_command", required=True)
    auth_status_parser = auth_subcommands.add_parser(
        "status",
        help="show auth broker status without reading secrets",
    )
    auth_status_parser.add_argument("--json", action="store_true", help="print JSON output")
    auth_explain_parser = auth_subcommands.add_parser(
        "explain",
        help="explain the auth broker boundary for an agent",
    )
    auth_explain_parser.add_argument("agent", choices=sorted(PROFILES), help="agent profile")
    auth_explain_parser.add_argument("--json", action="store_true", help="print JSON output")
    auth_log_parser = auth_subcommands.add_parser(
        "log",
        help="show recent auth broker decisions without secrets",
    )
    auth_log_parser.add_argument(
        "--limit",
        type=int,
        default=20,
        help="maximum entries to show; use 0 for all entries",
    )
    auth_log_parser.add_argument("--json", action="store_true", help="print JSON output")

    why_parser = subcommands.add_parser("why", help="explain RunHaven safety decisions")
    why_subcommands = why_parser.add_subparsers(dest="why_command", required=True)
    why_host_parser = why_subcommands.add_parser(
        "host",
        help="explain provider-host allowlist behavior",
    )
    why_host_parser.add_argument("host", help="host to explain")
    why_host_parser.add_argument("--port", type=int, default=443, help="provider port to check")
    why_host_parser.add_argument(
        "--agent",
        choices=sorted(PROFILES),
        help="agent profile whose bundled provider hosts should be checked",
    )

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
        "--codex-api-key-broker-env",
        metavar="NAME",
        help=(
            "Codex-only: read this host environment variable at run time and broker "
            "OpenAI Responses API requests without placing the raw value in the guest"
        ),
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

    validate_runtime_auth_broker_environment(plan)
    require_container_cli()
    with acquire_state_lock(plan.state_volume):
        if plan.network_mode == "provider":
            return run_provider_agent(plan)
        for command in plan.preflight:
            run_preflight(command)
        run_id = uuid.uuid4().hex
        git_before = capture_git_snapshot(plan.workspace)
        started_at = utc_timestamp()
        print(f"Run id: {run_id}", file=sys.stderr)
        write_active_run_record(plan, run_id=run_id, started_at=started_at)
        try:
            return_code = subprocess.call(plan.command)
        finally:
            stop_requested = active_run_stop_requested(run_id)
            remove_active_run_record(run_id)
        finished_at = utc_timestamp()
        git = summarize_git_change(git_before, capture_git_snapshot(plan.workspace))
        write_run_record(
            plan,
            run_id=run_id,
            started_at=started_at,
            finished_at=finished_at,
            return_code=return_code,
            status="stopped" if stop_requested else None,
            provider_decisions=(),
            auth_decisions=None,
            cleanup={"provider_network": "not-applicable"},
            git=git,
        )
        return return_code


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


def runs_command(args: argparse.Namespace) -> int:
    if args.runs_command == "list":
        return runs_list(limit=args.limit, json_output=args.json)
    if args.runs_command == "show":
        return runs_show(args.run_id, json_output=args.json)
    if args.runs_command == "log":
        return runs_log(args.run_id, json_output=args.json)
    if args.runs_command == "diff":
        return runs_diff(args.run_id)
    if args.runs_command == "active":
        return runs_active(json_output=args.json)
    if args.runs_command == "stop":
        return runs_stop(args.run_id)
    raise ValueError(f"unknown runs command: {args.runs_command}")


def egress_command(args: argparse.Namespace) -> int:
    if args.egress_command == "log":
        return egress_log(limit=args.limit, json_output=args.json)
    raise ValueError(f"unknown egress command: {args.egress_command}")


def auth_command(args: argparse.Namespace) -> int:
    if args.auth_command == "status":
        return auth_status(json_output=args.json)
    if args.auth_command == "explain":
        return auth_explain(args.agent, json_output=args.json)
    if args.auth_command == "log":
        return auth_log(limit=args.limit, json_output=args.json)
    raise ValueError(f"unknown auth command: {args.auth_command}")


def why_command(args: argparse.Namespace) -> int:
    if args.why_command == "host":
        return why_host(args.host, port=args.port, agent=args.agent)
    raise ValueError(f"unknown why command: {args.why_command}")


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
            codex_api_key_broker_env=args.codex_api_key_broker_env,
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
    if plan.codex_api_key_broker_env:
        print(
            "Codex API key broker: enabled from host environment variable "
            f"{plan.codex_api_key_broker_env}; value is not printed or planned."
        )
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


@dataclass(frozen=True)
class GitStatusEntry:
    path: str
    status: str


def run_provider_agent(plan: AgentRunPlan) -> int:
    if not plan.network_name:
        raise ValueError("provider network plan is missing an internal network")
    if not plan.provider_allowed_hosts:
        raise ValueError("provider network plan is missing provider hosts")

    codex_api_key = require_codex_api_key_broker_secret(plan)
    provider_network_created = False
    proxy: ThreadedAllowlistProxy | None = None
    proxy_thread: threading.Thread | None = None
    codex_broker: CodexApiKeyBrokerProxy | None = None
    codex_broker_thread: threading.Thread | None = None
    run_id = uuid.uuid4().hex
    started_at: str | None = None
    finished_at: str | None = None
    return_code: int | None = None
    provider_decisions: tuple[ProxyDecision, ...] = ()
    auth_decisions: tuple[BrokerDecision, ...] | None = None
    git: dict[str, object] | None = None
    stop_requested = False
    active_run_recorded = False
    cleanup: dict[str, object] = {
        "provider_network": "not-created",
        "provider_network_name": plan.network_name,
    }
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
        proxy_thread = worker
        proxy_url = f"http://{network_info.ipv4_gateway}:{proxy.server_address[1]}"

        if codex_api_key is None:
            command = with_provider_proxy_environment(plan, proxy_url)
        else:
            codex_broker = create_codex_api_key_broker(codex_api_key, network_info)
            broker_worker = threading.Thread(target=codex_broker.serve_forever, daemon=True)
            broker_worker.start()
            codex_broker_thread = broker_worker
            broker_url = (
                f"http://{network_info.ipv4_gateway}:"
                f"{codex_broker.server_address[1]}/v1"
            )
            command = with_provider_proxy_environment(
                plan,
                proxy_url,
                no_proxy_hosts=(network_info.ipv4_gateway,),
            )
            command = with_codex_api_key_broker_config(command, plan, broker_url)

        git_before = capture_git_snapshot(plan.workspace)
        started_at = utc_timestamp()
        print(f"Run id: {run_id}", file=sys.stderr)
        write_active_run_record(plan, run_id=run_id, started_at=started_at)
        active_run_recorded = True
        try:
            return_code = subprocess.call(command)
        finally:
            stop_requested = active_run_stop_requested(run_id)
        finished_at = utc_timestamp()
        git = summarize_git_change(git_before, capture_git_snapshot(plan.workspace))
        provider_decisions = tuple(proxy.policy_decisions())
        write_provider_policy_log(plan, provider_decisions, run_id=run_id)
        if codex_broker is not None:
            auth_decisions = tuple(codex_broker.broker_decisions())
            write_auth_broker_log(
                plan,
                auth_decisions,
                run_id=run_id,
                return_code=return_code,
            )
        print_provider_blocked_host_review(plan, provider_decisions, run_id=run_id)
        return return_code
    finally:
        if codex_broker is not None:
            if codex_broker_thread is not None:
                codex_broker.shutdown()
            codex_broker.server_close()
        if codex_broker_thread is not None:
            codex_broker_thread.join(timeout=5)
        if proxy is not None:
            if proxy_thread is not None:
                proxy.shutdown()
            proxy.server_close()
        if proxy_thread is not None:
            proxy_thread.join(timeout=5)
        if provider_network_created:
            cleanup = cleanup_provider_network(plan)
        if return_code is not None and started_at is not None and finished_at is not None:
            write_run_record(
                plan,
                run_id=run_id,
                started_at=started_at,
                finished_at=finished_at,
                return_code=return_code,
                status="stopped" if stop_requested else None,
                provider_decisions=provider_decisions,
                auth_decisions=auth_decisions,
                cleanup=cleanup,
                git=git or {"available": False, "reason": "git-snapshot-missing"},
            )
        if active_run_recorded:
            remove_active_run_record(run_id)


def require_codex_api_key_broker_secret(plan: AgentRunPlan) -> str | None:
    if plan.codex_api_key_broker_env is None:
        return None
    value = os.environ.get(plan.codex_api_key_broker_env)
    if value is None or not value.strip():
        raise ValueError(
            f"{plan.codex_api_key_broker_env} is not set on the host; export it before "
            "using --codex-api-key-broker-env"
        )
    return value


def validate_runtime_auth_broker_environment(plan: AgentRunPlan) -> None:
    require_codex_api_key_broker_secret(plan)


def with_provider_proxy_environment(
    plan: AgentRunPlan,
    proxy_url: str,
    *,
    no_proxy_hosts: Sequence[str] = (),
) -> tuple[str, ...]:
    image_index = plan.command.index(plan.image)
    no_proxy = ",".join(("localhost", "127.0.0.1", "::1", *no_proxy_hosts))
    proxy_environment = (
        ("HTTPS_PROXY", proxy_url),
        ("HTTP_PROXY", proxy_url),
        ("ALL_PROXY", proxy_url),
        ("https_proxy", proxy_url),
        ("http_proxy", proxy_url),
        ("all_proxy", proxy_url),
        ("NO_PROXY", no_proxy),
        ("no_proxy", no_proxy),
    )
    injected: list[str] = []
    for name, value in proxy_environment:
        injected.extend(("--env", f"{name}={value}"))
    return (*plan.command[:image_index], *injected, *plan.command[image_index:])


def with_codex_api_key_broker_config(
    command: tuple[str, ...],
    plan: AgentRunPlan,
    broker_base_url: str,
) -> tuple[str, ...]:
    image_index = command.index(plan.image)
    if image_index + 1 >= len(command) or command[image_index + 1] != "codex":
        raise ValueError("Codex API key broker requires the agent command to start with codex")
    broker_environment = (
        "--env",
        f"{CODEX_BROKER_PLACEHOLDER_ENV}={CODEX_BROKER_PLACEHOLDER_VALUE}",
    )
    command_with_env = (*command[:image_index], *broker_environment, *command[image_index:])
    codex_index = image_index + len(broker_environment) + 1
    config = (
        "-c",
        f'model_provider="{CODEX_BROKER_PROVIDER_ID}"',
        "-c",
        f'model_providers.{CODEX_BROKER_PROVIDER_ID}.name='
        '"RunHaven OpenAI API-key broker"',
        "-c",
        f'model_providers.{CODEX_BROKER_PROVIDER_ID}.base_url="{broker_base_url}"',
        "-c",
        f'model_providers.{CODEX_BROKER_PROVIDER_ID}.env_key='
        f'"{CODEX_BROKER_PLACEHOLDER_ENV}"',
        "-c",
        f'model_providers.{CODEX_BROKER_PROVIDER_ID}.wire_api="responses"',
    )
    return (
        *command_with_env[: codex_index + 1],
        *config,
        *command_with_env[codex_index + 1 :],
    )


def print_provider_blocked_host_review(
    plan: AgentRunPlan,
    decisions: Sequence[ProxyDecision],
    *,
    run_id: str,
) -> None:
    denials = tuple(decision for decision in decisions if decision.decision == "denied")
    if not denials:
        return
    total = sum(decision.count for decision in denials)
    plural = "request" if total == 1 else "requests"
    print(
        f"RunHaven provider proxy blocked {total} CONNECT {plural} "
        f"across {len(denials)} target(s).",
        file=sys.stderr,
    )
    print(f"Run id: {run_id}", file=sys.stderr)
    print("Review:", file=sys.stderr)
    for decision in denials:
        target = f"{decision.host}:{decision.port}"
        matched_rule = decision.matched_rule or "-"
        print(
            f"  - {target}  count={decision.count}  reason={decision.reason}  "
            f"rule={matched_rule}",
            file=sys.stderr,
        )
        print(
            f"    Next action: {provider_denial_next_action(plan, decision)}",
            file=sys.stderr,
        )
    print("Recent policy log: runhaven egress log --limit 20", file=sys.stderr)


def provider_denial_next_action(plan: AgentRunPlan, decision: ProxyDecision) -> str:
    host = decision.host
    if is_ip_literal(host):
        return "IP literal targets cannot be allowed; use a reviewed provider hostname."
    if decision.reason == "port-not-allowed":
        return "provider mode only allows HTTPS CONNECT on port 443."
    if decision.reason == "unsafe-resolved-address":
        return "do not add an override; the allowed host resolved to a non-public address."
    if decision.reason == "dns-resolution-failed":
        return "check DNS or provider availability before changing the allowlist."
    explanation = f"runhaven why host {host} --agent {plan.profile_name}"
    if match_provider_endpoints(host, profile=plan.profile_name):
        return (
            f"{explanation}; rerun with --provider-host {host} only if the documented "
            "purpose matches."
        )
    return f"{explanation}; add --provider-host {host} only after source review."


def utc_timestamp() -> str:
    return datetime.now(UTC).isoformat().replace("+00:00", "Z")


def capture_git_snapshot(workspace: Path) -> dict[str, object]:
    repo_root, unavailable_reason = git_repo_root(workspace)
    if repo_root is None:
        return {"available": False, "reason": unavailable_reason}

    head = git_head(repo_root)
    entries = read_git_status_entries(repo_root, workspace)
    if entries is None:
        return {
            "available": False,
            "reason": "git-status-failed",
            "repo_root": repo_root,
        }

    paths = tuple(entry.path for entry in entries)
    shown_paths = list(paths[:GIT_STATUS_PATH_LIMIT])
    return {
        "available": True,
        "repo_root": repo_root,
        "head": head,
        "dirty": bool(paths),
        "changed_count": len(paths),
        "paths": shown_paths,
        "truncated": len(paths) > GIT_STATUS_PATH_LIMIT,
    }


def read_git_status_entries(repo_root: str, workspace: Path) -> tuple[GitStatusEntry, ...] | None:
    resolved_workspace = str(workspace.resolve())
    status_result = run_git_for_metadata(
        ("git", "-C", repo_root, "status", "--porcelain=v1", "-z", "--", resolved_workspace),
    )
    if status_result is None or status_result.returncode != 0:
        return None
    return parse_git_status_entries(status_result.stdout)


def git_repo_root(workspace: Path) -> tuple[str | None, str]:
    result = run_git_for_metadata(
        ("git", "-C", str(workspace), "rev-parse", "--show-toplevel"),
        text=True,
    )
    if result is None:
        return None, "git-not-found"
    if result.returncode != 0:
        return None, "not-a-git-worktree"
    repo_root = result.stdout.strip()
    if not repo_root:
        return None, "not-a-git-worktree"
    return str(Path(repo_root).resolve()), ""


def git_head(repo_root: str) -> str | None:
    result = run_git_for_metadata(
        ("git", "-C", repo_root, "rev-parse", "HEAD"),
        text=True,
    )
    if result is None or result.returncode != 0:
        return None
    head = result.stdout.strip()
    return head or None


def run_git_for_metadata(
    command: tuple[str, ...],
    *,
    text: bool = False,
) -> subprocess.CompletedProcess[Any] | None:
    try:
        return subprocess.run(
            command,
            check=False,
            capture_output=True,
            text=text,
        )
    except (FileNotFoundError, OSError):
        return None


def parse_git_status_paths(output: bytes) -> tuple[str, ...]:
    return tuple(entry.path for entry in parse_git_status_entries(output))


def parse_git_status_entries(output: bytes) -> tuple[GitStatusEntry, ...]:
    entries = [entry for entry in output.split(b"\0") if entry]
    parsed: dict[str, GitStatusEntry] = {}
    index = 0
    while index < len(entries):
        entry = entries[index]
        if len(entry) >= 4:
            status = entry[:2]
            path = os.fsdecode(entry[3:])
            parsed[path] = GitStatusEntry(path=path, status=os.fsdecode(status))
            if b"R" in status or b"C" in status:
                index += 1
        index += 1
    return tuple(parsed[path] for path in sorted(parsed))


def summarize_git_change(
    before: dict[str, object],
    after: dict[str, object],
) -> dict[str, object]:
    before_available = before.get("available") is True
    after_available = after.get("available") is True
    if not before_available or not after_available:
        reason = after.get("reason") if not after_available else before.get("reason")
        metadata: dict[str, object] = {
            "available": False,
            "reason": reason if isinstance(reason, str) else "git-unavailable",
        }
        repo_root = after.get("repo_root") or before.get("repo_root")
        if isinstance(repo_root, str):
            metadata["repo_root"] = repo_root
        return metadata

    before_summary = git_snapshot_for_record(before)
    after_summary = git_snapshot_for_record(after)
    return {
        "available": True,
        "repo_root": after["repo_root"],
        "changed": before_summary != after_summary,
        "before": before_summary,
        "after": after_summary,
    }


def git_snapshot_for_record(snapshot: dict[str, object]) -> dict[str, object]:
    return {
        "head": snapshot.get("head"),
        "dirty": snapshot.get("dirty") is True,
        "changed_count": snapshot.get("changed_count", 0),
        "paths": snapshot.get("paths", []),
        "truncated": snapshot.get("truncated") is True,
    }


def cleanup_provider_network(plan: AgentRunPlan) -> dict[str, object]:
    if plan.network_name is None:
        return {"provider_network": "not-created", "provider_network_name": None}
    result: object = delete_container_network(plan.network_name)
    status = "deleted" if result == 0 else "delete-failed"
    cleanup: dict[str, object] = {
        "provider_network": status,
        "provider_network_name": plan.network_name,
    }
    if isinstance(result, int):
        cleanup["delete_return_code"] = result
    return cleanup


def write_run_record(
    plan: AgentRunPlan,
    *,
    run_id: str,
    started_at: str,
    finished_at: str,
    return_code: int,
    status: str | None = None,
    provider_decisions: Sequence[ProxyDecision],
    auth_decisions: Sequence[BrokerDecision] | None,
    cleanup: dict[str, object],
    git: dict[str, object],
) -> None:
    log_path = runs_log_path()
    log_path.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
    payload = {
        "timestamp": finished_at,
        "started_at": started_at,
        "finished_at": finished_at,
        "run_id": run_id,
        "profile": plan.profile_name,
        "workspace": str(plan.workspace),
        "network": plan.network_mode,
        "status": status or ("succeeded" if return_code == 0 else "failed"),
        "return_code": return_code,
        "provider_policy": summarize_provider_policy(provider_decisions),
        "auth_broker": summarize_auth_broker(auth_decisions),
        "cleanup": cleanup,
        "git": git,
    }
    with log_path.open("a", encoding="utf-8") as log_file:
        log_file.write(json.dumps(payload, sort_keys=True) + "\n")


def summarize_provider_policy(decisions: Sequence[ProxyDecision]) -> dict[str, int]:
    return {
        "entries": len(decisions),
        "allowed": sum(decision.count for decision in decisions if decision.decision == "allowed"),
        "denied": sum(decision.count for decision in decisions if decision.decision == "denied"),
    }


def summarize_auth_broker(decisions: Sequence[BrokerDecision] | None) -> dict[str, object]:
    if decisions is None:
        return {
            "broker": None,
            "entries": 0,
            "allowed": 0,
            "denied": 0,
            "no_requests": False,
        }
    return {
        "broker": "codex-api-key",
        "entries": len(decisions),
        "allowed": sum(decision.count for decision in decisions if decision.decision == "allowed"),
        "denied": sum(decision.count for decision in decisions if decision.decision == "denied"),
        "no_requests": not decisions,
    }


def write_active_run_record(plan: AgentRunPlan, *, run_id: str, started_at: str) -> None:
    payload: dict[str, object] = {
        "timestamp": started_at,
        "run_id": run_id,
        "profile": plan.profile_name,
        "workspace": str(plan.workspace),
        "network": plan.network_mode,
        "status": "running",
        "container_name": plan.container_name,
        "state_volume": plan.state_volume,
        "network_name": plan.network_name,
        "host_pid": os.getpid(),
    }
    write_active_run_payload(run_id, payload)


def write_active_run_payload(run_id: str, payload: dict[str, object]) -> None:
    path = active_run_path(run_id)
    path.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
    temporary_path = path.with_suffix(".tmp")
    temporary_path.write_text(json.dumps(payload, sort_keys=True) + "\n", encoding="utf-8")
    temporary_path.chmod(0o600)
    temporary_path.replace(path)


def find_active_run_record(run_id: str) -> dict[str, Any]:
    path = active_run_path(run_id)
    if not path.exists():
        raise ValueError(f"active run not found: {run_id}")
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise ValueError(f"active run record is invalid: {run_id}") from exc
    if not isinstance(payload, dict):
        raise ValueError(f"active run record is invalid: {run_id}")
    return payload


def mark_active_run_stop_requested(run_id: str, record: dict[str, Any]) -> None:
    updated = dict(record)
    updated["status"] = "stop-requested"
    updated["stop_requested_at"] = utc_timestamp()
    write_active_run_payload(run_id, updated)


def clear_active_run_stop_requested(run_id: str, record: dict[str, Any]) -> None:
    updated = dict(record)
    updated["status"] = "running"
    updated.pop("stop_requested_at", None)
    write_active_run_payload(run_id, updated)


def active_run_stop_requested(run_id: str) -> bool:
    try:
        record = find_active_run_record(run_id)
    except ValueError:
        return False
    return isinstance(record.get("stop_requested_at"), str)


def remove_active_run_record(run_id: str) -> None:
    try:
        active_run_path(run_id).unlink()
    except FileNotFoundError:
        pass


def active_run_path(run_id: str) -> Path:
    validate_run_id(run_id)
    return active_runs_dir() / f"{run_id}.json"


def active_runs_dir() -> Path:
    return runhaven_cache_root() / "active-runs"


def read_active_run_records() -> list[dict[str, Any]]:
    active_dir = active_runs_dir()
    if not active_dir.exists():
        return []
    records: list[dict[str, Any]] = []
    for path in sorted(active_dir.glob("*.json")):
        try:
            payload = json.loads(path.read_text(encoding="utf-8"))
        except json.JSONDecodeError:
            continue
        if not isinstance(payload, dict):
            continue
        run_id = payload.get("run_id")
        container_name = payload.get("container_name")
        if not isinstance(run_id, str) or not isinstance(container_name, str):
            continue
        try:
            validate_run_id(run_id)
            validate_runhaven_container_name(container_name)
        except ValueError:
            continue
        records.append(payload)
    return sorted(records, key=active_run_sort_key)


def active_run_sort_key(record: dict[str, Any]) -> tuple[str, str]:
    timestamp = record.get("timestamp")
    run_id = record.get("run_id")
    return (
        timestamp if isinstance(timestamp, str) else "",
        run_id if isinstance(run_id, str) else "",
    )


def validate_run_id(run_id: str) -> None:
    if not run_id or run_id.startswith("-"):
        raise ValueError(f"invalid run id: {run_id!r}")
    if any(character.isspace() or character in "/\\" for character in run_id):
        raise ValueError(f"invalid run id: {run_id!r}")


def runs_active(*, json_output: bool) -> int:
    records = read_active_run_records()
    if json_output:
        print(json.dumps(records, indent=2, sort_keys=True))
        return 0
    if not records:
        print("No active RunHaven runs found.")
        return 0
    for record in records:
        print(
            f"{record.get('timestamp', '<unknown>')}  "
            f"{record.get('profile', 'unknown')}  "
            f"{record.get('network', 'unknown')}  "
            f"{record.get('status', 'unknown')}  "
            f"run={record.get('run_id', '-')}  "
            f"workspace={record.get('workspace', '-')}  "
            f"container={record.get('container_name', '-')}"
        )
    return 0


def runs_stop(run_id: str) -> int:
    record = find_active_run_record(run_id)
    container_name = require_string(
        record.get("container_name"),
        "active run record is missing container name",
    )
    validate_runhaven_container_name(container_name)
    require_container_cli()
    mark_active_run_stop_requested(run_id, record)
    result = subprocess.run(("container", "stop", container_name), check=False)
    if result.returncode != 0:
        try:
            clear_active_run_stop_requested(run_id, record)
        except ValueError:
            pass
        return result.returncode
    print(f"Stop requested for run {run_id} ({container_name}).")
    return 0


def validate_runhaven_container_name(container_name: str) -> None:
    if not container_name.startswith("runhaven-"):
        raise ValueError(
            f"active run container {container_name!r} is not a RunHaven-owned container"
        )
    if container_name.startswith("-"):
        raise ValueError(f"invalid active run container name: {container_name!r}")
    if any(character.isspace() or character in "/\\," for character in container_name):
        raise ValueError(f"invalid active run container name: {container_name!r}")


def runs_list(*, limit: int, json_output: bool) -> int:
    if limit < 0:
        raise ValueError("--limit must be 0 or greater")
    records = read_run_records(limit=limit)
    if json_output:
        print(json.dumps(records, indent=2, sort_keys=True))
        return 0
    if not records:
        print("No RunHaven run records found.")
        return 0
    for record in records:
        provider_policy = record.get("provider_policy")
        auth_broker = record.get("auth_broker")
        cleanup = record.get("cleanup")
        provider_denied = (
            provider_policy.get("denied", 0) if isinstance(provider_policy, dict) else 0
        )
        auth_denied = auth_broker.get("denied", 0) if isinstance(auth_broker, dict) else 0
        cleanup_status = (
            cleanup.get("provider_network", "-") if isinstance(cleanup, dict) else "-"
        )
        print(
            f"{record.get('timestamp', '<unknown>')}  "
            f"{record.get('profile', 'unknown')}  "
            f"{record.get('network', 'unknown')}  "
            f"{record.get('status', 'unknown')}  "
            f"return={record.get('return_code', '-')}  "
            f"provider_denied={provider_denied}  "
            f"auth_denied={auth_denied}  "
            f"cleanup={cleanup_status}  "
            f"run={record.get('run_id', '-')}"
        )
    return 0


def runs_show(run_id: str, *, json_output: bool) -> int:
    record = find_run_record(run_id)
    if json_output:
        print(json.dumps(record, indent=2, sort_keys=True))
        return 0

    provider_policy = record.get("provider_policy")
    auth_broker = record.get("auth_broker")
    cleanup = record.get("cleanup")
    print(f"Run id: {record.get('run_id', '-')}")
    print(f"Started: {record.get('started_at', '-')}")
    print(f"Finished: {record.get('finished_at', '-')}")
    print(f"Profile: {record.get('profile', 'unknown')}")
    print(f"Workspace: {record.get('workspace', 'unknown')}")
    print(f"Network: {record.get('network', 'unknown')}")
    print(f"Status: {record.get('status', 'unknown')}")
    print(f"Return code: {record.get('return_code', '-')}")
    git = record.get("git")
    if isinstance(git, dict):
        print(format_git_summary(git))
    if isinstance(provider_policy, dict):
        print(
            "Provider policy: "
            f"entries={provider_policy.get('entries', 0)} "
            f"allowed={provider_policy.get('allowed', 0)} "
            f"denied={provider_policy.get('denied', 0)}"
        )
    if isinstance(auth_broker, dict):
        no_requests = str(auth_broker.get("no_requests", False)).lower()
        print(
            "Auth broker: "
            f"broker={auth_broker.get('broker') or '-'} "
            f"entries={auth_broker.get('entries', 0)} "
            f"allowed={auth_broker.get('allowed', 0)} "
            f"denied={auth_broker.get('denied', 0)} "
            f"no_requests={no_requests}"
        )
    if isinstance(cleanup, dict):
        print(f"Cleanup provider network: {cleanup.get('provider_network', '-')}")
    return 0


def format_git_summary(git: dict[str, Any]) -> str:
    if git.get("available") is not True:
        reason = git.get("reason")
        reason_text = reason if isinstance(reason, str) else "unknown"
        return f"Git: unavailable ({reason_text})"

    before = git.get("before")
    after = git.get("after")
    before_head = short_git_head(before.get("head") if isinstance(before, dict) else None)
    after_head = short_git_head(after.get("head") if isinstance(after, dict) else None)
    changed = str(git.get("changed") is True).lower()
    files = after.get("changed_count", 0) if isinstance(after, dict) else 0
    return f"Git: changed={changed} before={before_head} after={after_head} files={files}"


def short_git_head(head: object) -> str:
    if not isinstance(head, str) or not head:
        return "-"
    return head[:7]


def runs_diff(run_id: str) -> int:
    record = find_run_record(run_id)
    git = require_available_git_metadata(record, run_id)
    before = require_git_snapshot(git.get("before"), "before")
    after = require_git_snapshot(git.get("after"), "after")
    repo_root = require_string(git.get("repo_root"), "run git metadata is missing repo root")
    workspace = require_string(record.get("workspace"), "run record is missing workspace")
    before_head = require_string(
        before.get("head"),
        "run git metadata is missing a base HEAD; refusing live diff",
    )
    after_head = require_string(
        after.get("head"),
        "run git metadata is missing recorded HEAD; refusing live diff",
    )
    after_dirty = after.get("dirty") is True
    after_paths = git_snapshot_paths(after)

    if after.get("truncated") is True:
        raise ValueError("run git path list is truncated; refusing live diff")
    if not Path(repo_root).is_dir():
        raise ValueError("recorded git repo no longer exists; refusing live diff")
    if not Path(workspace).exists():
        raise ValueError("recorded workspace no longer exists; refusing live diff")

    current = capture_git_snapshot(Path(workspace))
    if current.get("available") is not True:
        raise ValueError("recorded workspace is no longer a git worktree; refusing live diff")
    if current.get("repo_root") != repo_root:
        raise ValueError(
            "workspace git repo no longer matches the recorded run; refusing live diff"
        )
    if current.get("head") != after_head:
        raise ValueError("git HEAD changed since the recorded run; refusing live diff")
    if not git_snapshot_matches(after, current):
        raise ValueError("git working tree changed since the recorded run; refusing live diff")

    if not after_dirty and before_head == after_head:
        print("No git changes recorded for this run.")
        return 0

    if after_dirty:
        print(
            "runhaven: showing a live working tree diff; RunHaven verified "
            "the recorded HEAD and path set, not file contents since the run.",
            file=sys.stderr,
        )

    diff_parts: list[str] = []
    if after_dirty:
        if before_head != after_head:
            diff_parts.append(
                run_git_diff(
                    (
                        "git",
                        "-C",
                        repo_root,
                        "diff",
                        "--no-ext-diff",
                        "--no-color",
                        before_head,
                        after_head,
                    ),
                )
            )
        entries = read_git_status_entries(repo_root, Path(workspace))
        if entries is None:
            raise ValueError("could not read current git status; refusing live diff")
        untracked_paths = {
            entry.path for entry in entries if entry.status == "??" and entry.path in after_paths
        }
        tracked_paths = [path for path in after_paths if path not in untracked_paths]
        if tracked_paths:
            diff_parts.append(
                run_git_diff(
                    (
                        "git",
                        "-C",
                        repo_root,
                        "diff",
                        "--no-ext-diff",
                        "--no-color",
                        after_head,
                        "--",
                        *tracked_paths,
                    ),
                )
            )
        for path in sorted(untracked_paths):
            diff_parts.append(run_untracked_git_diff(repo_root, path))
    else:
        diff_parts.append(
            run_git_diff(
                (
                    "git",
                    "-C",
                    repo_root,
                    "diff",
                    "--no-ext-diff",
                    "--no-color",
                    before_head,
                    after_head,
                ),
            )
        )

    printed = False
    for diff in diff_parts:
        if not diff:
            continue
        print(diff, end="" if diff.endswith("\n") else "\n")
        printed = True
    if not printed:
        print("No git diff output for recorded changes.")
    return 0


def require_available_git_metadata(record: dict[str, Any], run_id: str) -> dict[str, Any]:
    git = record.get("git")
    if not isinstance(git, dict):
        raise ValueError(f"git metadata is unavailable for run {run_id}")
    if git.get("available") is not True:
        reason = git.get("reason")
        reason_text = f": {reason}" if isinstance(reason, str) and reason else ""
        raise ValueError(f"git metadata is unavailable for run {run_id}{reason_text}")
    return git


def require_git_snapshot(value: object, name: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise ValueError(f"run git metadata is missing {name} snapshot")
    return value


def require_string(value: object, message: str) -> str:
    if not isinstance(value, str) or not value:
        raise ValueError(message)
    return value


def git_snapshot_paths(snapshot: dict[str, Any]) -> tuple[str, ...]:
    paths = snapshot.get("paths")
    if not isinstance(paths, list) or not all(isinstance(path, str) for path in paths):
        raise ValueError("run git metadata has invalid path list")
    return tuple(sorted(paths))


def git_snapshot_matches(recorded: dict[str, Any], current: dict[str, object]) -> bool:
    current_paths = current.get("paths")
    if not isinstance(current_paths, list) or not all(
        isinstance(path, str) for path in current_paths
    ):
        return False
    return (
        recorded.get("dirty") is (current.get("dirty") is True)
        and recorded.get("changed_count", 0) == current.get("changed_count", 0)
        and git_snapshot_paths(recorded) == tuple(sorted(current_paths))
        and recorded.get("truncated") is (current.get("truncated") is True)
    )


def run_git_diff(command: tuple[str, ...]) -> str:
    result = run_git_for_metadata(command, text=True)
    if result is None:
        raise ValueError("git diff is unavailable")
    if result.returncode != 0:
        detail = result.stderr.strip() if isinstance(result.stderr, str) else ""
        raise ValueError(f"git diff failed: {detail or result.returncode}")
    return result.stdout if isinstance(result.stdout, str) else ""


def run_untracked_git_diff(repo_root: str, path: str) -> str:
    full_path = safe_repo_path(repo_root, path)
    result = run_git_for_metadata(
        (
            "git",
            "-C",
            repo_root,
            "diff",
            "--no-ext-diff",
            "--no-color",
            "--no-index",
            "--",
            "/dev/null",
            str(full_path),
        ),
        text=True,
    )
    if result is None:
        raise ValueError("git diff is unavailable")
    if result.returncode not in (0, 1):
        detail = result.stderr.strip() if isinstance(result.stderr, str) else ""
        raise ValueError(f"git diff failed: {detail or result.returncode}")
    return result.stdout if isinstance(result.stdout, str) else ""


def safe_repo_path(repo_root: str, path: str) -> Path:
    root = Path(repo_root).resolve()
    full_path = (root / path).resolve()
    try:
        full_path.relative_to(root)
    except ValueError as exc:
        raise ValueError("git path escapes the recorded repository; refusing live diff") from exc
    return full_path


def runs_log(run_id: str, *, json_output: bool) -> int:
    record = find_run_record(run_id)
    provider_entries = log_entries_for_run(read_egress_policy_log(limit=0), run_id)
    auth_entries = log_entries_for_run(read_auth_broker_log(limit=0), run_id)
    payload = {
        "run": record,
        "provider_policy": provider_entries,
        "auth_broker": auth_entries,
    }
    if json_output:
        print(json.dumps(payload, indent=2, sort_keys=True))
        return 0

    runs_show(run_id, json_output=False)
    print("Provider policy decisions:")
    if provider_entries:
        for entry in provider_entries:
            host = entry.get("host", "<unknown>")
            port = entry.get("port", "?")
            matched_rule = entry.get("matched_rule") or "-"
            print(
                f"  - {entry.get('timestamp', '<unknown>')}  "
                f"{entry.get('decision', 'unknown')}  {host}:{port}  "
                f"count={entry.get('count', 1)}  "
                f"reason={entry.get('reason', 'unknown')}  rule={matched_rule}"
            )
    else:
        print("  none")

    print("Auth broker decisions:")
    if auth_entries:
        for entry in auth_entries:
            upstream_status = entry.get("upstream_status")
            status = upstream_status if upstream_status is not None else "-"
            print(
                f"  - {entry.get('timestamp', '<unknown>')}  "
                f"{entry.get('broker', 'unknown')}  "
                f"{entry.get('decision', 'unknown')}  "
                f"{entry.get('method', '-')} {entry.get('path', '-')}  "
                f"status={status}  count={entry.get('count', 1)}  "
                f"reason={entry.get('reason', 'unknown')}"
            )
    else:
        print("  none")
    return 0


def find_run_record(run_id: str) -> dict[str, Any]:
    for record in reversed(read_run_records(limit=0)):
        if record.get("run_id") == run_id:
            return record
    raise ValueError(f"run record not found: {run_id}")


def log_entries_for_run(entries: Sequence[dict[str, Any]], run_id: str) -> list[dict[str, Any]]:
    return [entry for entry in entries if entry.get("run_id") == run_id]


def read_run_records(*, limit: int) -> list[dict[str, Any]]:
    log_path = runs_log_path()
    if not log_path.exists():
        return []
    records: list[dict[str, Any]] = []
    for line in log_path.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        try:
            payload = json.loads(line)
        except json.JSONDecodeError:
            continue
        if isinstance(payload, dict):
            records.append(payload)
    if limit == 0:
        return records
    return records[-limit:]


def runs_log_path() -> Path:
    return runhaven_cache_root() / "runs.jsonl"


def write_provider_policy_log(
    plan: AgentRunPlan,
    decisions: Sequence[ProxyDecision],
    *,
    run_id: str,
) -> None:
    if not decisions:
        return
    log_path = egress_policy_log_path()
    log_path.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
    timestamp = datetime.now(UTC).isoformat().replace("+00:00", "Z")
    with log_path.open("a", encoding="utf-8") as log_file:
        for decision in decisions:
            payload = {
                "timestamp": timestamp,
                "run_id": run_id,
                "profile": plan.profile_name,
                "workspace": str(plan.workspace),
                "network": plan.network_mode,
                "host": decision.host,
                "port": decision.port,
                "decision": decision.decision,
                "reason": decision.reason,
                "matched_rule": decision.matched_rule,
                "count": decision.count,
            }
            log_file.write(json.dumps(payload, sort_keys=True) + "\n")


def egress_log(*, limit: int, json_output: bool) -> int:
    if limit < 0:
        raise ValueError("--limit must be 0 or greater")
    entries = read_egress_policy_log(limit=limit)
    if json_output:
        print(json.dumps(entries, indent=2, sort_keys=True))
        return 0
    if not entries:
        print("No RunHaven provider egress policy log entries found.")
        return 0
    for entry in entries:
        host = entry.get("host", "<unknown>")
        port = entry.get("port", "?")
        decision = entry.get("decision", "unknown")
        reason = entry.get("reason", "unknown")
        count = entry.get("count", 1)
        profile = entry.get("profile", "unknown")
        run_id = entry.get("run_id", "-")
        matched_rule = entry.get("matched_rule") or "-"
        print(
            f"{entry.get('timestamp', '<unknown>')}  {profile}  {decision}  "
            f"{host}:{port}  count={count}  reason={reason}  rule={matched_rule}  "
            f"run={run_id}"
        )
    return 0


def read_egress_policy_log(*, limit: int) -> list[dict[str, Any]]:
    log_path = egress_policy_log_path()
    if not log_path.exists():
        return []
    entries: list[dict[str, Any]] = []
    for line in log_path.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        try:
            payload = json.loads(line)
        except json.JSONDecodeError:
            continue
        if isinstance(payload, dict):
            entries.append(payload)
    if limit == 0:
        return entries
    return entries[-limit:]


def egress_policy_log_path() -> Path:
    return runhaven_cache_root() / "egress-policy.jsonl"


def write_auth_broker_log(
    plan: AgentRunPlan,
    decisions: Sequence[BrokerDecision],
    *,
    run_id: str,
    return_code: int,
) -> None:
    log_path = auth_broker_log_path()
    log_path.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
    timestamp = datetime.now(UTC).isoformat().replace("+00:00", "Z")
    entries = decisions or (
        BrokerDecision(
            method="-",
            path="-",
            decision="denied",
            reason="run-complete",
            upstream_status=None,
            count=0,
        ),
    )
    with log_path.open("a", encoding="utf-8") as log_file:
        for decision in entries:
            payload = {
                "timestamp": timestamp,
                "run_id": run_id,
                "profile": plan.profile_name,
                "workspace": str(plan.workspace),
                "network": plan.network_mode,
                "broker": "codex-api-key",
                "method": decision.method,
                "path": decision.path,
                "decision": "no-requests" if not decisions else decision.decision,
                "reason": decision.reason,
                "upstream_status": decision.upstream_status,
                "count": decision.count,
                "return_code": return_code,
            }
            log_file.write(json.dumps(payload, sort_keys=True) + "\n")


def auth_log(*, limit: int, json_output: bool) -> int:
    if limit < 0:
        raise ValueError("--limit must be 0 or greater")
    entries = read_auth_broker_log(limit=limit)
    if json_output:
        print(json.dumps(entries, indent=2, sort_keys=True))
        return 0
    if not entries:
        print("No RunHaven auth broker log entries found.")
        return 0
    for entry in entries:
        broker = entry.get("broker", "unknown")
        decision = entry.get("decision", "unknown")
        reason = entry.get("reason", "unknown")
        method = entry.get("method", "-")
        path = entry.get("path", "-")
        count = entry.get("count", 1)
        profile = entry.get("profile", "unknown")
        run_id = entry.get("run_id", "-")
        upstream_status = entry.get("upstream_status")
        status = upstream_status if upstream_status is not None else "-"
        print(
            f"{entry.get('timestamp', '<unknown>')}  {profile}  {broker}  "
            f"{decision}  {method} {path}  status={status}  count={count}  "
            f"reason={reason}  run={run_id}"
        )
    return 0


def read_auth_broker_log(*, limit: int) -> list[dict[str, Any]]:
    log_path = auth_broker_log_path()
    if not log_path.exists():
        return []
    entries: list[dict[str, Any]] = []
    for line in log_path.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        try:
            payload = json.loads(line)
        except json.JSONDecodeError:
            continue
        if isinstance(payload, dict):
            entries.append(payload)
    if limit == 0:
        return entries
    return entries[-limit:]


def auth_broker_log_path() -> Path:
    return runhaven_cache_root() / "auth-broker.jsonl"


def auth_status(*, json_output: bool) -> int:
    profiles = auth_broker_profiles()
    payload = {
        "status": AUTH_BROKER_STATUS,
        "runtime": AUTH_BROKER_RUNTIME,
        "credential_stores_inspected": False,
        "environment_values_inspected": False,
        "secrets_printed": False,
        "profiles": [profile.to_json() for profile in profiles],
    }
    if json_output:
        print(json.dumps(payload, indent=2, sort_keys=True))
        return 0

    print(f"Auth broker: {AUTH_BROKER_STATUS}")
    print(f"Runtime: {AUTH_BROKER_RUNTIME}")
    print("Credential stores inspected: no")
    print("Environment values inspected: no")
    print("Secrets printed: no")
    print("Profiles:")
    width = max(len(profile.name) for profile in profiles)
    for profile in profiles:
        print(f"  {profile.name:<{width}}  {profile.status}")
    print("Current safe paths:")
    print("  - authenticate inside the isolated agent state volume when interactive")
    print("  - use the Codex API-key broker for headless Codex API-key runs")
    print("  - pass one token with --env NAME only when explicitly needed")
    print("  - use --network provider to constrain provider egress separately")
    return 0


def auth_explain(agent: str, *, json_output: bool) -> int:
    profile = get_profile(agent)
    auth_profile = get_auth_broker_profile(profile.name)
    payload = {
        **auth_profile.to_json(),
        "runtime": AUTH_BROKER_RUNTIME,
        "credential_stores_inspected": False,
        "environment_values_inspected": False,
        "secrets_printed": False,
        "provider_hosts": profile.provider_hosts,
    }
    if json_output:
        print(json.dumps(payload, indent=2, sort_keys=True))
        return 0

    print(f"Profile: {profile.name}")
    print(f"Auth broker: {auth_profile.status}")
    print(f"Runtime: {AUTH_BROKER_RUNTIME}")
    print("Credential stores inspected: no")
    print("Environment values inspected: no")
    print("Secrets printed: no")
    print("Supported auth surfaces:")
    for item in auth_profile.supported_auth:
        print(f"  - {item}")
    print("Host keeps:")
    for item in auth_profile.host_keeps:
        print(f"  - {item}")
    print("Guest receives:")
    for item in auth_profile.guest_receives:
        print(f"  - {item}")
    if profile.provider_hosts:
        print(f"Provider hosts: {', '.join(profile.provider_hosts)}")
    else:
        print("Provider hosts: none bundled")
    print(f"Current safe path: {auth_profile.current_safe_path}")
    if auth_profile.notes:
        print("Notes:")
        for note in auth_profile.notes:
            print(f"  - {note}")
    return 0


def why_host(host: str, *, port: int, agent: str | None) -> int:
    if port < 1 or port > 65535:
        raise ValueError("--port must be between 1 and 65535")
    normalized = normalize_host(host)
    print(f"Host: {normalized}")
    print(f"Port: {port}")
    if is_ip_literal(normalized):
        print("Provider mode: denied")
        print("Reason: IP literal targets cannot be allowed in provider mode.")
        print("Next action: use a reviewed fully qualified provider hostname instead.")
        return 0
    if "." not in normalized:
        print("Provider mode: denied")
        print("Reason: provider hosts must be fully qualified, not single-label names.")
        print("Next action: use a specific hostname such as api.example.com.")
        return 0

    if agent is not None:
        profile = get_profile(agent)
        print(f"Provider profile: {profile.name}")
        if not profile.provider_hosts:
            print("Provider mode: no bundled provider hosts are defined for this profile.")
            print("Next action: use --provider-host only after reviewing a fully qualified host.")
            return 0
        policy = EgressPolicy(profile.provider_hosts)
        matched_rule = policy.match_rule(normalized, port)
        if matched_rule is not None:
            print("Provider mode: allowed by bundled provider profile")
            print(f"Matched rule: {matched_rule}")
            print("DNS safety: checked at runtime before the proxy opens the connection.")
            return 0
        print("Provider mode: not allowed by bundled provider profile")
        print(f"Bundled hosts: {', '.join(profile.provider_hosts)}")
        print_endpoint_matches(match_provider_endpoints(normalized, profile=profile.name))
        print(f"Next action: review before rerunning with --provider-host {normalized}.")
        return 0

    matches: list[str] = []
    for profile in PROFILES.values():
        if not profile.provider_hosts:
            continue
        policy = EgressPolicy(profile.provider_hosts)
        matched_rule = policy.match_rule(normalized, port)
        if matched_rule is not None:
            matches.append(f"{profile.name} ({matched_rule})")
    if matches:
        print("Provider mode: allowed by bundled profile(s)")
        print(f"Matches: {', '.join(matches)}")
    else:
        print("Provider mode: not allowed by any bundled provider profile")
        print_endpoint_matches(match_provider_endpoints(normalized))
        print(f"Next action: review before rerunning with --provider-host {normalized}.")
    print("DNS safety: checked at runtime before the proxy opens the connection.")
    return 0


def print_endpoint_matches(matches: Sequence[ProviderEndpoint]) -> None:
    if not matches:
        return
    print("Known endpoint record(s):")
    for endpoint in matches:
        print(f"  - {endpoint.profile}: {endpoint.status}; {endpoint.purpose}")
        if endpoint.note:
            print(f"    Note: {endpoint.note}")


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


def create_codex_api_key_broker(
    api_key: str,
    network_info: InternalNetworkInfo,
) -> CodexApiKeyBrokerProxy:
    try:
        return CodexApiKeyBrokerProxy(
            (network_info.ipv4_gateway, 0),
            api_key=api_key,
            allowed_client_subnets=(network_info.ipv4_subnet,),
        )
    except OSError:
        return CodexApiKeyBrokerProxy(
            ("0.0.0.0", 0),
            api_key=api_key,
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


def delete_container_network(name: str) -> int:
    return subprocess.run(("container", "network", "delete", name), check=False).returncode


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
    return runhaven_cache_root() / "locks" / f"{state_volume}.lock"


def runhaven_cache_root() -> Path:
    override = os.environ.get("RUNHAVEN_CACHE_HOME")
    if override:
        return Path(override)
    return Path.home() / "Library" / "Caches" / "runhaven"


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
