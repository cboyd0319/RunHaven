import { useEffect, useRef, useState } from "react"

import type {
  ManagedSessionList,
  ManagedSessionSummary,
} from "@/commands/runhaven"

import {
  HOME_NAV,
  navigationAfterSessionRemoval,
  type NavDestination,
} from "./navigation"
import {
  loadLastDestination,
  persistLastDestination,
} from "./uiStatePersistence"

// Owns the single selected destination. The app never navigates on its own
// while the user is looking at something; the one exception is when the
// selected session's record is removed, where selection moves to the nearest
// row per navigationAfterSessionRemoval (spec section 4). Status flaps,
// polls, and list changes never move the selection otherwise.
//
// The last destination persists across relaunch (uiStatePersistence). System
// destinations always exist, so they restore immediately. A persisted session
// is only a candidate: the app starts on Home and moves to the session only
// after the first backend session list confirms the id still exists
// (restoreNavigation, called by useManagedSessionsState after each successful
// load). Any manual navigation first cancels the restore.
export function useNavigation(managedSessions: ManagedSessionList) {
  const [restoreTarget] = useState(loadLastDestination)
  const [navigation, setNavigation] = useState<NavDestination>(
    restoreTarget && restoreTarget.kind !== "session"
      ? restoreTarget
      : HOME_NAV,
  )
  const pendingSessionRestoreRef = useRef(
    restoreTarget?.kind === "session" ? restoreTarget.id : null,
  )
  const selectedSessionId =
    navigation.kind === "session" ? navigation.id : null
  const selectedSessionIdRef = useRef<string | null>(null)
  const previousSessionsRef = useRef(managedSessions.sessions)

  useEffect(() => {
    if (
      selectedSessionId &&
      !managedSessions.sessions.some(
        (session) => session.id === selectedSessionId,
      )
    ) {
      const next = navigationAfterSessionRemoval(
        selectedSessionId,
        previousSessionsRef.current,
        managedSessions.sessions,
      )
      selectedSessionIdRef.current = next.kind === "session" ? next.id : null
      setNavigation(next)
    }
    previousSessionsRef.current = managedSessions.sessions
  }, [managedSessions.sessions, selectedSessionId])

  useEffect(() => {
    // While a session restore is pending, the rendered Home is provisional;
    // persisting it would clobber the stored destination before the backend
    // list has had a chance to confirm it.
    if (pendingSessionRestoreRef.current !== null) {
      return
    }
    persistLastDestination(navigation)
  }, [navigation])

  // Apply the persisted session selection once the backend session list is
  // in, and only if the session still exists; otherwise Home stands (spec
  // section 4 relaunch rule). Runs at most once per app start.
  function restoreNavigation(sessions: ManagedSessionSummary[]) {
    const pending = pendingSessionRestoreRef.current
    if (pending === null) {
      return
    }
    pendingSessionRestoreRef.current = null
    if (sessions.some((session) => session.id === pending)) {
      selectedSessionIdRef.current = pending
      setNavigation({ kind: "session", id: pending })
    } else {
      // The stored session is gone; record Home so the next relaunch starts
      // clean instead of retrying a dead id.
      persistLastDestination(HOME_NAV)
    }
  }

  function navigate(next: NavDestination) {
    // Manual navigation always wins over a pending relaunch restore.
    pendingSessionRestoreRef.current = null
    selectedSessionIdRef.current = next.kind === "session" ? next.id : null
    setNavigation(next)
  }

  function selectSession(sessionId: string) {
    navigate({ kind: "session", id: sessionId })
  }

  return {
    navigation,
    selectedSessionId,
    selectedSessionIdRef,
    navigate,
    selectSession,
    restoreNavigation,
  }
}
