import { useState } from "react"

import {
  getAuthLog,
  getAuthStatus,
  getEgressLog,
  type AuthLogResponse,
  type AuthStatusResponse,
  type EgressLogResponse,
} from "@/commands/runhaven"

import { errorMessage, type DiagnosticsState } from "./types"

export function useDiagnosticsState() {
  const [data, setData] = useState<DiagnosticsState | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState("")

  async function load() {
    setLoading(true)
    setError("")
    try {
      const [authStatus, authLog, egressLog] = await Promise.all([
        getAuthStatus(),
        getAuthLog(),
        getEgressLog(),
      ])
      setData((current) => {
        const nextData = { authStatus, authLog, egressLog }
        return current && sameDiagnosticsStateData(current, nextData)
          ? current
          : nextData
      })
    } catch (cause) {
      setData(null)
      setError(errorMessage(cause))
    } finally {
      setLoading(false)
    }
  }

  return { data, loading, error, load }
}

export function sameDiagnosticsStateData(
  current: DiagnosticsState,
  next: DiagnosticsState,
): boolean {
  return (
    sameAuthStatusResponse(current.authStatus, next.authStatus) &&
    sameAuthLogResponse(current.authLog, next.authLog) &&
    sameEgressLogResponse(current.egressLog, next.egressLog)
  )
}

export function sameAuthStatusResponse(
  current: AuthStatusResponse,
  next: AuthStatusResponse,
): boolean {
  return (
    current.status === next.status &&
    current.runtime === next.runtime &&
    current.profiles.length === next.profiles.length &&
    current.profiles.every((profile, index) => {
      const nextProfile = next.profiles[index]
      return (
        nextProfile !== undefined &&
        profile.name === nextProfile.name &&
        profile.status === nextProfile.status
      )
    })
  )
}

function sameAuthLogResponse(
  current: AuthLogResponse,
  next: AuthLogResponse,
): boolean {
  return (
    current.entries.length === next.entries.length &&
    current.entries.every((entry, index) => {
      const nextEntry = next.entries[index]
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

function sameEgressLogResponse(
  current: EgressLogResponse,
  next: EgressLogResponse,
): boolean {
  return (
    current.entries.length === next.entries.length &&
    current.entries.every((entry, index) => {
      const nextEntry = next.entries[index]
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
