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
from pathlib import Path as Path
from typing import TextIO

from . import provider_runtime
from .active_commands import (
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
from .cli_parser import build_parser
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
from .git_metadata import capture_git_snapshot, summarize_git_change
from .images import build_image_plan
from .plans import (
    AgentRunPlan,
    RunOptions,
    WorktreeRun,
    build_run_plan,
)
from .profiles import PROFILES, get_profile
from .provider_observability import utc_timestamp
from .run_history import (
    runs_diff,
    runs_list,
    runs_log,
    runs_show,
    write_run_record,
)
from .setup_guide import print_checks, print_setup_guide
from .worktree_lifecycle import (
    runs_worktree_discard,
    runs_worktree_keep,
    runs_worktree_merge,
    runs_worktree_recover,
)
from .worktrees import create_worktree_for_run, preview_worktree


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
    if args.worktree and args.dry_run:
        return print_worktree_dry_run(args)

    plan = make_run_plan(args)
    if args.worktree:
        provider_runtime.validate_runtime_auth_broker_environment(plan)
        require_container_cli()
        run_id = uuid.uuid4().hex
        worktree = create_worktree_for_run(
            args.workspace,
            workspace_scope=args.workspace_scope,
            allow_sensitive_workspace=args.allow_sensitive_workspace,
            profile_name=args.agent,
            run_id=run_id,
        )
        plan = make_run_plan(
            args,
            workspace_override=worktree.mounted_workspace,
            worktree=worktree,
            run_id=run_id,
        )
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
        run_id = plan.run_id or uuid.uuid4().hex
        git_before = capture_git_snapshot(plan.workspace)
        started_at = utc_timestamp()
        print(f"Run id: {run_id}", file=sys.stderr)
        write_active_run_record(plan, run_id=run_id, started_at=started_at)
        try:
            return_code = subprocess.call(plan.command)
        finally:
            terminal_status = active_run_terminal_status(run_id)
            remove_active_run_record(run_id)
        finished_at = utc_timestamp()
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
    if args.runs_command == "recover":
        return runs_worktree_recover(args.run_id)
    if args.runs_command == "merge":
        return runs_worktree_merge(args.run_id)
    if args.runs_command == "keep":
        return runs_worktree_keep(args.run_id)
    if args.runs_command == "discard":
        return runs_worktree_discard(args.run_id)
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


def print_worktree_dry_run(args: argparse.Namespace) -> int:
    make_run_plan(args)
    source_workspace, repo_root, base_head = preview_worktree(
        args.workspace,
        workspace_scope=args.workspace_scope,
        allow_sensitive_workspace=args.allow_sensitive_workspace,
    )
    print("Worktree: enabled")
    print(f"Source workspace: {source_workspace}")
    print(f"Source repo root: {repo_root}")
    print(f"Base HEAD: {base_head}")
    print("RunHaven will create a branch named runhaven/<agent>/<run-id>.")
    print("RunHaven will keep the worktree after the run and record recovery commands.")
    return 0


def make_run_plan(
    args: argparse.Namespace,
    *,
    workspace_override: Path | None = None,
    worktree: WorktreeRun | None = None,
    run_id: str | None = None,
) -> AgentRunPlan:
    profile = get_profile(args.agent)
    tty = args.tty == "always" or (
        args.tty == "auto" and sys.stdin.isatty() and sys.stdout.isatty()
    )
    return build_run_plan(
        RunOptions(
            profile=profile,
            workspace=workspace_override or args.workspace,
            agent_args=tuple(args.agent_args),
            image=args.image,
            cpus=args.cpus,
            memory=args.memory,
            network=args.network,
            workspace_scope=args.workspace_scope,
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
            worktree=worktree,
            run_id=run_id,
        )
    )


def print_run_plan(plan: AgentRunPlan) -> None:
    print(f"Workspace: {plan.workspace}")
    print(f"Workspace scope: {plan.workspace_scope}")
    if plan.workspace_scope_note:
        print(f"Workspace note: {plan.workspace_scope_note}")
    if plan.worktree is not None:
        print(f"Worktree: {plan.worktree.worktree_root}")
        print(f"Worktree branch: {plan.worktree.branch}")
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
