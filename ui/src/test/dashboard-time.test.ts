import { describe, expect, it } from "vitest"

import { lastCheckLabel, timeAgo } from "../app/dashboard/timeAgo"

const base = Date.parse("2026-07-04T18:00:00Z")

describe("lastCheckLabel", () => {
  it("counts seconds while recent, then minutes, then hours", () => {
    expect(lastCheckLabel("2026-07-04T18:00:00Z", base + 3_000)).toBe("just now")
    expect(lastCheckLabel("2026-07-04T18:00:00Z", base + 20_000)).toBe("20s ago")
    expect(lastCheckLabel("2026-07-04T18:00:00Z", base + 5 * 60_000)).toBe("5m ago")
    expect(lastCheckLabel("2026-07-04T18:00:00Z", base + 3 * 3_600_000)).toBe("3h ago")
  })

  it("clamps a future timestamp to just now and rejects a bad one", () => {
    expect(lastCheckLabel("2026-07-04T18:00:00Z", base - 5_000)).toBe("just now")
    expect(lastCheckLabel("not-a-date")).toBe("")
  })
})

describe("timeAgo", () => {
  it("is minute-granular for run rows and em-dashes a bad value", () => {
    expect(timeAgo("2026-07-04T18:00:00Z", base + 2 * 60_000)).toBe("2m ago")
    expect(timeAgo("2026-07-04T18:00:00Z", base + 3 * 3_600_000)).toBe("3h ago")
    expect(timeAgo("nope")).toBe("—")
  })
})
