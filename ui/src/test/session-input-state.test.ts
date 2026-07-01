import { describe, expect, it } from "vitest"

import {
  selectedSessionInputState,
  updateSessionInputState,
} from "../app/useSessionInputState"

describe("session input state", () => {
  it("keeps drafts isolated by managed session id", () => {
    const alpha = updateSessionInputState({}, "session-alpha", {
      draft: "alpha draft",
    })
    const both = updateSessionInputState(alpha, "session-beta", {
      draft: "beta draft",
    })

    expect(selectedSessionInputState(both, "session-alpha").draft).toBe(
      "alpha draft",
    )
    expect(selectedSessionInputState(both, "session-beta").draft).toBe(
      "beta draft",
    )
  })

  it("keeps in-flight send state on the originating session", () => {
    const sending = updateSessionInputState({}, "session-alpha", {
      draft: "hello",
      sending: true,
    })
    const withOtherSelection = updateSessionInputState(sending, "session-beta", {
      draft: "other",
    })
    const completed = updateSessionInputState(withOtherSelection, "session-alpha", {
      draft: "",
      sending: false,
      acceptedMessage: "Sent 5 bytes.",
    })

    expect(selectedSessionInputState(withOtherSelection, "session-alpha")).toMatchObject(
      {
        draft: "hello",
        sending: true,
        acceptedMessage: "",
      },
    )
    expect(selectedSessionInputState(completed, "session-alpha")).toMatchObject({
      draft: "",
      sending: false,
      acceptedMessage: "Sent 5 bytes.",
    })
    expect(selectedSessionInputState(completed, "session-beta").draft).toBe(
      "other",
    )
  })

  it("keeps per-session send feedback isolated", () => {
    const withAccepted = updateSessionInputState({}, "session-alpha", {
      acceptedMessage: "Sent 5 bytes.",
    })
    const withError = updateSessionInputState(withAccepted, "session-beta", {
      error: "Message not sent",
      sending: true,
    })

    expect(selectedSessionInputState(withError, "session-alpha")).toMatchObject({
      acceptedMessage: "Sent 5 bytes.",
      error: "",
      sending: false,
    })
    expect(selectedSessionInputState(withError, "session-beta")).toMatchObject({
      acceptedMessage: "",
      error: "Message not sent",
      sending: true,
    })
  })

  it("preserves record identity for no-op session input refreshes", () => {
    const current = updateSessionInputState({}, "session-alpha", {
      draft: "hello",
      sending: true,
      error: "",
      acceptedMessage: "",
    })

    expect(
      updateSessionInputState(current, "session-alpha", {
        draft: "hello",
        sending: true,
      }),
    ).toBe(current)
  })

  it("returns an empty view when no session is selected", () => {
    expect(selectedSessionInputState({}, null)).toEqual({
      draft: "",
      sending: false,
      error: "",
      acceptedMessage: "",
    })
  })
})
