import type { ManagedSessionSummary } from "@/commands/runhaven"
import { Button } from "@/components/ui/button"

import { DashboardPanel } from "../dashboard/DashboardPanel"
import { RunTable } from "../dashboard/RunTable"
import {
  sessionGroupFor,
  type RunListFilter,
} from "../navigation"
import { sessionAttention } from "../sessionAttention"

const FILTERS: Array<{ id: RunListFilter; label: string }> = [
  { id: "needs-you", label: "Needs you" },
  { id: "running", label: "Running" },
  { id: "past", label: "Past" },
  { id: "all", label: "All" },
]

// The full run history: every managed session, newest first, reached from the
// sidebar Runs counts and Home's "Open full run history". Restores the
// all-sessions reachability the variant-06 sidebar dropped (a keyboard user, and
// anyone looking for a run past the six shown on Home, has a real destination).
// A read-only summary of backend-owned state; opening a row is the only action.
export function RunsView({
  filter,
  sessions,
  onSelectSession,
  onFilterChange,
}: {
  filter: RunListFilter
  sessions: ManagedSessionSummary[]
  onSelectSession: (sessionId: string) => void
  onFilterChange: (filter: RunListFilter) => void
}) {
  const ordered = filterRuns(sessions, filter).slice().sort(
    (left, right) =>
      right.startedAt.localeCompare(left.startedAt) ||
      right.id.localeCompare(left.id, undefined, { numeric: true }),
  )
  return (
    <div className="flex flex-col gap-3">
      <div
        role="group"
        aria-label="Filter run history"
        className="flex flex-wrap gap-2"
      >
        {FILTERS.map((item) => (
          <Button
            key={item.id}
            type="button"
            size="sm"
            variant={filter === item.id ? "secondary" : "ghost"}
            aria-pressed={filter === item.id}
            onClick={() => onFilterChange(item.id)}
          >
            {item.label}
          </Button>
        ))}
      </div>
      <DashboardPanel title="Run history" count={ordered.length}>
        {ordered.length === 0 ? (
          <p className="py-2 text-sm text-muted-foreground">No runs yet.</p>
        ) : (
          <RunTable sessions={ordered} onSelectSession={onSelectSession} />
        )}
      </DashboardPanel>
    </div>
  )
}

export function filterRuns(
  sessions: ManagedSessionSummary[],
  filter: RunListFilter,
): ManagedSessionSummary[] {
  switch (filter) {
    case "needs-you":
      return sessions.filter((session) => sessionAttention(session) !== null)
    case "running":
      return sessions.filter((session) => sessionGroupFor(session) === "active")
    case "past":
      return sessions.filter((session) => sessionGroupFor(session) === "finished")
    case "all":
      return sessions
  }
}
