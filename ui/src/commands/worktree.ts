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

export async function mergeManagedSessionWorktree(
  sessionId: string,
  reviewId: string
): Promise<WorktreeActionResponse> {
  return call(
    "merge_managed_session_worktree",
    { request: { sessionId, confirm: true, reviewId } },
    () => previewManagedWorktreeAction(sessionId, "merge", reviewId)
  );
}

export async function recoverManagedSessionWorktree(
  sessionId: string,
  _reviewId?: string
): Promise<WorktreeActionResponse> {
  return call(
    "recover_managed_session_worktree",
    { request: { sessionId, confirm: true } },
    () => previewManagedWorktreeAction(sessionId, "recover")
  );
}

export async function discardManagedSessionWorktree(
  sessionId: string,
  reviewId: string
): Promise<WorktreeActionResponse> {
  return call(
    "discard_managed_session_worktree",
    { request: { sessionId, confirm: true, reviewId } },
    () => previewManagedWorktreeAction(sessionId, "discard", reviewId)
  );
}
