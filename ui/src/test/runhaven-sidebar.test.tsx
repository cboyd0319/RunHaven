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
    input: { attached: true, status: "attached", detail: "Session input is ready." },
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
          appVersion="9.9.9"
        />
      </SidebarProvider>
    </TooltipProvider>,
  )
}

describe("RunHavenSidebar", () => {
  it("shows the four destinations, the Runs summary, and the app version", () => {
    const html = renderSidebar([], { kind: "home" })

    const home = html.indexOf(">Home<")
    const agents = html.indexOf(">Agents<")
    const diagnostics = html.indexOf(">Diagnostics<")
    const maintenance = html.indexOf(">Maintenance<")
    const runs = html.indexOf(">Runs<")
    expect(home).toBeGreaterThanOrEqual(0)
    expect(agents).toBeGreaterThan(home)
    expect(diagnostics).toBeGreaterThan(agents)
    expect(maintenance).toBeGreaterThan(diagnostics)
    // The Runs summary sits below the destinations.
    expect(runs).toBeGreaterThan(maintenance)
    expect(html).toContain(">Needs review<")
    expect(html).toContain(">Active<")
    expect(html).toContain(">Finished<")
    expect(html).toContain("RunHaven 9.9.9")

    // Home is the one selected destination.
    expect(html.match(/aria-current="page"/g) ?? []).toHaveLength(1)
    expect(html.match(/data-active="true"/g) ?? []).toHaveLength(1)
  })

  it("summarizes the run pipeline as counts, never as session rows", () => {
    const html = renderSidebar(
      [
        session("run-a", { lifecycle: "running", title: "Claude - secret-alpha" }),
        session("run-b", { lifecycle: "running", title: "Codex - secret-beta" }),
        session("fin", {
          lifecycle: "finished",
          statusLabel: "Finished",
          title: "Claude - secret-gamma",
        }),
        session("bad", {
          lifecycle: "failed",
          statusLabel: "Failed",
          title: "Claude - secret-delta",
          project: "/Users/c/private/delta",
          detailCommand: "runhaven runs show --env TOKEN=value",
        }),
      ],
      { kind: "home" },
    )

    // Counts: 2 active, 2 finished (finished + failed), 1 needs review (failed).
    expect(html).toContain(">2<")
    expect(html).toContain(">1<")
    // Individual sessions are not listed in the sidebar anymore, so no session
    // title, raw path, or secret ever reaches a sidebar row.
    expect(html).not.toContain("secret-alpha")
    expect(html).not.toContain("secret-delta")
    expect(html).not.toContain("/Users/c/private")
    expect(html).not.toContain("TOKEN=value")
  })

  it("shows a Run terminal handle only while a run terminal is live", () => {
    // A live interactive run is not a managed session, so without this fixed row
    // navigating away would leave no way back to it (audit RT-1/RT-2).
    expect(renderSidebar([], { kind: "home" }, false)).not.toContain(">Run terminal<")

    const live = renderSidebar([], { kind: "run-terminal" }, true)
    expect(live).toContain(">Run terminal<")
    expect(live.indexOf(">Run terminal<")).toBeGreaterThan(live.indexOf(">Home<"))
    expect(live.match(/aria-current="page"/g) ?? []).toHaveLength(1)
  })

  it("marks the selected System destination with exactly one current", () => {
    const html = renderSidebar([], { kind: "diagnostics" })
    expect(html.match(/aria-current="page"/g) ?? []).toHaveLength(1)
    const currentIndex = html.indexOf('aria-current="page"')
    expect(html.indexOf(">Diagnostics<")).toBeGreaterThan(currentIndex)
  })
})
