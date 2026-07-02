import { useEffect, useRef, useState } from "react"

import {
  getManagedSessionStatus,
  getSessionWindowTarget,
  hasTauriRuntime,
  type ManagedSessionStatusResponse,
} from "@/commands/runhaven"

import { errorMessage } from "./types"
import { monitorPreviewSessionId } from "./windowMode"
import { sameManagedSessionStatus } from "./useManagedSessionStatus"

const MONITOR_POLL_MS = 4_000

// The exact backend error for a session id that is no longer in the read
// model (src-tauri/src/commands/sessions.rs). A removed record is the spec's
// end state, not a failure, so it renders as the gone notice.
const SESSION_GONE_ERROR = "managed session not found"

export type SessionMonitorState = {
  // Which session this window watches. Null while resolving or when the
  // window has no registered target (resolveError says why).
  sessionId: string | null
  resolveError: string
  // Last good status; kept through transient poll errors and after the
  // record disappears so the end state stays on screen.
  status: ManagedSessionStatusResponse | null
  // True once the backend reports the session record is gone.
  gone: boolean
  // Transient poll error (not the gone state); cleared by the next success.
  pollError: string
}

// Read-only poll loop for a per-session monitor window. Under Tauri the
// watched session comes from Rust (get_session_window_target, keyed by this
// window's label); the ?session= URL parameter is consulted only in the
// browser preview where no Tauri runtime exists. The hook only reads status;
// it never invokes a mutating command and never touches persisted UI state.
export function useSessionMonitor(): SessionMonitorState {
  const [sessionId, setSessionId] = useState<string | null>(null)
  const [resolveError, setResolveError] = useState("")
  const [status, setStatus] = useState<ManagedSessionStatusResponse | null>(null)
  const [gone, setGone] = useState(false)
  const [pollError, setPollError] = useState("")
  const statusRef = useRef<ManagedSessionStatusResponse | null>(null)

  useEffect(() => {
    let disposed = false
    async function resolveTarget() {
      try {
        const id = hasTauriRuntime()
          ? (await getSessionWindowTarget()).sessionId
          : monitorPreviewSessionId()
        if (disposed) {
          return
        }
        if (id) {
          setSessionId(id)
        } else {
          setResolveError(
            "This window has no session to watch. Close it and use Open in New Window from the main window.",
          )
        }
      } catch (cause) {
        if (!disposed) {
          setResolveError(errorMessage(cause))
        }
      }
    }
    void resolveTarget()
    return () => {
      disposed = true
    }
  }, [])

  useEffect(() => {
    if (!sessionId) {
      return
    }
    let disposed = false
    let inFlight = false

    async function poll() {
      if (inFlight) {
        return
      }
      inFlight = true
      try {
        const next = await getManagedSessionStatus(sessionId as string)
        if (disposed) {
          return
        }
        setPollError("")
        setGone(false)
        setStatus((current) =>
          current && sameManagedSessionStatus(current, next) ? current : next,
        )
        statusRef.current = next
      } catch (cause) {
        if (disposed) {
          return
        }
        const message = errorMessage(cause)
        if (message.includes(SESSION_GONE_ERROR)) {
          // End state: keep the last-known overview on screen and say the
          // record is gone. Never auto-close; the person closes the window.
          setGone(true)
          setPollError("")
        } else {
          setPollError(message)
        }
      } finally {
        inFlight = false
      }
    }

    void poll()
    const interval = window.setInterval(() => void poll(), MONITOR_POLL_MS)
    return () => {
      disposed = true
      window.clearInterval(interval)
    }
  }, [sessionId])

  return { sessionId, resolveError, status, gone, pollError }
}
