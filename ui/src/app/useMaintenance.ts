import { useEffect, useRef, useState } from "react"

import {
  getAgentSignInStatus,
  getImageBuildStatus,
  getNetworkPrunePreview,
  isTerminalState,
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

const agentSignInStatusChecked = {
  signInId: true,
  agent: true,
  state: true,
  label: true,
  detail: true,
  outputTail: true,
  updatedAt: true,
} satisfies Record<keyof AgentSignInStatusResponse, true>
void agentSignInStatusChecked

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

function sameAgentSignInStatus(
  current: AgentSignInStatusResponse,
  next: AgentSignInStatusResponse,
): boolean {
  return (
    current.signInId === next.signInId &&
    current.agent === next.agent &&
    current.state === next.state &&
    current.label === next.label &&
    current.detail === next.detail &&
    current.outputTail === next.outputTail &&
    current.updatedAt === next.updatedAt
  )
}

export function useImageRebuild() {
  const [confirm, setConfirm] = useState(false)
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
    if (busy || !confirm || !agent) {
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
  const signedInRef = useRef(false)
  const terminalRef = useRef(false)
  // Hold the callback in a ref so an inline lambda from the parent does not
  // re-arm the interval on every parent re-render (which can starve the poll).
  const onSignedInRef = useRef(onSignedIn)

  useEffect(() => {
    onSignedInRef.current = onSignedIn
  }, [onSignedIn])

  function finishIfSucceeded(state: string): void {
    if (state === "succeeded" && !signedInRef.current) {
      signedInRef.current = true
      onSignedInRef.current?.()
    }
  }

  // Key the poll on the sign-in id, not the whole status object, so progress
  // updates do not tear down and re-arm the interval every tick.
  const signInId = status?.signInId ?? ""

  useEffect(() => {
    if (!signInId || terminalRef.current) {
      return
    }
    let active = true
    const timer = setInterval(() => {
      void getAgentSignInStatus(signInId)
        .then((next) => {
          if (!active) {
            return
          }
          setError("")
          setStatus((current) =>
            current && sameAgentSignInStatus(current, next) ? current : next,
          )
          if (isTerminalState(next.state)) {
            terminalRef.current = true
            active = false
            clearInterval(timer)
            finishIfSucceeded(next.state)
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
    // finishIfSucceeded reads only refs, so it is intentionally left out of the
    // deps to keep the interval keyed solely on the sign-in id.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [signInId])

  async function signIn(agent: string) {
    if (busy || !agent) {
      return
    }
    setBusy(true)
    setError("")
    setStatus(null)
    signedInRef.current = false
    terminalRef.current = false
    try {
      const started = await startAgentSignIn(agent)
      terminalRef.current = isTerminalState(started.state)
      setStatus({
        signInId: started.signInId,
        agent: started.agent,
        state: started.state,
        label: started.label,
        detail: started.detail,
        outputTail: "",
        updatedAt: "",
      })
      finishIfSucceeded(started.state)
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
    confirm,
    setConfirm,
    busy,
    actionError,
    result,
    loadPreview,
    prune,
  }
}
