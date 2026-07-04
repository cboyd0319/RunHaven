import { useEffect, useRef, useState } from "react"

import {
  getManagedSessions,
  type ManagedSessionList,
} from "@/commands/runhaven"

import { sameManagedSessionList } from "./sessionList"
import { useNavigation } from "./useNavigation"

const MANAGED_SESSION_POLL_MS = 4_000
// With no session context, still poll on a slow baseline so a session started
// outside this window (the CLI, another window) shows up on its own instead of
// waiting for a manual Refresh. Kept slow and coalesced so it never spams.
const MANAGED_SESSION_BASELINE_POLL_MS = 30_000

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
    const intervalMs = hasSessionContext
      ? MANAGED_SESSION_POLL_MS
      : MANAGED_SESSION_BASELINE_POLL_MS

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
    }, intervalMs)

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
