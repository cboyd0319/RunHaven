import { useCallback, useEffect, useRef, useState } from "react"

import {
  getLaunchStatus,
  type LaunchStatusResponse,
} from "@/commands/runhaven"

import { errorMessage } from "./types"

const LAUNCH_STATUS_POLL_MS = 2_000

export function useLaunchStatus(runId: string | null) {
  const [data, setData] = useState<LaunchStatusResponse | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState("")
  const inFlight = useRef(false)
  const dataRef = useRef<LaunchStatusResponse | null>(null)

  const loadStatus = useCallback(async () => {
    if (!runId || inFlight.current) {
      return
    }
    inFlight.current = true
    setLoading(dataRef.current === null)
    setError("")
    try {
      const nextStatus = await getLaunchStatus(runId)
      dataRef.current = nextStatus
      setData((current) =>
        current && sameLaunchStatus(current, nextStatus) ? current : nextStatus,
      )
    } catch (cause) {
      setError(errorMessage(cause))
    } finally {
      inFlight.current = false
      setLoading(false)
    }
  }, [runId])

  useEffect(() => {
    dataRef.current = null
    setData(null)
    setError("")
    setLoading(Boolean(runId))
    inFlight.current = false
    if (!runId) {
      setLoading(false)
      return
    }
    void loadStatus()
    const interval = window.setInterval(() => void loadStatus(), LAUNCH_STATUS_POLL_MS)
    return () => window.clearInterval(interval)
  }, [loadStatus, runId])

  return { data, loading, error, loadStatus }
}

export function sameLaunchStatus(
  current: LaunchStatusResponse,
  next: LaunchStatusResponse,
): boolean {
  return (
    current.runId === next.runId &&
    current.state === next.state &&
    current.label === next.label &&
    current.detail === next.detail &&
    current.updatedAt === next.updatedAt
  )
}
