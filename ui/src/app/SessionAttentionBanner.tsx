import { CircleAlert, GitBranch, LogIn } from "lucide-react"

import type { ManagedSessionSummary } from "@/commands/runhaven"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Button } from "@/components/ui/button"

import type { NavDestination } from "./navigation"
import { sessionAttention } from "./sessionAttention"
import { agentLabel, displayValue } from "./sessionIdentity"

// The session view's needs-you banner: first in reading order, one state at a
// time (the same priority helper the sidebar badges use), always leading with
// the one fix. Sign-in navigates to Agents; repair and review name their
// surface instead of duplicating it, because the confirm-gated repair control
// and the worktree review panel sit directly below this banner and each
// mutation must keep exactly one confirmation surface.
export function SessionAttentionBanner({
  session,
  onNavigate,
}: {
  session: ManagedSessionSummary
  onNavigate: (destination: NavDestination) => void
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
              Sign in to {agentLabel(session.agent)} in Agents, then come back
              and refresh.
            </span>
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={() => onNavigate({ kind: "agents" })}
            >
              Open Agents
            </Button>
          </AlertDescription>
        </Alert>
      )
    case "not-responding":
      return (
        <Alert variant="warning" role="alert">
          <CircleAlert />
          <AlertTitle>Not responding</AlertTitle>
          <AlertDescription>
            The sandbox for this run is gone. Open More actions under Run actions
            to clear the stale record.
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
