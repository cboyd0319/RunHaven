import { describe, expect, it } from "vitest"

import {
  groupSessions,
  navigationAfterSessionRemoval,
  sessionGroupFor,
} from "../app/navigation"
import type { ManagedSessionSummary } from "../commands/runhaven"

function session(
  id: string,
  patch: Partial<ManagedSessionSummary> = {},
): ManagedSessionSummary {
  return {
    id,
    runId: `run-${id}`,
    agent: "codex",
    session: id,
    title: `Codex - ${id}`,
    project: id,
    status: "running",
    statusLabel: "Running",
    authStatus: "ready",
    authScope: "project",
    network: "provider",
    startedAt: "2026-07-01T01:00:00Z",
    containerName: `runhaven-${id}`,
    detailCommand: `runhaven runs show run-${id}`,
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

describe("sessionGroupFor", () => {
  it("treats starting, running, and stop/kill-requested as active", () => {
    for (const lifecycle of [
      "starting",
      "running",
      "stop-requested",
      "kill-requested",
    ]) {
      expect(sessionGroupFor(session("a", { lifecycle }))).toBe("active")
    }
  })

  it("puts finished, failed, and unknown lifecycles in finished", () => {
    for (const lifecycle of ["finished", "failed", "removed", "cleaned-up", "mystery"]) {
      expect(sessionGroupFor(session("a", { lifecycle }))).toBe("finished")
    }
  })
})

describe("groupSessions", () => {
  it("orders each group newest-first by startedAt with a stable id tiebreak", () => {
    const groups = groupSessions([
      session("old-active", { startedAt: "2026-07-01T01:00:00Z" }),
      session("new-active", { startedAt: "2026-07-01T03:00:00Z" }),
      session("tie-2", { startedAt: "2026-07-01T02:00:00Z" }),
      session("tie-10", { startedAt: "2026-07-01T02:00:00Z" }),
      session("done", { lifecycle: "failed", startedAt: "2026-07-01T02:30:00Z" }),
    ])

    expect(groups.active.map((item) => item.id)).toEqual([
      "new-active",
      "tie-10",
      "tie-2",
      "old-active",
    ])
    expect(groups.finished.map((item) => item.id)).toEqual(["done"])
  })

  it("does not reorder rows on a status flap within the same lifecycle group", () => {
    const before = groupSessions([
      session("a", { startedAt: "2026-07-01T02:00:00Z" }),
      session("b", { startedAt: "2026-07-01T01:00:00Z" }),
    ])
    const after = groupSessions([
      session("a", {
        startedAt: "2026-07-01T02:00:00Z",
        lifecycle: "stop-requested",
        statusLabel: "Stop requested",
      }),
      session("b", { startedAt: "2026-07-01T01:00:00Z" }),
    ])

    expect(after.active.map((item) => item.id)).toEqual(
      before.active.map((item) => item.id),
    )
  })
})

describe("navigationAfterSessionRemoval", () => {
  const active = [
    session("top", { startedAt: "2026-07-01T03:00:00Z" }),
    session("middle", { startedAt: "2026-07-01T02:00:00Z" }),
    session("bottom", { startedAt: "2026-07-01T01:00:00Z" }),
  ]

  it("selects the next row below in the same group", () => {
    const next = active.filter((item) => item.id !== "middle")
    expect(navigationAfterSessionRemoval("middle", active, next)).toEqual({
      kind: "session",
      id: "bottom",
    })
  })

  it("selects the nearest row above when nothing remains below", () => {
    const next = active.filter((item) => item.id !== "bottom")
    expect(navigationAfterSessionRemoval("bottom", active, next)).toEqual({
      kind: "session",
      id: "middle",
    })
  })

  it("falls back to the other group's first row when its group empties", () => {
    const finished = session("done", {
      lifecycle: "finished",
      startedAt: "2026-07-01T00:30:00Z",
    })
    const previous = [active[0], finished]
    expect(
      navigationAfterSessionRemoval(active[0].id, previous, [finished]),
    ).toEqual({ kind: "session", id: "done" })
  })

  it("falls back to Home when no sessions remain", () => {
    expect(navigationAfterSessionRemoval("top", [active[0]], [])).toEqual({
      kind: "home",
    })
  })

  it("falls back deliberately even when the removed row was never seen", () => {
    expect(
      navigationAfterSessionRemoval("ghost", [], [active[1]]),
    ).toEqual({ kind: "session", id: "middle" })
    expect(navigationAfterSessionRemoval("ghost", [], [])).toEqual({
      kind: "home",
    })
  })
})
