import type { ManagedSessionSummary } from "@/commands/runhaven"

import { DashboardPanel } from "../dashboard/DashboardPanel"
import { NeedsReviewPanel } from "../dashboard/NeedsReviewPanel"
import { RunTable } from "../dashboard/RunTable"
import { sessionAttention } from "../sessionAttention"
import { userSessions } from "../sessionList"

// History owns all run state so Home can remain a launchpad. Runs that need a
// user action lead with their exact reason and next step; every other run stays
// in the chronological table below. A run appears in exactly one section.
export function RunsView({
  sessions,
  onSelectSession,
  onSignIn,
}: {
  sessions: ManagedSessionSummary[]
  onSelectSession: (sessionId: string) => void
  onSignIn: (agent: string) => void
}) {
  const ordered = userSessions(sessions).slice().sort(
    (left, right) =>
      right.startedAt.localeCompare(left.startedAt) ||
      right.id.localeCompare(left.id, undefined, { numeric: true }),
  )
  const attention = ordered.filter(
    (session) => sessionAttention(session) !== null,
  )
  const other = ordered.filter((session) => sessionAttention(session) === null)

  return (
    <div className="flex flex-col gap-3">
      {attention.length > 0 ? (
        <NeedsReviewPanel
          sessions={attention}
          onSelectSession={onSelectSession}
          onSignIn={onSignIn}
        />
      ) : null}

      {other.length > 0 || attention.length === 0 ? (
        <DashboardPanel
          title={attention.length > 0 ? "Other runs" : "Run history"}
          count={other.length}
        >
          {other.length === 0 ? (
          <p className="py-2 text-sm text-muted-foreground">No runs yet.</p>
          ) : (
            <RunTable sessions={other} onSelectSession={onSelectSession} />
          )}
        </DashboardPanel>
      ) : null}
    </div>
  )
}
