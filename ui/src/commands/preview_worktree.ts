import { previewSession, resolvePreviewSessionWorktree } from "./preview_sessions";
import type {
  ManagedSessionSummary,
  WorktreeActionResponse,
  WorktreeReviewResponse
} from "./types";

function requireWorktreeSession(sessionId: string): ManagedSessionSummary {
  const session = previewSession(sessionId);
  if (session.cleanup?.available !== true || session.cleanup.target !== "worktree") {
    throw new Error("Changes are no longer available for this session.");
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
    statusSummary: "2 file changes are ready to review.",
    worktreeChanges: 2,
    sourceChanges: 0,
    clean: false,
    fileChanges: [
      { path: "src/app.ts", kind: "changed" },
      { path: "src/session.ts", kind: "added" }
    ],
    fileChangesTruncated: false,
    sourceFileChanges: [],
    sourceFileChangesTruncated: false,
    reviewId: "0".repeat(64),
    availableActions: ["merge", "recover", "discard"]
  };
}

export function previewManagedWorktreeAction(
  sessionId: string,
  action: string,
  reviewId?: string
): WorktreeActionResponse {
  const session = requireWorktreeSession(sessionId);
  if ((action === "merge" || action === "discard") && reviewId !== "0".repeat(64)) {
    throw new Error(
      "The protected copy changed since you reviewed it. Review the changes again before you continue."
    );
  }
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
              "Keep the protected copy so nothing is lost.",
              "Finish or undo other changes in your project.",
              "Return to this review and choose Apply changes again."
            ]
          : []
    }
  };
}
