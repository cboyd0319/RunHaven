// @vitest-environment happy-dom
import { describe, expect, it, vi } from "vitest"

import { installSystemAppearance } from "../app/systemAppearance"

describe("system appearance", () => {
  it("follows the Mac color scheme and keeps following changes", () => {
    let listener: ((event: { matches: boolean }) => void) | undefined
    const query = {
      matches: false,
      addEventListener: vi.fn((_name: string, next: typeof listener) => {
        listener = next
      }),
      removeEventListener: vi.fn(),
    }

    const remove = installSystemAppearance(document.documentElement, query)
    expect(document.documentElement.classList.contains("dark")).toBe(false)

    listener?.({ matches: true })
    expect(document.documentElement.classList.contains("dark")).toBe(true)

    remove()
    expect(query.removeEventListener).toHaveBeenCalledTimes(1)
  })
})
