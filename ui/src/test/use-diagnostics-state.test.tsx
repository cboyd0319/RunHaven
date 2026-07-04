// @vitest-environment happy-dom
import { beforeEach, describe, expect, it, vi } from "vitest"

import { useDiagnosticsState } from "../app/useDiagnosticsState"
import type {
  AuthLogResponse,
  AuthStatusResponse,
  EgressLogResponse,
} from "../commands/runhaven"

import { act, renderHook } from "./render-hook"

vi.mock("@/commands/runhaven", async (importActual) => {
  const actual = await importActual<typeof import("@/commands/runhaven")>()
  return {
    ...actual,
    getAuthStatus: vi.fn(),
    getAuthLog: vi.fn(),
    getEgressLog: vi.fn(),
  }
})

const { getAuthStatus, getAuthLog, getEgressLog } = await import(
  "@/commands/runhaven"
)
const authStatusMock = vi.mocked(getAuthStatus)
const authLogMock = vi.mocked(getAuthLog)
const egressLogMock = vi.mocked(getEgressLog)

const authStatus: AuthStatusResponse = {
  status: "ok",
  runtime: "running",
  profiles: [{ name: "codex", status: "api-key-broker" }],
}
const authLog: AuthLogResponse = { entries: [] }
const egressLog: EgressLogResponse = { entries: [] }

beforeEach(() => {
  authStatusMock.mockReset()
  authLogMock.mockReset()
  egressLogMock.mockReset()
})

describe("useDiagnosticsState", () => {
  it("keeps last-good data when an explicit reload fails", async () => {
    authStatusMock.mockResolvedValue(authStatus)
    authLogMock.mockResolvedValue(authLog)
    egressLogMock.mockResolvedValue(egressLog)

    const hook = await renderHook(() => useDiagnosticsState(), undefined)
    await act(async () => {
      await hook.result.current.load()
    })
    expect(hook.result.current.data?.authStatus.status).toBe("ok")
    expect(hook.result.current.error).toBe("")

    // One failing sub-call on reload must not blank the whole panel: the
    // last-good auth/egress data stays put and only the error surfaces.
    egressLogMock.mockRejectedValueOnce(new Error("egress log unavailable"))
    await act(async () => {
      await hook.result.current.load()
    })
    expect(hook.result.current.data?.authStatus.status).toBe("ok")
    expect(hook.result.current.error).toContain("unavailable")
    await hook.unmount()
  })
})
