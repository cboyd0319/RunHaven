// @vitest-environment happy-dom
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"

import { useManagedSessionsState } from "../app/useManagedSessionsState"
import type { ManagedSessionList } from "../commands/runhaven"

import { act, renderHook } from "./render-hook"

vi.mock("@/commands/runhaven", async (importActual) => {
  const actual = await importActual<typeof import("@/commands/runhaven")>()
  return { ...actual, getManagedSessions: vi.fn() }
})

const { getManagedSessions } = await import("@/commands/runhaven")
const getSessions = vi.mocked(getManagedSessions)

const EMPTY: ManagedSessionList = { sessions: [] }

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
