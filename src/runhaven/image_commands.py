from __future__ import annotations

import json
import subprocess
from collections.abc import Callable
from dataclasses import dataclass
from typing import Protocol

from .profiles import PROFILES, AgentProfile, get_profile


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


@dataclass(frozen=True)
class ImageDoctorEntry:
    profile: AgentProfile
    present: bool


def image_doctor(
    agent: str | None,
    *,
    require_container: Callable[[], None],
    run_container: ContainerRunner,
) -> int:
    require_container()
    local_names = list_local_image_names(run_container=run_container)
    entries = tuple(
        ImageDoctorEntry(profile=profile, present=image_is_present(profile.image, local_names))
        for profile in selected_profiles(agent)
    )

    print("Image doctor")
    for entry in entries:
        status = "ok" if entry.present else "missing"
        print(f"{status} {entry.profile.name}: {entry.profile.image}")
        if not entry.present:
            print(f"fix: runhaven image rebuild {entry.profile.name}")
    print_preflight_recovery(agent)
    return 0 if all(entry.present for entry in entries) else 1


def selected_profiles(agent: str | None) -> tuple[AgentProfile, ...]:
    if agent is not None:
        return (get_profile(agent),)
    return tuple(PROFILES[name] for name in sorted(PROFILES))


def image_is_present(image: str, local_names: set[str]) -> bool:
    return any(name in local_names for name in candidate_image_names(image))


def candidate_image_names(image: str) -> tuple[str, ...]:
    return (image, f"docker.io/{image}")


def list_local_image_names(*, run_container: ContainerRunner) -> set[str]:
    result = run_container(
        ("container", "image", "list", "--format", "json"),
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        raise SystemExit(result.returncode)
    try:
        payload = json.loads(result.stdout)
    except json.JSONDecodeError as exc:
        raise ValueError("could not parse Apple container image list JSON") from exc
    if not isinstance(payload, list):
        raise ValueError("could not parse Apple container image list JSON")

    names: set[str] = set()
    for item in payload:
        if not isinstance(item, dict):
            continue
        configuration = item.get("configuration")
        if not isinstance(configuration, dict):
            continue
        name = configuration.get("name")
        if isinstance(name, str):
            names.add(name)
        descriptor = configuration.get("descriptor")
        if not isinstance(descriptor, dict):
            continue
        annotations = descriptor.get("annotations")
        if not isinstance(annotations, dict):
            continue
        for value in annotations.values():
            if isinstance(value, str):
                names.add(value)
    return names


def print_preflight_recovery(agent: str | None) -> None:
    agent_label = agent or "AGENT"
    print("Preflight recovery")
    print(f"- Rebuild a missing or stale bundled image: runhaven image rebuild {agent_label}")
    print("- Inspect RunHaven-managed networks: runhaven network list")
    print("- Remove stale managed networks after review: runhaven network prune --yes")
    print(
        "- Reset interrupted isolated home state only when you want to discard it: "
        f"runhaven state reset {agent_label} --workspace PATH --yes"
    )
