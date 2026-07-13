// @vitest-environment happy-dom
import { beforeEach, describe, expect, it, vi } from "vitest"

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
    requestedLines: 200,
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
      authStatus: "ready",
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

beforeEach(() => {
  getSessionSnapshot.mockReset()
})

describe("useRunOutput", () => {
  it("never reads output until the user explicitly asks", async () => {
    const hook = await renderHook((props: Props) => useRunOutput(props), sessionProps)
    await act(async () => {
      await Promise.resolve()
    })
    expect(getSessionSnapshot).not.toHaveBeenCalled()
    await hook.unmount()
  })

  it("loads one bounded snapshot with native confirmation intent", async () => {
    getSessionSnapshot.mockResolvedValue(sessionResponse(baseSnapshot()))
    const hook = await renderHook((props: Props) => useRunOutput(props), sessionProps)

    await act(async () => {
      await hook.result.current.loadSnapshot()
    })

    expect(getSessionSnapshot).toHaveBeenCalledWith("s1", {
      confirmSensitiveOutput: true,
    })
    expect(hook.result.current.visibleSnapshot?.text).toBe("line one\n")
    await hook.unmount()
  })

  it("keeps the same object when an explicit refresh returns equal content", async () => {
    getSessionSnapshot
      .mockResolvedValueOnce(sessionResponse(baseSnapshot()))
      .mockResolvedValueOnce(
        sessionResponse(baseSnapshot({ capturedAt: "2026-07-01T02:00:03Z" })),
      )
    const hook = await renderHook((props: Props) => useRunOutput(props), sessionProps)
    await act(async () => {
      await hook.result.current.loadSnapshot()
    })
    const before = hook.result.current.visibleSnapshot
    await act(async () => {
      await hook.result.current.loadSnapshot()
    })
    expect(hook.result.current.visibleSnapshot).toBe(before)
    await hook.unmount()
  })

  it("drops the snapshot immediately when the target changes", async () => {
    getSessionSnapshot.mockResolvedValue(sessionResponse(baseSnapshot()))
    const hook = await renderHook((props: Props) => useRunOutput(props), sessionProps)
    await act(async () => {
      await hook.result.current.loadSnapshot()
    })
    await hook.rerender({
      targetId: "session:s2",
      sessionId: "s2",
      runId: null,
    })
    expect(hook.result.current.visibleSnapshot).toBeNull()
    await hook.unmount()
  })

  it("discards a stale response after the target changes", async () => {
    const pending = deferred<ManagedSessionLogSnapshotResponse>()
    getSessionSnapshot.mockReturnValueOnce(pending.promise)
    const hook = await renderHook((props: Props) => useRunOutput(props), sessionProps)
    let loadPromise: Promise<void> = Promise.resolve()
    await act(async () => {
      loadPromise = hook.result.current.loadSnapshot()
    })
    await hook.rerender({
      targetId: "session:s2",
      sessionId: "s2",
      runId: null,
    })
    await act(async () => {
      pending.resolve(sessionResponse(baseSnapshot({ text: "stale\n" })))
      await loadPromise
    })
    expect(hook.result.current.visibleSnapshot).toBeNull()
    await hook.unmount()
  })

  it("loads a new target while the old target request is pending", async () => {
    const oldTarget = deferred<ManagedSessionLogSnapshotResponse>()
    getSessionSnapshot
      .mockReturnValueOnce(oldTarget.promise)
      .mockResolvedValueOnce(
        sessionResponse(baseSnapshot({ runId: "run-2", text: "new target\n" })),
      )
    const hook = await renderHook((props: Props) => useRunOutput(props), sessionProps)
    let oldLoad: Promise<void> = Promise.resolve()
    await act(async () => {
      oldLoad = hook.result.current.loadSnapshot()
    })
    await hook.rerender({
      targetId: "session:s2",
      sessionId: "s2",
      runId: null,
    })
    await act(async () => {
      await hook.result.current.loadSnapshot()
    })
    expect(hook.result.current.visibleSnapshot?.text).toBe("new target\n")
    await act(async () => {
      oldTarget.resolve(sessionResponse(baseSnapshot({ text: "old target\n" })))
      await oldLoad
    })
    expect(hook.result.current.visibleSnapshot?.text).toBe("new target\n")
    await hook.unmount()
  })
})
