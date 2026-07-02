// @vitest-environment happy-dom
import { createElement } from "react"
import { createRoot } from "react-dom/client"
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"

import {
  CLEAR_TOOLTIP,
  ClaudeSignIn,
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

// -- component: age, rotation prompt, path A opt-in ------------------------

describe("ClaudeSignIn component", () => {
  beforeEach(() => {
    resetMocks()
  })

  async function render(reauthDays: number) {
    const container = document.createElement("div")
    const root = createRoot(container)
    await act(async () => {
      root.render(createElement(ClaudeSignIn, { reauthDays }))
    })
    await act(async () => {}) // flush the mount status read
    return { container, root }
  }

  it("shows the token age and prompts a fresh sign-in past the cadence", async () => {
    getStatus.mockResolvedValue(status({ status: "signed_in", ageDays: 100 }))
    const { container, root } = await render(90)

    expect(container.textContent).toContain("Saved 100 days ago")
    // Past the 90-day cadence the reminder prompt shows, as a nudge not a cutoff.
    expect(container.textContent).toContain("Time to sign in again")
    expect(container.textContent).toContain("only a reminder, not a cutoff")
    const again = Array.from(container.querySelectorAll("button")).find((b) =>
      b.textContent?.includes("Sign in again"),
    )
    expect(again).toBeDefined()
    await act(async () => {
      root.unmount()
    })
  })

  it("shows the age but no prompt while the token is within the cadence", async () => {
    getStatus.mockResolvedValue(status({ status: "signed_in", ageDays: 10 }))
    const { container, root } = await render(90)

    expect(container.textContent).toContain("Saved 10 days ago")
    expect(container.textContent).not.toContain("Time to sign in again")
    // The confirm-gated remove action is offered.
    expect(container.textContent).toContain("Remove saved token")
    await act(async () => {
      root.unmount()
    })
  })

  it("offers the on-your-Mac primary action when not signed in", async () => {
    getStatus.mockResolvedValue(status())
    const { container, root } = await render(90)

    expect(container.textContent).toContain("Sign in on your Mac")
    // No secret input anywhere.
    expect(container.querySelector("input")).toBeNull()
    await act(async () => {
      root.unmount()
    })
  })

  it("reveals the unchanged in-sandbox terminal login only when the opt-in is chosen", async () => {
    getStatus.mockResolvedValue(status())
    const { container, root } = await render(90)

    // Collapsed by default: the terminal command is not shown.
    expect(container.textContent).not.toContain("runhaven login claude")
    const optIn = Array.from(container.querySelectorAll("button")).find((b) =>
      b.textContent?.includes("Sign in inside the sandbox instead"),
    )
    expect(optIn).toBeDefined()
    await act(async () => {
      optIn?.click()
    })
    expect(container.textContent).toContain("runhaven login claude")
    expect(container.textContent).toContain("Copy command")
    await act(async () => {
      root.unmount()
    })
  })
})
