import { describe, expect, it } from "vitest"

import { errorMessage } from "../app/types"

describe("errorMessage", () => {
  it("treats native confirmation cancellation as a quiet user choice", () => {
    for (const message of [
      "Raw output view canceled.",
      "Network cleanup canceled; no networks were removed.",
      "Stop canceled before changing this session.",
      "Discard canceled; the worktree was left in place.",
    ]) {
      expect(errorMessage(new Error(message))).toBe("")
    }
  })

  it("preserves real failures", () => {
    expect(errorMessage(new Error("Container service failed."))).toBe(
      "Container service failed.",
    )
  })
})
