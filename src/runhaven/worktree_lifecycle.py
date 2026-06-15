from __future__ import annotations

import json
import os
import shlex
import shutil
import subprocess
from collections.abc import Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import cast

from .git_metadata import capture_git_snapshot, parse_git_status_entries, safe_repo_path
from .project_checks import SuggestedCheck, suggest_project_checks
from .run_history import find_run_record
from .validators import require_string, validate_run_id


@dataclass(frozen=True)
class WorktreeLifecycle:
    run_id: str
    source_repo_root: Path
    worktree_root: Path
    mounted_workspace: Path
    branch: str
    base_head: str


def runs_worktree_keep(run_id: str) -> int:
    lifecycle = load_worktree_lifecycle(run_id)
    verify_lifecycle(lifecycle)
    suggested_checks = suggest_project_checks(lifecycle.mounted_workspace)
    print(f"Worktree kept for run {run_id}")
    print(f"Source repo: {lifecycle.source_repo_root}")
    print(f"Worktree: {lifecycle.worktree_root}")
    print(f"Mounted workspace: {lifecycle.mounted_workspace}")
    print(f"Branch: {lifecycle.branch}")
    print(f"Review: runhaven runs diff {run_id}")
    print(f"Recover: runhaven runs recover {run_id}")
    print(f"Merge: runhaven runs merge {run_id}")
    print(f"Discard: runhaven runs discard {run_id}")
    print_suggested_checks(suggested_checks)
    return 0


def runs_worktree_recover(run_id: str, *, json_output: bool = False) -> int:
    lifecycle = load_worktree_lifecycle(run_id)
    verify_lifecycle(lifecycle)
    payload = worktree_recovery_payload(lifecycle)

    if json_output:
        print(json.dumps(payload, indent=2, sort_keys=True))
        return 0

    commands = cast(dict[str, str], payload["commands"])
    next_steps = cast(list[str], payload["next_steps"])
    source_status = cast(list[str], payload["source_status"])
    worktree_status = cast(list[str], payload["worktree_status"])
    suggested_checks = cast(list[SuggestedCheck], payload["suggested_checks"])
    print(f"Manual recovery for worktree run {run_id}")
    print(f"Source repo: {payload['source_repo_root']}")
    print(f"Worktree: {payload['worktree_root']}")
    print(f"Mounted workspace: {payload['mounted_workspace']}")
    print(f"Branch: {payload['branch']}")
    print(f"Base HEAD: {payload['base_head']}")
    print(f"Source HEAD: {payload['source_head']}")
    print(f"Worktree HEAD: {payload['worktree_head']}")
    print_status("Source status", source_status)
    print_status("Worktree status", worktree_status)

    print("Manual recovery steps:")
    print(f"1. {next_steps[0]}: {commands['diff']}")
    print(f"2. {next_steps[1]}: {commands['source_status']}")
    print("   Commit, stash, or remove source-local changes before retrying.")
    print(f"3. {next_steps[2]}: {commands['worktree_status']}")
    print("   Resolve conflicts or commit finished work in the worktree if needed.")
    print(f"4. {next_steps[3]}: {commands['merge']}")
    print(f"5. {next_steps[4]}: {commands['keep']}")
    print(f"6. {next_steps[5]}: {commands['discard']}")
    print_suggested_checks(suggested_checks)
    return 0


def worktree_recovery_payload(lifecycle: WorktreeLifecycle) -> dict[str, object]:
    return {
        "run_id": lifecycle.run_id,
        "source_repo_root": str(lifecycle.source_repo_root),
        "worktree_root": str(lifecycle.worktree_root),
        "mounted_workspace": str(lifecycle.mounted_workspace),
        "branch": lifecycle.branch,
        "base_head": lifecycle.base_head,
        "source_head": git_stdout(lifecycle.source_repo_root, "rev-parse", "HEAD"),
        "worktree_head": git_stdout(lifecycle.worktree_root, "rev-parse", "HEAD"),
        "source_status": list(git_status_lines(lifecycle.source_repo_root)),
        "worktree_status": list(git_status_lines(lifecycle.worktree_root)),
        "commands": worktree_recovery_commands(lifecycle),
        "next_steps": worktree_recovery_next_steps(),
        "suggested_checks": suggest_project_checks(lifecycle.mounted_workspace),
    }


