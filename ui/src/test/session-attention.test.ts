import { describe, expect, it } from "vitest"

import { sessionAttention } from "../app/sessionAttention"
import type { ManagedSessionSummary } from "../commands/runhaven"

function session(
  patch: Partial<ManagedSessionSummary> = {},
): ManagedSessionSummary {
  return {
    id: "s1",
    runId: "run-s1",
    agent: "codex",
    session: "alpha",
    title: "Codex - alpha",
    project: "alpha",
    status: "running",
    statusLabel: "Running",
    authStatus: "ready",
    authScope: "project",
    network: "provider",
    startedAt: "2026-07-01T01:00:00Z",
    containerName: "runhaven-s1",
    detailCommand: "runhaven runs show run-s1",
    lifecycle: "running",
    input: {
      attached: true,
      status: "attached",
      detail: "Session input is ready.",
    },
    nextAction: {
      kind: "monitor",
      label: "Monitor session",
      detail: "RunHaven is tracking this session.",
    },
    events: [],
    ...patch,
  }
}

const worktreeCleanup = {
  available: true,
  target: "worktree",
  label: "Discard saved copy",
  detail: "This session saved an isolated copy.",
}

describe("sessionAttention", () => {
  it("returns null for a session that needs nothing", () => {
    expect(sessionAttention(session())).toBeNull()
    expect(sessionAttention(session({ lifecycle: "finished" }))).toBeNull()
  })

  it("treats an ended interactive run as neutral, not a failure", () => {
    // The in-app run terminal ends with lifecycle "ended"; it must not raise the
    // red failure signal.
    const ended = session({ lifecycle: "ended", statusLabel: "Ended" })
    expect(sessionAttention(ended)).toBeNull()
  })

  it("flags a failed session as red", () => {
    expect(sessionAttention(session({ lifecycle: "failed" }))).toEqual({
      kind: "failed",
      label: "Failed",
      tone: "failed",
    })
  })

  it("flags needs-sign-in from the next action or the auth status", () => {
    const fromAction = session({
      nextAction: { kind: "auth_attention", label: "Finish auth", detail: "" },
    })
    const fromStatus = session({ authStatus: "needs-sign-in" })
    for (const needy of [fromAction, fromStatus]) {
      expect(sessionAttention(needy)).toEqual({
        kind: "needs-sign-in",
        label: "Needs sign-in",
        tone: "needs-you",
      })
    }
  })

  it("flags a recover next action as not responding", () => {
    expect(
      sessionAttention(
        session({
          nextAction: { kind: "recover", label: "Repair", detail: "" },
        }),
      ),
    ).toEqual({
      kind: "not-responding",
      label: "Not responding",
      tone: "needs-you",
    })
  })

  it("flags a finished worktree session as ready to review with the ready tone", () => {
    // Ready to review is a positive, ready state (VIS-2), so it carries the teal
    // success tone, not warning amber.
    expect(
      sessionAttention(
        session({ lifecycle: "finished", cleanup: worktreeCleanup }),
      ),
    ).toEqual({
      kind: "ready-to-review",
      label: "Ready to review",
      tone: "ready",
    })
  })

  it("applies the spec priority order: failed, sign-in, recover, review", () => {
    const everything = session({
      lifecycle: "failed",
      authStatus: "needs-sign-in",
      nextAction: { kind: "recover", label: "Repair", detail: "" },
      cleanup: worktreeCleanup,
    })
    expect(sessionAttention(everything)?.kind).toBe("failed")

    const noFailure = session({
      authStatus: "needs-sign-in",
      nextAction: { kind: "recover", label: "Repair", detail: "" },
      cleanup: worktreeCleanup,
    })
    expect(sessionAttention(noFailure)?.kind).toBe("needs-sign-in")

    const recoverAndReview = session({
      nextAction: { kind: "recover", label: "Repair", detail: "" },
      cleanup: worktreeCleanup,
    })
    expect(sessionAttention(recoverAndReview)?.kind).toBe("not-responding")
  })
})
