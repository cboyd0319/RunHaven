import { useEffect, useRef } from "react"

import { listenForMenuCommands } from "@/commands/menu_events"

import {
  helpNavigation,
  NEW_SESSION_NAV,
  type NavDestination,
} from "./navigation"
import type { RunControlAction } from "./types"

// Translates native menu activations into the same UI actions a click takes.
// New Session and Help open their pages, the View preference toggles, and Run
// actions activate the same on-screen buttons, so the Rust-owned dialog still
// gates mutations.
// Send Message focuses the composer instead of sending a draft. Ids that do not
// apply right now (no selection or control not on screen) no-op; unknown ids are
// ignored.
export type MenuActions = {
  navigation: NavDestination
  launchFlow: { setOpen: (open: boolean) => void }
  settings: { dockBadge: boolean; setDockBadge: (enabled: boolean) => void }
  navigate: (destination: NavDestination) => void
  runControl: (action: RunControlAction) => Promise<void>
}

// Stop and cleanup commands use the selected session directly. The trusted
// Rust command still owns approval and revalidation, so the menu does not need
// a duplicate hidden button in the WebView.
export const MENU_CONTROL_ACTIONS: Record<string, RunControlAction> = {
  "stop-session": "stop",
  "hard-stop-session": "kill",
  "cleanup-session": "cleanup",
}

export const MENU_FOCUS_SELECTORS: Record<string, string> = {
  "send-message": 'textarea[placeholder="Type a message for this session."]',
}

// The shadcn sidebar trigger; clicking it is the exact toggle a user clicks.
const SIDEBAR_TRIGGER_SELECTOR = '[data-sidebar="trigger"]'

function focusMenuTarget(selector: string) {
  document.querySelector<HTMLElement>(selector)?.focus()
}

export function handleMenuCommand(id: string, app: MenuActions): void {
  const controlAction = MENU_CONTROL_ACTIONS[id]
  if (controlAction) {
    if (app.navigation.kind === "session") {
      void app.runControl(controlAction)
    }
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
    case "show-help":
      app.navigate(helpNavigation(app.navigation))
      return
    case "show-attention-badge":
      app.settings.setDockBadge(!app.settings.dockBadge)
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
