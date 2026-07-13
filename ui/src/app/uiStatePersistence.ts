// The sidebar open/collapsed state and the bounded accessibility text scale are
// the only WebView UI preferences RunHaven persists. Never add composer drafts,
// session output, tokens, project paths, commands, or navigation history here.
export const TEXT_SIZES = [80, 100, 125, 150, 175, 200] as const
export type TextSize = (typeof TEXT_SIZES)[number]
export const TEXT_SIZE_EVENT = "runhaven:text-size"

const TEXT_SIZE_KEY = "runhaven.text-size"

export function persistTextSize(value: TextSize): void {
  if (!TEXT_SIZES.includes(value)) {
    return
  }
  try {
    window.localStorage.setItem(TEXT_SIZE_KEY, String(value))
  } catch {
    // Persistence is a convenience; a blocked store never breaks the app.
  }
}

export function loadTextSize(): TextSize {
  try {
    const value = Number(window.localStorage.getItem(TEXT_SIZE_KEY))
    return TEXT_SIZES.includes(value as TextSize) ? (value as TextSize) : 100
  } catch {
    return 100
  }
}

export function applyStoredTextSize(): void {
  document.documentElement.style.fontSize = `${loadTextSize()}%`
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
