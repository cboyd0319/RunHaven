import { useEffect, useRef, useState } from "react"

import {
  clearClaudeOAuth,
  getClaudeOAuthStatus,
  startClaudeOAuthSignIn,
  type ClaudeOAuthStatusResponse,
} from "@/commands/runhaven"

import { errorMessage } from "./types"

const STATUS_POLL_MS = 1500
// How long a sign-in can sit in "signing" before we tell the user they can stop
// and start over. Browser sign-in can take a bit, so keep this calm, not alarming.
const STALL_MS = 45000

// Drives Claude path C (sign in on your Mac). It triggers the host sign-in
// (Rust shows a native confirmation first), polls the sanitized status while the
// host capture runs, and can remove the saved token. No secret is handled here:
// the token lives only in the Mac's Keychain and never reaches this hook. The
// status carries a plain `ageDays` from the item's own non-secret creation date,
// used to drive the re-sign-in cadence prompt.
export function useClaudeOAuthSignIn(onSignedIn?: () => void) {
  const [status, setStatus] = useState<ClaudeOAuthStatusResponse | null>(null)
  const [busy, setBusy] = useState(false)
  // A failed start of the sign-in (for example the native confirm was canceled).
  const [error, setError] = useState("")
  // A failed read of the saved-token state, kept apart from `error` so a status
  // read that could not run is never reported as "sign-in did not start".
  const [statusError, setStatusError] = useState("")
  // The user chose to stop waiting on an in-flight sign-in. This stops the client
  // poll and frees the UI to start over even if the host capture is still open.
  const [canceled, setCanceled] = useState(false)
  // The in-flight sign-in has been waiting long enough that we surface a plain
  // "you can start over" note. It is only a hint; the sign-in can still finish.
  const [stalled, setStalled] = useState(false)
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
      setStatusError("")
      setStatus((current) =>
        current && sameStatus(current, next) ? current : next,
      )
    } catch (cause) {
      setStatusError(errorMessage(cause))
    }
  }

  // Reflect the current saved-token state on mount.
  useEffect(() => {
    void refreshStatus()
    // refreshStatus reads no reactive inputs; run it once.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  // Poll while the host reports an in-flight sign-in, whether or not the user
  // stopped waiting. Keeping the poll alive means a sign-in that finishes after
  // "Stop waiting" is still caught and still routes success through
  // finishIfSignedIn -> onSignedIn; `canceled` only changes what the card shows,
  // never whether we track the result. Keyed on the boolean so routine polls do
  // not tear down and re-arm the interval.
  const polling = status?.status === "signing"
  // What the waiting UI shows: the active "finish in your browser" view runs only
  // until the user stops waiting. The poll above keeps running underneath.
  const signing = polling && !canceled
  useEffect(() => {
    if (!polling) {
      return
    }
    let active = true
    const timer = setInterval(() => {
      void getClaudeOAuthStatus()
        .then((next) => {
          if (!active) {
            return
          }
          setStatusError("")
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
            setStatusError(errorMessage(cause))
          }
        })
    }, STATUS_POLL_MS)
    return () => {
      active = false
      clearInterval(timer)
    }
    // finishIfSignedIn reads only refs, so the interval stays keyed on `polling`.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [polling])

  // After a reasonable wait, surface a calm "you can start over" hint. Reset
  // whenever the poll is not active so a fresh sign-in starts the clock over.
  useEffect(() => {
    if (!signing) {
      setStalled(false)
      return
    }
    const timer = setTimeout(() => setStalled(true), STALL_MS)
    return () => clearTimeout(timer)
  }, [signing])

  async function signIn(): Promise<void> {
    if (busy) {
      return
    }
    setBusy(true)
    setError("")
    setStatusError("")
    setClearMessage("")
    setClearError("")
    setCanceled(false)
    setStalled(false)
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

  // Stop showing the in-flight waiting UI without giving up on the result. There
  // is no backend cancel command, so the host capture may still finish on its
  // own; the poll above stays alive to catch that and still fire onSignedIn. We
  // only flip the display to a calm "you stopped waiting" message. We do not
  // re-read here on purpose: a one-shot read cannot fire onSignedIn (only the
  // poll can), so letting the poll own the result avoids saving a token the app
  // never notices. Backend follow-up: a real cancel command would also stop the
  // host capture (out of this file's scope).
  function cancel(): void {
    setCanceled(true)
    setStalled(false)
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
    statusError,
    signing,
    polling,
    stalled,
    canceled,
    cancel,
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
