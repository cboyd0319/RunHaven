import { useEffect, useRef, useState } from "react"

import type { ManagedSessionList } from "@/commands/runhaven"

import {
  HOME_NAV,
  navigationAfterSessionRemoval,
  type NavDestination,
} from "./navigation"

// Owns the single selected destination. The app never navigates on its own
// while the user is looking at something; the one exception is when the
// selected session's record is removed, where selection moves to the nearest
// row per navigationAfterSessionRemoval (spec section 4). Status flaps,
// polls, and list changes never move the selection otherwise.
export function useNavigation(managedSessions: ManagedSessionList) {
  const [navigation, setNavigation] = useState<NavDestination>(HOME_NAV)
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

  function navigate(next: NavDestination) {
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
  }
}
