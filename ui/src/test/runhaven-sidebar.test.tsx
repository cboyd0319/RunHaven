import { renderToStaticMarkup } from "react-dom/server"
import { describe, expect, it } from "vitest"

import { RunHavenSidebar } from "../app/RunHavenSidebar"
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

describe("RunHavenSidebar", () => {
  it("shows distinct session identity in the session picker", () => {
    const html = renderToStaticMarkup(
      <TooltipProvider>
        <SidebarProvider>
          <RunHavenSidebar
            sessions={[
              session("alpha"),
              session("beta", {
                agent: "codex",
                session: "beta",
                title: "Codex - beta",
                project: "beta",
                authStatus: "ready",
                authScope: "project",
                network: "internet",
                input: {
                  attached: false,
                  status: "detached",
                  detail: "Input is not attached for this session.",
                },
                statusLabel: "Needs auth",
              }),
            ]}
            selectedSessionId="beta"
            onStartSelect={() => undefined}
            onSessionSelect={() => undefined}
          />
        </SidebarProvider>
      </TooltipProvider>,
    )

    expect(html).toContain("Claude - alpha")
    expect(html).toContain("Codex - beta")
    expect(html).toContain("API key broker")
    expect(html).toContain("Shared agent login")
    expect(html).toContain("Project")
    expect(html).toContain("Provider only")
    expect(html).toContain("Internet")
    expect(html).toContain("Input ready")
    expect(html).toContain("Input blocked")
  })
})
