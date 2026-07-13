// @vitest-environment happy-dom
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"

import { useManagedSessionsState } from "../app/useManagedSessionsState"
import type {
  ManagedSessionList,
  ManagedSessionSummary,
} from "../commands/runhaven"

import { act, renderHook } from "./render-hook"

vi.mock("@/commands/runhaven", async (importActual) => {
  const actual = await importActual<typeof import("@/commands/runhaven")>()
  return { ...actual, getManagedSessions: vi.fn() }
})

const { getManagedSessions } = await import("@/commands/runhaven")
const getSessions = vi.mocked(getManagedSessions)

const EMPTY: ManagedSessionList = { sessions: [] }

function session(id: string, runId: string): ManagedSessionSummary {
  return {
    id,
    runId,
    agent: "copilot",
    session: "project",
    title: "Copilot - project",
    project: "project",
    status: "running",
    statusLabel: "Running",
    authStatus: "ready",
    authScope: "project",
    network: "provider",
    startedAt: "2026-07-13T10:00:00Z",
    containerName: "runhaven-copilot-project",
    detailCommand: "runhaven runs show run-1",
    lifecycle: "running",
    input: { attached: true, status: "attached", detail: "Ready." },
    nextAction: { kind: "monitor", label: "Monitor", detail: "Running." },
    events: [],
  }
}

beforeEach(() => {
  vi.useFakeTimers()
  getSessions.mockReset()
  getSessions.mockResolvedValue(EMPTY)
})

afterEach(() => {
  vi.useRealTimers()
})

describe("useManagedSessionsState polling", () => {
  it("runs a slow baseline poll even with no session context", async () => {
    const hook = await renderHook(
      (runId: string | null) => useManagedSessionsState(runId),
      null as string | null,
    )

    // No context: the fast 4s interval is not armed, so nothing polls yet. This
    // is the bug's blast radius: a CLI-started session was invisible until a
    // manual Refresh.
    await act(async () => {
      await vi.advanceTimersByTimeAsync(4_000)
    })
    expect(getSessions).not.toHaveBeenCalled()

    // The 30s baseline still fires, so an externally-created session surfaces on
    // its own.
    await act(async () => {
      await vi.advanceTimersByTimeAsync(26_000)
    })
    expect(getSessions).toHaveBeenCalledTimes(1)
    await hook.unmount()
  })

  it("polls on the fast interval once a launch gives it session context", async () => {
    const hook = await renderHook(
      (runId: string | null) => useManagedSessionsState(runId),
      "run-1" as string | null,
    )

    await act(async () => {
      await vi.advanceTimersByTimeAsync(4_000)
    })
    expect(getSessions).toHaveBeenCalledTimes(1)
    await hook.unmount()
  })

  it("opens a newly launched session when its record appears later", async () => {
    getSessions.mockResolvedValue({
      sessions: [session("session-1", "run-1")],
    })
    const hook = await renderHook(
      (runId: string | null) => useManagedSessionsState(runId),
      "run-1" as string | null,
    )

    await act(async () => {
      hook.result.current.navigate({ kind: "runs" })
      await vi.advanceTimersByTimeAsync(4_000)
    })

    expect(hook.result.current.navigation).toEqual({
      kind: "session",
      id: "session-1",
    })
    await hook.unmount()
  })

  it("does not steal focus when the user leaves History before startup finishes", async () => {
    getSessions.mockResolvedValue({
      sessions: [session("session-1", "run-1")],
    })
    const hook = await renderHook(
      (runId: string | null) => useManagedSessionsState(runId),
      "run-1" as string | null,
    )

    await act(async () => {
      hook.result.current.navigate({ kind: "home" })
      await vi.advanceTimersByTimeAsync(4_000)
    })
    expect(hook.result.current.navigation).toEqual({ kind: "home" })

    await act(async () => {
      hook.result.current.navigate({ kind: "runs" })
    })
    expect(hook.result.current.navigation).toEqual({ kind: "runs" })
    await hook.unmount()
  })

  it("stops polling after unmount", async () => {
    const hook = await renderHook(
      (runId: string | null) => useManagedSessionsState(runId),
      "run-1" as string | null,
    )
    await act(async () => {
      await vi.advanceTimersByTimeAsync(4_000)
    })
    expect(getSessions).toHaveBeenCalledTimes(1)

    await hook.unmount()
    await act(async () => {
      await vi.advanceTimersByTimeAsync(60_000)
    })
    expect(getSessions).toHaveBeenCalledTimes(1)
  })
})
