import { useCallback, useEffect, useRef, useState } from "react"

import {
  getManagedSessionStatus,
  type ManagedSessionRecovery,
  type ManagedSessionStatusResponse,
  type RunStatusContainer,
} from "@/commands/runhaven"

import { sameManagedSession } from "./sessionList"

const MANAGED_SESSION_STATUS_POLL_MS = 4_000

export function useManagedSessionStatus(sessionId: string | null) {
  const [data, setData] = useState<ManagedSessionStatusResponse | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState("")
  const dataRef = useRef<ManagedSessionStatusResponse | null>(null)
  const requestSeq = useRef(0)
  const inFlight = useRef(false)

  const loadStatus = useCallback(async () => {
    if (!sessionId || inFlight.current) {
      return
    }
    const requestId = requestSeq.current + 1
    requestSeq.current = requestId
    inFlight.current = true
    setLoading(dataRef.current === null)
    setError("")
    try {
      const nextStatus = await getManagedSessionStatus(sessionId)
      if (requestSeq.current !== requestId) {
        return
      }
      dataRef.current = nextStatus
      setData((current) =>
        current && sameManagedSessionStatus(current, nextStatus)
          ? current
          : nextStatus,
      )
    } catch (cause) {
      if (requestSeq.current === requestId) {
        setError(errorMessage(cause))
        dataRef.current = null
        setData(null)
      }
    } finally {
      if (requestSeq.current === requestId) {
        setLoading(false)
        inFlight.current = false
      }
    }
  }, [sessionId])

  useEffect(() => {
    requestSeq.current += 1
    inFlight.current = false
    dataRef.current = null
    setData(null)
    setError("")
    setLoading(Boolean(sessionId))
    if (!sessionId) {
      setLoading(false)
      return
    }

    void loadStatus()
    const interval = window.setInterval(() => void loadStatus(), MANAGED_SESSION_STATUS_POLL_MS)
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

function errorMessage(cause: unknown): string {
  return cause instanceof Error ? cause.message : String(cause)
}
