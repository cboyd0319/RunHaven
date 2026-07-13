import { useEffect, useRef } from "react"

import { listenForMenuCommands } from "@/commands/menu_events"

import {
  helpNavigation,
  NEW_SESSION_NAV,
  type NavDestination,
} from "./navigation"
import {
  persistTextSize,
  TEXT_SIZE_EVENT,
  TEXT_SIZES,
  type TextSize,
} from "./uiStatePersistence"
import type { RunControlAction } from "./types"
import { SESSION_MESSAGE_MENU_TARGET } from "./SessionInputCard"

// Translates native menu activations into the same UI actions a click takes.
// New Session and Help open their pages, and Session
// actions activate the same on-screen buttons, so the Rust-owned dialog still
// gates mutations.
// Send Message focuses the composer instead of sending a draft. Ids that do not
// apply right now (no selection or control not on screen) no-op; unknown ids are
// ignored.
export type MenuActions = {
  navigation: NavDestination
  launchFlow: { setOpen: (open: boolean) => void }
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
  "send-message": `[data-menu-target="${SESSION_MESSAGE_MENU_TARGET}"]`,
}

// The shadcn sidebar trigger; clicking it is the exact toggle a user clicks.
const SIDEBAR_TRIGGER_SELECTOR = '[data-sidebar="trigger"]'

function focusMenuTarget(selector: string) {
  document.querySelector<HTMLElement>(selector)?.focus()
}

function changeTextSize(direction: -1 | 0 | 1) {
  const current = Number.parseInt(document.documentElement.style.fontSize, 10) || 100
  const defaultIndex = TEXT_SIZES.indexOf(100)
  const foundIndex = TEXT_SIZES.findIndex((value) => value === current)
  const currentIndex = foundIndex < 0 ? defaultIndex : foundIndex
  const nextIndex = direction === 0
    ? defaultIndex
    : Math.min(Math.max(currentIndex + direction, 0), TEXT_SIZES.length - 1)
  const next = TEXT_SIZES[nextIndex] as TextSize
  document.documentElement.style.fontSize = `${next}%`
  persistTextSize(next)
  window.dispatchEvent(new CustomEvent<TextSize>(TEXT_SIZE_EVENT, { detail: next }))
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
    case "toggle-sidebar":
      document
        .querySelector<HTMLElement>(SIDEBAR_TRIGGER_SELECTOR)
        ?.click()
      return
    case "increase-text-size":
      changeTextSize(1)
      return
    case "decrease-text-size":
      changeTextSize(-1)
      return
    case "actual-text-size":
      changeTextSize(0)
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
