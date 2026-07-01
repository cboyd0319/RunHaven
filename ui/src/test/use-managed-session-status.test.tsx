// @vitest-environment happy-dom
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"

import { useManagedSessionStatus } from "../app/useManagedSessionStatus"
import type { ManagedSessionStatusResponse } from "../commands/runhaven"
import { act, renderHook } from "./render-hook"

vi.mock("@/commands/runhaven", async (importActual) => {
  const actual = await importActual<typeof import("@/commands/runhaven")>()
  return { ...actual, getManagedSessionStatus: vi.fn() }
})

const { getManagedSessionStatus } = await import("@/commands/runhaven")
const getStatus = vi.mocked(getManagedSessionStatus)

function status(
  sessionId: string,
  patch: Partial<ManagedSessionStatusResponse> = {},
): ManagedSessionStatusResponse {
  return {
    session: {
      id: sessionId,
      runId: sessionId,
      agent: "codex",
      session: "project",
      title: `Codex - ${sessionId}`,
      project: "project",
      status: "running",
      statusLabel: "Running",
      authStatus: "api-key-broker",
      authScope: "project",
      network: "provider",
      startedAt: "2026-07-01T02:00:00Z",
      containerName: `runhaven-codex-${sessionId}`,
      detailCommand: `runhaven runs show ${sessionId}`,
      lifecycle: "running",
      input: { attached: true, status: "attached", detail: "ready" },
      nextAction: { kind: "monitor", label: "Monitor", detail: "Tracking" },
      events: [],
    },
    container: null,
    statusUnavailable: null,
    recovery: null,
    ...patch,
  }
}

function deferred<T>() {
  let resolve!: (value: T) => void
  let reject!: (reason: unknown) => void
  const promise = new Promise<T>((res, rej) => {
    resolve = res
    reject = rej
  })
  return { promise, resolve, reject }
}

beforeEach(() => {
  vi.useFakeTimers()
  getStatus.mockReset()
})

afterEach(() => {
  vi.useRealTimers()
})

describe("useManagedSessionStatus", () => {
  it("loads the current session status on mount", async () => {
    getStatus.mockResolvedValue(status("a"))
    const hook = await renderHook((id: string) => useManagedSessionStatus(id), "a" as string)
    expect(hook.result.current.data?.session.id).toBe("a")
    expect(hook.result.current.error).toBe("")
    await hook.unmount()
  })

  it("discards a stale response after the target changes (requestSeq guard)", async () => {
    const first = deferred<ManagedSessionStatusResponse>()
    const second = deferred<ManagedSessionStatusResponse>()
    getStatus.mockReturnValueOnce(first.promise) // for "a"
    getStatus.mockReturnValueOnce(second.promise) // for "b"

    const hook = await renderHook((id: string) => useManagedSessionStatus(id), "a" as string)
    // "a" is still in flight, so no stale data renders yet.
    expect(hook.result.current.data).toBeNull()

    await hook.rerender("b")
    // Switching to "b" must not render "a" for a frame, and "b" is still loading.
    expect(hook.result.current.data).toBeNull()

    // "a" resolves late; its response must be discarded (requestSeq mismatch).
    await act(async () => {
      first.resolve(status("a"))
      await first.promise.catch(() => undefined)
    })
    expect(hook.result.current.data).toBeNull()

    // "b" resolves and wins.
    await act(async () => {
      second.resolve(status("b"))
      await second.promise
    })
    expect(hook.result.current.data?.session.id).toBe("b")
    await hook.unmount()
  })

  it("keeps last-good data when a background poll fails", async () => {
    getStatus.mockResolvedValueOnce(status("a"))
    const hook = await renderHook((id: string) => useManagedSessionStatus(id), "a" as string)
    expect(hook.result.current.data?.session.id).toBe("a")

    getStatus.mockRejectedValueOnce(new Error("container status unavailable"))
    await act(async () => {
      await vi.advanceTimersByTimeAsync(4_000)
    })

    // The card keeps its last-good data; only the error surfaces.
    expect(hook.result.current.data?.session.id).toBe("a")
    expect(hook.result.current.error).toContain("unavailable")
    await hook.unmount()
  })

  it("skips a re-render when a poll returns an equal snapshot", async () => {
    getStatus.mockResolvedValueOnce(status("a"))
    const hook = await renderHook((id: string) => useManagedSessionStatus(id), "a" as string)
    const firstData = hook.result.current.data
    expect(firstData?.session.id).toBe("a")

    // Same content, new object: the equality guard should keep the old reference.
    getStatus.mockResolvedValueOnce(status("a"))
    await act(async () => {
      await vi.advanceTimersByTimeAsync(4_000)
    })
    expect(hook.result.current.data).toBe(firstData)
    await hook.unmount()
  })

  it("takes a fresh load when an explicit reload arrives during an in-flight poll", async () => {
    getStatus.mockResolvedValueOnce(status("a"))
    const hook = await renderHook((id: string) => useManagedSessionStatus(id), "a" as string)
    expect(hook.result.current.data?.session.id).toBe("a")

    // A background poll hangs in flight (pre-action snapshot).
    const stale = deferred<ManagedSessionStatusResponse>()
    getStatus.mockReturnValueOnce(stale.promise)
    await act(async () => {
      await vi.advanceTimersByTimeAsync(4_000)
    })

    // An explicit reload (post-action refresh) must not silently return: it
    // waits for the in-flight poll, then fetches once more and resolves with
    // the fresh status.
    const fresh = status("a", { statusUnavailable: "stopped" })
    getStatus.mockResolvedValueOnce(fresh)
    await act(async () => {
      const explicit = hook.result.current.loadStatus()
      stale.resolve(status("a"))
      await explicit
    })
    expect(getStatus).toHaveBeenCalledTimes(3)
    expect(hook.result.current.data?.statusUnavailable).toBe("stopped")
    await hook.unmount()
  })

  it("still skips overlapping background polls", async () => {
    const hanging = deferred<ManagedSessionStatusResponse>()
    getStatus.mockReturnValueOnce(hanging.promise)
    const hook = await renderHook((id: string) => useManagedSessionStatus(id), "a" as string)

    // Two poll ticks while the first request is still in flight: no stacking.
    await act(async () => {
      await vi.advanceTimersByTimeAsync(8_000)
    })
    expect(getStatus).toHaveBeenCalledTimes(1)

    await act(async () => {
      hanging.resolve(status("a"))
      await hanging.promise
    })
    expect(hook.result.current.data?.session.id).toBe("a")
    await hook.unmount()
  })

  it("stops polling after unmount", async () => {
    getStatus.mockResolvedValue(status("a"))
    const hook = await renderHook((id: string) => useManagedSessionStatus(id), "a" as string)
    expect(getStatus).toHaveBeenCalledTimes(1)

    await hook.unmount()
    await act(async () => {
      await vi.advanceTimersByTimeAsync(12_000)
    })
    expect(getStatus).toHaveBeenCalledTimes(1)
  })
})
