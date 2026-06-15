from __future__ import annotations

import re
import subprocess
from collections.abc import Callable
from typing import Protocol

from .plans import VOLUME_PREP_NETWORK

_PROJECT_ID = r"[0-9a-f]{16}"
_INTERNAL_NETWORK_RE = re.compile(rf"^runhaven-{_PROJECT_ID}-internal$")
_PROVIDER_NETWORK_RE = re.compile(
    rf"^runhaven-(?:antigravity|claude|codex|copilot|gemini|shell)-{_PROJECT_ID}-provider$"
)


class ContainerRunner(Protocol):
    def __call__(
        self,
        args: tuple[str, ...],
        *,
        check: bool = False,
        capture_output: bool = False,
        text: bool = False,
    ) -> subprocess.CompletedProcess[str]:
        ...


def network_list(
    *,
    require_container: Callable[[], None],
    run_container: ContainerRunner,
) -> int:
    require_container()
    networks = list_managed_networks(run_container=run_container)
    if not networks:
        print("No RunHaven managed networks found.")
        return 0
    for network in networks:
        print(network)
    return 0


def network_prune(
    *,
    confirm: bool,
    require_container: Callable[[], None],
    run_container: ContainerRunner,
) -> int:
    require_container()
    networks = list_managed_networks(run_container=run_container)
    if not networks:
        print("No RunHaven managed networks found.")
        return 0
    if not confirm:
        for network in networks:
            print(network)
        print("Rerun with --yes to delete these networks.")
        return 2
    for network in networks:
        result = run_container(("container", "network", "delete", network), check=False)
        if result.returncode != 0:
            return result.returncode
    return 0


def list_managed_networks(*, run_container: ContainerRunner) -> tuple[str, ...]:
    result = run_container(
        ("container", "network", "list", "--quiet"),
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        raise SystemExit(result.returncode)
    return tuple(
        network
        for network in (line.strip() for line in result.stdout.splitlines())
        if is_runhaven_managed_network(network)
    )


def is_runhaven_managed_network(name: str) -> bool:
    if name == VOLUME_PREP_NETWORK:
        return True
    return bool(_INTERNAL_NETWORK_RE.fullmatch(name) or _PROVIDER_NETWORK_RE.fullmatch(name))
