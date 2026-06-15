from __future__ import annotations

import shlex
import subprocess
from pathlib import Path

from .cache_paths import worktrees_dir
from .git_metadata import capture_git_snapshot, git_head, git_repo_root
from .plans import WorkspaceScope, WorktreeRun, apply_workspace_scope, validate_workspace


def preview_worktree(
    workspace: Path,
    *,
    workspace_scope: WorkspaceScope,
    allow_sensitive_workspace: bool,
) -> tuple[Path, Path, str]:
    source_workspace = resolve_worktree_source(
        workspace,
        workspace_scope=workspace_scope,
        allow_sensitive_workspace=allow_sensitive_workspace,
    )
    repo_root = require_git_repo_root(source_workspace)
    base_head = require_git_head(repo_root)
    ensure_clean_source_repo(repo_root)
    return source_workspace, repo_root, base_head


def create_worktree_for_run(
    workspace: Path,
    *,
    workspace_scope: WorkspaceScope,
    allow_sensitive_workspace: bool,
    profile_name: str,
    run_id: str,
) -> WorktreeRun:
    source_workspace, repo_root, base_head = preview_worktree(
        workspace,
        workspace_scope=workspace_scope,
        allow_sensitive_workspace=allow_sensitive_workspace,
    )
    worktree_root = worktrees_dir() / run_id
    worktree_root.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
    branch = f"runhaven/{profile_name}/{run_id}"
    command = (
        "git",
        "-C",
        str(repo_root),
        "worktree",
        "add",
        "-b",
        branch,
        str(worktree_root),
        base_head,
    )
    result = subprocess.run(command, check=False, capture_output=True, text=True)
    if result.returncode != 0:
        if worktree_root.exists() and not any(worktree_root.iterdir()):
            worktree_root.rmdir()
        detail = result.stderr.strip() or result.stdout.strip() or str(result.returncode)
        raise ValueError(f"could not create git worktree: {detail}")

    mounted_workspace = worktree_root / source_workspace.relative_to(repo_root)
    mounted_workspace.mkdir(parents=True, exist_ok=True)
    return WorktreeRun(
        source_workspace=source_workspace,
        source_repo_root=repo_root,
        worktree_root=worktree_root.resolve(),
        mounted_workspace=mounted_workspace.resolve(),
        branch=branch,
        base_head=base_head,
        recovery_commands=recovery_commands(repo_root, worktree_root, branch),
    )


def resolve_worktree_source(
    workspace: Path,
    *,
    workspace_scope: WorkspaceScope,
    allow_sensitive_workspace: bool,
) -> Path:
    try:
        resolved = workspace.expanduser().resolve()
    except OSError as exc:
        raise ValueError(f"could not resolve workspace path: {exc}") from exc
    if not resolved.exists():
        raise ValueError(f"workspace does not exist: {resolved}")
    if not resolved.is_dir():
        raise ValueError(f"workspace is not a directory: {resolved}")
    resolved, _ = apply_workspace_scope(resolved, scope=workspace_scope)
    validate_workspace(resolved, allow_sensitive=allow_sensitive_workspace)
    return resolved


def require_git_repo_root(workspace: Path) -> Path:
    repo_root, reason = git_repo_root(workspace)
    if repo_root is None:
        raise ValueError(f"--worktree requires a git worktree: {reason}")
    return Path(repo_root).resolve()


def require_git_head(repo_root: Path) -> str:
    head = git_head(str(repo_root))
    if not head:
        raise ValueError("--worktree requires a git repository with a committed HEAD")
    return head


def ensure_clean_source_repo(repo_root: Path) -> None:
    snapshot = capture_git_snapshot(repo_root)
    if snapshot.get("available") is not True:
        reason = snapshot.get("reason")
        reason_text = f": {reason}" if isinstance(reason, str) and reason else ""
        raise ValueError(f"could not inspect source git worktree{reason_text}")
    if snapshot.get("dirty") is True:
        raise ValueError(
            "--worktree requires a clean source git worktree.\n"
            "Options:\n"
            "1. Commit or stash source changes, then retry with --worktree.\n"
            "2. Run without --worktree to use the source checkout directly.\n"
            "3. Start from a clean clone or git worktree if you want isolation."
        )


def recovery_commands(
    repo_root: Path,
    worktree_root: Path,
    branch: str,
) -> tuple[tuple[str, str], ...]:
    quoted_repo = shlex.quote(str(repo_root))
    quoted_worktree = shlex.quote(str(worktree_root))
    quoted_branch = shlex.quote(branch)
    return (
        ("status", f"git -C {quoted_worktree} status --short"),
        ("diff", f"git -C {quoted_worktree} diff HEAD"),
        ("merge", f"git -C {quoted_repo} merge {quoted_branch}"),
        ("remove_worktree", f"git -C {quoted_repo} worktree remove {quoted_worktree}"),
        ("delete_branch", f"git -C {quoted_repo} branch -D {quoted_branch}"),
    )


def worktree_record(worktree: WorktreeRun) -> dict[str, object]:
    return {
        "source_workspace": str(worktree.source_workspace),
        "source_repo_root": str(worktree.source_repo_root),
        "worktree_root": str(worktree.worktree_root),
        "mounted_workspace": str(worktree.mounted_workspace),
        "branch": worktree.branch,
        "base_head": worktree.base_head,
        "recovery_commands": dict(worktree.recovery_commands),
    }
