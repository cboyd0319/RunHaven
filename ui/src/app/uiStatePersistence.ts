import type { NavDestination } from "./navigation"

// UI state persisted in the WebView (localStorage plus the shadcn sidebar
// cookie). The allowed set, in full:
// - the last navigation destination kind
// - for a session destination, the backend-generated opaque session id
// - the sidebar open/collapsed state
// Nothing else may be persisted here: never composer drafts, run output,
// tokens, project paths, or commands. Session ids are opaque UUIDs minted in
// Rust (runhaven-core new_run_id), so this store never holds a secret. The
// stored value also has no authority: a restored session id is re-verified
// against the live backend session list before the app navigates to it
// (useNavigation), so a tampered value can at most select a session that
// already exists.
const LAST_DESTINATION_KEY = "runhaven.lastDestination"

const SYSTEM_KINDS = ["home", "agents", "diagnostics", "maintenance"] as const

// Opaque-token shape for a persisted session id: letters, digits, dash, and
// underscore only, bounded length. Anything path- or secret-shaped is
// rejected outright.
const OPAQUE_SESSION_ID = /^[A-Za-z0-9][A-Za-z0-9_-]{0,63}$/

export function persistLastDestination(destination: NavDestination): void {
  const sanitized =
    destination.kind === "session"
      ? { kind: destination.kind, id: destination.id }
      : { kind: destination.kind }
  try {
    window.localStorage.setItem(LAST_DESTINATION_KEY, JSON.stringify(sanitized))
  } catch {
    // Persistence is a convenience; a blocked store never breaks navigation.
  }
}

// The persisted destination, or null when nothing valid is stored. Garbage,
// unknown kinds, and malformed session ids all read as null so a bad store
// can only ever mean "start on Home".
export function loadLastDestination(): NavDestination | null {
  try {
    const raw = window.localStorage.getItem(LAST_DESTINATION_KEY)
    if (!raw) {
      return null
    }
    const parsed: unknown = JSON.parse(raw)
    if (typeof parsed !== "object" || parsed === null) {
      return null
    }
    const kind = (parsed as { kind?: unknown }).kind
    for (const system of SYSTEM_KINDS) {
      if (kind === system) {
        return { kind: system }
      }
    }
    if (kind === "session") {
      const id = (parsed as { id?: unknown }).id
      if (typeof id === "string" && OPAQUE_SESSION_ID.test(id)) {
        return { kind: "session", id }
      }
    }
    return null
  } catch {
    return null
  }
}

const SIDEBAR_STATE_COOKIE = "sidebar_state"

// The shadcn sidebar provider already writes its open/collapsed state to this
// cookie on every toggle; reading it back at mount makes the toggle stick
// across relaunch. The sidebar has no drag-resize today, so this boolean is
// the only sidebar geometry there is to remember (the width is a CSS
// constant).
export function sidebarDefaultOpen(): boolean {
  try {
    const entry = document.cookie
      .split("; ")
      .find((candidate) => candidate.startsWith(`${SIDEBAR_STATE_COOKIE}=`))
    return entry ? entry.slice(SIDEBAR_STATE_COOKIE.length + 1) !== "false" : true
  } catch {
    return true
  }
}