def worktree_recovery_commands(lifecycle: WorktreeLifecycle) -> dict[str, str]:
    source = shlex.quote(str(lifecycle.source_repo_root))
    worktree = shlex.quote(str(lifecycle.worktree_root))
    run_id = lifecycle.run_id
    return {
        "diff": f"runhaven runs diff {run_id}",
        "recover": f"runhaven runs recover {run_id}",
        "recover_json": f"runhaven runs recover {run_id} --json",
        "source_status": f"git -C {source} status --short",
        "worktree_status": f"git -C {worktree} status --short",
        "merge": f"runhaven runs merge {run_id}",
        "keep": f"runhaven runs keep {run_id}",
        "discard": f"runhaven runs discard {run_id}",
    }


def worktree_recovery_next_steps() -> list[str]:
    return [
        "Review recorded changes",
        "Inspect the source checkout",
        "Inspect the worktree",
        "Retry guarded merge",
        "Keep for manual review",
        "Discard only after review",
    ]


def runs_worktree_merge(run_id: str) -> int:
    lifecycle = load_worktree_lifecycle(run_id)
    verify_lifecycle(lifecycle)
    try:
        merge_worktree_changes(lifecycle)
    except ValueError as exc:
        raise ValueError(format_merge_recovery(lifecycle, str(exc))) from exc
    cleanup_worktree(lifecycle)
    print(f"Merged worktree run {run_id}")
    print(f"Source repo: {lifecycle.source_repo_root}")
    return 0


def runs_worktree_discard(run_id: str) -> int:
    lifecycle = load_worktree_lifecycle(run_id)
    verify_lifecycle(lifecycle)
    cleanup_worktree(lifecycle)
    print(f"Discarded worktree run {run_id}")
    print(f"Source repo: {lifecycle.source_repo_root}")
    return 0


def load_worktree_lifecycle(run_id: str) -> WorktreeLifecycle:
    validate_run_id(run_id)
    record = find_run_record(run_id)
    worktree = record.get("worktree")
    if not isinstance(worktree, dict):
        raise ValueError(f"run {run_id} is not a worktree run")

    source_repo_root = Path(
        require_string(
            worktree.get("source_repo_root"),
            "worktree run record is missing source repo root",
        )
    ).expanduser()
    worktree_root = Path(
        require_string(worktree.get("worktree_root"), "worktree run record is missing worktree")
    ).expanduser()
    mounted_workspace = Path(
        require_string(
            worktree.get("mounted_workspace"),
            "worktree run record is missing mounted workspace",
        )
    ).expanduser()
    branch = require_string(worktree.get("branch"), "worktree run record is missing branch")
    base_head = require_string(
        worktree.get("base_head"),
        "worktree run record is missing base HEAD",
    )
    return WorktreeLifecycle(
        run_id=run_id,
        source_repo_root=source_repo_root.resolve(),
        worktree_root=worktree_root.resolve(),
        mounted_workspace=mounted_workspace.resolve(),
        branch=branch,
        base_head=base_head,
    )


