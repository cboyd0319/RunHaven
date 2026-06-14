from __future__ import annotations

import shlex
from dataclasses import dataclass
from importlib.resources import files
from pathlib import Path

from .plans import validate_image_reference
from .profiles import AgentProfile


@dataclass(frozen=True)
class ImageBuildPlan:
    command: tuple[str, ...]
    context: Path
    containerfile: Path
    tag: str

    def shell_command(self) -> str:
        return shlex.join(self.command)


def build_image_plan(profile: AgentProfile, *, tag: str | None = None) -> ImageBuildPlan:
    if profile.image_context is None:
        raise ValueError(f"agent {profile.name!r} does not have a bundled image template")

    context = files("runhaven").joinpath("images")
    context_path = Path(str(context))
    containerfile = context_path / profile.image_context / "Containerfile"
    if not containerfile.exists():
        raise ValueError(f"missing bundled Containerfile for agent {profile.name!r}")

    image_tag = tag or profile.image
    validate_image_reference(image_tag, "image tag")
    command = (
        "container",
        "build",
        "-t",
        image_tag,
        "-f",
        str(containerfile),
        str(context_path),
    )
    return ImageBuildPlan(
        command=command,
        context=context_path,
        containerfile=containerfile,
        tag=image_tag,
    )
