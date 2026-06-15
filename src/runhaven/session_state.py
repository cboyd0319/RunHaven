from __future__ import annotations

import hashlib
import re

SESSION_DEFAULT = "default"
SESSION_MARKER = "-s-"
_SESSION_RE = re.compile(r"^[a-z0-9][a-z0-9_.-]{0,62}$")


def validate_session_name(name: str) -> str:
    if not _SESSION_RE.match(name):
        raise ValueError(
            "invalid session name: use lowercase letters, numbers, dots, underscores, "
            "or dashes"
        )
    if name == SESSION_DEFAULT:
        raise ValueError("invalid session name: 'default' is reserved")
    return name


def state_volume_name(profile_name: str, project_id: str, session: str | None) -> str:
    if session is None:
        return f"runhaven-{profile_name}-{project_id}-home"
    session = validate_session_name(session)
    digest = session_digest(session)
    prefix = f"runhaven-{profile_name}-{project_id}{SESSION_MARKER}"
    suffix = f"-{digest}-home"
    budget = 63 - len(prefix) - len(suffix)
    if budget < 1:
        raise ValueError("session state volume name would be too long")
    return f"{prefix}{session[:budget]}{suffix}"


def session_digest(session: str) -> str:
    return hashlib.sha256(session.encode("utf-8")).hexdigest()[:8]


def volume_matches_session(volume: str, session: str | None) -> bool:
    if not is_runhaven_state_volume(volume):
        return False
    if session is None:
        return True
    session = validate_session_name(session)
    return f"{SESSION_MARKER}{session}-" in volume or volume.endswith(
        f"-{session_digest(session)}-home"
    )


def is_runhaven_state_volume(volume: str) -> bool:
    return volume.startswith("runhaven-") and volume.endswith("-home")
