// @vitest-environment happy-dom
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"

import { useAgentSignIn, useImageRebuild } from "../app/useMaintenance"
import type {
  AgentSignInStatusResponse,
  ImageBuildStatusResponse,
} from "../commands/runhaven"
import { act, renderHook } from "./render-hook"

vi.mock("@/commands/runhaven", async (importActual) => {
  const actual = await importActual<typeof import("@/commands/runhaven")>()
  return {
    ...actual,
    startAgentSignIn: vi.fn(),
    getAgentSignInStatus: vi.fn(),
    rebuildAgentImage: vi.fn(),
    getImageBuildStatus: vi.fn(),
  }
})

const commands = await import("@/commands/runhaven")
const startSignIn = vi.mocked(commands.startAgentSignIn)
const getSignInStatus = vi.mocked(commands.getAgentSignInStatus)
const rebuildImage = vi.mocked(commands.rebuildAgentImage)
const getBuildStatus = vi.mocked(commands.getImageBuildStatus)

function signInStatus(
  state: string,
  patch: Partial<AgentSignInStatusResponse> = {},
): AgentSignInStatusResponse {
  return {
    signInId: "si1",
    agent: "codex",
    state,
    label: "Signing in",
    detail: "Open the link to finish.",
    outputTail: "",
    updatedAt: "2026-07-01T02:00:00Z",
    ...patch,
  }
}

function buildStatus(
  state: string,
  patch: Partial<ImageBuildStatusResponse> = {},
): ImageBuildStatusResponse {
  return {
    buildId: "b1",
    agent: "codex",
    tag: "runhaven/codex:0.1.0",
    state,
    label: "Building",
    detail: "Building image.",
    outputTail: "",
    updatedAt: "2026-07-01T02:00:00Z",
    ...patch,
  }
}

beforeEach(() => {
  vi.useFakeTimers()
  startSignIn.mockReset()
  getSignInStatus.mockReset()
  rebuildImage.mockReset()
  getBuildStatus.mockReset()
})

afterEach(() => {
  vi.clearAllTimers()
  vi.useRealTimers()
})

