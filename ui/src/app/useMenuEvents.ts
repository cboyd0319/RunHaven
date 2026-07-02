import { useEffect, useRef } from "react"

import type { ManagedSessionSummary } from "@/commands/runhaven"
import { listenForMenuCommands } from "@/commands/menu_events"

import { groupSessions, HOME_NAV, type NavDestination } from "./navigation"
import { sessionAttention, type SessionAttentionKind } from "./sessionAttention"

// Translates native menu bar activations (src-tauri/src/menu.rs) into the
// same UI actions a click would take. The menu never becomes a new mutation
// path: New Run opens the existing launch sheet, Settings opens the existing
// settings dialog, navigation ids move the existing selection, and session
// actions only focus the existing confirm control, so the confirm step plus
// the Rust-owned native dialog still gate every mutation. Menus are static,
// so ids that do not apply right now
// (no sessions, no selection, control not on screen) no-op here; unknown ids
// are ignored.
export type MenuActions = {
  navigation: NavDestination
  managedSessions: { sessions: ManagedSessionSummary[] }
  setupReady: boolean
  launchFlow: { setOpen: (open: boolean) => void }
  settings: { setOpen: (open: boolean) => void }
  navigate: (destination: NavDestination) => void
  selectSession: (sessionId: string) => void
  refresh: () => Promise<void>
  // Opens the read-only monitor window for a session through the validated
  // open_session_window command. The one menu id that invokes a command:
  // window creation mutates nothing, so it carries no confirm step.
  openSessionInNewWindow: (sessionId: string) => void
}

// Session-menu ids focus the matching on-screen control instead of acting.
// The selectors mirror labels owned by the session view components
// (RunStatusCard, SessionInputCard, RunOutputCard); the menu-events test
// reads those sources and fails if a label drifts away from this map.
export const MENU_FOCUS_SELECTORS: Record<string, string> = {
  "stop-session": '[aria-label="Confirm stopping this session."]',
  "hard-stop-session": '[aria-label="Confirm hard-stopping this session."]',
  "repair-session":
    `[aria-label="Confirm clearing this session's stale session record."]`,
  "cleanup-session":
    `[aria-label="Confirm deleting this session's saved worktree."]`,
  "send-message": 'textarea[placeholder="Type a message for this session."]',
  "show-output":
    '[aria-label="I understand raw output may include private data."]',
}

// The shadcn sidebar trigger; clicking it is the exact toggle a user clicks.
const SIDEBAR_TRIGGER_SELECTOR = '[data-sidebar="trigger"]'

const ATTENTION_PRIORITY: Record<SessionAttentionKind, number> = {
  failed: 0,
  "needs-sign-in": 1,
  "not-responding": 2,
  "ready-to-review": 3,
}

// Sessions in the order the sidebar renders them: Active group first, then
// Finished, newest-first inside each group.
function sidebarOrder(
  sessions: ManagedSessionSummary[],
): ManagedSessionSummary[] {
  const groups = groupSessions(sessions)
  return [...groups.active, ...groups.finished]
}

function selectedIndex(ordered: ManagedSessionSummary[], nav: NavDestination) {
  return nav.kind === "session"
    ? ordered.findIndex((session) => session.id === nav.id)
    : -1
}

function focusMenuTarget(selector: string) {
  document.querySelector<HTMLElement>(selector)?.focus()
}

export function handleMenuCommand(id: string, app: MenuActions): void {
  const focusSelector = MENU_FOCUS_SELECTORS[id]
  if (focusSelector) {
    focusMenuTarget(focusSelector)
    return
  }
  if (id.startsWith("select-session-")) {
    const ordered = sidebarOrder(app.managedSessions.sessions)
    const index = Number(id.slice("select-session-".length)) - 1
    const target = ordered[index]
    if (target) {
      app.selectSession(target.id)
    }
    return
  }
  switch (id) {
    case "new-run":
      // While setup is blocked, Home leads with the blocker and the disabled
      // New Run reason, matching the Home button's own gating.
      if (app.setupReady) {
        app.launchFlow.setOpen(true)
      } else {
        app.navigate(HOME_NAV)
      }
      return
    case "settings":
      app.settings.setOpen(true)
      return
    case "go-home":
      app.navigate(HOME_NAV)
      return
    case "go-agents":
    case "agent-sign-in":
    case "agent-rebuild-image":
    case "agent-auth-details":
      // Agent menu items land on the Agents view, where sign-in, rebuild,
      // and auth details live behind their existing confirm flows.
      app.navigate({ kind: "agents" })
      return
    case "go-diagnostics":
      app.navigate({ kind: "diagnostics" })
      return
    case "go-maintenance":
      app.navigate({ kind: "maintenance" })
      return
    case "next-session":
    case "previous-session": {
      const ordered = sidebarOrder(app.managedSessions.sessions)
      if (ordered.length === 0) {
        return
      }
      const step = id === "next-session" ? 1 : -1
      const current = selectedIndex(ordered, app.navigation)
      const target =
        current === -1
          ? ordered[step === 1 ? 0 : ordered.length - 1]
          : ordered[(current + step + ordered.length) % ordered.length]
      app.selectSession(target.id)
      return
    }
    case "jump-needs-you": {
      // Highest-priority needs-you session first; if it is already selected,
      // move to the next one so repeated jumps walk every waiting session.
      const needsYou = sidebarOrder(app.managedSessions.sessions)
        .map((session, order) => {
          const attention = sessionAttention(session)
          return attention
            ? { session, order, rank: ATTENTION_PRIORITY[attention.kind] }
            : null
        })
        .filter((entry) => entry !== null)
        .sort(
          (left, right) => left.rank - right.rank || left.order - right.order,
        )
      if (needsYou.length === 0) {
        return
      }
      const nav = app.navigation
      const current =
        nav.kind === "session"
          ? needsYou.findIndex((entry) => entry.session.id === nav.id)
          : -1
      app.selectSession(needsYou[(current + 1) % needsYou.length].session.id)
      return
    }
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
    case "check-again":
      void app.refresh()
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
