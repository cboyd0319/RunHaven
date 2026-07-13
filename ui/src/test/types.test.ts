import { describe, expect, it } from "vitest"

import { errorMessage } from "../app/types"

describe("errorMessage", () => {
  it("treats native confirmation cancellation as a quiet user choice", () => {
    for (const message of [
      "Raw output view canceled.",
      "Network cleanup canceled; no networks were removed.",
      "Stop canceled before changing this session.",
      "Force Stop canceled; the session is still running.",
      "Message send canceled; nothing was sent.",
      "Apply changes canceled; the protected copy was left in place.",
      "Discard changes canceled; the protected copy was left in place.",
      "Save canceled; the protected copy was left in place.",
      "Recovery steps canceled; the protected copy was left in place.",
      "Remove canceled. This session was not changed.",
    ]) {
      expect(errorMessage(new Error(message))).toBe("")
    }
  })

  it("preserves real failures", () => {
    expect(errorMessage(new Error("Container service failed."))).toBe(
      "Container service failed.",
    )
    expect(errorMessage(new Error("Hard-stop canceled unexpectedly."))).toBe(
      "Hard-stop canceled unexpectedly.",
    )
    expect(errorMessage(new Error("Merge canceled unexpectedly."))).toBe(
      "Merge canceled unexpectedly.",
    )
    expect(errorMessage(new Error("Repair canceled unexpectedly."))).toBe(
      "Repair canceled unexpectedly.",
    )
  })
})
