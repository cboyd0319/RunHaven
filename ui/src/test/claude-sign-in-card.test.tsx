// @vitest-environment happy-dom
import { createElement } from "react"
import { createRoot } from "react-dom/client"
import { beforeEach, describe, expect, it, vi } from "vitest"

import { ClaudeSignIn } from "../app/ClaudeSignIn"
import type { ClaudeOAuthStatusResponse } from "../commands/runhaven"
import { act } from "./render-hook"

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

// -- component: age, rotation prompt, stop-waiting, path A opt-in ----------

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

  it("offers a stop-waiting control while a sign-in is in flight", async () => {
    getStatus.mockResolvedValue(status({ status: "signing" }))
    const { container, root } = await render(90)

    expect(container.textContent).toContain("Signing in on your Mac")
    const stop = Array.from(container.querySelectorAll("button")).find((b) =>
      b.textContent?.includes("Stop waiting"),
    )
    expect(stop).toBeDefined()
    await act(async () => {
      root.unmount()
    })
  })

  it("hides the restart while still signing after the user stops waiting", async () => {
    // While the host is still signing, stopping must not offer a restart: the
    // backend refuses a concurrent capture, so a second "Sign in" would double
    // capture. The card shows only the honest "may still finish" message.
    getStatus.mockResolvedValue(status({ status: "signing" }))
    const { container, root } = await render(90)

    const stop = Array.from(container.querySelectorAll("button")).find((b) =>
      b.textContent?.includes("Stop waiting"),
    )
    expect(stop).toBeDefined()
    await act(async () => {
      stop?.click()
    })

    expect(container.textContent).toContain(
      "You stopped waiting. The sign-in may still finish on its own.",
    )
    // No restart control while the backend is still signing.
    const restart = Array.from(container.querySelectorAll("button")).find((b) =>
      b.textContent?.includes("Sign in on your Mac"),
    )
    expect(restart).toBeUndefined()
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
