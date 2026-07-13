import type { ManagedSessionSummary } from "@/commands/runhaven"
import { Button } from "@/components/ui/button"

import {
  sessionAttention,
  type SessionAttentionKind,
} from "../sessionAttention"
import { displayValue, sessionDisplayTitle } from "../sessionIdentity"
import { AgentCell } from "./AgentCell"
import { DashboardPanel } from "./DashboardPanel"
import { timeAgo } from "./timeAgo"

// History "Needs attention": the sessions waiting on the user. Stacked rows
// keep the session, reason, and primary action readable at the minimum window
// width. Empty state hides the panel (the caller checks the count).
export function NeedsReviewPanel({
  sessions,
  onSelectSession,
  onSignIn,
}: {
  sessions: ManagedSessionSummary[]
  onSelectSession: (sessionId: string) => void
  onSignIn: (agent: string) => void
}) {
  return (
    <DashboardPanel title="Needs attention" count={sessions.length}>
      <ul className="divide-y text-sm">
        {sessions.map((session) => {
          const attention = sessionAttention(session)
          const title = sessionDisplayTitle(session)
          return (
            <li
              key={session.id}
              className="flex flex-col gap-3 py-3 sm:flex-row sm:items-start"
            >
              <div className="min-w-0 flex-1">
                <div className="flex min-w-0 items-center gap-2 font-medium">
                  <span
                    aria-hidden
                    className={`size-2 shrink-0 rounded-full ${attentionDotClass(
                      attention?.tone,
                    )}`}
                  />
                  <span className="min-w-0 break-words">{title}</span>
                </div>
                <div className="mt-1 flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-muted-foreground">
                  <AgentCell agent={session.agent} />
                  <span className="tabular-nums">{timeAgo(session.startedAt)}</span>
                </div>
                <p className="mt-2 font-medium text-foreground">
                  {attention?.label ?? "Needs review"}
                </p>
                <p className="text-muted-foreground">
                  {attentionDetail(session, attention?.kind)}
                </p>
              </div>
              <Button
                type="button"
                variant="outline"
                size="sm"
                className="shrink-0 self-start"
                aria-label={`${attentionAction(attention?.kind)} ${title}`}
                onClick={() => {
                  if (attention?.kind === "needs-sign-in") {
                    onSignIn(session.agent)
                  } else {
                    onSelectSession(session.id)
                  }
                }}
              >
                {attentionAction(attention?.kind)}
              </Button>
            </li>
          )
        })}
      </ul>
    </DashboardPanel>
  )
}

function attentionDetail(
  session: ManagedSessionSummary,
  kind: SessionAttentionKind | undefined,
): string {
  if (kind === "ready-to-review") {
    return "The assistant changed files in the protected copy."
  }
  if (kind === "review-unavailable") {
    return "RunHaven couldn’t read the saved changes. Open this session and check again."
  }
  if (session.nextAction.detail.trim()) {
    return displayValue(session.nextAction.detail)
  }
  switch (kind) {
    case "failed":
      return "RunHaven couldn’t determine why this session ended."
    case "not-responding":
      return "RunHaven can no longer find this session."
    case "needs-sign-in":
      return "Sign in to continue."
    default:
      return "Open this session for the next step."
  }
}

function attentionAction(kind: SessionAttentionKind | undefined): string {
  switch (kind) {
    case "failed":
      return "Open session"
    case "needs-sign-in":
      return "Sign in"
    case "not-responding":
      return "Open repair"
    case "ready-to-review":
      return "Review changes"
    case "review-unavailable":
      return "Open session"
    default:
      return "Open"
  }
}

function attentionDotClass(
  tone: "failed" | "needs-you" | "ready" | undefined,
): string {
  switch (tone) {
    case "failed":
      return "bg-destructive"
    case "needs-you":
      return "bg-warning"
    case "ready":
      return "bg-success"
    default:
      return "bg-muted-foreground"
  }
}
