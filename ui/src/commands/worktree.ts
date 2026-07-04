import { call } from "./client";
import {
  previewManagedSessionWorktree,
  previewManagedWorktreeAction
} from "./preview";
import type { WorktreeActionResponse, WorktreeReviewResponse } from "./types";

// Worktree review commands for finished managed sessions. Each wrapper routes
// through the shared `call` helper in client.ts, so `invoke` stays centralized
// and the browser preview falls back to the preview fixtures.

export async function getManagedSessionWorktree(
  sessionId: string
): Promise<WorktreeReviewResponse> {
  return call(
    "get_managed_session_worktree",
    { request: { sessionId } },
    () => previewManagedSessionWorktree(sessionId)
  );
}

export async function keepManagedSessionWorktree(
  sessionId: string
): Promise<WorktreeActionResponse> {
  return call(
    "keep_managed_session_worktree",
    { request: { sessionId, confirm: true } },
    () => previewManagedWorktreeAction(sessionId, "keep")
  );
}

export async function mergeManagedSessionWorktree(
  sessionId: string
): Promise<WorktreeActionResponse> {
  return call(
    "merge_managed_session_worktree",
    { request: { sessionId, confirm: true } },
    () => previewManagedWorktreeAction(sessionId, "merge")
  );
}

export async function recoverManagedSessionWorktree(
  sessionId: string
): Promise<WorktreeActionResponse> {
  return call(
    "recover_managed_session_worktree",
    { request: { sessionId, confirm: true } },
    () => previewManagedWorktreeAction(sessionId, "recover")
  );
}

export async function discardManagedSessionWorktree(
  sessionId: string
): Promise<WorktreeActionResponse> {
  return call(
    "discard_managed_session_worktree",
    { request: { sessionId, confirm: true } },
    () => previewManagedWorktreeAction(sessionId, "discard")
  );
}
