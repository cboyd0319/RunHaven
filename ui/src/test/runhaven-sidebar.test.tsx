import { renderToStaticMarkup } from "react-dom/server"
import { describe, expect, it } from "vitest"

import { RunHavenSidebar } from "../app/RunHavenSidebar"
import type { NavDestination } from "../app/navigation"
import { SidebarProvider } from "../components/ui/sidebar"
import { TooltipProvider } from "../components/ui/tooltip"
import type { ManagedSessionSummary } from "../commands/runhaven"

function session(
  id: string,
  patch: Partial<ManagedSessionSummary> = {},
): ManagedSessionSummary {
  return {
    id,
    runId: `run-${id}`,
    agent: "claude",
    session: "alpha",
    title: "Claude - alpha",
    project: "alpha",
    status: "running",
    statusLabel: "Running",
    authStatus: "api-key-broker",
    authScope: "agent",
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

function renderSidebar(
  sessions: ManagedSessionSummary[],
  navigation: NavDestination,
  runTerminalActive = false,
): string {
  return renderToStaticMarkup(
    <TooltipProvider>
      <SidebarProvider>
        <RunHavenSidebar
          sessions={sessions}
          navigation={navigation}
          onNavigate={() => undefined}
          runTerminalActive={runTerminalActive}
        />
      </SidebarProvider>
    </TooltipProvider>,
  )
}

describe("RunHavenSidebar", () => {
  it("shows Home first, lifecycle groups newest-first, then System", () => {
    const html = renderSidebar(
      [
        session("older", {
          title: "Claude - older",
          startedAt: "2026-07-01T01:00:00Z",
        }),
        session("newer", {
          agent: "codex",
          title: "Codex - newer",
          startedAt: "2026-07-01T03:00:00Z",
        }),
        session("done", {
          title: "Claude - done",
          lifecycle: "finished",
          statusLabel: "Finished",
          startedAt: "2026-07-01T02:00:00Z",
        }),
      ],
      { kind: "home" },
    )

    // Lifecycle taxonomy replaces the old per-agent grouping.
    expect(html).not.toContain("Sessions (")
    expect(html).not.toContain('aria-label="Claude sessions"')
    expect(html).toContain('aria-label="Active sessions"')
    expect(html).toContain('aria-label="Finished sessions"')

    const home = html.indexOf(">Home<")
    const active = html.indexOf(">Active<")
    const newer = html.indexOf("Codex - newer")
    const older = html.indexOf("Claude - older")
    const finished = html.indexOf(">Finished<")
    const done = html.indexOf("Claude - done")
    const system = html.indexOf(">System<")
    const agents = html.indexOf(">Agents<")
    const diagnostics = html.indexOf(">Diagnostics<")
    const maintenance = html.indexOf(">Maintenance<")
    expect(home).toBeGreaterThanOrEqual(0)
    // Home first, Active (newest first), Finished, then System last.
    expect(active).toBeGreaterThan(home)
    expect(newer).toBeGreaterThan(active)
    expect(older).toBeGreaterThan(newer)
    expect(finished).toBeGreaterThan(older)
    expect(done).toBeGreaterThan(finished)
    expect(system).toBeGreaterThan(done)
    expect(agents).toBeGreaterThan(system)
    expect(diagnostics).toBeGreaterThan(agents)
    expect(maintenance).toBeGreaterThan(diagnostics)

    // Home is the selected destination.
    expect(html.match(/aria-current="page"/g) ?? []).toHaveLength(1)
    expect(html.match(/data-active="true"/g) ?? []).toHaveLength(1)
  })

  it("shows a Run terminal handle only while a run terminal is live", () => {
    // A live interactive run is not a managed session, so without this fixed row
    // navigating away would leave no way back to it (audit RT-1/RT-2).
    expect(renderSidebar([], { kind: "home" }, false)).not.toContain(
      ">Run terminal<",
    )

    const live = renderSidebar([], { kind: "run-terminal" }, true)
    expect(live).toContain(">Run terminal<")
    // It sits right after Home and is the selected destination.
    expect(live.indexOf(">Run terminal<")).toBeGreaterThan(live.indexOf(">Home<"))
    expect(live.match(/aria-current="page"/g) ?? []).toHaveLength(1)
    const current = live.indexOf('aria-current="page"')
    expect(live.indexOf(">Run terminal<")).toBeGreaterThan(current)
  })

  it("hides empty session groups and marks the selected System destination", () => {
    const html = renderSidebar([], { kind: "diagnostics" })

    expect(html).not.toContain('aria-label="Active sessions"')
    expect(html).not.toContain('aria-label="Finished sessions"')
    expect(html).toContain(">Home<")
    expect(html).toContain(">Agents<")
    expect(html).toContain(">Maintenance<")
    // Exactly one selection: the Diagnostics destination.
    expect(html.match(/aria-current="page"/g) ?? []).toHaveLength(1)
    const currentIndex = html.indexOf('aria-current="page"')
    expect(html.indexOf(">Diagnostics<")).toBeGreaterThan(currentIndex)
  })

  it("shows one prioritized attention badge per row that needs you", () => {
    const html = renderSidebar(
      [
        session("broken", {
          title: "Claude - broken",
          lifecycle: "failed",
          statusLabel: "Failed",
          startedAt: "2026-07-01T04:00:00Z",
        }),
        session("auth", {
          agent: "codex",
          title: "Codex - auth",
          startedAt: "2026-07-01T03:00:00Z",
          nextAction: {
            kind: "auth_attention",
            label: "Finish auth",
            detail: "Sign in to continue.",
          },
        }),
        session("review", {
          title: "Claude - review",
          lifecycle: "finished",
          statusLabel: "Finished",
          startedAt: "2026-07-01T02:00:00Z",
          cleanup: {
            available: true,
            target: "worktree",
            label: "Discard saved copy",
            detail: "This session saved an isolated copy.",
          },
        }),
        session("calm", {
          title: "Claude - calm",
          startedAt: "2026-07-01T01:00:00Z",
        }),
      ],
      { kind: "home" },
    )

    // One symbol-in-dot badge per needy row, none for the calm one, and the
    // paired status text names the state (never color or symbol alone).
    expect(html).toContain('data-attention="failed"')
    expect(html).toContain('data-attention="needs-sign-in"')
    expect(html).toContain('data-attention="ready-to-review"')
    expect(html.match(/data-attention=/g) ?? []).toHaveLength(3)
    expect(html).toContain("Needs sign-in")
    expect(html).toContain("Ready to review")
    // Badges carry states, never counts.
    expect(html).not.toContain(">3<")
  })

  it("shows only the primary label and one status line per row", () => {
    const html = renderSidebar(
      [
        session("alpha", {
          session: "planning",
          title: "Claude - docs",
          project: "docs",
          containerName: "runhaven-codex-hidden-home",
          detailCommand:
            "runhaven runs show /Users/c/private/docs --env TOKEN=value",
        }),
        session("beta", {
          agent: "claude",
          session: "review",
          title: "Claude - review",
          project: "/Users/c/private/docs",
          authStatus: "ready",
          authScope: "project",
          network: "internal",
          input: {
            attached: false,
            status: "detached",
            detail: "Input is not attached for this session.",
          },
          statusLabel: "Waiting",
        }),
        session("gamma", {
          agent: "codex",
          session: "gamma",
          title: "Codex - gamma",
          project: "sandbox",
          statusLabel: "Running",
        }),
      ],
      { kind: "session", id: "beta" },
    )

    // Spec section 2 row anatomy: primary label plus a single status line.
    expect(html).toContain("Claude - docs")
    expect(html).toContain("Claude - review")
    expect(html).toContain("Codex - gamma")
    expect(html).toContain("Waiting")
    expect(html).toContain("Running")
    // The auth, scope, network, input, and identity detail no longer lives in
    // the row; it stays in the session view and its Details disclosure.
    expect(html).not.toContain("Project docs / Session")
    expect(html).not.toContain("API key broker")
    expect(html).not.toContain("Shared agent login")
    expect(html).not.toContain("Provider only")
    expect(html).not.toContain("Local only")
    expect(html).not.toContain("Input ready")
    expect(html).not.toContain("Input blocked")
    // One selection, and no secret ever reaches a row.
    expect(html.match(/data-active="true"/g) ?? []).toHaveLength(1)
    expect(html).toContain('aria-current="page"')
    expect(html).not.toContain("/Users/c/private")
    expect(html).not.toContain("TOKEN=value")
    expect(html).not.toContain("runhaven-codex-hidden-home")
  })

  it("keeps rows secret-safe when auth values carry sensitive text", () => {
    const html = renderSidebar(
      [
        session("signin", {
          agent: "codex",
          session: "signin",
          title: "Codex - signin",
          project: "signin",
          authStatus: "needs-sign-in",
          statusLabel: "Needs auth",
        }),
        session("unsafe", {
          agent: "gemini",
          session: "unsafe",
          title: "Gemini - unsafe",
          project: "unsafe",
          authStatus: "/Users/c/private/TOKEN=value",
          detailCommand:
            "runhaven runs show /Users/c/private/unsafe --env TOKEN=value",
        }),
      ],
      { kind: "session", id: "signin" },
    )

    // The attention status names the state; raw auth values never render.
    expect(html).toContain("Codex - signin")
    expect(html).toContain("Needs sign-in")
    expect(html).not.toContain("/Users/c/private")
    expect(html).not.toContain("TOKEN=value")
  })
})
