// @vitest-environment happy-dom
import { beforeEach, describe, expect, it } from "vitest"

import { useNavigation } from "../app/useNavigation"
import type {
  ManagedSessionList,
  ManagedSessionSummary,
} from "../commands/runhaven"

import { installLocalStorageStub } from "./local-storage"
import { act, renderHook } from "./render-hook"

installLocalStorageStub()

function session(
  id: string,
  patch: Partial<ManagedSessionSummary> = {},
): ManagedSessionSummary {
  return {
    id,
    runId: `run-${id}`,
    agent: "codex",
    session: id,
    title: `Codex - ${id}`,
    project: id,
    status: "running",
    statusLabel: "Running",
    authStatus: "ready",
    authScope: "project",
    network: "provider",
    startedAt: "2026-07-01T01:00:00Z",
    containerName: `runhaven-${id}`,
    detailCommand: `runhaven runs show run-${id}`,
    lifecycle: "running",
    input: {
      attached: true,
      status: "attached",
      detail: "Session input is ready.",
    },
    nextAction: {
      kind: "monitor",
      label: "Monitor session",
      detail: "RunHaven is tracking this session.",
    },
    events: [],
    ...patch,
  }
}

function list(...sessions: ManagedSessionSummary[]): ManagedSessionList {
  return { sessions }
}

describe("useNavigation", () => {
  beforeEach(() => {
    window.localStorage.clear()
  })

  it("starts on New session and navigates to destinations and sessions", async () => {
    const hook = await renderHook(useNavigation, list(session("alpha")))
    expect(hook.result.current.navigation).toEqual({ kind: "new-session" })
    expect(hook.result.current.selectedSessionId).toBeNull()

    await act(async () => {
      hook.result.current.navigate({ kind: "diagnostics" })
    })
    expect(hook.result.current.navigation).toEqual({ kind: "diagnostics" })

    await act(async () => {
      hook.result.current.selectSession("alpha")
    })
    expect(hook.result.current.navigation).toEqual({
      kind: "session",
      id: "alpha",
    })
    expect(hook.result.current.selectedSessionId).toBe("alpha")
    expect(hook.result.current.selectedSessionIdRef.current).toBe("alpha")
  })

  it("keeps the selection when the session list changes around it", async () => {
    const alpha = session("alpha", { startedAt: "2026-07-01T02:00:00Z" })
    const beta = session("beta", { startedAt: "2026-07-01T01:00:00Z" })
    const hook = await renderHook(useNavigation, list(alpha, beta))

    await act(async () => {
      hook.result.current.selectSession("alpha")
    })
    // A status flap, a new arrival, and another row's removal never move the
    // selection: the app does not self-navigate while you look at something.
    await hook.rerender(
      list(
        session("alpha", {
          startedAt: "2026-07-01T02:00:00Z",
          lifecycle: "stop-requested",
        }),
        session("new", { startedAt: "2026-07-01T03:00:00Z" }),
      ),
    )
    expect(hook.result.current.navigation).toEqual({
      kind: "session",
      id: "alpha",
    })
  })

  it("does not navigate away from a destination when sessions change", async () => {
    const hook = await renderHook(useNavigation, list(session("alpha")))
    await act(async () => {
      hook.result.current.navigate({ kind: "runs" })
    })
    await hook.rerender(list())
    expect(hook.result.current.navigation).toEqual({ kind: "runs" })
  })

  it("moves to the adjacent row when the selected session is removed", async () => {
    const top = session("top", { startedAt: "2026-07-01T03:00:00Z" })
    const middle = session("middle", { startedAt: "2026-07-01T02:00:00Z" })
    const bottom = session("bottom", { startedAt: "2026-07-01T01:00:00Z" })
    const hook = await renderHook(useNavigation, list(top, middle, bottom))

    await act(async () => {
      hook.result.current.selectSession("middle")
    })
    await hook.rerender(list(top, bottom))
    expect(hook.result.current.navigation).toEqual({
      kind: "session",
      id: "bottom",
    })
    expect(hook.result.current.selectedSessionIdRef.current).toBe("bottom")
  })

  it("falls back to New session only when no session remains", async () => {
    const only = session("only")
    const hook = await renderHook(useNavigation, list(only))
    await act(async () => {
      hook.result.current.selectSession("only")
    })
    await hook.rerender(list())
    expect(hook.result.current.navigation).toEqual({ kind: "new-session" })
    expect(hook.result.current.selectedSessionIdRef.current).toBeNull()
  })

  it("always starts on New session even when an older build stored a destination", async () => {
    window.localStorage.setItem(
      "runhaven.lastDestination",
      JSON.stringify({ kind: "diagnostics" }),
    )
    const hook = await renderHook(useNavigation, list())
    expect(hook.result.current.navigation).toEqual({ kind: "new-session" })
  })
})
