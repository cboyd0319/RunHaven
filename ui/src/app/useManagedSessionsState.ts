import { useEffect, useState } from "react"

import {
  getManagedSessions,
  type ManagedSessionList,
} from "@/commands/runhaven"

import { sameManagedSessionList } from "./sessionList"
import { useSelectedSessionId } from "./useSelectedSessionId"

const MANAGED_SESSION_POLL_MS = 4_000

export function useManagedSessionsState(lastLaunchRunId: string | null) {
  const [managedSessions, setManagedSessions] = useState<ManagedSessionList>({
    sessions: [],
  })
  const { selectedSessionId, selectedSessionIdRef, selectSession } =
    useSelectedSessionId(managedSessions)

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
    const nextSessions = await getManagedSessions()
    setManagedSessions((current) =>
      sameManagedSessionList(current, nextSessions) ? current : nextSessions,
    )
    return nextSessions
  }

  return {
    managedSessions,
    selectedSessionId,
    selectedSessionIdRef,
    selectSession,
    loadManagedSessions,
  }
}
