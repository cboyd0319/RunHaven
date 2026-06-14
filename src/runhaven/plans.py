from __future__ import annotations

import hashlib
import os
import re
import shlex
from collections.abc import Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import Literal

from .egress import is_ip_literal, normalize_host
from .profiles import AgentProfile

NetworkMode = Literal["internet", "internal", "provider"]
SUPPORTED_NETWORK_MODES = ("internet", "internal", "provider")
_ENV_NAME_RE = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*$")
_IMAGE_REF_RE = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._:/@+-]*$")
_CPUS_RE = re.compile(r"^[1-9][0-9]*(?:\.[0-9]+)?$")
_MEMORY_RE = re.compile(r"^[1-9][0-9]*(?:[KMGTPkmgmtp])?$")
_USER_RE = re.compile(r"^(?:[A-Za-z_][A-Za-z0-9_.-]*|[0-9]+)(?::[0-9]+)?$")
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
    interactive: bool = True
    tty: bool = True
    allow_sensitive_workspace: bool = False
    allow_root_user: bool = False
    provider_hosts: tuple[str, ...] = ()


@dataclass(frozen=True)
class AgentRunPlan:
    command: tuple[str, ...]
    preflight: tuple[tuple[str, ...], ...]
    workspace: Path
    state_volume: str
    network_name: str | None
    network_mode: NetworkMode
    egress_summary: str
    image: str
    provider_allowed_hosts: tuple[str, ...] = ()

    def shell_command(self) -> str:
        return shlex.join(self.command)

    def shell_preflight(self) -> tuple[str, ...]:
        return tuple(shlex.join(command) for command in self.preflight)


def build_run_plan(options: RunOptions) -> AgentRunPlan:
    try:
        workspace = options.workspace.expanduser().resolve()
    except OSError as exc:
        raise ValueError(f"could not resolve workspace path: {exc}") from exc
    if not workspace.exists():
        raise ValueError(f"workspace does not exist: {workspace}")
    if not workspace.is_dir():
        raise ValueError(f"workspace is not a directory: {workspace}")
    validate_workspace(workspace, allow_sensitive=options.allow_sensitive_workspace)
    validate_network_mode(options.network)

    for name in options.env:
        validate_env_name(name)
    validate_resource_options(options.cpus, options.memory, options.user)
    if uses_root_identity(options.user) and not options.allow_root_user:
        raise ValueError("root user or group requires --allow-root-user")
    provider_allowed_hosts = provider_hosts_for_options(options)

    project_id = project_identifier(workspace)
    state_volume = safe_resource_name(f"runhaven-{options.profile.name}-{project_id}-home")
    network_name = safe_resource_name(f"runhaven-{project_id}-internal")
    image = options.image or options.profile.image
    validate_image_reference(image, "image")

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
    if options.interactive:
        command.append("--interactive")
    if options.tty:
        command.append("--tty")

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
    elif options.network == "provider":
        active_network = safe_resource_name(
            f"runhaven-{options.profile.name}-{project_id}-provider"
        )
        preflight.append(("container", "network", "create", "--internal", active_network))
        command.extend(("--network", active_network))

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
        network_mode=options.network,
        egress_summary=network_egress_summary(options.network, provider_allowed_hosts),
        image=image,
        provider_allowed_hosts=provider_allowed_hosts,
    )


def provider_hosts_for_options(options: RunOptions) -> tuple[str, ...]:
    if options.network != "provider":
        return ()
    hosts = normalize_provider_hosts((*options.profile.provider_hosts, *options.provider_hosts))
    if not hosts:
        raise ValueError(
            "provider hosts are required for --network provider. "
            "Use a bundled provider profile or pass --provider-host HOST."
        )
    return hosts


def normalize_provider_hosts(hosts: Sequence[str]) -> tuple[str, ...]:
    normalized_hosts: list[str] = []
    seen: set[str] = set()
    for host in hosts:
        normalized = normalize_host(host)
        if is_ip_literal(normalized):
            raise ValueError("provider hosts cannot be IP literals")
        if normalized not in seen:
            normalized_hosts.append(normalized)
            seen.add(normalized)
    return tuple(normalized_hosts)


def validate_env_name(name: str) -> None:
    if "=" in name:
        raise ValueError("pass only environment variable names, not NAME=value pairs")
    if not _ENV_NAME_RE.match(name):
        raise ValueError(f"invalid environment variable name: {name!r}")


def validate_workspace(workspace: Path, *, allow_sensitive: bool) -> None:
    if "," in str(workspace):
        raise ValueError("workspace paths containing a comma cannot be mounted safely")
    if allow_sensitive:
        return

    root_paths, secret_paths = sensitive_workspace_paths()
    for path in root_paths:
        if workspace == path:
            raise ValueError(
                f"sensitive workspace requires --allow-sensitive-workspace: {workspace}"
            )
    for path in secret_paths:
        if workspace == path or workspace in path.parents or path in workspace.parents:
            raise ValueError(
                f"sensitive workspace requires --allow-sensitive-workspace: {workspace}"
            )


def sensitive_workspace_paths() -> tuple[tuple[Path, ...], tuple[Path, ...]]:
    home = Path.home().resolve()
    root_paths = tuple(
        path.resolve()
        for path in (
            Path("/"),
            Path("/Users"),
            Path("/private"),
            Path("/var"),
            home,
        )
    )
    secret_paths = (
        Path("/Applications"),
        Path("/Library"),
        Path("/System"),
        Path("/etc"),
        Path("/private/etc"),
        Path("/private/var/audit"),
        Path("/private/var/db"),
        Path("/private/var/log"),
        Path("/private/var/root"),
        Path("/private/var/run"),
        home / ".ssh",
        home / ".aws",
        home / ".azure",
        home / ".config" / "gcloud",
        home / ".docker",
        home / ".gnupg",
        home / ".kube",
        home / "Library" / "Application Support" / "Google" / "Chrome",
        home / "Library" / "Application Support" / "Firefox",
        home / "Library" / "Keychains",
    )
    return root_paths, tuple(path.resolve() for path in secret_paths)


def validate_resource_options(cpus: str, memory: str, user: str) -> None:
    if not _CPUS_RE.match(cpus):
        raise ValueError(f"invalid cpus value: {cpus!r}")
    if not _MEMORY_RE.match(memory):
        raise ValueError(f"invalid memory value: {memory!r}")
    if not _USER_RE.match(user):
        raise ValueError(f"invalid user value: {user!r}")


def validate_network_mode(network: str) -> None:
    if network not in SUPPORTED_NETWORK_MODES:
        raise ValueError(f"invalid network mode: {network!r}")


def network_egress_summary(
    network: NetworkMode,
    provider_allowed_hosts: Sequence[str] = (),
) -> str:
    if network == "internet":
        return "unrestricted internet egress; domain allowlisting is not enforced"
    if network == "internal":
        return "host-only internal network; internet egress disabled"
    hosts = ", ".join(provider_allowed_hosts)
    return f"provider allowlist egress through runtime proxy: {hosts}"


def uses_root_identity(user: str) -> bool:
    parts = user.split(":", maxsplit=1)
    for part in parts:
        if part == "root":
            return True
        if part.isdigit() and int(part) == 0:
            return True
    return False


def validate_image_reference(value: str, label: str) -> None:
    if not value or value.startswith("-"):
        raise ValueError(f"invalid {label}: {value!r}")
    if "," in value or any(character.isspace() for character in value):
        raise ValueError(f"invalid {label}: {value!r}")
    if "://" in value or not _IMAGE_REF_RE.match(value):
        raise ValueError(f"invalid {label}: {value!r}")


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
