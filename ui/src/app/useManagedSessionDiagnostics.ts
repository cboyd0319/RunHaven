import { useCallback, useEffect, useRef, useState } from "react"

import {
  getManagedSessionDiagnostics,
  type ManagedSessionDiagnosticsResponse,
} from "@/commands/runhaven"

import { errorMessage } from "./types"
import { sameManagedSession } from "./sessionList"

export function useManagedSessionDiagnostics(sessionId: string | null) {
  const [data, setData] = useState<ManagedSessionDiagnosticsResponse | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState("")
  const dataRef = useRef<ManagedSessionDiagnosticsResponse | null>(null)
  const requestSeq = useRef(0)

  useEffect(() => {
    requestSeq.current += 1
    dataRef.current = null
    setData(null)
    setError("")
    setLoading(false)
  }, [sessionId])

  const load = useCallback(async () => {
    if (!sessionId) {
      return
    }
    const requestId = requestSeq.current + 1
    requestSeq.current = requestId
    setLoading(dataRef.current === null)
    setError("")
    try {
      const nextDiagnostics = await getManagedSessionDiagnostics(sessionId)
      if (requestSeq.current !== requestId) {
        return
      }
      dataRef.current = nextDiagnostics
      setData((current) =>
        current && sameManagedSessionDiagnostics(current, nextDiagnostics)
          ? current
          : nextDiagnostics,
      )
    } catch (cause) {
      if (requestSeq.current === requestId) {
        dataRef.current = null
        setData(null)
        setError(errorMessage(cause))
      }
    } finally {
      if (requestSeq.current === requestId) {
        setLoading(false)
      }
    }
  }, [sessionId])

  return { data, loading, error, load }
}

export function sameManagedSessionDiagnostics(
  current: ManagedSessionDiagnosticsResponse,
  next: ManagedSessionDiagnosticsResponse,
): boolean {
  return (
    sameManagedSession(current.session, next.session) &&
    current.authStatus.name === next.authStatus.name &&
    current.authStatus.status === next.authStatus.status &&
    sameAuthLogEntries(current.authLogEntries, next.authLogEntries) &&
    sameEgressLogEntries(current.egressLogEntries, next.egressLogEntries)
  )
}

function sameAuthLogEntries(
  current: ManagedSessionDiagnosticsResponse["authLogEntries"],
  next: ManagedSessionDiagnosticsResponse["authLogEntries"],
): boolean {
  return (
    current.length === next.length &&
    current.every((entry, index) => {
      const nextEntry = next[index]
      return (
        nextEntry !== undefined &&
        entry.timestamp === nextEntry.timestamp &&
        entry.profile === nextEntry.profile &&
        entry.broker === nextEntry.broker &&
        entry.decision === nextEntry.decision &&
        entry.method === nextEntry.method &&
        entry.path === nextEntry.path &&
        entry.upstreamStatus === nextEntry.upstreamStatus &&
        entry.count === nextEntry.count &&
        entry.reason === nextEntry.reason &&
        entry.runId === nextEntry.runId
      )
    })
  )
}

function sameEgressLogEntries(
  current: ManagedSessionDiagnosticsResponse["egressLogEntries"],
  next: ManagedSessionDiagnosticsResponse["egressLogEntries"],
): boolean {
  return (
    current.length === next.length &&
    current.every((entry, index) => {
      const nextEntry = next[index]
      return (
        nextEntry !== undefined &&
        entry.timestamp === nextEntry.timestamp &&
        entry.profile === nextEntry.profile &&
        entry.decision === nextEntry.decision &&
        entry.host === nextEntry.host &&
        entry.port === nextEntry.port &&
        entry.count === nextEntry.count &&
        entry.reason === nextEntry.reason &&
        entry.matchedRule === nextEntry.matchedRule &&
        entry.runId === nextEntry.runId
      )
    })
  )
}
