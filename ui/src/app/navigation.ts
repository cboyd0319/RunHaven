import type { ManagedSessionSummary } from "@/commands/runhaven"

// The one navigation truth for the main window. Exactly one destination is
// selected at all times: Home, a session, or a System view. See
// docs/design/native-gui-navigation-spec.md sections 2 and 4.
export type NavDestination =
  | { kind: "home" }
  | { kind: "session"; id: string }
  | { kind: "agents" }
  | { kind: "diagnostics" }
  | { kind: "maintenance" }

export const HOME_NAV: NavDestination = { kind: "home" }

// Sidebar lifecycle groups. Active covers runs that are starting, running, or
// waiting to stop. Everything else (finished, failed, and any unknown state)
// lands in Finished so no session ever disappears from navigation.
const ACTIVE_LIFECYCLES = new Set([
  "starting",
  "running",
  "stop-requested",
  "kill-requested",
])

export type SessionGroups = {
  active: ManagedSessionSummary[]
  finished: ManagedSessionSummary[]
}

export function sessionGroupFor(
  session: ManagedSessionSummary,
): keyof SessionGroups {
  return ACTIVE_LIFECYCLES.has(session.lifecycle.trim().toLowerCase())
    ? "active"
    : "finished"
}

// Newest-first inside each group. The sort keys (startedAt, id) never change
// for a session, so rows never reorder on a status flap; a row only moves
// between groups on a lifecycle change.
export function groupSessions(
  sessions: ManagedSessionSummary[],
): SessionGroups {
  const groups: SessionGroups = { active: [], finished: [] }
  for (const session of sessions) {
    groups[sessionGroupFor(session)].push(session)
  }
  groups.active.sort(newestFirst)
  groups.finished.sort(newestFirst)
  return groups
}

function newestFirst(
  left: ManagedSessionSummary,
  right: ManagedSessionSummary,
): number {
  return (
    right.startedAt.localeCompare(left.startedAt) ||
    right.id.localeCompare(left.id, undefined, { numeric: true })
  )
}

// Where selection goes when the selected session's record is removed, per spec
// section 4: the adjacent row in the same group (next below, else above), else
// the first row of the other session group, else Home. This replaces the old
// reset-to-wizard behavior; it is the only navigation the app makes on its own.
export function navigationAfterSessionRemoval(
  removedId: string,
  previousSessions: ManagedSessionSummary[],
  nextSessions: ManagedSessionSummary[],
): NavDestination {
  const remainingIds = new Set(nextSessions.map((session) => session.id))
  const previousGroups = groupSessions(previousSessions)
  const removedGroup = previousGroups.active.some((s) => s.id === removedId)
    ? "active"
    : previousGroups.finished.some((s) => s.id === removedId)
      ? "finished"
      : null

  // The removed session was never in the list we knew about, so there is no
  // neighbor the user was actually looking at. Go Home instead of selecting a
  // remaining session the user never chose (spec invariant 10: the app never
  // self-navigates onto a destination the user did not pick).
  if (removedGroup === null) {
    return HOME_NAV
  }

  const rows = previousGroups[removedGroup]
  const index = rows.findIndex((session) => session.id === removedId)
  for (let below = index + 1; below < rows.length; below += 1) {
    if (remainingIds.has(rows[below].id)) {
      return { kind: "session", id: rows[below].id }
    }
  }
  for (let above = index - 1; above >= 0; above -= 1) {
    if (remainingIds.has(rows[above].id)) {
      return { kind: "session", id: rows[above].id }
    }
  }

  const nextGroups = groupSessions(nextSessions)
  const fallbackRows =
    removedGroup === "active"
      ? [...nextGroups.finished, ...nextGroups.active]
      : [...nextGroups.active, ...nextGroups.finished]
  const fallback = fallbackRows[0]
  return fallback ? { kind: "session", id: fallback.id } : HOME_NAV
}
