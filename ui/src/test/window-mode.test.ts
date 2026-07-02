// @vitest-environment happy-dom
import { afterEach, describe, expect, it } from "vitest"

import {
  isSessionMonitorWindow,
  monitorPreviewSessionId,
} from "../app/windowMode"

afterEach(() => {
  window.history.replaceState(null, "", "/")
})

describe("isSessionMonitorWindow", () => {
  it("is false for the main window URL", () => {
    expect(isSessionMonitorWindow()).toBe(false)
  })

  it("is true when Rust opened the window with the monitor flag", () => {
    window.history.replaceState(null, "", "/?monitor=1")
    expect(isSessionMonitorWindow()).toBe(true)
  })

  it("ignores unrelated query parameters", () => {
    window.history.replaceState(null, "", "/?other=1")
    expect(isSessionMonitorWindow()).toBe(false)
  })
})

describe("monitorPreviewSessionId", () => {
  it("returns a valid opaque token", () => {
    window.history.replaceState(null, "", "/?monitor=1&session=run_ABC-123")
    expect(monitorPreviewSessionId()).toBe("run_ABC-123")
  })

  it("rejects path-, markup-, and secret-shaped values", () => {
    for (const bad of [
      "../../etc/passwd",
      "has space",
      "<img%20src=x>",
      "a".repeat(65),
      "",
    ]) {
      window.history.replaceState(
        null,
        "",
        `/?monitor=1&session=${encodeURIComponent(bad)}`,
      )
      expect(monitorPreviewSessionId(), bad).toBeNull()
    }
  })

  it("is null without the parameter", () => {
    window.history.replaceState(null, "", "/?monitor=1")
    expect(monitorPreviewSessionId()).toBeNull()
  })
})
