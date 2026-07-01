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

  async function loadImageStatus(agentName: string) {
    const requestId = imageStatusRequestSeq.current + 1
    imageStatusRequestSeq.current = requestId
    setImageLoading(true)
    setImageError("")
    try {
      const nextImageStatus = await getImageStatus(agentName)
      if (imageStatusRequestSeq.current !== requestId) {
        return
      }
      setImageStatus((current) =>
        current && sameImageStatus(current, nextImageStatus)
          ? current
          : nextImageStatus,
      )
    } catch (cause) {
      if (imageStatusRequestSeq.current === requestId) {
        setImageStatus(null)
        setImageError(errorMessage(cause))
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
    setRunStatusLoading(true)
    setRunStatusError("")
    try {
      const nextRunStatus = await getRunStatus(runId)
      if (runStatusRequestSeq.current !== requestId) {
        return
      }
      setRunStatus((current) =>
        current && sameRunStatus(current, nextRunStatus) ? current : nextRunStatus,
      )
    } catch (cause) {
      if (runStatusRequestSeq.current === requestId) {
        setRunStatus(null)
        setRunStatusError(errorMessage(cause))
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
