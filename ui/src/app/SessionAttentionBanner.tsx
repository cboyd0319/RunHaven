import { CircleAlert, GitBranch, LogIn } from "lucide-react"

import type { ManagedSessionSummary } from "@/commands/runhaven"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Button } from "@/components/ui/button"

import type { NavDestination } from "./navigation"
import { sessionAttention } from "./sessionAttention"
import { agentLabel, displayValue } from "./sessionIdentity"

// The session view's needs-you banner: first in reading order, one state at a
// time (the same priority helper Home and the Dock badge use), always leading
// with the one fix. Sign-in and repair expose one direct action; worktree review
// points at the full review immediately below. Rust keeps the one trusted native
// confirmation surface for mutations.
export function SessionAttentionBanner({
  session,
  onNavigate,
  onSignIn,
  onRepair,
}: {
  session: ManagedSessionSummary
  onNavigate: (destination: NavDestination) => void
  onSignIn?: (agent: string) => void
  onRepair?: () => void
}) {
  const attention = sessionAttention(session)
  if (!attention) {
    return null
  }

  switch (attention.kind) {
    case "failed":
      return (
        <Alert variant="destructive" role="alert">
          <CircleAlert />
          <AlertTitle>This run failed</AlertTitle>
          <AlertDescription>
            {session.nextAction.detail.trim()
              ? displayValue(session.nextAction.detail)
              : "Check the latest output below, then clean up the run."}
          </AlertDescription>
        </Alert>
      )
    case "needs-sign-in":
      return (
        <Alert variant="warning" role="alert">
          <LogIn />
          <AlertTitle>Needs sign-in</AlertTitle>
          <AlertDescription className="flex flex-col items-start gap-2">
            <span>
              Sign in to {agentLabel(session.agent)}, then return to this run.
            </span>
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={() =>
                onSignIn
                  ? onSignIn(session.agent)
                  : onNavigate({ kind: "new-session" })
              }
            >
              Sign in
            </Button>
          </AlertDescription>
        </Alert>
      )
    case "not-responding":
      return (
        <Alert variant="warning" role="alert">
          <CircleAlert />
          <AlertTitle>Not responding</AlertTitle>
          <AlertDescription className="flex flex-col items-start gap-2">
            <span>
              The sandbox for this run is gone. Clear its stale run record to
              remove it from active work.
            </span>
            {onRepair ? (
              <Button type="button" variant="outline" size="sm" onClick={onRepair}>
                Repair run record
              </Button>
            ) : null}
          </AlertDescription>
        </Alert>
      )
    case "ready-to-review":
      return (
        <Alert role="status">
          <GitBranch />
          <AlertTitle>Ready to review</AlertTitle>
          <AlertDescription>
            Review the isolated copy below to merge, keep, or discard its
            changes.
          </AlertDescription>
        </Alert>
      )
  }
}
