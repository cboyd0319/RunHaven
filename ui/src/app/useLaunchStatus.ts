import { useCallback, useEffect, useRef, useState } from "react"

import {
  getLaunchStatus,
  type LaunchStatusResponse,
} from "@/commands/runhaven"

import { errorMessage } from "./types"

const LAUNCH_STATUS_POLL_MS = 2_000

type StoredLaunchStatus = {
  runId: string
  data: LaunchStatusResponse
}

export function useLaunchStatus(runId: string | null) {
  // Pair the fetched status with its runId so the render derives null the moment
  // runId changes, before the reset effect runs after paint.
  const [stored, setStored] = useState<StoredLaunchStatus | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState("")
  const inFlight = useRef(false)
  const storedRef = useRef<StoredLaunchStatus | null>(null)
  const requestSeq = useRef(0)

  const data = stored?.runId === runId ? stored.data : null

  const loadStatus = useCallback(
    async ({ background = false }: { background?: boolean } = {}) => {
      if (!runId || inFlight.current) {
        return
      }
      const requestId = requestSeq.current + 1
      requestSeq.current = requestId
      inFlight.current = true
      setLoading(storedRef.current?.runId !== runId)
      try {
        const nextStatus = await getLaunchStatus(runId)
        if (requestSeq.current !== requestId) {
          return
        }
        const nextStored: StoredLaunchStatus = { runId, data: nextStatus }
        storedRef.current = nextStored
        setError("")
        setStored((current) =>
          current &&
          current.runId === runId &&
          sameLaunchStatus(current.data, nextStatus)
            ? current
            : nextStored,
        )
      } catch (cause) {
        if (requestSeq.current === requestId) {
          setError(errorMessage(cause))
          // Keep the last-good status across a failed background poll; only an
          // explicit reload clears it to surface the error.
          if (!background) {
            if (storedRef.current?.runId === runId) {
              storedRef.current = null
            }
            setStored((current) => (current?.runId === runId ? null : current))
          }
        }
      } finally {
        if (requestSeq.current === requestId) {
          inFlight.current = false
          setLoading(false)
        }
      }
    },
    [runId],
  )

  useEffect(() => {
    requestSeq.current += 1
    storedRef.current = null
    setStored(null)
    setError("")
    setLoading(Boolean(runId))
    inFlight.current = false
    if (!runId) {
      setLoading(false)
      return
    }
    void loadStatus()
    const interval = window.setInterval(
      () => void loadStatus({ background: true }),
      LAUNCH_STATUS_POLL_MS,
    )
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