def verify_lifecycle(lifecycle: WorktreeLifecycle) -> None:
    if not lifecycle.branch.startswith("runhaven/"):
        raise ValueError("recorded branch is not RunHaven-owned; refusing worktree action")
    if not lifecycle.branch.endswith(f"/{lifecycle.run_id}"):
        raise ValueError("recorded branch does not match the run id; refusing worktree action")
    if lifecycle.worktree_root.name != lifecycle.run_id:
        raise ValueError("recorded worktree path does not match the run id; refusing action")
    if not lifecycle.source_repo_root.is_dir():
        raise ValueError("recorded source repository no longer exists")
    if not lifecycle.worktree_root.is_dir():
        raise ValueError("recorded worktree no longer exists")
    if not lifecycle.mounted_workspace.exists():
        raise ValueError("recorded mounted workspace no longer exists")
    if lifecycle.source_repo_root == lifecycle.worktree_root:
        raise ValueError("recorded worktree matches source repository; refusing action")

    source_root = git_stdout(lifecycle.source_repo_root, "rev-parse", "--show-toplevel")
    if Path(source_root).resolve() != lifecycle.source_repo_root:
        raise ValueError("recorded source repository does not match git toplevel")
    worktree_root = git_stdout(lifecycle.worktree_root, "rev-parse", "--show-toplevel")
    if Path(worktree_root).resolve() != lifecycle.worktree_root:
        raise ValueError("recorded worktree does not match git toplevel")
    branch_ref = f"refs/heads/{lifecycle.branch}"
    git_checked(
        lifecycle.source_repo_root,
        "rev-parse",
        "--verify",
        "--quiet",
        branch_ref,
        action="verify recorded branch",
    )
    current_branch = git_stdout(lifecycle.worktree_root, "branch", "--show-current")
    if current_branch != lifecycle.branch:
        raise ValueError("recorded worktree is not on the recorded branch; refusing action")
    git_checked(
        lifecycle.source_repo_root,
        "merge-base",
        "--is-ancestor",
        lifecycle.base_head,
        lifecycle.branch,
        action="verify recorded branch ancestry",
    )


def ensure_source_ready_for_merge(lifecycle: WorktreeLifecycle) -> None:
    current_head = git_stdout(lifecycle.source_repo_root, "rev-parse", "HEAD")
    if current_head != lifecycle.base_head:
        raise ValueError("source repository HEAD changed since the worktree run; refusing merge")
    snapshot = capture_git_snapshot(lifecycle.source_repo_root)
    if snapshot.get("available") is not True:
        raise ValueError("could not inspect source repository before merge")
    if snapshot.get("dirty") is True:
        raise ValueError("source repository has uncommitted changes; refusing merge")


def merge_worktree_changes(lifecycle: WorktreeLifecycle) -> None:
    ensure_source_ready_for_merge(lifecycle)
    worktree_head = git_stdout(lifecycle.worktree_root, "rev-parse", "HEAD")
    if worktree_head != lifecycle.base_head:
        git_checked(
            lifecycle.source_repo_root,
            "merge",
            "--ff-only",
            lifecycle.branch,
            action="fast-forward source repository",
        )

    apply_worktree_dirty_changes(lifecycle)


def format_merge_recovery(lifecycle: WorktreeLifecycle, reason: str) -> str:
    return "\n".join(
        (
            f"could not complete merge for run {lifecycle.run_id}: {reason}",
            "No cleanup was attempted; review the recorded worktree before retrying.",
            f"Source repo: {lifecycle.source_repo_root}",
            f"Worktree: {lifecycle.worktree_root}",
            f"Branch: {lifecycle.branch}",
            f"Review changes: runhaven runs diff {lifecycle.run_id}",
            f"Inspect source: git -C {lifecycle.source_repo_root} status --short",
            f"Inspect worktree: git -C {lifecycle.worktree_root} status --short",
            f"Manual recovery guide: runhaven runs recover {lifecycle.run_id}",
            f"Retry after fixing the source checkout: runhaven runs merge {lifecycle.run_id}",
            f"Keep for manual review: runhaven runs keep {lifecycle.run_id}",
            f"Discard after review: runhaven runs discard {lifecycle.run_id}",
        )
    )


def apply_worktree_dirty_changes(lifecycle: WorktreeLifecycle) -> None:
    paths = untracked_paths(lifecycle.worktree_root)
    ensure_untracked_destinations_available(
        lifecycle.worktree_root,
        lifecycle.source_repo_root,
        paths,
    )
    patch = git_bytes(
        lifecycle.worktree_root,
        "diff",
        "--binary",
        "HEAD",
        action="read worktree diff",
    )
    if patch:
        git_checked(
            lifecycle.source_repo_root,
            "apply",
            "--check",
            "--binary",
            input_bytes=patch,
            action="verify worktree patch",
        )
        git_checked(
            lifecycle.source_repo_root,
            "apply",
            "--binary",
            input_bytes=patch,
            action="apply worktree patch",
        )

    for path in paths:
        copy_untracked_path(lifecycle.worktree_root, lifecycle.source_repo_root, path)


