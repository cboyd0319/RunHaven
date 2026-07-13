import type { ManagedSessionSummary } from "@/commands/runhaven"
import { Button } from "@/components/ui/button"

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
  onStart,
}: {
  sessions: ManagedSessionSummary[]
  onSelectSession: (sessionId: string) => void
  onSignIn: (agent: string) => void
  onStart: () => void
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

  if (ordered.length === 0) {
    return (
      <section className="mx-auto flex max-w-md flex-col items-center gap-3 py-14 text-center">
        <h2 className="text-lg font-semibold">No sessions yet</h2>
        <p className="text-sm text-muted-foreground">
          Sessions you start will appear here.
        </p>
        <Button type="button" onClick={onStart}>
          Start a session
        </Button>
      </section>
    )
  }

  return (
    <div className="flex flex-col gap-3">
      {attention.length > 0 ? (
        <NeedsReviewPanel
          sessions={attention}
          onSelectSession={onSelectSession}
          onSignIn={onSignIn}
        />
      ) : null}

      {other.length > 0 ? (
        <DashboardPanel
          title={attention.length > 0 ? "Other sessions" : "Session history"}
          count={other.length}
        >
          <RunTable sessions={other} onSelectSession={onSelectSession} />
        </DashboardPanel>
      ) : null}
    </div>
  )
}
