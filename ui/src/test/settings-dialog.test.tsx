// @vitest-environment happy-dom
import { beforeEach, describe, expect, it } from "vitest"

import { useSettings } from "../app/useSettings"

import { installLocalStorageStub } from "./local-storage"
import { act, renderHook } from "./render-hook"

installLocalStorageStub()

describe("useSettings", () => {
  beforeEach(() => {
    window.localStorage.clear()
  })

  it("starts with the app icon count enabled", async () => {
    const { result } = await renderHook(() => useSettings(), undefined)
    expect(result.current.dockBadge).toBe(true)
  })

  it("persists each choice so a fresh mount reads it back", async () => {
    const first = await renderHook(() => useSettings(), undefined)
    await act(async () => {
      first.result.current.setDockBadge(false)
    })
    expect(first.result.current.dockBadge).toBe(false)
    await first.unmount()

    const second = await renderHook(() => useSettings(), undefined)
    expect(second.result.current.dockBadge).toBe(false)
  })
})
