import { useEffect, useRef, useState } from "react"

import {
  clearClaudeOAuth,
  getClaudeOAuthStatus,
  startClaudeOAuthSignIn,
  type ClaudeOAuthStatusResponse,
} from "@/commands/runhaven"

import { errorMessage } from "./types"

const STATUS_POLL_MS = 1500

// Drives Claude path C (sign in on your Mac). It triggers the host sign-in
// (Rust shows a native confirmation first), polls the sanitized status while the
// host capture runs, and can remove the saved token. No secret is handled here:
// the token lives only in the Mac's Keychain and never reaches this hook. The
// status carries a plain `ageDays` from the item's own non-secret creation date,
// used to drive the re-sign-in cadence prompt.
export function useClaudeOAuthSignIn(onSignedIn?: () => void) {
  const [status, setStatus] = useState<ClaudeOAuthStatusResponse | null>(null)
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState("")
  const [clearing, setClearing] = useState(false)
  const [clearError, setClearError] = useState("")
  const [clearMessage, setClearMessage] = useState("")

  const signedInRef = useRef(false)
  const onSignedInRef = useRef(onSignedIn)
  useEffect(() => {
    onSignedInRef.current = onSignedIn
  }, [onSignedIn])

  function finishIfSignedIn(next: ClaudeOAuthStatusResponse): void {
    if (next.status === "signed_in" && !signedInRef.current) {
      signedInRef.current = true
      onSignedInRef.current?.()
    }
  }

  async function refreshStatus(): Promise<void> {
    try {
      const next = await getClaudeOAuthStatus()
      setStatus((current) =>
        current && sameStatus(current, next) ? current : next,
      )
    } catch (cause) {
      setError(errorMessage(cause))
    }
  }

  // Reflect the current saved-token state on mount.
  useEffect(() => {
    void refreshStatus()
    // refreshStatus reads no reactive inputs; run it once.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  // Poll only while a sign-in is actually running on the host, so a signed-in or
  // signed-out control does no background work. Keyed on the boolean so routine
  // polls do not tear down and re-arm the interval.
  const signing = status?.status === "signing"
  useEffect(() => {
    if (!signing) {
      return
    }
    let active = true
    const timer = setInterval(() => {
      void getClaudeOAuthStatus()
        .then((next) => {
          if (!active) {
            return
          }
          setError("")
          setStatus((current) =>
            current && sameStatus(current, next) ? current : next,
          )
          if (next.status !== "signing") {
            active = false
            clearInterval(timer)
            finishIfSignedIn(next)
          }
        })
        .catch((cause) => {
          if (active) {
            setError(errorMessage(cause))
          }
        })
    }, STATUS_POLL_MS)
    return () => {
      active = false
      clearInterval(timer)
    }
    // finishIfSignedIn reads only refs, so the interval stays keyed on `signing`.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [signing])

  async function signIn(): Promise<void> {
    if (busy) {
      return
    }
    setBusy(true)
    setError("")
    setClearMessage("")
    setClearError("")
    signedInRef.current = false
    try {
      const started = await startClaudeOAuthSignIn()
      // Reflect the acknowledgement; the poll above then tracks host progress
      // and flips to signed_in or failed.
      setStatus({
        status: started.state,
        reason: null,
        signInId: started.signInId,
        ageDays: null,
      })
    } catch (cause) {
      setError(errorMessage(cause))
      // The sign-in never started (for example the native confirm was
      // canceled), so re-read the true, unchanged state.
      void refreshStatus()
    } finally {
      setBusy(false)
    }
  }

  async function clear(): Promise<void> {
    if (clearing) {
      return
    }
    setClearing(true)
    setClearError("")
    setClearMessage("")
    try {
      const outcome = await clearClaudeOAuth()
      setClearMessage(outcome.detail)
      // Allow a later sign-in to fire onSignedIn again.
      signedInRef.current = false
      await refreshStatus()
    } catch (cause) {
      setClearError(errorMessage(cause))
    } finally {
      setClearing(false)
    }
  }

  return {
    status,
    busy,
    error,
    signIn,
    clearing,
    clearError,
    clearMessage,
    clear,
  }
}

function sameStatus(
  current: ClaudeOAuthStatusResponse,
  next: ClaudeOAuthStatusResponse,
): boolean {
  return (
    current.status === next.status &&
    current.reason === next.reason &&
    current.signInId === next.signInId &&
    current.ageDays === next.ageDays
  )
}
