import type { ManagedSessionSummary } from "@/commands/runhaven"

import { DashboardPanel } from "../dashboard/DashboardPanel"
import { RunTable } from "../dashboard/RunTable"

// The full run history: every managed session, newest first, reached from the
// sidebar Runs counts and Home's "Open full run history". Restores the
// all-sessions reachability the variant-06 sidebar dropped (a keyboard user, and
// anyone looking for a run past the six shown on Home, has a real destination).
// A read-only summary of backend-owned state; opening a row is the only action.
export function RunsView({
  sessions,
  onSelectSession,
}: {
  sessions: ManagedSessionSummary[]
  onSelectSession: (sessionId: string) => void
}) {
  const ordered = sessions.slice().sort(
    (left, right) =>
      right.startedAt.localeCompare(left.startedAt) ||
      right.id.localeCompare(left.id, undefined, { numeric: true }),
  )
  return (
    <DashboardPanel title="All runs" count={sessions.length}>
      {ordered.length === 0 ? (
        <p className="py-2 text-sm text-muted-foreground">No runs yet.</p>
      ) : (
        <RunTable sessions={ordered} onSelectSession={onSelectSession} />
      )}
    </DashboardPanel>
  )
}
