import type {
  ManagedSessionSummary,
  WorktreeActionResult,
} from "@/commands/runhaven"

export type WorktreeAction = "merge" | "recover" | "discard"

export type WorktreeActionDefinition = {
  action: WorktreeAction
  button: string
}

// The two decisions that belong in the normal UI. Leaving the page already
// keeps the protected copy for later. Recovery appears only after Apply fails.
export const WORKTREE_ACTIONS: WorktreeActionDefinition[] = [
  {
    action: "merge",
    button: "Apply changes…",
  },
  {
    action: "discard",
    button: "Discard changes…",
  },
]

// The review shows only for a finished session that saved an isolated copy.
export function sessionAllowsWorktreeReview(
  session: ManagedSessionSummary,
): boolean {
  return (
    session.cleanup?.available === true &&
    session.cleanup.target === "worktree" &&
    (session.cleanup.reviewState === "changes" ||
      session.cleanup.reviewState === "unknown")
  )
}

export type WorktreeReviewState = NonNullable<
  ManagedSessionSummary["cleanup"]
>["reviewState"]

export function sessionWorktreeReviewState(
  session: ManagedSessionSummary,
): WorktreeReviewState {
  if (
    session.cleanup?.available !== true ||
    session.cleanup.target !== "worktree"
  ) {
    return "none"
  }
  return session.cleanup.reviewState
}

export function sessionHasWorktreeChanges(
  session: ManagedSessionSummary,
): boolean {
  return sessionWorktreeReviewState(session) === "changes"
}

export function sessionWorktreeReviewUnavailable(
  session: ManagedSessionSummary,
): boolean {
  return sessionWorktreeReviewState(session) === "unknown"
}

// Merge and discard remove the copy, so the review should disappear after them.
export function worktreeActionRemovesWorktree(action: WorktreeAction): boolean {
  return action === "merge" || action === "discard"
}

export function worktreeActionSuccessMessage(
  result: WorktreeActionResult,
): string {
  switch (result.action) {
    case "merge":
      return "Applied the assistant's changes to your project."
    case "recover":
      return "Follow these steps to open the protected copy."
    case "discard":
      return "Discarded the assistant's changes."
    default:
      return "RunHaven finished the change review."
  }
}
