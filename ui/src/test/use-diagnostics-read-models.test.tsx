// @vitest-environment happy-dom
import { beforeEach, describe, expect, it, vi } from "vitest"

import { useDiagnosticsReadModels } from "../app/useDiagnosticsReadModels"

import { renderHook } from "./render-hook"
import { providerAgent } from "./runhaven-fixtures"

vi.mock("@/commands/runhaven", async (importActual) => {
  const actual = await importActual<typeof import("@/commands/runhaven")>()
  return {
    ...actual,
    getImageStatus: vi.fn(),
    getNetworkPrunePreview: vi.fn(),
  }
})

const { getImageStatus, getNetworkPrunePreview } = await import(
  "@/commands/runhaven"
)
const getImage = vi.mocked(getImageStatus)
const getNetworks = vi.mocked(getNetworkPrunePreview)

beforeEach(() => {
  getImage.mockReset()
  getNetworks.mockReset()
})

describe("useDiagnosticsReadModels", () => {
  it("does not query runtime data before setup is ready", async () => {
    getImage.mockRejectedValue(new Error("runtime unavailable"))
    getNetworks.mockResolvedValue({ count: 0 })
    const hook = await renderHook(
      () => useDiagnosticsReadModels([providerAgent], false),
      undefined,
    )

    expect(getImage).not.toHaveBeenCalled()
    expect(getNetworks).not.toHaveBeenCalled()
    expect(hook.result.current.loading).toBe(false)
    await hook.unmount()
  })
})
