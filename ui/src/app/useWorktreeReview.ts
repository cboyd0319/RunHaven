import { useEffect, useRef, useState } from "react"

import {
  discardManagedSessionWorktree,
  getManagedSessionWorktree,
  mergeManagedSessionWorktree,
  recoverManagedSessionWorktree,
  type WorktreeActionResponse,
  type WorktreeReviewResponse,
} from "@/commands/runhaven"

import { errorMessage } from "./types"
import {
  worktreeActionRemovesWorktree,
  worktreeActionSuccessMessage,
  type WorktreeAction,
} from "./worktreeReview"

// Each review action maps to exactly one typed client call. Exported so tests
// can prove the mapping without driving the whole hook.
export const worktreeActionClients: Record<
  WorktreeAction,
  (sessionId: string, reviewId: string) => Promise<WorktreeActionResponse>
> = {
  merge: mergeManagedSessionWorktree,
  recover: recoverManagedSessionWorktree,
  discard: discardManagedSessionWorktree,
}

export function useWorktreeReview(
  sessionId: string,
  onResolved: () => Promise<void> | void,
) {
  const [review, setReview] = useState<WorktreeReviewResponse | null>(null)
  const [loading, setLoading] = useState(false)
  const [loadError, setLoadError] = useState("")
  const [busy, setBusy] = useState(false)
  const [message, setMessage] = useState("")
  const [nextSteps, setNextSteps] = useState<string[]>([])
  const [actionError, setActionError] = useState("")
  const [loadAttempt, setLoadAttempt] = useState(0)
  const onResolvedRef = useRef(onResolved)

  useEffect(() => {
    onResolvedRef.current = onResolved
  }, [onResolved])

  useEffect(() => {
    let active = true
    setReview(null)
    setLoadError("")
    setMessage("")
    setNextSteps([])
    setActionError("")
    setLoading(true)
    void getManagedSessionWorktree(sessionId)
      .then(async (next) => {
        if (!active) {
          return
        }
        if (loadAttempt > 0) {
          await onResolvedRef.current()
        }
        if (active) {
          setReview(next)
        }
      })
      .catch((cause) => {
        if (active) {
          setLoadError(errorMessage(cause))
        }
      })
      .finally(() => {
        if (active) {
          setLoading(false)
        }
      })
    return () => {
      active = false
    }
  }, [sessionId, loadAttempt])

  async function runAction(action: WorktreeAction) {
    if (busy || review === null) {
      return
    }
    setBusy(true)
    setActionError("")
    setMessage("")
    setNextSteps([])
    try {
      const { result } = await worktreeActionClients[action](
        sessionId,
        review.reviewId,
      )
      setMessage(worktreeActionSuccessMessage(result))
      setNextSteps(result.nextSteps)
      if (worktreeActionRemovesWorktree(action)) {
        await onResolved()
      } else {
        const next = await getManagedSessionWorktree(sessionId)
        setReview(next)
      }
    } catch (cause) {
      setActionError(errorMessage(cause))
    } finally {
      setBusy(false)
    }
  }

  return {
    review,
    loading,
    loadError,
    busy,
    message,
    nextSteps,
    actionError,
    retry: () => setLoadAttempt((attempt) => attempt + 1),
    runAction,
  }
}
