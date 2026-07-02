// @vitest-environment happy-dom
import { beforeEach, describe, expect, it, vi } from "vitest"

import type { ManagedSessionSummary } from "../commands/runhaven"
import {
  attentionBadgeCount,
  useAttentionBadge,
} from "../app/useAttentionBadge"

import { renderHook } from "./render-hook"

vi.mock("@/commands/runhaven", async (importActual) => {
  const actual = await importActual<typeof import("@/commands/runhaven")>()
  return { ...actual, setAttentionBadge: vi.fn(async () => undefined) }
})

const { setAttentionBadge } = await import("@/commands/runhaven")
const setBadge = vi.mocked(setAttentionBadge)

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

const failed = (id: string) => session(id, { lifecycle: "failed" })

describe("attentionBadgeCount", () => {
  it("counts exactly the sessions in a needs-you state", () => {
    expect(attentionBadgeCount([])).toBe(0)
    expect(
      attentionBadgeCount([
        session("calm"),
        failed("broken"),
        session("waiting", {
          nextAction: {
            kind: "auth_attention",
            label: "Sign in",
            detail: "This agent needs a sign-in.",
          },
        }),
      ]),
    ).toBe(2)
  })
})

type BadgeProps = { sessions: ManagedSessionSummary[]; enabled: boolean }

async function renderBadge(sessions: ManagedSessionSummary[], enabled = true) {
  return renderHook(
    (props: BadgeProps) => useAttentionBadge(props.sessions, props.enabled),
    { sessions, enabled },
  )
}

describe("useAttentionBadge", () => {
  beforeEach(() => {
    setBadge.mockClear()
  })

  it("reports the needs-you count and clears it back to zero", async () => {
    const hook = await renderBadge([failed("broken")])
    expect(setBadge).toHaveBeenLastCalledWith(1)

    await hook.rerender({ sessions: [session("recovered")], enabled: true })
    expect(setBadge).toHaveBeenLastCalledWith(0)
  })

  it("does not resend an unchanged count on a poll", async () => {
    const hook = await renderBadge([failed("broken")])
    const calls = setBadge.mock.calls.length

    // Same count from a fresh list identity, as a background poll produces.
    await hook.rerender({ sessions: [failed("broken")], enabled: true })
    expect(setBadge.mock.calls.length).toBe(calls)
  })

  it("clamps the reported count to what the Dock can show", async () => {
    const many = Array.from({ length: 120 }, (_, index) =>
      failed(`broken-${index}`),
    )
    await renderBadge(many)
    expect(setBadge).toHaveBeenLastCalledWith(99)
  })

  it("clears the badge while the Settings toggle is off, then restores", async () => {
    const hook = await renderBadge([failed("broken")], false)
    // Disabled reports 0 (which clears the Dock number) and nothing else,
    // even while sessions need attention.
    expect(setBadge).toHaveBeenLastCalledWith(0)
    const calls = setBadge.mock.calls.length

    await hook.rerender({
      sessions: [failed("broken"), failed("also-broken")],
      enabled: false,
    })
    expect(setBadge.mock.calls.length).toBe(calls)

    await hook.rerender({
      sessions: [failed("broken"), failed("also-broken")],
      enabled: true,
    })
    expect(setBadge).toHaveBeenLastCalledWith(2)
  })
})
