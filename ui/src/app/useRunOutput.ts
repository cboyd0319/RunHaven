import { useCallback, useEffect, useRef, useState } from "react"

import {
  getLogSnapshot,
  getManagedSessionLogSnapshot,
  type LogSnapshotResponse,
} from "@/commands/runhaven"

import { errorMessage } from "./types"

const LIVE_OUTPUT_REFRESH_MS = 3_000

export function useRunOutput({
  targetId,
  sessionId,
  runId,
}: {
  targetId: string | null
  sessionId: string | null
  runId: string | null
}) {
  const [snapshot, setSnapshot] = useState<LogSnapshotResponse | null>(null)
  const [snapshotTargetId, setSnapshotTargetId] = useState<string | null>(null)
  // The acknowledgement is bound to the target it was given for. Deriving
  // `acknowledged` from a target match means a target change flips it false in
  // the same render, before any effect runs, so the live poll can never fetch
  // bounded output for a target the user has not re-acknowledged. Fail-closed.
  const [acknowledgedTargetId, setAcknowledgedTargetId] = useState<string | null>(
    null,
  )
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState("")
  const [liveEnabled, setLiveEnabled] = useState(false)
  const [liveRefreshing, setLiveRefreshing] = useState(false)
  const targetRef = useRef<string | null>(targetId)
  const requestInFlight = useRef(false)

  const acknowledged = targetId !== null && acknowledgedTargetId === targetId
  const setAcknowledged = useCallback(
    (value: boolean) => {
      setAcknowledgedTargetId(value ? targetId : null)
    },
    [targetId],
  )
  const visibleSnapshot = snapshotTargetId === targetId ? snapshot : null
  const liveAvailable = Boolean(sessionId)

  const loadForTarget = useCallback(
    async ({
      expectedTargetId,
      expectedSessionId,
      expectedRunId,
      background,
    }: {
      expectedTargetId: string
      expectedSessionId: string | null
      expectedRunId: string | null
      background: boolean
    }) => {
      if (requestInFlight.current) {
        return
      }
      requestInFlight.current = true
      if (background) {
        setLiveRefreshing(true)
      } else {
        setLoading(true)
      }
      setError("")
      try {
        if (expectedSessionId) {
          const response = await getManagedSessionLogSnapshot(expectedSessionId, {
            confirmSensitiveOutput: acknowledged,
          })
          if (targetRef.current !== expectedTargetId) {
            return
          }
          setSnapshot((current) =>
            current && sameLogSnapshot(current, response.snapshot)
              ? current
              : response.snapshot,
          )
        } else if (expectedRunId) {
          const response = await getLogSnapshot(expectedRunId, {
            confirmSensitiveOutput: acknowledged,
          })
          if (targetRef.current !== expectedTargetId) {
            return
          }
          setSnapshot((current) =>
            current && sameLogSnapshot(current, response) ? current : response,
          )
        }
        if (targetRef.current === expectedTargetId) {
          setSnapshotTargetId(expectedTargetId)
        }
      } catch (cause) {
        if (targetRef.current === expectedTargetId) {
          if (!background) {
            setSnapshot(null)
            setSnapshotTargetId(null)
          }
          setError(errorMessage(cause))
        }
      } finally {
        requestInFlight.current = false
        if (background) {
          setLiveRefreshing(false)
        } else {
          setLoading(false)
        }
      }
    },
    [acknowledged],
  )

  const loadSnapshot = useCallback(async () => {
    if (!targetId) {
      return
    }
    await loadForTarget({
      expectedTargetId: targetId,
      expectedSessionId: sessionId,
      expectedRunId: runId,
      background: false,
    })
  }, [loadForTarget, runId, sessionId, targetId])

  useEffect(() => {
    targetRef.current = targetId
  }, [targetId])

  useEffect(() => {
    setAcknowledgedTargetId(null)
    setSnapshot(null)
    setSnapshotTargetId(null)
    setError("")
    setLiveEnabled(false)
    setLiveRefreshing(false)
  }, [targetId])

  useEffect(() => {
    if (!acknowledged) {
      setLiveEnabled(false)
      setLiveRefreshing(false)
    }
  }, [acknowledged])

  useEffect(() => {
    if (!liveEnabled || !acknowledged || !sessionId || !targetId) {
      return
    }

    const expectedTargetId = targetId
    const expectedSessionId = sessionId
    let cancelled = false
    async function poll() {
      if (cancelled) {
        return
      }
      await loadForTarget({
        expectedTargetId,
        expectedSessionId,
        expectedRunId: null,
        background: true,
      })
    }

    void poll()
    const interval = window.setInterval(
      () => void poll(),
      LIVE_OUTPUT_REFRESH_MS,
    )

    return () => {
      cancelled = true
      window.clearInterval(interval)
    }
  }, [acknowledged, liveEnabled, loadForTarget, sessionId, targetId])

  return {
    acknowledged,
    loading,
    error,
    visibleSnapshot,
    liveAvailable,
    liveEnabled,
    liveRefreshing,
    setAcknowledged,
    setLiveEnabled,
    loadSnapshot,
  }
}

// Guards the 3s live-output poll so an unchanged snapshot does not re-render the
// whole app. Compares the content fields the output card renders; ignores
// volatile bookkeeping (capturedAt, requestedLines, runId, source) that does
// not change what the user sees.
export function sameLogSnapshot(
  current: LogSnapshotResponse,
  next: LogSnapshotResponse,
): boolean {
  return (
    current.text === next.text &&
    current.returnedLines === next.returnedLines &&
    current.truncated === next.truncated &&
    sameStringList(current.warnings, next.warnings)
  )
}

function sameStringList(current: string[], next: string[]): boolean {
  return (
    current.length === next.length &&
    current.every((value, index) => value === next[index])
  )
}
