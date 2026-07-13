// @vitest-environment happy-dom
import { createElement } from "react"
import { createRoot } from "react-dom/client"
import { beforeEach, describe, expect, it, vi } from "vitest"

import { MaintenanceView } from "../app/views/MaintenanceView"

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

describe("network cleanup flow", () => {
  it("warns when one or more stale networks could not be removed", async () => {
    getPreview.mockResolvedValueOnce({ count: 1 }).mockResolvedValueOnce({ count: 0 })
    prune.mockResolvedValue({ removed: 0, failed: 1 })
    const container = document.createElement("div")
    const root = createRoot(container)
    await act(async () => {
      root.render(createElement(MaintenanceView, { setupReady: true }))
    })

    const button = (label: string) =>
      Array.from(container.querySelectorAll("button")).find((item) =>
        item.textContent?.includes(label),
      )
    await act(async () => {
      button("Check for stale networks")?.click()
    })
    expect(container.textContent).toContain("Remove stale networks")
    await act(async () => {
      button("Remove stale networks")?.click()
    })

    expect(container.textContent).toContain("Could not remove 1")
    expect(container.textContent).not.toContain("Cleanup finished")
    await act(async () => {
      root.unmount()
    })
  })
})
