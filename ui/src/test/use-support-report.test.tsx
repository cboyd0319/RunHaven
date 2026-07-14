// @vitest-environment happy-dom
import { beforeEach, describe, expect, it, vi } from "vitest"

import { act, renderHook } from "./render-hook"

const { createSupportReport, hasTauriRuntime } = vi.hoisted(() => ({
  createSupportReport: vi.fn(),
  hasTauriRuntime: vi.fn(),
}))

vi.mock("@/commands/runhaven", () => ({
  createSupportReport,
  hasTauriRuntime,
}))

const { useSupportReport } = await import("../app/useSupportReport")

beforeEach(() => {
  vi.clearAllMocks()
  hasTauriRuntime.mockReturnValue(true)
})

describe("useSupportReport", () => {
  it("keeps cancellation quiet", async () => {
    createSupportReport.mockResolvedValue(null)
    const hook = await renderHook(() => useSupportReport(), undefined)

    await act(async () => hook.result.current.create())

    expect(hook.result.current.message).toBe("")
    expect(hook.result.current.error).toBe("")
    await hook.unmount()
  })

  it("explains the explicit review and submit step after success", async () => {
    createSupportReport.mockResolvedValue({
      fileName: "RunHaven-support.txt",
      issueOpened: true,
      finderRevealed: true,
    })
    const hook = await renderHook(() => useSupportReport(), undefined)

    await act(async () => hook.result.current.create())

    expect(hook.result.current.message).toContain("Review the selected file in Finder")
    expect(hook.result.current.message).toContain("GitHub uploads it when you attach it")
    expect(hook.result.current.message).toContain("You decide whether to submit the issue")
    await hook.unmount()
  })

  it("does not mutate from the browser preview", async () => {
    hasTauriRuntime.mockReturnValue(false)
    const hook = await renderHook(() => useSupportReport(), undefined)

    await act(async () => hook.result.current.create())

    expect(createSupportReport).not.toHaveBeenCalled()
    expect(hook.result.current.error).toContain("desktop app")
    await hook.unmount()
  })

  it("coalesces rapid duplicate requests before React rerenders", async () => {
    let resolve: (value: null) => void = () => undefined
    createSupportReport.mockImplementation(
      () => new Promise<null>((done) => (resolve = done)),
    )
    const hook = await renderHook(() => useSupportReport(), undefined)

    await act(async () => {
      const first = hook.result.current.create()
      const duplicate = hook.result.current.create()
      expect(createSupportReport).toHaveBeenCalledTimes(1)
      resolve(null)
      await Promise.all([first, duplicate])
    })

    expect(hook.result.current.busy).toBe(false)
    await hook.unmount()
  })
})
