import type { ManagedSessionSummary } from "@/commands/runhaven"

import { sessionAllowsWorktreeReview } from "./worktreeReview"

// The one needs-you signal per session row. Priority order comes from
// docs/design/native-gui-navigation-spec.md section 4: Failed (red) beats
// Needs sign-in, then Not responding, then Ready to review (all amber). The
// highest state wins; the badge is a symbol in a dot and the label names the
// state in text, so the signal is never color-only and never a count.
export type SessionAttentionKind =
  | "failed"
  | "needs-sign-in"
  | "not-responding"
  | "ready-to-review"

export type SessionAttention = {
  kind: SessionAttentionKind
  label: string
  tone: "failed" | "needs-you"
}

export function sessionAttention(
  session: ManagedSessionSummary,
): SessionAttention | null {
  if (session.lifecycle.trim().toLowerCase() === "failed") {
    return { kind: "failed", label: "Failed", tone: "failed" }
  }
  if (
    session.nextAction.kind === "auth_attention" ||
    session.authStatus.trim().toLowerCase() === "needs-sign-in"
  ) {
    return { kind: "needs-sign-in", label: "Needs sign-in", tone: "needs-you" }
  }
  if (session.nextAction.kind === "recover") {
    return {
      kind: "not-responding",
      label: "Not responding",
      tone: "needs-you",
    }
  }
  if (sessionAllowsWorktreeReview(session)) {
    return {
      kind: "ready-to-review",
      label: "Ready to review",
      tone: "needs-you",
    }
  }
  return null
}
