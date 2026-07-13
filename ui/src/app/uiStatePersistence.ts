import { OPAQUE_TOKEN, isSessionMonitorWindow } from "./windowMode"

// UI state persisted in the WebView (localStorage plus the shadcn sidebar
// cookie). The allowed set, in full:
// - the sidebar open/collapsed state
// - the Settings choices: the Dock badge on/off boolean and the default
//   agent's name (an opaque agent identifier from the Rust dashboard)
// - GUI-only session display labels: a map from the opaque session id to a
//   short user-typed name. These are the one field here that is free-form user
//   input, so they are sanitized on every write and read (control characters
//   stripped, length capped, blank labels dropped) and are shown as display
//   text only. They never rename the run's real volume identity key.
// Nothing else may be persisted here: never composer drafts, run output,
// tokens, project paths, commands, or navigation history. Session ids used as
// label keys are opaque UUIDs minted in Rust (runhaven-core new_run_id), so this
// store never holds a secret. The stored values also have no authority: a
// restored default agent is re-verified against the
// live agent list before it picks anything (useLaunchFlow), so a tampered
// value can at most select something that already exists.
//
// Only the main window writes here. Per-session monitor windows share the
// WebView storage (same origin), so every persist function no-ops in a
// monitor window; the monitor app also never mounts the components that
// persist. Two layers, one rule: a monitor can never move the main window's
// remembered settings or labels.

const SETTINGS_KEY = "runhaven.settings"

// The whole inline Settings surface. Only convenience choices
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
  if (isSessionMonitorWindow()) {
    return
  }
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

const SESSION_LABELS_KEY = "runhaven.sessionLabels"

// A GUI-only display name is short by design: long enough for "Fix the login
// bug" but capped so a row or title cannot be stuffed with a wall of text.
export const MAX_SESSION_LABEL_LENGTH = 60

// Session labels are the only free-form user input this store keeps, so every
// value passes through here on write and read. Control characters (including
// newlines and DEL) are stripped so a label stays a single plain line, the
// text is trimmed, and the length is capped. A label that sanitizes to empty
// is treated as "no label" by callers and dropped from the store.
export function sanitizeSessionLabel(raw: string): string {
  // eslint-disable-next-line no-control-regex
  const stripped = raw.replace(/[\u0000-\u001F\u007F]/g, "")
  return stripped.trim().slice(0, MAX_SESSION_LABEL_LENGTH).trim()
}

// Persist the session-label map. Keys must be opaque session ids and values
// are sanitized; anything else is dropped, so a tampered or oversized store can
// never smuggle a path, secret, or unbounded string back into the UI. Labels
// live only in this Mac's WebView storage; CLI parity and cross-machine
// persistence (labels in the run record itself) are a deferred follow-up, not
// something this cosmetic display state tries to do.
export function persistSessionLabels(labels: Record<string, string>): void {
  if (isSessionMonitorWindow()) {
    return
  }
  const sanitized: Record<string, string> = {}
  for (const [id, value] of Object.entries(labels)) {
    if (!OPAQUE_TOKEN.test(id)) {
      continue
    }
    const clean = sanitizeSessionLabel(value)
    if (clean) {
      sanitized[id] = clean
    }
  }
  try {
    window.localStorage.setItem(SESSION_LABELS_KEY, JSON.stringify(sanitized))
  } catch {
    // Persistence is a convenience; a blocked store never breaks labels.
  }
}

// The persisted labels, with every non-token key, non-string value, or label
// that sanitizes to empty dropped, so a bad store can only mean "no labels".
export function loadSessionLabels(): Record<string, string> {
  try {
    const raw = window.localStorage.getItem(SESSION_LABELS_KEY)
    if (!raw) {
      return {}
    }
    const parsed: unknown = JSON.parse(raw)
    if (typeof parsed !== "object" || parsed === null) {
      return {}
    }
    const result: Record<string, string> = {}
    for (const [id, value] of Object.entries(parsed as Record<string, unknown>)) {
      if (!OPAQUE_TOKEN.test(id) || typeof value !== "string") {
        continue
      }
      const clean = sanitizeSessionLabel(value)
      if (clean) {
        result[id] = clean
      }
    }
    return result
  } catch {
    return {}
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
