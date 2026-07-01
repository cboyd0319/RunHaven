import { useRef, useState } from "react"

import {
  getImageStatus,
  getRunStatus,
  type ImageStatusResponse,
  type RunStatusResponse,
} from "@/commands/runhaven"

import { sameImageStatus, sameRunStatus } from "./appStatusEquality"
import { errorMessage } from "./types"

export function useStatusLoaders() {
  const [imageLoading, setImageLoading] = useState(false)
  const [imageStatus, setImageStatus] = useState<ImageStatusResponse | null>(null)
  const [imageError, setImageError] = useState("")
  const [runStatus, setRunStatus] = useState<RunStatusResponse | null>(null)
  const [runStatusLoading, setRunStatusLoading] = useState(false)
  const [runStatusError, setRunStatusError] = useState("")
  const imageStatusRequestSeq = useRef(0)
  const runStatusRequestSeq = useRef(0)
  // Track the target each card last has good data for, so a transient failure
  // keeps the last-good card and only a genuine target change clears it.
  const imageStatusTarget = useRef<string | null>(null)
  const runStatusTarget = useRef<string | null>(null)

  async function loadImageStatus(agentName: string) {
    const requestId = imageStatusRequestSeq.current + 1
    imageStatusRequestSeq.current = requestId
    const sameTarget = imageStatusTarget.current === agentName
    setImageLoading(true)
    if (!sameTarget) {
      setImageError("")
    }
    try {
      const nextImageStatus = await getImageStatus(agentName)
      if (imageStatusRequestSeq.current !== requestId) {
        return
      }
      imageStatusTarget.current = agentName
      setImageError("")
      setImageStatus((current) =>
        current && sameImageStatus(current, nextImageStatus)
          ? current
          : nextImageStatus,
      )
    } catch (cause) {
      if (imageStatusRequestSeq.current === requestId) {
        setImageError(errorMessage(cause))
        // Keep last-good status for the same agent; drop only when the target
        // changed so a different agent never shows stale image data.
        if (!sameTarget) {
          setImageStatus(null)
        }
      }
    } finally {
      if (imageStatusRequestSeq.current === requestId) {
        setImageLoading(false)
      }
    }
  }

  async function loadRunStatus(runId: string) {
    const requestId = runStatusRequestSeq.current + 1
    runStatusRequestSeq.current = requestId
    const sameTarget = runStatusTarget.current === runId
    setRunStatusLoading(true)
    if (!sameTarget) {
      setRunStatusError("")
    }
    try {
      const nextRunStatus = await getRunStatus(runId)
      if (runStatusRequestSeq.current !== requestId) {
        return
      }
      runStatusTarget.current = runId
      setRunStatusError("")
      setRunStatus((current) =>
        current && sameRunStatus(current, nextRunStatus) ? current : nextRunStatus,
      )
    } catch (cause) {
      if (runStatusRequestSeq.current === requestId) {
        setRunStatusError(errorMessage(cause))
        if (!sameTarget) {
          setRunStatus(null)
        }
      }
    } finally {
      if (runStatusRequestSeq.current === requestId) {
        setRunStatusLoading(false)
      }
    }
  }

  return {
    imageLoading,
    imageStatus,
    imageError,
    runStatus,
    runStatusLoading,
    runStatusError,
    loadImageStatus,
    loadRunStatus,
  }
}
