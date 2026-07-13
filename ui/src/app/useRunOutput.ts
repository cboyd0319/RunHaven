import { useCallback, useEffect, useRef, useState } from "react"

import {
  getLogSnapshot,
  getManagedSessionLogSnapshot,
  type LogSnapshotResponse,
} from "@/commands/runhaven"

import { errorMessage } from "./types"

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
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState("")
  const targetRef = useRef<string | null>(targetId)
  const requestsInFlight = useRef(new Set<string>())

  const visibleSnapshot = snapshotTargetId === targetId ? snapshot : null

  const loadForTarget = useCallback(
    async ({
      expectedTargetId,
      expectedSessionId,
      expectedRunId,
    }: {
      expectedTargetId: string
      expectedSessionId: string | null
      expectedRunId: string | null
    }) => {
      if (requestsInFlight.current.has(expectedTargetId)) {
        return
      }
      requestsInFlight.current.add(expectedTargetId)
      setLoading(true)
      setError("")
      try {
        if (expectedSessionId) {
          const response = await getManagedSessionLogSnapshot(expectedSessionId, {
            confirmSensitiveOutput: true,
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
            confirmSensitiveOutput: true,
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
          setSnapshot(null)
          setSnapshotTargetId(null)
          setError(errorMessage(cause))
        }
      } finally {
        requestsInFlight.current.delete(expectedTargetId)
        if (targetRef.current === expectedTargetId) {
          setLoading(false)
        }
      }
    },
    [],
  )

  const loadSnapshot = useCallback(async () => {
    if (!targetId) {
      return
    }
    await loadForTarget({
      expectedTargetId: targetId,
      expectedSessionId: sessionId,
      expectedRunId: runId,
    })
  }, [loadForTarget, runId, sessionId, targetId])

  useEffect(() => {
    targetRef.current = targetId
  }, [targetId])

  useEffect(() => {
    setSnapshot(null)
    setSnapshotTargetId(null)
    setError("")
  }, [targetId])

  return {
    loading,
    error,
    visibleSnapshot,
    loadSnapshot,
  }
}

// Avoids replacing an unchanged snapshot after an explicit refresh. Compares
// the content fields the output card renders; ignores
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
