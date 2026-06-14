from __future__ import annotations

import hashlib
import os
import re
import shlex
from collections.abc import Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import Literal

from .profiles import AgentProfile

NetworkMode = Literal["internet", "internal"]

_ENV_NAME_RE = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*$")
_SAFE_NAME_RE = re.compile(r"[^a-z0-9_.-]+")
DEFAULT_ENV_PASSTHROUGH = ("TERM", "COLORTERM", "LANG", "LC_ALL", "NO_COLOR")
CONTAINER_PATH = (
    "/opt/runhaven-agent/node_modules/.bin:"
    "/home/agent/.local/bin:"
    "/usr/local/bin:/usr/bin:/bin"
)
VOLUME_PREP_IMAGE = (
    "debian:trixie-slim@"
    "sha256:4e401d95de7083948053197a9c3913343cd06b706bf15eb6a0c3ccd26f436a0e"
)
VOLUME_PREP_NETWORK = "runhaven-volume-prep-internal"


@dataclass(frozen=True)
class RunOptions:
    profile: AgentProfile
    workspace: Path
    agent_args: tuple[str, ...] = ()
    image: str | None = None
    cpus: str = "4"
    memory: str = "4g"
    network: NetworkMode = "internet"
    read_only_workspace: bool = False
    ssh: bool = False
    env: tuple[str, ...] = ()
    user: str = "agent"


@dataclass(frozen=True)
class AgentRunPlan:
    command: tuple[str, ...]
    preflight: tuple[tuple[str, ...], ...]
    workspace: Path
    state_volume: str
    network_name: str | None

    def shell_command(self) -> str:
        return shlex.join(self.command)

    def shell_preflight(self) -> tuple[str, ...]:
        return tuple(shlex.join(command) for command in self.preflight)


def build_run_plan(options: RunOptions) -> AgentRunPlan:
    workspace = options.workspace.expanduser().resolve()
    if not workspace.exists():
        raise ValueError(f"workspace does not exist: {workspace}")
    if not workspace.is_dir():
        raise ValueError(f"workspace is not a directory: {workspace}")

    for name in options.env:
        validate_env_name(name)

    project_id = project_identifier(workspace)
    state_volume = safe_resource_name(f"runhaven-{options.profile.name}-{project_id}-home")
    network_name = safe_resource_name(f"runhaven-{project_id}-internal")
    image = options.image or options.profile.image

    command: list[str] = [
        "container",
        "run",
        "--rm",
        "--init",
        "--read-only",
        "--tmpfs",
        "/tmp",
        "--cap-drop",
        "ALL",
        "--cpus",
        options.cpus,
        "--memory",
        options.memory,
        "--user",
        options.user,
        "--workdir",
        "/workspace",
        "--mount",
        bind_mount(workspace, "/workspace", read_only=options.read_only_workspace),
        "--mount",
        volume_mount(state_volume, "/home/agent"),
        "--env",
        "HOME=/home/agent",
        "--env",
        f"PATH={CONTAINER_PATH}",
    ]

    for name in DEFAULT_ENV_PASSTHROUGH:
        if name in os.environ:
            command.extend(("--env", name))

    for key, value in options.profile.env().items():
        command.extend(("--env", f"{key}={value}"))

    for name in options.env:
        command.extend(("--env", name))

    preflight: list[tuple[str, ...]] = []
    if options.user == "agent":
        preflight.append(("container", "network", "create", "--internal", VOLUME_PREP_NETWORK))
        home_setup = home_setup_command(options.profile)
        preflight.append(
            (
                "container",
                "run",
                "--rm",
                "--init",
                "--read-only",
                "--no-dns",
                "--network",
                VOLUME_PREP_NETWORK,
                "--cpus",
                "1",
                "--memory",
                "256m",
                "--user",
                "root",
                "--entrypoint",
                "/bin/sh",
                "--mount",
                volume_mount(state_volume, "/home/agent"),
                VOLUME_PREP_IMAGE,
                "-c",
                home_setup,
            )
        )

    active_network: str | None = None
    if options.network == "internal":
        active_network = network_name
        preflight.append(("container", "network", "create", "--internal", network_name))
        command.extend(("--network", network_name))

    if options.ssh:
        command.append("--ssh")

    agent_command = strip_remainder_separator(options.agent_args)
    if not agent_command:
        agent_command = options.profile.command

    command.append(image)
    command.extend(agent_command)

    return AgentRunPlan(
        command=tuple(command),
        preflight=tuple(preflight),
        workspace=workspace,
        state_volume=state_volume,
        network_name=active_network,
    )


def validate_env_name(name: str) -> None:
    if "=" in name:
        raise ValueError("pass only environment variable names, not NAME=value pairs")
    if not _ENV_NAME_RE.match(name):
        raise ValueError(f"invalid environment variable name: {name!r}")


def bind_mount(source: Path, target: str, *, read_only: bool) -> str:
    parts = ["type=bind", f"source={source}", f"target={target}"]
    if read_only:
        parts.append("readonly")
    return ",".join(parts)


def volume_mount(source: str, target: str) -> str:
    return f"type=volume,source={source},target={target}"


def home_setup_command(profile: AgentProfile) -> str:
    commands = ["chown 1000:1000 /home/agent", "chmod 700 /home/agent"]
    for value in profile.env().values():
        if value.startswith("/home/agent/"):
            quoted = shlex.quote(value)
            commands.append(f"mkdir -p {quoted}")
            commands.append(f"chown -R 1000:1000 {quoted}")
    return " && ".join(commands)


def project_identifier(workspace: Path) -> str:
    digest = hashlib.sha256(str(workspace).encode("utf-8")).hexdigest()
    return digest[:16]


def safe_resource_name(value: str) -> str:
    normalized = _SAFE_NAME_RE.sub("-", value.lower()).strip("-")
    return normalized[:63] or "runhaven"


def strip_remainder_separator(args: Sequence[str]) -> tuple[str, ...]:
    if args and args[0] == "--":
        return tuple(args[1:])
    return tuple(args)
