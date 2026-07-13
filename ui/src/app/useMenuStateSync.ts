import { useEffect, useMemo, useRef } from "react"

import type { ManagedSessionSummary, MenuState } from "@/commands/runhaven"
import { syncMenuState } from "@/commands/runhaven"

import {
  sessionAllowsCleanup,
  sessionAllowsRunControl,
} from "./ManagedSessionCard"
import type { NavDestination } from "./navigation"
import { sessionAllowsWorktreeReview } from "./worktreeReview"

// Which selection-scoped menu items apply right now. Computed from the exact
// same helpers the session view uses, so the
// grayed-out menu always matches what the window actually offers:
// - run control (Stop/Hard Stop/Repair) shares the run-control card's gate
// - the message composer shows only for a monitor session
// - the output action and Open in New Window need a real selection
// - clean up mirrors the session view, which hides it while the worktree
//   review owns removal
export function computeMenuState(
  sessions: ManagedSessionSummary[],
  navigation: NavDestination,
): MenuState {
  const selected =
    navigation.kind === "session"
      ? (sessions.find((session) => session.id === navigation.id) ?? null)
      : null
  const hasSelected = selected !== null
  return {
    selectedSessionCanRunControl: selected
      ? sessionAllowsRunControl(selected)
      : false,
    selectedSessionCanSendInput: selected
      ? selected.nextAction.kind === "monitor"
      : false,
    selectedSessionCanViewOutput: hasSelected,
    selectedSessionHasCleanup: selected
      ? sessionAllowsCleanup(selected) && !sessionAllowsWorktreeReview(selected)
      : false,
    selectedSessionCanOpenWindow: hasSelected,
  }
}

// Stable key for dedupe: MenuState is a flat primitive object, so a fixed-order
// join is enough and never re-invokes on an unchanged state.
function menuStateKey(state: MenuState): string {
  return [
    state.selectedSessionCanRunControl,
    state.selectedSessionCanSendInput,
    state.selectedSessionCanViewOutput,
    state.selectedSessionHasCleanup,
    state.selectedSessionCanOpenWindow,
  ].join("|")
}

// Tell Rust which native menu items apply so it can gray out the rest. Fires
// only when the derived state actually changes, not on every status poll: the
// dedupe key drops identical states. Safe in the browser preview and tests:
// syncMenuState no-ops without a Tauri runtime, so nothing touches Tauri APIs.
export function useMenuStateSync(
  sessions: ManagedSessionSummary[],
  navigation: NavDestination,
) {
  const state = useMemo(
    () => computeMenuState(sessions, navigation),
    [sessions, navigation],
  )
  const lastSent = useRef<string | null>(null)
  useEffect(() => {
    const key = menuStateKey(state)
    if (lastSent.current === key) {
      return
    }
    lastSent.current = key
    // Best effort: a failed sync only leaves a menu item in its last state
    // until the next change, never blocks the UI.
    void syncMenuState(state).catch(() => undefined)
  }, [state])
}
