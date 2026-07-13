// @vitest-environment happy-dom
import { createElement } from "react"
import { createRoot } from "react-dom/client"
import { afterEach, describe, expect, it, vi } from "vitest"

import { DiagnosticsView } from "../app/views/DiagnosticsView"

import { dashboard } from "./main-view-props"
import { act } from "./render-hook"

const recovery = vi.hoisted(() => vi.fn())

vi.mock("@/commands/runhaven", async (importActual) => ({
  ...(await importActual<typeof import("@/commands/runhaven")>()),
  performSetupRecovery: recovery,
}))

afterEach(() => {
  recovery.mockReset()
})

describe("DiagnosticsView container-service recovery", () => {
  it("does not report a browser preview fallback as a successful repair", async () => {
    recovery.mockResolvedValue({
      actionId: "restart-container-system",
      status: "preview",
      label: "Desktop app required",
      detail: "Open RunHaven on your Mac to repair setup.",
    })
    const container = document.createElement("div")
    const root = createRoot(container)

    await act(async () => {
      root.render(
        createElement(DiagnosticsView, {
          dashboard: dashboard(true),
          refreshing: false,
          onRefresh: () => {},
        }),
      )
    })
    const restart = Array.from(container.querySelectorAll("button")).find(
      (button) => button.textContent === "Restart container service…",
    )
    await act(async () => {
      restart?.click()
    })

    expect(container.textContent).toContain("Desktop app required")
    expect(container.textContent).not.toContain("Internet service refreshed")

    await act(async () => {
      root.unmount()
    })
  })

  it("refreshes readiness after a failed restart", async () => {
    recovery.mockRejectedValue(
      new Error(
        "The container service stopped, but RunHaven could not start it again. Choose Check again, then Start container service.",
      ),
    )
    let refreshes = 0
    const container = document.createElement("div")
    const root = createRoot(container)

    await act(async () => {
      root.render(
        createElement(DiagnosticsView, {
          dashboard: dashboard(true),
          refreshing: false,
          onRefresh: () => {
            refreshes += 1
          },
        }),
      )
    })

    const restart = Array.from(container.querySelectorAll("button")).find(
      (button) => button.textContent === "Restart container service…",
    )
    expect(restart).toBeDefined()
    await act(async () => {
      restart?.click()
    })

    expect(recovery).toHaveBeenCalledWith("restart-container-system")
    expect(refreshes).toBe(1)
    expect(container.textContent).toContain("Could not restart the service")
    expect(container.textContent).toContain("Start container service")

    await act(async () => {
      root.unmount()
    })
  })
})
