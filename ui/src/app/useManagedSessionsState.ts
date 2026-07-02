import { useEffect, useRef, useState } from "react"

import {
  getManagedSessions,
  type ManagedSessionList,
} from "@/commands/runhaven"

import { sameManagedSessionList } from "./sessionList"
import { useNavigation } from "./useNavigation"

const MANAGED_SESSION_POLL_MS = 4_000

export function useManagedSessionsState(lastLaunchRunId: string | null) {
  const [managedSessions, setManagedSessions] = useState<ManagedSessionList>({
    sessions: [],
  })
  const managedSessionsRef = useRef(managedSessions)
  const requestSeq = useRef(0)
  const {
    navigation,
    selectedSessionId,
    selectedSessionIdRef,
    navigate,
    selectSession,
    restoreNavigation,
  } = useNavigation(managedSessions)

  useEffect(() => {
    managedSessionsRef.current = managedSessions
  }, [managedSessions])

  useEffect(() => {
    const hasSessionContext =
      managedSessions.sessions.length > 0 ||
      selectedSessionId !== null ||
      lastLaunchRunId !== null
    if (!hasSessionContext) {
      return
    }

    let inFlight = false
    const interval = window.setInterval(() => {
      if (inFlight) {
        return
      }
      inFlight = true
      void loadManagedSessions()
        .catch(() => undefined)
        .finally(() => {
          inFlight = false
        })
    }, MANAGED_SESSION_POLL_MS)

    return () => window.clearInterval(interval)
  }, [lastLaunchRunId, managedSessions.sessions.length, selectedSessionId])

  async function loadManagedSessions() {
    const requestId = requestSeq.current + 1
    requestSeq.current = requestId
    const nextSessions = await getManagedSessions()
    if (requestSeq.current !== requestId) {
      return managedSessionsRef.current
    }
    managedSessionsRef.current = nextSessions
    setManagedSessions((current) =>
      sameManagedSessionList(current, nextSessions) ? current : nextSessions,
    )
    // Backend truth is in: reconcile the relaunch-restored selection against
    // it (no-op after the first successful load or any manual navigation).
    restoreNavigation(nextSessions.sessions)
    return nextSessions
  }

  return {
    managedSessions,
    navigation,
    selectedSessionId,
    selectedSessionIdRef,
    navigate,
    selectSession,
    loadManagedSessions,
  }
}
