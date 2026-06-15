from __future__ import annotations

import argparse
import fcntl
import os
import shutil
import subprocess
import sys
import threading
import uuid
from collections.abc import Iterator, Sequence
from contextlib import contextmanager
from pathlib import Path
from typing import TextIO

from . import provider_runtime
from .active_commands import (
    DEFAULT_LOG_FOLLOW_LINES,
    runs_active,
    runs_attach,
    runs_kill,
    runs_logs_follow,
    runs_repair,
    runs_status,
    runs_stop,
)
from .active_records import (
    active_run_terminal_status,
    remove_active_run_record,
    write_active_run_record,
)
from .auth_broker import CodexApiKeyBrokerProxy
from .cache_paths import state_lock_path
from .diagnostic_commands import (
    auth_explain,
    auth_log,
    auth_status,
    egress_log,
    read_auth_broker_log,
    read_egress_policy_log,
    why_host,
)
from .doctor import collect_checks
from .egress import (
    EgressPolicy,
    ThreadedAllowlistProxy,
)
from .images import build_image_plan
from .plans import (
    SUPPORTED_NETWORK_MODES,
    AgentRunPlan,
    RunOptions,
    build_run_plan,
)
from .profiles import PROFILES, get_profile
from .run_history import (
    capture_git_snapshot,
    runs_diff,
    runs_list,
    runs_log,
    runs_show,
    summarize_git_change,
    write_run_record,
)
from .setup_guide import print_checks, print_setup_guide


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
        if args.command == "setup":
            return setup(args.agent)
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
    setup_parser = subcommands.add_parser(
        "setup",
        help="guide first-run prerequisites and next commands",
    )
    setup_parser.add_argument(
        "--agent",
        choices=sorted(PROFILES),
        default="claude",
        help="agent profile to prepare; defaults to claude",
    )

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
    runs_status_parser = runs_subcommands.add_parser(
        "status",
        help="show sanitized status for an active RunHaven run",
    )
    runs_status_parser.add_argument("run_id", help="active run id to inspect")
    runs_status_parser.add_argument("--json", action="store_true", help="print JSON output")
    runs_attach_parser = runs_subcommands.add_parser(
        "attach",
        help="open a shell or command in an active RunHaven run",
        epilog="Use -- before a custom command, for example: runhaven runs attach RUN_ID -- pwd",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    runs_attach_parser.add_argument("run_id", help="active run id to attach to")
    runs_attach_parser.add_argument(
        "--user",
        default="agent",
        help="container user for the attached process; defaults to non-root agent",
    )
    runs_attach_parser.add_argument(
        "--allow-root-user",
        action="store_true",
        help="allow attaching as root inside the active container",
    )
    runs_attach_parser.add_argument(
        "--workdir",
        default="/workspace",
        help="container working directory for the attached process",
    )
    runs_attach_parser.add_argument(
        "--tty",
        choices=("auto", "always", "never"),
        default="auto",
        help="allocate a TTY for the attached process; auto follows the current terminal",
    )
    runs_logs_follow_parser = runs_subcommands.add_parser(
        "logs-follow",
        help="follow logs from an active RunHaven run",
    )
    runs_logs_follow_parser.add_argument("run_id", help="active run id to follow")
    runs_logs_follow_parser.add_argument(
        "--lines",
        type=int,
        default=DEFAULT_LOG_FOLLOW_LINES,
        help="recent log lines to show before following",
    )
    runs_stop_parser = runs_subcommands.add_parser(
        "stop",
        help="stop an active RunHaven run",
    )
    runs_stop_parser.add_argument("run_id", help="active run id to stop")
    runs_kill_parser = runs_subcommands.add_parser(
        "kill",
        help="hard-stop an active RunHaven run",
    )
    runs_kill_parser.add_argument("run_id", help="active run id to kill")
    runs_repair_parser = runs_subcommands.add_parser(
        "repair",
        help="remove a stale active marker after confirming its container is gone",
    )
    runs_repair_parser.add_argument("run_id", nargs="?", help="active run id to repair")
    runs_repair_parser.add_argument(
        "--all",
        action="store_true",
        help="inspect all active markers and remove only confirmed-stale markers",
    )
    runs_repair_parser.add_argument("--json", action="store_true", help="print JSON output")

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
    print_checks(checks)
    return 0 if all(check.ok for check in checks) else 1


def setup(agent: str) -> int:
    return print_setup_guide(agent, collect_checks())


def plan_run(args: argparse.Namespace) -> int:
    plan = make_run_plan(args)
    print_run_plan(plan)
    return 0


def run_agent(args: argparse.Namespace) -> int:
    plan = make_run_plan(args)
    if args.dry_run:
        print_run_plan(plan)
        return 0

    provider_runtime.validate_runtime_auth_broker_environment(plan)
    require_container_cli()
    with acquire_state_lock(plan.state_volume):
        if plan.network_mode == "provider":
            return run_provider_agent(plan)
        for command in plan.preflight:
            run_preflight(command)
        run_id = uuid.uuid4().hex
        git_before = capture_git_snapshot(plan.workspace)
        started_at = provider_runtime.utc_timestamp()
        print(f"Run id: {run_id}", file=sys.stderr)
        write_active_run_record(plan, run_id=run_id, started_at=started_at)
        try:
            return_code = subprocess.call(plan.command)
        finally:
            terminal_status = active_run_terminal_status(run_id)
            remove_active_run_record(run_id)
        finished_at = provider_runtime.utc_timestamp()
        git = summarize_git_change(git_before, capture_git_snapshot(plan.workspace))
        write_run_record(
            plan,
            run_id=run_id,
            started_at=started_at,
            finished_at=finished_at,
            return_code=return_code,
            status=terminal_status,
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
        return runs_log(
            args.run_id,
            json_output=args.json,
            read_provider_log=lambda: read_egress_policy_log(limit=0),
            read_auth_log=lambda: read_auth_broker_log(limit=0),
        )
    if args.runs_command == "diff":
        return runs_diff(args.run_id)
    if args.runs_command == "active":
        return runs_active(json_output=args.json)
    if args.runs_command == "status":
        return runs_status(
            args.run_id,
            json_output=args.json,
            require_container=require_container_cli,
            run_container=subprocess.run,
        )
    if args.runs_command == "attach":
        return runs_attach(
            args.run_id,
            user=args.user,
            workdir=args.workdir,
            tty_mode=args.tty,
            allow_root_user=args.allow_root_user,
            command_args=tuple(args.agent_args),
            require_container=require_container_cli,
            call_container=subprocess.call,
            stdin_is_tty=sys.stdin.isatty,
            stdout_is_tty=sys.stdout.isatty,
        )
    if args.runs_command == "logs-follow":
        return runs_logs_follow(
            args.run_id,
            lines=args.lines,
            require_container=require_container_cli,
            call_container=subprocess.call,
        )
    if args.runs_command == "stop":
        return runs_stop(
            args.run_id,
            require_container=require_container_cli,
            run_container=subprocess.run,
        )
    if args.runs_command == "kill":
        return runs_kill(
            args.run_id,
            require_container=require_container_cli,
            run_container=subprocess.run,
        )
    if args.runs_command == "repair":
        return runs_repair(
            args.run_id,
            repair_all=args.all,
            json_output=args.json,
            require_container=require_container_cli,
            run_container=subprocess.run,
        )
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


def run_provider_agent(plan: AgentRunPlan) -> int:
    return provider_runtime.run_provider_agent(
        plan,
        deps=provider_runtime.ProviderRuntimeDependencies(
            run_preflight=run_preflight,
            inspect_internal_network=inspect_internal_network,
            create_provider_proxy=create_provider_proxy,
            create_codex_api_key_broker=create_codex_api_key_broker,
            thread_factory=threading.Thread,
            call_container=subprocess.call,
            delete_container_network=delete_container_network,
        ),
    )


def create_provider_proxy(
    policy: EgressPolicy,
    network_info: provider_runtime.InternalNetworkInfo,
) -> ThreadedAllowlistProxy:
    return provider_runtime.create_provider_proxy(policy, network_info)


def create_codex_api_key_broker(
    api_key: str,
    network_info: provider_runtime.InternalNetworkInfo,
) -> CodexApiKeyBrokerProxy:
    return provider_runtime.create_codex_api_key_broker(api_key, network_info)


def inspect_internal_network(name: str) -> provider_runtime.InternalNetworkInfo:
    return provider_runtime.inspect_internal_network(name)


def delete_container_network(name: str) -> int:
    return provider_runtime.delete_container_network(name)


def ensure_internal_network(name: str) -> None:
    provider_runtime.ensure_internal_network(name)


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
