import { CircleAlert, Files, LogIn } from "lucide-react"

import type { ManagedSessionSummary } from "@/commands/runhaven"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Button } from "@/components/ui/button"

import type { NavDestination } from "./navigation"
import { sessionAttention } from "./sessionAttention"
import { agentLabel, displayValue } from "./sessionIdentity"
import { isPastOutputUnavailable } from "./useRunOutput"

// The session view's needs-you banner: first in reading order, one state at a
// time, always leading with the reason and the shortest available recovery.
// Sign-in and repair expose one direct action; worktree review points at the
// full review immediately below. Rust keeps the one trusted native confirmation
// surface for mutations.
export function SessionAttentionBanner({
  session,
  onNavigate,
  onShowOutput,
  onStartNew,
  onSignIn,
  onRepair,
  outputError = "",
}: {
  session: ManagedSessionSummary
  onNavigate: (destination: NavDestination) => void
  onShowOutput: () => void
  onStartNew: (agent: string) => void
  onSignIn?: (agent: string) => void
  onRepair?: () => void
  outputError?: string
}) {
  const attention = sessionAttention(session)
  if (!attention) {
    return null
  }

  switch (attention.kind) {
    case "failed": {
      const outputUnavailable =
        session.outputAvailable === false || isPastOutputUnavailable(outputError)
      return (
        <Alert variant="destructive" role="alert">
          <CircleAlert />
          <AlertTitle>Session ended</AlertTitle>
          <AlertDescription className="flex flex-col items-start gap-2">
            <span>
              {session.nextAction.detail.trim()
                ? displayValue(session.nextAction.detail)
                : "RunHaven couldn’t determine why this session ended."}
            </span>
            {outputUnavailable ? (
              <span>
                Output was not kept. RunHaven keeps session output only until you
                quit.
              </span>
            ) : null}
            <div className="flex flex-wrap gap-2">
              {!outputUnavailable ? (
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  onClick={onShowOutput}
                >
                  Show output…
                </Button>
              ) : null}
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={() => onStartNew(session.agent)}
              >
                Start new session…
              </Button>
            </div>
          </AlertDescription>
        </Alert>
      )
    }
    case "needs-sign-in":
      return (
        <Alert variant="warning" role="alert">
          <LogIn />
          <AlertTitle>Sign in to continue</AlertTitle>
          <AlertDescription className="flex flex-col items-start gap-2">
            <span>
              Sign in to {agentLabel(session.agent)}, then return to this session.
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
              Sign in…
            </Button>
          </AlertDescription>
        </Alert>
      )
    case "not-responding":
      return (
        <Alert variant="warning" role="alert">
          <CircleAlert />
          <AlertTitle>Session ended unexpectedly</AlertTitle>
          <AlertDescription className="flex flex-col items-start gap-2">
            <span>
              RunHaven can no longer find this session. Remove it from the active
              list.
            </span>
            {onRepair ? (
              <Button type="button" variant="outline" size="sm" onClick={onRepair}>
                Remove from active list…
              </Button>
            ) : null}
          </AlertDescription>
        </Alert>
      )
    case "review-unavailable":
      return (
        <Alert variant="warning" role="alert">
          <CircleAlert />
          <AlertTitle>Couldn’t check the protected copy</AlertTitle>
          <AlertDescription>
            RunHaven kept the protected copy. Check again below before you
            decide what to do with it.
          </AlertDescription>
        </Alert>
      )
    case "ready-to-review":
      return (
        <Alert role="status">
          <Files aria-hidden />
          <AlertTitle>Changes ready to review</AlertTitle>
          <AlertDescription>
            Review the files below. Then choose Apply changes or Discard changes.
          </AlertDescription>
        </Alert>
      )
  }
}
