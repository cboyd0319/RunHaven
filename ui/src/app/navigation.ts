import type { ManagedSessionSummary } from "@/commands/runhaven"

// The one navigation truth for the main window. Exactly one destination is
// selected at all times, including menu-only destinations such as Help. See
// docs/design/native-gui-navigation-spec.md sections 2 and 4.
export type HelpReturnDestination =
  | { kind: "new-session" }
  | { kind: "session"; id: string }
  | { kind: "runs" }
  | { kind: "diagnostics" }
  // The interactive-agent run terminal (Claude Code) as a main-content
  // destination, not a modal. A live coding session fills the bounded content
  // area and stays closeable; it is ephemeral, so it is never persisted or
  // restored. Every app launch starts on New session instead.
  | { kind: "run-terminal"; id?: string }

export type NavDestination =
  | HelpReturnDestination
  | { kind: "help"; returnTo: HelpReturnDestination }

export const NEW_SESSION_NAV: HelpReturnDestination = { kind: "new-session" }

export function helpNavigation(returnTo: NavDestination): NavDestination {
  return {
    kind: "help",
    returnTo: returnTo.kind === "help" ? returnTo.returnTo : returnTo,
  }
}

export function helpReturnLabel(destination: HelpReturnDestination): string {
  switch (destination.kind) {
    case "new-session":
      return "New session"
    case "session":
      return "Session"
    case "runs":
      return "History"
    case "diagnostics":
      return "System"
    case "run-terminal":
      return "Terminal workspace"
  }
}

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
// the first row of the other session group, else New session. This replaces the old
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
  // neighbor the user was actually looking at. Go to New session instead of selecting a
  // remaining session the user never chose (spec invariant 10: the app never
  // self-navigates onto a destination the user did not pick).
  if (removedGroup === null) {
    return NEW_SESSION_NAV
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
  return fallback ? { kind: "session", id: fallback.id } : NEW_SESSION_NAV
}
