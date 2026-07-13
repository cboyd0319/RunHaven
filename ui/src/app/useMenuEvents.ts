import { useEffect, useRef } from "react"

import { listenForMenuCommands } from "@/commands/menu_events"

import { NEW_SESSION_NAV, type NavDestination } from "./navigation"

// Translates native menu activations into the same UI actions a click takes.
// New Session opens its page, Settings expands inline, and Run actions activate
// the same on-screen buttons, so the Rust-owned dialog still gates mutations.
// Send Message focuses the composer instead of sending a draft. Ids that do not
// apply right now (no selection or control not on screen) no-op; unknown ids are
// ignored.
export type MenuActions = {
  navigation: NavDestination
  launchFlow: { setOpen: (open: boolean) => void }
  settings: { setOpen: (open: boolean) => void }
  navigate: (destination: NavDestination) => void
  // Opens the read-only monitor window for a session through the validated
  // open_session_window command. The one menu id that invokes a command:
  // window creation mutates nothing, so it carries no confirm step.
  openSessionInNewWindow: (sessionId: string) => void
}

// Session-menu ids focus the matching on-screen control instead of acting.
// The selectors mirror labels owned by the session view components
// (RunStatusCard, SessionInputCard, RunOutputCard); the menu-events test
// reads those sources and fails if a label drifts away from this map.
export const MENU_ACTION_SELECTORS: Record<string, string> = {
  "stop-session": '[aria-label="Stop run"]',
  "hard-stop-session": '[aria-label="Hard stop"]',
  "cleanup-session": '[aria-label="Clean up run"]',
  "show-output": '[aria-label="Show latest output"]',
}

export const MENU_FOCUS_SELECTORS: Record<string, string> = {
  "send-message": 'textarea[placeholder="Type a message for this run."]',
}

// The shadcn sidebar trigger; clicking it is the exact toggle a user clicks.
const SIDEBAR_TRIGGER_SELECTOR = '[data-sidebar="trigger"]'

function focusMenuTarget(selector: string) {
  document.querySelector<HTMLElement>(selector)?.focus()
}

export function handleMenuCommand(id: string, app: MenuActions): void {
  const actionSelector = MENU_ACTION_SELECTORS[id]
  if (actionSelector) {
    document.querySelector<HTMLElement>(actionSelector)?.click()
    return
  }
  const focusSelector = MENU_FOCUS_SELECTORS[id]
  if (focusSelector) {
    focusMenuTarget(focusSelector)
    return
  }
  switch (id) {
    case "new-run":
      app.launchFlow.setOpen(false)
      app.navigate(NEW_SESSION_NAV)
      return
    case "settings":
      app.settings.setOpen(true)
      return
    case "open-session-window":
      // Acts on the selected session only; a monitor window is read-only,
      // so this needs a real selection, never a guess.
      if (app.navigation.kind === "session") {
        app.openSessionInNewWindow(app.navigation.id)
      }
      return
    case "toggle-sidebar":
      document
        .querySelector<HTMLElement>(SIDEBAR_TRIGGER_SELECTOR)
        ?.click()
      return
    default:
      // Placeholder and future ids fall through harmlessly.
      return
  }
}

// Mount-once listener for native menu activations. Safe in the browser
// preview and under tests: without a Tauri runtime the subscription is a
// no-op, so e2e and vitest never touch Tauri APIs.
export function useMenuEvents(app: MenuActions) {
  const appRef = useRef(app)
  useEffect(() => {
    appRef.current = app
  })
  useEffect(() => {
    let disposed = false
    let unlisten: (() => void) | undefined
    void listenForMenuCommands((id) =>
      handleMenuCommand(id, appRef.current),
    ).then((stop) => {
      if (disposed) {
        stop()
      } else {
        unlisten = stop
      }
    })
    return () => {
      disposed = true
      unlisten?.()
    }
  }, [])
}
