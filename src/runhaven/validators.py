from __future__ import annotations


def require_string(value: object, message: str) -> str:
    if not isinstance(value, str) or not value:
        raise ValueError(message)
    return value


def validate_run_id(run_id: str) -> None:
    if not run_id or run_id.startswith("-"):
        raise ValueError(f"invalid run id: {run_id!r}")
    if any(character.isspace() or character in "/\\" for character in run_id):
        raise ValueError(f"invalid run id: {run_id!r}")


def validate_runhaven_container_name(container_name: str) -> None:
    if not container_name.startswith("runhaven-"):
        raise ValueError(
            f"active run container {container_name!r} is not a RunHaven-owned container"
        )
    if container_name.startswith("-"):
        raise ValueError(f"invalid active run container name: {container_name!r}")
    if any(character.isspace() or character in "/\\," for character in container_name):
        raise ValueError(f"invalid active run container name: {container_name!r}")
