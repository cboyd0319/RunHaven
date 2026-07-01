import { useEffect, useRef, useState } from "react"

import type { ManagedSessionList } from "@/commands/runhaven"

export function useSelectedSessionId(managedSessions: ManagedSessionList) {
  const [selectedSessionId, setSelectedSessionId] = useState<string | null>(null)
  const selectedSessionIdRef = useRef<string | null>(null)

  useEffect(() => {
    if (
      selectedSessionId &&
      !managedSessions.sessions.some((session) => session.id === selectedSessionId)
    ) {
      selectedSessionIdRef.current = null
      setSelectedSessionId(null)
    }
  }, [managedSessions.sessions, selectedSessionId])

  useEffect(() => {
    selectedSessionIdRef.current = selectedSessionId
  }, [selectedSessionId])

  function selectSession(sessionId: string | null) {
    selectedSessionIdRef.current = sessionId
    setSelectedSessionId(sessionId)
  }

  return {
    selectedSessionId,
    selectedSessionIdRef,
    selectSession,
  }
}
