// @vitest-environment happy-dom
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"

import { useRunOutput } from "../app/useRunOutput"
import type {
  LogSnapshotResponse,
  ManagedSessionLogSnapshotResponse,
} from "../commands/runhaven"
import { act, renderHook } from "./render-hook"

vi.mock("@/commands/runhaven", async (importActual) => {
  const actual = await importActual<typeof import("@/commands/runhaven")>()
  return {
    ...actual,
    getManagedSessionLogSnapshot: vi.fn(),
    getLogSnapshot: vi.fn(),
  }
})

const { getManagedSessionLogSnapshot } = await import("@/commands/runhaven")
const getSessionSnapshot = vi.mocked(getManagedSessionLogSnapshot)

type Props = {
  targetId: string | null
  sessionId: string | null
  runId: string | null
}

function baseSnapshot(patch: Partial<LogSnapshotResponse> = {}): LogSnapshotResponse {
  return {
    runId: "run-1",
    capturedAt: "2026-07-01T02:00:00Z",
    cursor: "cursor-1",
    requestedLines: 200,
    entries: [
      {
        source: "container-stdio",
        timestamp: "ts-1",
        text: "line one\n",
        lineCount: 1,
      },
    ],
    text: "line one\n",
    returnedLines: 1,
    truncated: false,
    source: "session-log",
    warnings: [],
    ...patch,
  }
}

function sessionResponse(
  snapshot: LogSnapshotResponse,
): ManagedSessionLogSnapshotResponse {
  return {
    session: {
      id: "s1",
      runId: "run-1",
      agent: "codex",
      session: "project",
      title: "Codex - project",
      project: "project",
      status: "running",
      statusLabel: "Running",
      authStatus: "api-key-broker",
      authScope: "project",
      network: "provider",
      startedAt: "2026-07-01T02:00:00Z",
      containerName: "runhaven-codex-project",
      detailCommand: "runhaven runs show run-1",
      lifecycle: "running",
      input: { attached: true, status: "attached", detail: "ready" },
      nextAction: { kind: "monitor", label: "Monitor", detail: "Tracking" },
      events: [],
    },
    snapshot,
    cursor: snapshot.cursor,
    entries: [],
  }
}

function deferred<T>() {
  let resolve!: (value: T) => void
  const promise = new Promise<T>((res) => {
    resolve = res
  })
  return { promise, resolve }
}

const sessionProps: Props = {
  targetId: "session:s1",
  sessionId: "s1",
  runId: null,
}

type Hook = Awaited<ReturnType<typeof renderHook<Props, ReturnType<typeof useRunOutput>>>>

async function mountLive(): Promise<Hook> {
  getSessionSnapshot.mockResolvedValue(sessionResponse(baseSnapshot()))
  const hook = await renderHook((props: Props) => useRunOutput(props), sessionProps)
  // Explicit load, then acknowledge and enable the live poll.
  await act(async () => {
    await hook.result.current.loadSnapshot()
  })
  await act(async () => {
    hook.result.current.setAcknowledged(true)
  })
  await act(async () => {
    hook.result.current.setLiveEnabled(true)
  })
  return hook
}

// Stop the live poll and drain any in-flight work before unmount, so no async
// state from one test strands React's work loop in the next.
async function teardown(hook: Hook): Promise<void> {
  getSessionSnapshot.mockResolvedValue(sessionResponse(baseSnapshot()))
  await act(async () => {
    hook.result.current.setLiveEnabled(false)
  })
  await act(async () => {
    await Promise.resolve()
  })
  await hook.unmount()
}

beforeEach(() => {
  vi.useFakeTimers()
  getSessionSnapshot.mockReset()
})

afterEach(() => {
  vi.clearAllTimers()
  vi.useRealTimers()
})

describe("useRunOutput", () => {
  it("renders the loaded snapshot for the current target", async () => {
    getSessionSnapshot.mockResolvedValue(sessionResponse(baseSnapshot()))
    const hook = await renderHook(
      (props: Props) => useRunOutput(props),
      sessionProps,
    )
    await act(async () => {
      await hook.result.current.loadSnapshot()
    })
    expect(hook.result.current.visibleSnapshot?.cursor).toBe("cursor-1")
    await hook.unmount()
  })

  it("keeps the same snapshot object when a poll returns equal data (C4 guard)", async () => {
    const hook = await mountLive()
    const before = hook.result.current.visibleSnapshot
    expect(before?.cursor).toBe("cursor-1")

    // A poll that returns identical content must not swap the object reference.
    getSessionSnapshot.mockResolvedValue(sessionResponse(baseSnapshot()))
    await act(async () => {
      await vi.advanceTimersByTimeAsync(3_000)
    })
    expect(hook.result.current.visibleSnapshot).toBe(before)

    // A poll with new content does replace it.
    getSessionSnapshot.mockResolvedValue(
      sessionResponse(baseSnapshot({ cursor: "cursor-2", text: "line two\n" })),
    )
    await act(async () => {
      await vi.advanceTimersByTimeAsync(3_000)
    })
    expect(hook.result.current.visibleSnapshot?.cursor).toBe("cursor-2")
    expect(hook.result.current.visibleSnapshot).not.toBe(before)
    await teardown(hook)
  })

  it("keeps the last snapshot when a background poll fails", async () => {
    const hook = await mountLive()
    const before = hook.result.current.visibleSnapshot

    getSessionSnapshot.mockRejectedValue(new Error("snapshot unavailable"))
    await act(async () => {
      await vi.advanceTimersByTimeAsync(3_000)
    })
    expect(hook.result.current.visibleSnapshot).toBe(before)
    expect(hook.result.current.error).toContain("unavailable")
    await teardown(hook)
  })

  it("drops the snapshot immediately when the target changes", async () => {
    getSessionSnapshot.mockResolvedValue(sessionResponse(baseSnapshot()))
    const hook = await renderHook(
      (props: Props) => useRunOutput(props),
      sessionProps,
    )
    await act(async () => {
      await hook.result.current.loadSnapshot()
    })
    expect(hook.result.current.visibleSnapshot?.cursor).toBe("cursor-1")

    // Switching target must drop the prior snapshot; with live off nothing
    // repopulates it, so the new target starts empty (no stale card frame).
    await hook.rerender({ targetId: "session:s2", sessionId: "s2", runId: null })
    expect(hook.result.current.visibleSnapshot).toBeNull()
    await hook.unmount()
  })

  it("discards a stale response after the target changes", async () => {
    const pending = deferred<ManagedSessionLogSnapshotResponse>()
    getSessionSnapshot.mockReturnValueOnce(pending.promise)
    const hook = await renderHook(
      (props: Props) => useRunOutput(props),
      sessionProps,
    )
    // Start an explicit load that will resolve after the target changes.
    let loadPromise: Promise<void> = Promise.resolve()
    await act(async () => {
      loadPromise = hook.result.current.loadSnapshot()
    })

    await hook.rerender({ targetId: "session:s2", sessionId: "s2", runId: null })
    await act(async () => {
      pending.resolve(sessionResponse(baseSnapshot({ cursor: "stale" })))
      await loadPromise
    })
    // The stale s1 response must not surface under s2.
    expect(hook.result.current.visibleSnapshot).toBeNull()
    await hook.unmount()
  })

  it("stops polling after unmount", async () => {
    const hook = await mountLive()
    const callsBefore = getSessionSnapshot.mock.calls.length

    await hook.unmount()
    await act(async () => {
      await vi.advanceTimersByTimeAsync(9_000)
    })
    expect(getSessionSnapshot.mock.calls.length).toBe(callsBefore)
  })
})
