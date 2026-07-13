// @vitest-environment happy-dom
import { beforeEach, describe, expect, it, vi } from "vitest"

import type { ManagedSessionSummary, MenuState } from "../commands/runhaven"
import type { NavDestination } from "../app/navigation"
import {
  computeMenuState,
  useMenuStateSync,
} from "../app/useMenuStateSync"

import { renderHook } from "./render-hook"

const { syncMenuState } = vi.hoisted(() => ({ syncMenuState: vi.fn() }))

vi.mock("@/commands/runhaven", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../commands/runhaven")>()),
  syncMenuState,
}))

function session(patch: Partial<ManagedSessionSummary>): ManagedSessionSummary {
  return {
    id: "s1",
    runId: "run-s1",
    agent: "codex",
    session: "alpha",
    title: "Codex - alpha",
    project: "alpha",
    status: "running",
    statusLabel: "Running",
    authStatus: "ready",
    authScope: "project",
    network: "provider",
    startedAt: "2026-07-01T01:00:00Z",
    containerName: "runhaven-s1",
    detailCommand: "runhaven runs show run-s1",
    lifecycle: "running",
    input: { attached: true, status: "attached", detail: "Ready." },
    nextAction: {
      kind: "monitor",
      label: "Monitor session",
      detail: "Tracking.",
    },
    events: [],
    ...patch,
  }
}

const HOME: NavDestination = { kind: "new-session" }
const selection = (id: string): NavDestination => ({ kind: "session", id })

describe("computeMenuState", () => {
  it("disables every session-scoped item when nothing is selected", () => {
    const sessions = [session({ id: "a1" }), session({ id: "a2" })]
    const state = computeMenuState(sessions, HOME)
    expect(state).toEqual<MenuState>({
      sidebarOpen: true,
      selectedSessionCanRunControl: false,
      selectedSessionCanSendInput: false,
      selectedSessionHasCleanup: false,
    })
  })

  it("enables run control and the composer for a running session", () => {
    const running = session({ id: "a1", lifecycle: "running" })
    const state = computeMenuState([running], selection("a1"))
    expect(state.selectedSessionCanRunControl).toBe(true)
    expect(state.selectedSessionCanSendInput).toBe(true)
    // A running session has no standalone clean-up.
    expect(state.selectedSessionHasCleanup).toBe(false)
  })

  it("disables the composer when the running session input is detached", () => {
    const detached = session({
      id: "a1",
      input: {
        attached: false,
        status: "detached",
        detail: "Use the session terminal.",
      },
    })
    const state = computeMenuState([detached], selection("a1"))

    expect(state.selectedSessionCanRunControl).toBe(true)
    expect(state.selectedSessionCanSendInput).toBe(false)
  })

  it("disables run control and composer for a finished session", () => {
    const finished = session({
      id: "f1",
      lifecycle: "finished",
      nextAction: { kind: "recover", label: "Recover", detail: "" },
    })
    const state = computeMenuState([finished], selection("f1"))
    expect(state.selectedSessionCanRunControl).toBe(false)
    expect(state.selectedSessionCanSendInput).toBe(false)
  })

  it("does not enable Send Message when the composer gate rejects the session", () => {
    const endedMonitor = session({ id: "f1", lifecycle: "finished" })
    expect(
      computeMenuState([endedMonitor], selection("f1")).selectedSessionCanSendInput,
    ).toBe(false)
  })

  it("offers clean-up only when it is not a worktree review", () => {
    const plain = session({
      id: "f1",
      lifecycle: "finished",
      cleanup: {
        available: true,
        reviewState: "none",
        target: "session",
        label: "Clean up",
        detail: "",
      },
    })
    expect(
      computeMenuState([plain], selection("f1")).selectedSessionHasCleanup,
    ).toBe(true)

    // A worktree copy is owned by the review card, so the menu clean-up hides.
    const worktree = session({
      id: "f2",
      lifecycle: "finished",
      cleanup: {
        available: true,
        reviewState: "changes",
        target: "worktree",
        label: "Review",
        detail: "",
      },
    })
    expect(
      computeMenuState([worktree], selection("f2")).selectedSessionHasCleanup,
    ).toBe(false)
  })

})

describe("useMenuStateSync", () => {
  beforeEach(() => {
    syncMenuState.mockClear()
    syncMenuState.mockResolvedValue(undefined)
  })

  it("syncs once, dedupes an unchanged state, and re-syncs on a real change", async () => {
    const sessions = [session({ id: "a1" })]
    const props: {
      sessions: ManagedSessionSummary[]
      navigation: NavDestination
      sidebarOpen: boolean
    } = {
      sessions,
      navigation: HOME,
      sidebarOpen: true,
    }
    const hook = await renderHook(
      (p: {
        sessions: ManagedSessionSummary[]
        navigation: NavDestination
        sidebarOpen: boolean
      }) =>
        useMenuStateSync(
          p.sessions,
          p.navigation,
          p.sidebarOpen,
        ),
      props,
    )
    expect(syncMenuState).toHaveBeenCalledTimes(1)
    expect(syncMenuState).toHaveBeenLastCalledWith(
      expect.objectContaining({ selectedSessionCanRunControl: false }),
    )

    // A new array with the same derived state must not re-invoke.
    await hook.rerender({
      sessions: [session({ id: "a1" })],
      navigation: HOME,
      sidebarOpen: true,
    })
    expect(syncMenuState).toHaveBeenCalledTimes(1)

    // Selecting the session changes the derived state, so it syncs again.
    await hook.rerender({
      sessions,
      navigation: selection("a1"),
      sidebarOpen: true,
    })
    expect(syncMenuState).toHaveBeenCalledTimes(2)
    expect(syncMenuState).toHaveBeenLastCalledWith(
      expect.objectContaining({ selectedSessionCanRunControl: true }),
    )

    await hook.unmount()
  })

  it("syncs whether the View menu should say Show or Hide Sidebar", async () => {
    const hook = await renderHook(
      (sidebarOpen: boolean) => useMenuStateSync([], HOME, sidebarOpen),
      false,
    )
    expect(syncMenuState).toHaveBeenLastCalledWith(
      expect.objectContaining({ sidebarOpen: false }),
    )
    await hook.unmount()
  })
})
