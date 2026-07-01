import type {
  ManagedSessionSummary,
  WorktreeActionResult,
} from "@/commands/runhaven"

export type WorktreeAction = "keep" | "merge" | "recover" | "discard"

export type WorktreeActionDefinition = {
  action: WorktreeAction
  button: string
  confirmLabel: string
  description: string
}

// The four review actions, in the order RunHaven shows them. Merge and discard
// remove the isolated copy; keep and recover leave it in place.
export const WORKTREE_ACTIONS: WorktreeActionDefinition[] = [
  {
    action: "merge",
    button: "Merge into project",
    confirmLabel: "Confirm merging the isolated copy back into your project.",
    description:
      "Bring the agent's changes back into your project files, then remove the copy.",
  },
  {
    action: "keep",
    button: "Keep for later",
    confirmLabel: "Confirm keeping the isolated copy for now.",
    description:
      "Leave the isolated copy and its branch in place so you can review it later.",
  },
  {
    action: "recover",
    button: "Recover by hand",
    confirmLabel: "Confirm getting manual steps to recover the isolated copy.",
    description:
      "Show plain steps to open the copy yourself. Nothing is merged or deleted.",
  },
  {
    action: "discard",
    button: "Discard changes",
    confirmLabel: "Confirm discarding the isolated copy and its changes.",
    description:
      "Throw away the agent's changes and remove the copy. This cannot be undone.",
  },
]

// The review shows only for a finished session that saved an isolated copy.
export function sessionAllowsWorktreeReview(
  session: ManagedSessionSummary,
): boolean {
  return (
    session.cleanup?.available === true &&
    session.cleanup.target === "worktree"
  )
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
      return "Merged the isolated copy back into your project."
    case "keep":
      return "Kept the isolated copy so you can review it later."
    case "recover":
      return "Here are the manual steps to recover the isolated copy."
    case "discard":
      return "Discarded the isolated copy and its changes."
    default:
      return `Worktree ${result.status}.`
  }
}
