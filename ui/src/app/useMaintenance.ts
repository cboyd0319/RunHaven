import { useEffect, useRef, useState } from "react"

import {
  getAgentSignInStatus,
  getImageBuildStatus,
  getNetworkPrunePreview,
  pruneManagedNetworks,
  rebuildAgentImage,
  startAgentSignIn,
  type AgentSignInStatusResponse,
  type ImageBuildStatusResponse,
  type NetworkPruneResponse,
} from "@/commands/runhaven"

import { errorMessage } from "./types"

const BUILD_POLL_MS = 1500
const SIGN_IN_POLL_MS = 1500

// The read-only status commands the hook uses are exported so tests can prove
// the client mapping without driving polling timers.
export const maintenanceClients = {
  rebuildAgentImage,
  getImageBuildStatus,
  getNetworkPrunePreview,
  pruneManagedNetworks,
  startAgentSignIn,
  getAgentSignInStatus,
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

// Drives an in-app agent sign-in: it triggers the backend sign-in (which shows
// a native confirmation before anything runs), then polls sanitized progress so
// the sign-in link and short-lived device code show. On success it calls
// `onSignedIn` once so the app can refresh auth status. No secret is handled
// here; the credential stays with RunHaven core.
export function useAgentSignIn(onSignedIn?: () => void) {
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState("")
  const [status, setStatus] = useState<AgentSignInStatusResponse | null>(null)
  const signInIdRef = useRef<string>("")
  const signedInRef = useRef(false)

  useEffect(() => {
    if (!status) {
      return
    }
    if (isTerminal(status.state)) {
      if (status.state === "succeeded" && !signedInRef.current) {
        signedInRef.current = true
        onSignedIn?.()
      }
      return
    }
    const signInId = signInIdRef.current
    if (!signInId) {
      return
    }
    let active = true
    const timer = setInterval(() => {
      void getAgentSignInStatus(signInId)
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
    }, SIGN_IN_POLL_MS)
    return () => {
      active = false
      clearInterval(timer)
    }
  }, [status, onSignedIn])

  async function signIn(agent: string) {
    if (busy || !agent) {
      return
    }
    setBusy(true)
    setError("")
    setStatus(null)
    signedInRef.current = false
    try {
      const started = await startAgentSignIn(agent)
      signInIdRef.current = started.signInId
      setStatus({
        signInId: started.signInId,
        agent: started.agent,
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

  return { busy, error, status, signIn }
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
