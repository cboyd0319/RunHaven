import { useCallback, useEffect, useRef, useState } from "react"

import {
  getManagedSessionStatus,
  type ManagedSessionRecovery,
  type ManagedSessionStatusResponse,
  type RunStatusContainer,
} from "@/commands/runhaven"

import { sameRunStatusContainer } from "./appStatusEquality"
import { sameManagedSession } from "./sessionList"
import { errorMessage } from "./types"

// Compile-time drift guards for the equality helpers below; see
// appStatusEquality.ts for the pattern.
const managedSessionStatusResponseChecked = {
  session: true,
  container: true,
  statusUnavailable: true,
  recovery: true,
} satisfies Record<keyof ManagedSessionStatusResponse, true>
void managedSessionStatusResponseChecked

const managedSessionRecoveryChecked = {
  status: true,
  title: true,
  detail: true,
  remedy: true,
  repairAvailable: true,
} satisfies Record<keyof ManagedSessionRecovery, true>
void managedSessionRecoveryChecked

const MANAGED_SESSION_STATUS_POLL_MS = 4_000

type StoredStatus = {
  sessionId: string
  data: ManagedSessionStatusResponse
}

export function useManagedSessionStatus(sessionId: string | null) {
  // Pairing the fetched status with the id it belongs to lets the render derive
  // null the instant `sessionId` changes, before the reset effect runs after
  // paint. Without this, one frame renders the previous session's card.
  const [stored, setStored] = useState<StoredStatus | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState("")
  const storedRef = useRef<StoredStatus | null>(null)
  const requestSeq = useRef(0)
  const inFlight = useRef<Promise<void> | null>(null)

  const data = stored?.sessionId === sessionId ? stored.data : null

  const loadStatus = useCallback(
    async ({ background = false }: { background?: boolean } = {}) => {
      if (!sessionId) {
        return
      }
      const fetchStatus = async (isBackground: boolean) => {
        const requestId = requestSeq.current + 1
        requestSeq.current = requestId
        setLoading(storedRef.current?.sessionId !== sessionId)
        try {
          const nextStatus = await getManagedSessionStatus(sessionId)
          if (requestSeq.current !== requestId) {
            return
          }
          const nextStored: StoredStatus = { sessionId, data: nextStatus }
          storedRef.current = nextStored
          setError("")
          setStored((current) =>
            current &&
            current.sessionId === sessionId &&
            sameManagedSessionStatus(current.data, nextStatus)
              ? current
              : nextStored,
          )
        } catch (cause) {
          if (requestSeq.current === requestId) {
            setError(errorMessage(cause))
            // A single failed background poll must not blank the last-good card.
            // An explicit (user-initiated) reload may clear to surface the error.
            if (!isBackground) {
              if (storedRef.current?.sessionId === sessionId) {
                storedRef.current = null
              }
              setStored((current) =>
                current?.sessionId === sessionId ? null : current,
              )
            }
          }
        } finally {
          if (requestSeq.current === requestId) {
            setLoading(false)
          }
        }
      }
      const pending = inFlight.current
      if (pending) {
        if (background) {
          return
        }
        // An explicit (user-initiated) reload must observe status at least as
        // fresh as the moment it was requested. Wait for the in-flight request
        // to settle, then take one follow-up load; if another explicit caller
        // already started that follow-up, share it instead of stacking more.
        await pending
        if (inFlight.current) {
          await inFlight.current
          return
        }
      }
      const request = fetchStatus(background)
      inFlight.current = request
      try {
        await request
      } finally {
        if (inFlight.current === request) {
          inFlight.current = null
        }
      }
    },
    [sessionId],
  )

  useEffect(() => {
    requestSeq.current += 1
    inFlight.current = null
    storedRef.current = null
    setStored(null)
    setError("")
    setLoading(Boolean(sessionId))
    if (!sessionId) {
      setLoading(false)
      return
    }

    void loadStatus()
    const interval = window.setInterval(
      () => void loadStatus({ background: true }),
      MANAGED_SESSION_STATUS_POLL_MS,
    )
    return () => window.clearInterval(interval)
  }, [loadStatus, sessionId])

  return {
    data,
    loading,
    error,
    loadStatus,
  }
}

export function sameManagedSessionStatus(
  current: ManagedSessionStatusResponse,
  next: ManagedSessionStatusResponse,
): boolean {
  return (
    sameManagedSession(current.session, next.session) &&
    current.statusUnavailable === next.statusUnavailable &&
    sameRecovery(current.recovery, next.recovery) &&
    sameContainerStatus(current.container, next.container)
  )
}

function sameRecovery(
  current: ManagedSessionRecovery | null,
  next: ManagedSessionRecovery | null,
): boolean {
  if (current === null || next === null) {
    return current === next
  }
  return (
    current.status === next.status &&
    current.title === next.title &&
    current.detail === next.detail &&
    current.remedy === next.remedy &&
    current.repairAvailable === next.repairAvailable
  )
}

function sameContainerStatus(
  current: RunStatusContainer | null,
  next: RunStatusContainer | null,
): boolean {
  if (current === null || next === null) {
    return current === next
  }
  return sameRunStatusContainer(current, next)
}
