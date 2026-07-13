// UI state persisted in the WebView (localStorage plus the shadcn sidebar
// cookie). The allowed set, in full:
// - the sidebar open/collapsed state
// - the View-menu choice: the Dock badge on/off boolean
// Nothing else may be persisted here: never composer drafts, session output,
// tokens, project paths, commands, or navigation history.
//

const SETTINGS_KEY = "runhaven.settings"

// The one persisted View-menu preference. Only convenience choices
// live here, never security defaults or confirmation suppression.
export type AppSettings = {
  // Show the needs-you count on the app's Dock icon.
  dockBadge: boolean
}

export const DEFAULT_SETTINGS: AppSettings = {
  dockBadge: true,
}

export function persistSettings(settings: AppSettings): void {
  const sanitized: AppSettings = {
    dockBadge: settings.dockBadge !== false,
  }
  try {
    window.localStorage.setItem(SETTINGS_KEY, JSON.stringify(sanitized))
  } catch {
    // Persistence is a convenience; a blocked store never breaks the app.
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
    const { dockBadge } = parsed as {
      dockBadge?: unknown
    }
    return {
      dockBadge: dockBadge !== false,
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
