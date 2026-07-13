// @vitest-environment happy-dom
import { createElement } from "react"
import { createRoot } from "react-dom/client"
import { beforeEach, describe, expect, it, vi } from "vitest"

import { DiagnosticsView } from "../app/views/DiagnosticsView"
import { dashboard } from "./main-view-props"
import { act } from "./render-hook"

vi.mock("@/commands/runhaven", async (importActual) => {
  const actual = await importActual<typeof import("@/commands/runhaven")>()
  return {
    ...actual,
    getNetworkPrunePreview: vi.fn(),
    pruneManagedNetworks: vi.fn(),
  }
})

const { getNetworkPrunePreview, pruneManagedNetworks } = await import(
  "@/commands/runhaven"
)
const getPreview = vi.mocked(getNetworkPrunePreview)
const prune = vi.mocked(pruneManagedNetworks)

beforeEach(() => {
  getPreview.mockReset()
  prune.mockReset()
})

describe("System details network cleanup", () => {
  it("loads on entry, removes in one click, and reports partial failure", async () => {
    getPreview.mockResolvedValueOnce({ count: 1 }).mockResolvedValueOnce({ count: 0 })
    prune.mockResolvedValue({ removed: 0, failed: 1 })
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

    const remove = Array.from(container.querySelectorAll("button")).find(
      (button) => button.textContent?.includes("Remove old networks"),
    )
    expect(remove).toBeDefined()
    expect(remove?.disabled).toBe(false)

    await act(async () => {
      remove?.click()
    })

    expect(prune).toHaveBeenCalledTimes(1)
    expect(container.textContent).toContain("Could not remove 1 network")
    expect(container.textContent).not.toContain("Cleanup complete")
    await act(async () => root.unmount())
  })
})
