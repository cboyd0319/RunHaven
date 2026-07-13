import { useEffect, useRef, useState } from "react"

import {
  getImageBuildStatus,
  getNetworkPrunePreview,
  isTerminalState,
  pruneManagedNetworks,
  rebuildAgentImage,
  type ImageBuildStatusResponse,
  type NetworkPruneResponse,
} from "@/commands/runhaven"

import { errorMessage } from "./types"

const BUILD_POLL_MS = 1500
// Compile-time drift guards for the equality helpers below; see
// appStatusEquality.ts for the pattern.
const imageBuildStatusChecked = {
  buildId: true,
  agent: true,
  tag: true,
  state: true,
  label: true,
  detail: true,
  outputTail: true,
  updatedAt: true,
} satisfies Record<keyof ImageBuildStatusResponse, true>
void imageBuildStatusChecked

// The read-only status commands the hook uses are exported so tests can prove
// the client mapping without driving polling timers.
export const maintenanceClients = {
  rebuildAgentImage,
  getImageBuildStatus,
  getNetworkPrunePreview,
  pruneManagedNetworks,
}
function sameImageBuildStatus(
  current: ImageBuildStatusResponse,
  next: ImageBuildStatusResponse,
): boolean {
  return (
    current.buildId === next.buildId &&
    current.agent === next.agent &&
    current.tag === next.tag &&
    current.state === next.state &&
    current.label === next.label &&
    current.detail === next.detail &&
    current.outputTail === next.outputTail &&
    current.updatedAt === next.updatedAt
  )
}

export function useImageRebuild() {
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState("")
  const [status, setStatus] = useState<ImageBuildStatusResponse | null>(null)
  const terminalRef = useRef(false)

  // Key the poll on the build id, not the whole status object, so a routine
  // progress update does not tear down and re-arm the interval every tick.
  const buildId = status?.buildId ?? ""

  useEffect(() => {
    if (!buildId || terminalRef.current) {
      return
    }
    let active = true
    const timer = setInterval(() => {
      void getImageBuildStatus(buildId)
        .then((next) => {
          if (!active) {
            return
          }
          setError("")
          setStatus((current) =>
            current && sameImageBuildStatus(current, next) ? current : next,
          )
          if (isTerminalState(next.state)) {
            terminalRef.current = true
            active = false
            clearInterval(timer)
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
  }, [buildId])

  async function rebuild(agent: string) {
    if (busy || !agent) {
      return
    }
    setBusy(true)
    setError("")
    setStatus(null)
    terminalRef.current = false
    try {
      const started = await rebuildAgentImage(agent)
      terminalRef.current = isTerminalState(started.state)
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
    } catch (cause) {
      setError(errorMessage(cause))
    } finally {
      setBusy(false)
    }
  }

  return { busy, error, status, rebuild }
}

export function useNetworkCleanup() {
  const [count, setCount] = useState<number | null>(null)
  const [loading, setLoading] = useState(false)
  const [loadError, setLoadError] = useState("")
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
    if (busy) {
      return
    }
    setBusy(true)
    setActionError("")
    try {
      const outcome = await pruneManagedNetworks()
      setResult(outcome)
      // Refresh the count directly; loadPreview would clear the success alert.
      setCount((await getNetworkPrunePreview()).count)
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
    busy,
    actionError,
    result,
    loadPreview,
    prune,
  }
}
