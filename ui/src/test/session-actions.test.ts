import { describe, expect, it } from "vitest"

import { sessionActionFailureMessage } from "../app/sessionActions"

describe("session action feedback", () => {
  it("treats canceling optional cleanup as a silent no-op", () => {
    expect(
      sessionActionFailureMessage(
        "cleanup",
        new Error("Cleanup canceled. Nothing was deleted."),
      ),
    ).toBe("")
  })

  it("keeps real cleanup failures visible", () => {
    expect(
      sessionActionFailureMessage(
        "cleanup",
        new Error("The protected copy could not be verified."),
      ),
    ).toBe("The protected copy could not be verified.")
  })
})
