// @vitest-environment happy-dom
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"

import {
  CLEAR_TOOLTIP,
  PATH_A_TOOLTIP,
  PATH_C_TOOLTIP,
} from "../app/ClaudeSignIn"
import { useClaudeOAuthSignIn } from "../app/useClaudeOAuthSignIn"
import type { ClaudeOAuthStatusResponse } from "../commands/runhaven"
import { act, renderHook } from "./render-hook"

vi.mock("@/commands/runhaven", async (importActual) => {
  const actual = await importActual<typeof import("@/commands/runhaven")>()
  return {
    ...actual,
    startClaudeOAuthSignIn: vi.fn(),
    getClaudeOAuthStatus: vi.fn(),
    clearClaudeOAuth: vi.fn(),
  }
})

const commands = await import("@/commands/runhaven")
const startSignIn = vi.mocked(commands.startClaudeOAuthSignIn)
const getStatus = vi.mocked(commands.getClaudeOAuthStatus)
const clearToken = vi.mocked(commands.clearClaudeOAuth)

function status(
  patch: Partial<ClaudeOAuthStatusResponse> = {},
): ClaudeOAuthStatusResponse {
  return {
    status: "not_signed_in",
    reason: null,
    signInId: null,
    ageDays: null,
    ...patch,
  }
}

function resetMocks(): void {
  startSignIn.mockReset()
  getStatus.mockReset()
  clearToken.mockReset()
  getStatus.mockResolvedValue(status())
}

// -- the approved verbatim strings ship unchanged -------------------------

describe("Claude sign-in tooltip strings", () => {
  it("ships the design's approved wording verbatim, with no paraphrase", () => {
    expect(PATH_C_TOOLTIP).toBe(
      "Sign in on your Mac. RunHaven saves your Claude token to your Mac's Keychain and uses it to sign your runs. The token never goes inside the sandbox.",
    )
    expect(PATH_A_TOOLTIP).toBe(
      "Sign in inside the sandbox instead. Your Claude login is saved in the agent's own space, not on your Mac's Keychain.",
    )
    expect(CLEAR_TOOLTIP).toBe(
      "Remove the Claude token RunHaven saved. This deletes only RunHaven's Keychain item, not your other logins.",
    )
  })
})

// -- path C flow: start -> signing -> signed_in / failed ------------------

