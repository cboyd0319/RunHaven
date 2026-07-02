import type { NavDestination } from "./navigation"

// UI state persisted in the WebView (localStorage plus the shadcn sidebar
// cookie). The allowed set, in full:
// - the last navigation destination kind
// - for a session destination, the backend-generated opaque session id
// - the sidebar open/collapsed state
// - the Settings choices: the Dock badge on/off boolean and the default
//   agent's name (an opaque agent identifier from the Rust dashboard)
// Nothing else may be persisted here: never composer drafts, run output,
// tokens, project paths, or commands. Session ids are opaque UUIDs minted in
// Rust (runhaven-core new_run_id), so this store never holds a secret. The
// stored values also have no authority: a restored session id is re-verified
// against the live backend session list before the app navigates to it
// (useNavigation), and a restored default agent is re-verified against the
// live agent list before it picks anything (useLaunchFlow), so a tampered
// value can at most select something that already exists.
const LAST_DESTINATION_KEY = "runhaven.lastDestination"

const SYSTEM_KINDS = ["home", "agents", "diagnostics", "maintenance"] as const

// Opaque-token shape for a persisted session id or agent name: letters,
// digits, dash, and underscore only, bounded length. Anything path- or
// secret-shaped is rejected outright.
const OPAQUE_TOKEN = /^[A-Za-z0-9][A-Za-z0-9_-]{0,63}$/

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
      if (typeof id === "string" && OPAQUE_TOKEN.test(id)) {
        return { kind: "session", id }
      }
    }
    return null
  } catch {
    return null
  }
}

const SETTINGS_KEY = "runhaven.settings"

// The whole Settings surface (spec 3.9, shipped lean as an in-window dialog;
// see docs/plans/native-gui-navigation-gap.md B3). Only convenience choices
// live here, never security defaults or confirmation suppression.
export type AppSettings = {
  // Show the needs-you count on the app's Dock icon.
  dockBadge: boolean
  // Agent preselected for new runs; null lets RunHaven choose.
  defaultAgent: string | null
}

export const DEFAULT_SETTINGS: AppSettings = {
  dockBadge: true,
  defaultAgent: null,
}

export function persistSettings(settings: AppSettings): void {
  const sanitized: AppSettings = {
    dockBadge: settings.dockBadge !== false,
    defaultAgent:
      typeof settings.defaultAgent === "string" &&
      OPAQUE_TOKEN.test(settings.defaultAgent)
        ? settings.defaultAgent
        : null,
  }
  try {
    window.localStorage.setItem(SETTINGS_KEY, JSON.stringify(sanitized))
  } catch {
    // Persistence is a convenience; a blocked store never breaks Settings.
  }
}

// The persisted settings, with every missing, malformed, or non-token field
// reading as its safe default, so a bad store can only mean default behavior.
export function loadSettings(): AppSettings {
  try {
    const raw = window.localStorage.getItem(SETTINGS_KEY)
    if (!raw) {
      return DEFAULT_SETTINGS
    }
    const parsed: unknown = JSON.parse(raw)
    if (typeof parsed !== "object" || parsed === null) {
      return DEFAULT_SETTINGS
    }
    const { dockBadge, defaultAgent } = parsed as {
      dockBadge?: unknown
      defaultAgent?: unknown
    }
    return {
      dockBadge: dockBadge !== false,
      defaultAgent:
        typeof defaultAgent === "string" && OPAQUE_TOKEN.test(defaultAgent)
          ? defaultAgent
          : null,
    }
  } catch {
    return DEFAULT_SETTINGS
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
