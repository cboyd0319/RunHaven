import { useCallback, useEffect, useRef, useState } from "react"

import {
  getManagedSessionStatus,
  type ManagedSessionRecovery,
  type ManagedSessionStatusResponse,
  type RunStatusContainer,
} from "@/commands/runhaven"

import { sameManagedSession } from "./sessionList"
import { errorMessage } from "./types"

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
  const inFlight = useRef(false)

  const data = stored?.sessionId === sessionId ? stored.data : null

  const loadStatus = useCallback(
    async ({ background = false }: { background?: boolean } = {}) => {
      if (!sessionId || inFlight.current) {
        return
      }
      const requestId = requestSeq.current + 1
      requestSeq.current = requestId
      inFlight.current = true
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
          if (!background) {
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
          inFlight.current = false
        }
      }
    },
    [sessionId],
  )

  useEffect(() => {
    requestSeq.current += 1
    inFlight.current = false
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
  return (
    current.state === next.state &&
    current.image === next.image &&
    current.startedAt === next.startedAt &&
    current.resources.cpus === next.resources.cpus &&
    current.resources.memoryBytes === next.resources.memoryBytes &&
    current.networks.length === next.networks.length &&
    current.networks.every((network, index) => {
      const nextNetwork = next.networks[index]
      return (
        nextNetwork !== undefined &&
        network.network === nextNetwork.network &&
        network.hostname === nextNetwork.hostname &&
        network.ipv4Address === nextNetwork.ipv4Address &&
        network.ipv4Gateway === nextNetwork.ipv4Gateway &&
        network.ipv6Address === nextNetwork.ipv6Address
      )
    })
  )
}
