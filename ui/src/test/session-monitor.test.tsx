// @vitest-environment happy-dom
import { renderToStaticMarkup } from "react-dom/server"
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"

import { SessionMonitorView } from "../app/SessionMonitorApp"
import { useSessionMonitor } from "../app/useSessionMonitor"
import type {
  ManagedSessionStatusResponse,
  ManagedSessionSummary,
} from "../commands/runhaven"

import { act, renderHook } from "./render-hook"

vi.mock("@/commands/runhaven", async (importActual) => {
  const actual = await importActual<typeof import("@/commands/runhaven")>()
  return {
    ...actual,
    hasTauriRuntime: vi.fn(),
    getSessionWindowTarget: vi.fn(),
    getManagedSessionStatus: vi.fn(),
  }
})

const runhaven = await import("@/commands/runhaven")
const hasTauriRuntime = vi.mocked(runhaven.hasTauriRuntime)
const getSessionWindowTarget = vi.mocked(runhaven.getSessionWindowTarget)
const getManagedSessionStatus = vi.mocked(runhaven.getManagedSessionStatus)

function session(patch: Partial<ManagedSessionSummary> = {}): ManagedSessionSummary {
  return {
    id: "run-1",
    runId: "run-1",
    agent: "codex",
    session: "alpha",
    title: "Codex - alpha",
    project: "alpha",
    status: "running",
    statusLabel: "Running",
    authStatus: "api-key-broker",
    authScope: "project",
    network: "provider",
    startedAt: "2026-07-01T02:00:00Z",
    containerName: "runhaven-codex-alpha",
    detailCommand: "runhaven runs show run-1",
    lifecycle: "running",
    input: { attached: false, status: "detached", detail: "Not attached." },
    nextAction: {
      kind: "monitor",
      label: "Monitor session",
      detail: "RunHaven is tracking this session.",
    },
    events: [],
    ...patch,
  }
}

function status(
  patch: Partial<ManagedSessionStatusResponse> = {},
): ManagedSessionStatusResponse {
  return {
    session: session(),
    container: null,
    statusUnavailable: null,
    recovery: null,
    ...patch,
  }
}

const noopDiagnostics = {
  data: null,
  loading: false,
  error: "",
  load: async () => {},
}

beforeEach(() => {
  hasTauriRuntime.mockReturnValue(true)
  getSessionWindowTarget.mockResolvedValue({ sessionId: "run-1" })
  getManagedSessionStatus.mockResolvedValue(status())
})

afterEach(() => {
  vi.clearAllMocks()
  window.history.replaceState(null, "", "/")
})

async function flush() {
  // The monitor resolves its target, then starts polling: two chained async
  // effects, so flush a few microtask rounds inside act.
  await act(async () => {
    for (let round = 0; round < 4; round += 1) {
      await Promise.resolve()
    }
  })
}

describe("useSessionMonitor", () => {
  it("resolves the target from Rust and polls that session's status", async () => {
    const hook = await renderHook(useSessionMonitor, undefined)
    await flush()
    expect(getSessionWindowTarget).toHaveBeenCalledTimes(1)
    expect(getManagedSessionStatus).toHaveBeenCalledWith("run-1")
    expect(hook.result.current.sessionId).toBe("run-1")
    expect(hook.result.current.status?.session.id).toBe("run-1")
    expect(hook.result.current.gone).toBe(false)
    await hook.unmount()
  })

  it("uses the preview URL parameter only without a Tauri runtime", async () => {
    hasTauriRuntime.mockReturnValue(false)
    window.history.replaceState(null, "", "/?monitor=1&session=run-preview")
    getManagedSessionStatus.mockResolvedValue(
      status({ session: session({ id: "run-preview" }) }),
    )
    const hook = await renderHook(useSessionMonitor, undefined)
    await flush()
    expect(getSessionWindowTarget).not.toHaveBeenCalled()
    expect(hook.result.current.sessionId).toBe("run-preview")
    await hook.unmount()
  })

  it("reports the gone end state when the record disappears, keeping the last view", async () => {
    vi.useFakeTimers()
    try {
      const hook = await renderHook(useSessionMonitor, undefined)
      await flush()
      expect(hook.result.current.status).not.toBeNull()
      expect(hook.result.current.gone).toBe(false)

      getManagedSessionStatus.mockRejectedValue(
        new Error("managed session not found"),
      )
      await act(async () => {
        await vi.advanceTimersByTimeAsync(4_000)
      })
      expect(hook.result.current.gone).toBe(true)
      // The end state stays on screen: last-known status is kept, and the
      // gone signal is not treated as a poll failure.
      expect(hook.result.current.status?.session.id).toBe("run-1")
      expect(hook.result.current.pollError).toBe("")
      await hook.unmount()
    } finally {
      vi.useRealTimers()
    }
  })

  it("keeps the last-good view through a transient poll error", async () => {
    vi.useFakeTimers()
    try {
      const hook = await renderHook(useSessionMonitor, undefined)
      await flush()
      getManagedSessionStatus.mockRejectedValue(new Error("container timed out"))
      await act(async () => {
        await vi.advanceTimersByTimeAsync(4_000)
      })
      expect(hook.result.current.pollError).toContain("timed out")
      expect(hook.result.current.gone).toBe(false)
      expect(hook.result.current.status?.session.id).toBe("run-1")
      await hook.unmount()
    } finally {
      vi.useRealTimers()
    }
  })

  it("fails closed when the window has no registered session", async () => {
    getSessionWindowTarget.mockRejectedValue(
      new Error("No session is registered for this window."),
    )
    const hook = await renderHook(useSessionMonitor, undefined)
    await flush()
    expect(hook.result.current.resolveError).toContain("No session is registered")
    expect(getManagedSessionStatus).not.toHaveBeenCalled()
    await hook.unmount()
  })
})

describe("SessionMonitorView", () => {
  it("renders a read-only monitor: overview plus the controls notice, no controls", () => {
    const markup = renderToStaticMarkup(
      <SessionMonitorView
        monitor={{
          sessionId: "run-1",
          resolveError: "",
          status: status(),
          gone: false,
          pollError: "",
        }}
        diagnostics={noopDiagnostics}
      />,
    )
    expect(markup).toContain("Controls live in the main window")
    expect(markup).toContain("Codex - alpha")
    expect(markup).toContain("Show details")
    // No mutating affordances of the main session view may appear here.
    for (const forbidden of [
      "Stop run",
      "Hard stop",
      "Repair",
      "Clean up",
      "Send",
      "Load output",
      "Sign in",
      "Rebuild",
    ]) {
      expect(markup, forbidden).not.toContain(forbidden)
    }
  })

  it("shows the gone end state and keeps the last-known overview", () => {
    const markup = renderToStaticMarkup(
      <SessionMonitorView
        monitor={{
          sessionId: "run-1",
          resolveError: "",
          status: status(),
          gone: true,
          pollError: "",
        }}
        diagnostics={noopDiagnostics}
      />,
    )
    expect(markup).toContain("This session is gone")
    expect(markup).toContain("Cmd-W")
    expect(markup).toContain("Codex - alpha")
    // Details hide with the record: nothing left to load.
    expect(markup).not.toContain("Show details")
  })

  it("explains an unresolved window instead of guessing a session", () => {
    const markup = renderToStaticMarkup(
      <SessionMonitorView
        monitor={{
          sessionId: null,
          resolveError: "This window has no session to watch.",
          status: null,
          gone: false,
          pollError: "",
        }}
        diagnostics={noopDiagnostics}
      />,
    )
    expect(markup).toContain("Nothing to watch")
    expect(markup).toContain("no session to watch")
  })
})