describe("useClaudeOAuthSignIn", () => {
  beforeEach(() => {
    vi.useFakeTimers()
    resetMocks()
  })
  afterEach(() => {
    vi.clearAllTimers()
    vi.useRealTimers()
  })

  it("drives start -> signing -> signed_in and fires onSignedIn once", async () => {
    const onSignedIn = vi.fn()
    const hook = await renderHook(
      (cb: () => void) => useClaudeOAuthSignIn(cb),
      onSignedIn,
    )
    await act(async () => {}) // flush the mount status read (not signed in)

    startSignIn.mockResolvedValue({
      signInId: "c1",
      state: "signing",
      label: "Signing in",
      detail: "Started on your Mac.",
    })
    getStatus.mockReset()
    getStatus
      .mockResolvedValueOnce(status({ status: "signing", signInId: "c1" }))
      .mockResolvedValue(status({ status: "signed_in", signInId: "c1", ageDays: 3 }))

    await act(async () => {
      await hook.result.current.signIn()
    })
    expect(hook.result.current.status?.status).toBe("signing")

    await act(async () => {
      await vi.advanceTimersByTimeAsync(1500) // still signing
    })
    expect(onSignedIn).not.toHaveBeenCalled()

    await act(async () => {
      await vi.advanceTimersByTimeAsync(1500) // signed in
    })
    expect(hook.result.current.status?.status).toBe("signed_in")
    expect(hook.result.current.status?.ageDays).toBe(3)
    expect(onSignedIn).toHaveBeenCalledTimes(1)

    // Polling stops after the terminal state; onSignedIn never re-fires.
    const callsAfter = getStatus.mock.calls.length
    await act(async () => {
      await vi.advanceTimersByTimeAsync(6000)
    })
    expect(getStatus.mock.calls.length).toBe(callsAfter)
    expect(onSignedIn).toHaveBeenCalledTimes(1)
    await hook.unmount()
  })

  it("reflects a failed host capture without firing onSignedIn", async () => {
    const onSignedIn = vi.fn()
    const hook = await renderHook(
      (cb: () => void) => useClaudeOAuthSignIn(cb),
      onSignedIn,
    )
    await act(async () => {})

    startSignIn.mockResolvedValue({
      signInId: "c2",
      state: "signing",
      label: "Signing in",
      detail: "Started.",
    })
    getStatus.mockReset()
    getStatus.mockResolvedValue(
      status({
        status: "failed",
        signInId: "c2",
        reason: "Claude sign-in did not finish; nothing was saved.",
      }),
    )

    await act(async () => {
      await hook.result.current.signIn()
    })
    await act(async () => {
      await vi.advanceTimersByTimeAsync(1500)
    })
    expect(hook.result.current.status?.status).toBe("failed")
    expect(hook.result.current.status?.reason).toContain("nothing was saved")
    expect(onSignedIn).not.toHaveBeenCalled()
    await hook.unmount()
  })

  it("surfaces a canceled native confirm as an error and keeps prior state", async () => {
    const hook = await renderHook(
      (cb: () => void) => useClaudeOAuthSignIn(cb),
      () => undefined,
    )
    await act(async () => {})

    startSignIn.mockRejectedValue(
      new Error("Claude sign-in canceled before it started."),
    )
    getStatus.mockReset()
    getStatus.mockResolvedValue(status()) // the true, unchanged state

    await act(async () => {
      await hook.result.current.signIn()
    })
    await act(async () => {}) // flush the follow-up refresh
    expect(hook.result.current.error).toContain("canceled")
    expect(hook.result.current.status?.status).toBe("not_signed_in")
    await hook.unmount()
  })

  it("stops the waiting UI without stopping the poll", async () => {
    const hook = await renderHook(
      (cb: () => void) => useClaudeOAuthSignIn(cb),
      () => undefined,
    )
    await act(async () => {})

    startSignIn.mockResolvedValue({
      signInId: "c3",
      state: "signing",
      label: "Signing in",
      detail: "Started.",
    })
    // The host keeps reporting "signing" (the browser step is still open).
    getStatus.mockReset()
    getStatus.mockResolvedValue(status({ status: "signing", signInId: "c3" }))

    await act(async () => {
      await hook.result.current.signIn()
    })
    expect(hook.result.current.signing).toBe(true)

    // Stopping hides the active waiting UI but the host still says "signing".
    await act(async () => {
      hook.result.current.cancel()
    })
    expect(hook.result.current.signing).toBe(false)
    expect(hook.result.current.canceled).toBe(true)
    // The backend is still signing, so the poll stays alive to catch a result.
    expect(hook.result.current.polling).toBe(true)
    expect(hook.result.current.status?.status).toBe("signing")

    // The poll is NOT torn down by cancel: advancing time keeps reading status.
    const callsAfterCancel = getStatus.mock.calls.length
    await act(async () => {
      await vi.advanceTimersByTimeAsync(6000)
    })
    expect(getStatus.mock.calls.length).toBeGreaterThan(callsAfterCancel)
    await hook.unmount()
  })

  it("still fires onSignedIn when the host capture finishes after cancel", async () => {
    // The exact race the review flagged: the user hits "Stop waiting", then the
    // host `claude setup-token` capture succeeds. onSignedIn must still fire and
    // the UI must show signed in, with no manual reload.
    const onSignedIn = vi.fn()
    const hook = await renderHook(
      (cb: () => void) => useClaudeOAuthSignIn(cb),
      onSignedIn,
    )
    await act(async () => {})

    startSignIn.mockResolvedValue({
      signInId: "c5",
      state: "signing",
      label: "Signing in",
      detail: "Started.",
    })
    getStatus.mockReset()
    getStatus.mockResolvedValue(status({ status: "signing", signInId: "c5" }))

    await act(async () => {
      await hook.result.current.signIn()
    })
    expect(hook.result.current.signing).toBe(true)

    // Stop waiting while the capture is still open.
    await act(async () => {
      hook.result.current.cancel()
    })
    expect(hook.result.current.signing).toBe(false)
    expect(onSignedIn).not.toHaveBeenCalled()

    // The host capture completes; the still-live poll reads signed_in.
    getStatus.mockResolvedValue(
      status({ status: "signed_in", signInId: "c5", ageDays: 0 }),
    )
    await act(async () => {
      await vi.advanceTimersByTimeAsync(1500)
    })

    expect(hook.result.current.status?.status).toBe("signed_in")
    expect(onSignedIn).toHaveBeenCalledTimes(1)
    // Once signed in, the poll settles and onSignedIn never re-fires.
    const callsAfterSignedIn = getStatus.mock.calls.length
    await act(async () => {
      await vi.advanceTimersByTimeAsync(6000)
    })
    expect(getStatus.mock.calls.length).toBe(callsAfterSignedIn)
    expect(onSignedIn).toHaveBeenCalledTimes(1)
    await hook.unmount()
  })

  it("surfaces a start-over hint after a long wait", async () => {
    const hook = await renderHook(
      (cb: () => void) => useClaudeOAuthSignIn(cb),
      () => undefined,
    )
    await act(async () => {})

    startSignIn.mockResolvedValue({
      signInId: "c4",
      state: "signing",
      label: "Signing in",
      detail: "Started.",
    })
    getStatus.mockReset()
    getStatus.mockResolvedValue(status({ status: "signing", signInId: "c4" }))

    await act(async () => {
      await hook.result.current.signIn()
    })
    expect(hook.result.current.stalled).toBe(false)

    await act(async () => {
      await vi.advanceTimersByTimeAsync(45000)
    })
    expect(hook.result.current.stalled).toBe(true)
    await hook.unmount()
  })

  it("reports a failed status read apart from a failed start", async () => {
    // A failed initial status read is a status-read error, not "sign-in did not
    // start".
    getStatus.mockReset()
    getStatus.mockRejectedValueOnce(new Error("Could not reach the Keychain."))
    const hook = await renderHook(
      (cb: () => void) => useClaudeOAuthSignIn(cb),
      () => undefined,
    )
    await act(async () => {})
    expect(hook.result.current.statusError).toContain("Could not reach")
    expect(hook.result.current.error).toBe("")

    // A failed start is the start error, and re-reading the true state clears the
    // earlier status-read error.
    startSignIn.mockRejectedValue(
      new Error("Claude sign-in canceled before it started."),
    )
    getStatus.mockReset()
    getStatus.mockResolvedValue(status())
    await act(async () => {
      await hook.result.current.signIn()
    })
    await act(async () => {}) // flush the follow-up refresh
    expect(hook.result.current.error).toContain("canceled")
    expect(hook.result.current.statusError).toBe("")
    await hook.unmount()
  })

  it("removes the token and refreshes back to not signed in", async () => {
    getStatus.mockResolvedValue(status({ status: "signed_in", ageDays: 5 }))
    const hook = await renderHook(
      (cb: () => void) => useClaudeOAuthSignIn(cb),
      () => undefined,
    )
    await act(async () => {})
    expect(hook.result.current.status?.status).toBe("signed_in")

    clearToken.mockResolvedValue({
      outcome: "cleared",
      detail: "RunHaven removed the Claude token it saved.",
    })
    getStatus.mockReset()
    getStatus.mockResolvedValue(status()) // now absent

    await act(async () => {
      await hook.result.current.clear()
    })
    expect(hook.result.current.clearMessage).toContain("removed the Claude token")
    expect(hook.result.current.status?.status).toBe("not_signed_in")
    await hook.unmount()
  })
})