def untracked_paths(worktree_root: Path) -> tuple[str, ...]:
    result = git_result(
        worktree_root,
        "status",
        "--porcelain=v1",
        "-z",
        "--untracked-files=all",
        "--",
        str(worktree_root),
    )
    entries = parse_git_status_entries(result.stdout)
    return tuple(entry.path for entry in entries if entry.status == "??")


def ensure_untracked_destinations_available(
    worktree_root: Path,
    source_repo_root: Path,
    paths: tuple[str, ...],
) -> None:
    for path in paths:
        safe_repo_path(str(worktree_root), path)
        destination = safe_repo_path(str(source_repo_root), path)
        if destination.exists() or destination.is_symlink():
            raise ValueError(f"source path already exists while merging untracked file: {path}")


def copy_untracked_path(worktree_root: Path, source_repo_root: Path, path: str) -> None:
    source_path = safe_repo_path(str(worktree_root), path)
    destination = safe_repo_path(str(source_repo_root), path)
    if destination.exists() or destination.is_symlink():
        raise ValueError(f"source path already exists while merging untracked file: {path}")
    destination.parent.mkdir(parents=True, exist_ok=True)
    if source_path.is_symlink():
        destination.symlink_to(os.readlink(source_path))
    elif source_path.is_dir():
        shutil.copytree(source_path, destination, symlinks=True)
    else:
        shutil.copy2(source_path, destination, follow_symlinks=False)


def cleanup_worktree(lifecycle: WorktreeLifecycle) -> None:
    git_checked(
        lifecycle.source_repo_root,
        "worktree",
        "remove",
        "--force",
        str(lifecycle.worktree_root),
        action="remove recorded worktree",
    )
    git_checked(
        lifecycle.source_repo_root,
        "branch",
        "-D",
        lifecycle.branch,
        action="delete recorded branch",
    )


def git_stdout(cwd: Path, *args: str) -> str:
    result = git_result(cwd, *args)
    return result.stdout.decode("utf-8", errors="replace").strip()


def git_bytes(cwd: Path, *args: str, action: str) -> bytes:
    result = git_result(cwd, *args, action=action)
    return result.stdout


def git_status_lines(cwd: Path) -> tuple[str, ...]:
    result = git_result(cwd, "status", "--short")
    text = result.stdout.decode("utf-8", errors="replace")
    return tuple(line for line in text.splitlines() if line)


def print_status(title: str, lines: Sequence[str]) -> None:
    print(f"{title}:")
    if not lines:
        print("  clean")
        return
    for line in lines:
        print(f"  {line}")


def print_suggested_checks(checks: Sequence[SuggestedCheck]) -> None:
    if not checks:
        return
    print("Suggested checks:")
    for index, check in enumerate(checks, start=1):
        print(f"{index}. {check['label']}: {check['command']}")
        print(f"   {check['reason']}")


def git_checked(
    cwd: Path,
    *args: str,
    input_bytes: bytes | None = None,
    action: str,
) -> None:
    git_result(cwd, *args, input_bytes=input_bytes, action=action)


def git_result(
    cwd: Path,
    *args: str,
    input_bytes: bytes | None = None,
    action: str = "run git command",
) -> subprocess.CompletedProcess[bytes]:
    try:
        result = subprocess.run(
            ("git", "-C", str(cwd), *args),
            check=False,
            capture_output=True,
            input=input_bytes,
        )
    except (FileNotFoundError, OSError) as exc:
        raise ValueError(f"{action} failed: git is unavailable") from exc
    if result.returncode != 0:
        detail = result.stderr.decode("utf-8", errors="replace").strip()
        raise ValueError(f"{action} failed: {detail or result.returncode}")
    return result
