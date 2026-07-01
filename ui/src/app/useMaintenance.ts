import { useEffect, useRef, useState } from "react"

import {
  getImageBuildStatus,
  getNetworkPrunePreview,
  pruneManagedNetworks,
  rebuildAgentImage,
  type ImageBuildStatusResponse,
  type NetworkPruneResponse,
} from "@/commands/runhaven"

import { errorMessage } from "./types"

const BUILD_POLL_MS = 1500

// The read-only status commands the hook uses are exported so tests can prove
// the client mapping without driving polling timers.
export const maintenanceClients = {
  rebuildAgentImage,
  getImageBuildStatus,
  getNetworkPrunePreview,
  pruneManagedNetworks,
}

function isTerminal(state: string): boolean {
  return state === "succeeded" || state === "failed"
}

export function useImageRebuild() {
  const [confirm, setConfirm] = useState(false)
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState("")
  const [status, setStatus] = useState<ImageBuildStatusResponse | null>(null)
  const buildIdRef = useRef<string>("")

  useEffect(() => {
    if (!status || isTerminal(status.state)) {
      return
    }
    const buildId = buildIdRef.current
    if (!buildId) {
      return
    }
    let active = true
    const timer = setInterval(() => {
      void getImageBuildStatus(buildId)
        .then((next) => {
          if (active) {
            setStatus(next)
          }
        })
        .catch((cause) => {
          if (active) {
            setError(errorMessage(cause))
          }
        })
    }, BUILD_POLL_MS)
    return () => {
      active = false
      clearInterval(timer)
    }
  }, [status])

  async function rebuild(agent: string) {
    if (busy || !confirm || !agent) {
      return
    }
    setBusy(true)
    setError("")
    setStatus(null)
    try {
      const started = await rebuildAgentImage(agent)
      buildIdRef.current = started.buildId
      setStatus({
        buildId: started.buildId,
        agent: started.agent,
        tag: started.tag,
        state: started.state,
        label: started.label,
        detail: started.detail,
        outputTail: "",
        updatedAt: "",
      })
      setConfirm(false)
    } catch (cause) {
      setError(errorMessage(cause))
    } finally {
      setBusy(false)
    }
  }

  return { confirm, setConfirm, busy, error, status, rebuild }
}

export function useNetworkCleanup() {
  const [count, setCount] = useState<number | null>(null)
  const [loading, setLoading] = useState(false)
  const [loadError, setLoadError] = useState("")
  const [confirm, setConfirm] = useState(false)
  const [busy, setBusy] = useState(false)
  const [actionError, setActionError] = useState("")
  const [result, setResult] = useState<NetworkPruneResponse | null>(null)

  async function loadPreview() {
    setLoading(true)
    setLoadError("")
    setResult(null)
    try {
      const preview = await getNetworkPrunePreview()
      setCount(preview.count)
    } catch (cause) {
      setLoadError(errorMessage(cause))
    } finally {
      setLoading(false)
    }
  }

  async function prune() {
    if (busy || !confirm) {
      return
    }
    setBusy(true)
    setActionError("")
    try {
      const outcome = await pruneManagedNetworks()
      setResult(outcome)
      setConfirm(false)
      await loadPreview()
    } catch (cause) {
      setActionError(errorMessage(cause))
    } finally {
      setBusy(false)
    }
  }

  return {
    count,
    loading,
    loadError,
    confirm,
    setConfirm,
    busy,
    actionError,
    result,
    loadPreview,
    prune,
  }
}
