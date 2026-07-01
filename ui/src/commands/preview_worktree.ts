import { previewSession, resolvePreviewSessionWorktree } from "./preview_sessions";
import type {
  ManagedSessionSummary,
  WorktreeActionResponse,
  WorktreeReviewResponse
} from "./types";

function requireWorktreeSession(sessionId: string): ManagedSessionSummary {
  const session = previewSession(sessionId);
  if (session.cleanup?.available !== true || session.cleanup.target !== "worktree") {
    throw new Error("Worktree review is only available for saved worktree sessions.");
  }
  return session;
}

function previewWorktreeBranch(session: ManagedSessionSummary): string {
  return `runhaven/${session.agent}-${session.session}`;
}

export function previewManagedSessionWorktree(sessionId: string): WorktreeReviewResponse {
  const session = requireWorktreeSession(sessionId);
  return {
    runId: session.runId,
    branch: previewWorktreeBranch(session),
    statusSummary: "2 files changed in the isolated copy.",
    worktreeChanges: 2,
    sourceChanges: 0,
    clean: false,
    availableActions: ["merge", "keep", "recover", "discard"]
  };
}

export function previewManagedWorktreeAction(
  sessionId: string,
  action: string
): WorktreeActionResponse {
  const session = requireWorktreeSession(sessionId);
  const statusByAction: Record<string, string> = {
    keep: "kept",
    merge: "merged",
    recover: "recovered",
    discard: "discarded"
  };
  if (action === "merge" || action === "discard") {
    resolvePreviewSessionWorktree(sessionId);
  }
  return {
    session,
    result: {
      runId: session.runId,
      action,
      branch: previewWorktreeBranch(session),
      status: statusByAction[action] ?? "kept",
      nextSteps:
        action === "recover"
          ? [
              "Open the isolated copy in your terminal.",
              "Review the changes before you merge or discard them."
            ]
          : []
    }
  };
}