describe("useAgentSignIn", () => {
  it("calls onSignedIn exactly once on terminal success then stops polling", async () => {
    startSignIn.mockResolvedValue({
      signInId: "si1",
      agent: "codex",
      state: "starting",
      label: "Signing in",
      detail: "Starting.",
    })
    getSignInStatus
      .mockResolvedValueOnce(signInStatus("needs_browser"))
      .mockResolvedValueOnce(signInStatus("succeeded"))
      .mockResolvedValue(signInStatus("succeeded"))

    const onSignedIn = vi.fn()
    const hook = await renderHook(
      (cb: () => void) => useAgentSignIn(cb),
      onSignedIn,
    )
    await act(async () => {
      await hook.result.current.signIn("codex")
    })

    await act(async () => {
      await vi.advanceTimersByTimeAsync(1_500) // needs_browser
    })
    expect(onSignedIn).not.toHaveBeenCalled()

    await act(async () => {
      await vi.advanceTimersByTimeAsync(1_500) // succeeded
    })
    expect(hook.result.current.status?.state).toBe("succeeded")
    expect(onSignedIn).toHaveBeenCalledTimes(1)

    const callsAfterSuccess = getSignInStatus.mock.calls.length
    await act(async () => {
      await vi.advanceTimersByTimeAsync(6_000)
    })
    // Polling stops after the terminal state; onSignedIn never re-fires.
    expect(getSignInStatus.mock.calls.length).toBe(callsAfterSuccess)
    expect(onSignedIn).toHaveBeenCalledTimes(1)
    await hook.unmount()
  })

  it("stops the waiting UI without stopping the poll", async () => {
    startSignIn.mockResolvedValue({
      signInId: "si1",
      agent: "codex",
      state: "starting",
      label: "Signing in",
      detail: "Starting.",
    })
    getSignInStatus.mockResolvedValue(signInStatus("needs_browser"))

    const hook = await renderHook(
      (cb: () => void) => useAgentSignIn(cb),
      () => undefined,
    )
    await act(async () => {
      await hook.result.current.signIn("codex")
    })
    await act(async () => {
      await vi.advanceTimersByTimeAsync(1_500)
    })
    expect(hook.result.current.signing).toBe(true)

    // Stopping hides the active waiting UI but does not tear down the poll.
    await act(async () => {
      hook.result.current.cancel()
    })
    expect(hook.result.current.signing).toBe(false)
    expect(hook.result.current.canceled).toBe(true)
    expect(hook.result.current.polling).toBe(true)

    const callsAfterCancel = getSignInStatus.mock.calls.length
    await act(async () => {
      await vi.advanceTimersByTimeAsync(6_000)
    })
    // The poll keeps reading status after cancel.
    expect(getSignInStatus.mock.calls.length).toBeGreaterThan(callsAfterCancel)
    await hook.unmount()
  })

  it("still fires onSignedIn when the device login finishes after cancel", async () => {
    // The user hits "Stop waiting", then the device login succeeds. onSignedIn
    // must still fire once, with no manual reload.
    startSignIn.mockResolvedValue({
      signInId: "si1",
      agent: "codex",
      state: "starting",
      label: "Signing in",
      detail: "Starting.",
    })
    getSignInStatus.mockResolvedValue(signInStatus("needs_browser"))

    const onSignedIn = vi.fn()
    const hook = await renderHook(
      (cb: () => void) => useAgentSignIn(cb),
      onSignedIn,
    )
    await act(async () => {
      await hook.result.current.signIn("codex")
    })
    await act(async () => {
      await vi.advanceTimersByTimeAsync(1_500)
    })
    expect(hook.result.current.signing).toBe(true)

    // Stop waiting while the device login is still open.
    await act(async () => {
      hook.result.current.cancel()
    })
    expect(hook.result.current.signing).toBe(false)
    expect(onSignedIn).not.toHaveBeenCalled()

    // The device login completes; the still-live poll reads succeeded.
    getSignInStatus.mockResolvedValue(signInStatus("succeeded"))
    await act(async () => {
      await vi.advanceTimersByTimeAsync(1_500)
    })
    expect(hook.result.current.status?.state).toBe("succeeded")
    expect(onSignedIn).toHaveBeenCalledTimes(1)

    // Once succeeded, the poll settles and onSignedIn never re-fires.
    const callsAfterSuccess = getSignInStatus.mock.calls.length
    await act(async () => {
      await vi.advanceTimersByTimeAsync(6_000)
    })
    expect(getSignInStatus.mock.calls.length).toBe(callsAfterSuccess)
    expect(onSignedIn).toHaveBeenCalledTimes(1)
    await hook.unmount()
  })

  it("surfaces a start-over hint after a long wait", async () => {
    startSignIn.mockResolvedValue({
      signInId: "si1",
      agent: "codex",
      state: "starting",
      label: "Signing in",
      detail: "Starting.",
    })
    getSignInStatus.mockResolvedValue(signInStatus("needs_browser"))

    const hook = await renderHook(
      (cb: () => void) => useAgentSignIn(cb),
      () => undefined,
    )
    await act(async () => {
      await hook.result.current.signIn("codex")
    })
    expect(hook.result.current.stalled).toBe(false)

    await act(async () => {
      await vi.advanceTimersByTimeAsync(45_000)
    })
    expect(hook.result.current.stalled).toBe(true)
    await hook.unmount()
  })

  it("does not re-arm the interval when the onSignedIn identity changes", async () => {
    startSignIn.mockResolvedValue({
      signInId: "si1",
      agent: "codex",
      state: "starting",
      label: "Signing in",
      detail: "Starting.",
    })
    getSignInStatus.mockResolvedValue(signInStatus("needs_browser"))

    const hook = await renderHook(
      (cb: () => void) => useAgentSignIn(cb),
      () => undefined,
    )
    await act(async () => {
      await hook.result.current.signIn("codex")
    })
    await act(async () => {
      await vi.advanceTimersByTimeAsync(1_500)
    })
    const callsBefore = getSignInStatus.mock.calls.length

    // A new inline callback (as parents pass) must not restart the poll.
    await hook.rerender(() => undefined)
    await act(async () => {
      await vi.advanceTimersByTimeAsync(1_500)
    })
    // Exactly one more poll for the one interval tick, not a re-armed burst.
    expect(getSignInStatus.mock.calls.length).toBe(callsBefore + 1)
    await hook.unmount()
  })

  it("keeps the status object stable and clears error on recovery", async () => {
    startSignIn.mockResolvedValue({
      signInId: "si1",
      agent: "codex",
      state: "starting",
      label: "Signing in",
      detail: "Starting.",
    })
    getSignInStatus
      .mockResolvedValueOnce(signInStatus("needs_browser"))
      .mockRejectedValueOnce(new Error("status probe failed"))
      .mockResolvedValue(signInStatus("needs_browser"))

    const hook = await renderHook(
      (cb: () => void) => useAgentSignIn(cb),
      () => undefined,
    )
    await act(async () => {
      await hook.result.current.signIn("codex")
    })
    await act(async () => {
      await vi.advanceTimersByTimeAsync(1_500)
    })
    const stable = hook.result.current.status
    expect(stable?.state).toBe("needs_browser")

    await act(async () => {
      await vi.advanceTimersByTimeAsync(1_500) // rejects
    })
    expect(hook.result.current.error).toContain("failed")
    // Equal status content keeps the same object reference.
    expect(hook.result.current.status).toBe(stable)

    await act(async () => {
      await vi.advanceTimersByTimeAsync(1_500) // recovers
    })
    expect(hook.result.current.error).toBe("")
    await hook.unmount()
  })
})

describe("useImageRebuild", () => {
  it("polls until terminal, guards equal status, and stops after unmount", async () => {
    rebuildImage.mockResolvedValue({
      buildId: "b1",
      agent: "codex",
      tag: "runhaven/codex:0.1.0",
      state: "building",
      label: "Building",
      detail: "Queued.",
    })
    getBuildStatus
      .mockResolvedValueOnce(buildStatus("building"))
      .mockResolvedValueOnce(buildStatus("building"))
      .mockResolvedValue(buildStatus("succeeded"))

    const hook = await renderHook(() => useImageRebuild(), undefined)
    await act(async () => {
      hook.result.current.setConfirm(true)
    })
    await act(async () => {
      await hook.result.current.rebuild("codex")
    })

    await act(async () => {
      await vi.advanceTimersByTimeAsync(1_500)
    })
    const buildingStatus = hook.result.current.status
    expect(buildingStatus?.state).toBe("building")

    // Second poll returns identical content: reference stays stable.
    await act(async () => {
      await vi.advanceTimersByTimeAsync(1_500)
    })
    expect(hook.result.current.status).toBe(buildingStatus)

    await act(async () => {
      await vi.advanceTimersByTimeAsync(1_500) // succeeded
    })
    expect(hook.result.current.status?.state).toBe("succeeded")

    const callsAfterTerminal = getBuildStatus.mock.calls.length
    await act(async () => {
      await vi.advanceTimersByTimeAsync(6_000)
    })
    expect(getBuildStatus.mock.calls.length).toBe(callsAfterTerminal)
    await hook.unmount()
  })
})
