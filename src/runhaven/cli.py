from __future__ import annotations

import argparse
import importlib
import os
import shutil
import subprocess
import sys
from collections.abc import Iterator, Sequence
from contextlib import contextmanager
from pathlib import Path
from typing import Any, TextIO

from .doctor import collect_checks
from .images import build_image_plan
from .plans import AgentRunPlan, RunOptions, build_run_plan
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

    plan_parser = subcommands.add_parser("plan", help="print the Apple container run plan")
    add_run_arguments(plan_parser)

    run_parser = subcommands.add_parser("run", help="run an agent through Apple container")
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

    return parser


def add_run_arguments(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("agent", choices=sorted(PROFILES), help="agent profile to run")
    parser.add_argument(
        "--workspace",
        type=Path,
        default=Path.cwd(),
        help="host project directory to mount at /workspace",
    )
    parser.add_argument("--image", help="override the profile image")
    parser.add_argument("--cpus", default="4", help="virtual CPUs for the container")
    parser.add_argument("--memory", default="4g", help="memory limit for the container")
    parser.add_argument(
        "--network",
        choices=("internet", "internal"),
        default="internet",
        help="internet uses the default network; internal creates a host-only network",
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


def make_run_plan(args: argparse.Namespace) -> AgentRunPlan:
    profile = get_profile(args.agent)
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
        )
    )


def print_run_plan(plan: AgentRunPlan) -> None:
    print(f"Workspace: {plan.workspace}")
    print(f"State volume: {plan.state_volume}")
    print(f"Network: {plan.network_name or 'default internet network'}")
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


def ensure_internal_network(name: str) -> None:
    existing = subprocess.run(
        ("container", "network", "inspect", name),
        check=False,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    if existing.returncode == 0:
        return

    created = subprocess.run(("container", "network", "create", "--internal", name), check=False)
    if created.returncode != 0:
        raise SystemExit(created.returncode)


@contextmanager
def acquire_state_lock(state_volume: str) -> Iterator[None]:
    path = state_lock_path(state_volume)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.touch(exist_ok=True)
    with path.open("r+", encoding="utf-8") as lock_file:
        prepare_lock_file(lock_file)
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


def prepare_lock_file(lock_file: TextIO) -> None:
    if not is_windows():
        return
    lock_file.seek(0)
    if lock_file.read(1):
        lock_file.seek(0)
        return
    lock_file.write("\0")
    lock_file.flush()
    lock_file.seek(0)


def lock_state_file(lock_file: TextIO) -> None:
    if is_windows():
        msvcrt = lock_module("msvcrt")
        try:
            msvcrt.locking(lock_file.fileno(), msvcrt.LK_NBLCK, 1)
        except OSError as exc:
            raise BlockingIOError from exc
        return

    fcntl = lock_module("fcntl")
    fcntl.flock(lock_file, fcntl.LOCK_EX | fcntl.LOCK_NB)


def unlock_state_file(lock_file: TextIO) -> None:
    if is_windows():
        msvcrt = lock_module("msvcrt")
        lock_file.seek(0)
        msvcrt.locking(lock_file.fileno(), msvcrt.LK_UNLCK, 1)
        return

    fcntl = lock_module("fcntl")
    fcntl.flock(lock_file, fcntl.LOCK_UN)


def lock_module(name: str) -> Any:
    return importlib.import_module(name)


def is_windows() -> bool:
    return sys.platform == "win32"


def state_lock_path(state_volume: str) -> Path:
    override = os.environ.get("RUNHAVEN_CACHE_HOME")
    if override:
        cache_root = Path(override)
    elif sys.platform == "darwin":
        cache_root = Path.home() / "Library" / "Caches" / "runhaven"
    else:
        cache_root = Path(os.environ.get("XDG_CACHE_HOME", Path.home() / ".cache"))
        cache_root = cache_root / "runhaven"
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
