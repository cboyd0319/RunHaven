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
  it("groups sessions by agent with compact counts", () => {
    const html = renderToStaticMarkup(
      <TooltipProvider>
        <SidebarProvider>
          <RunHavenSidebar
            sessions={[
              session("gamma", {
                agent: "codex",
                session: "gamma",
                title: "Codex - gamma",
                project: "sandbox",
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
              session("alpha"),
              session("beta", {
                agent: "claude",
                session: "beta",
                title: "Claude - beta",
                project: "beta",
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
            ]}
            selectedSessionId="gamma"
            onStartSelect={() => undefined}
            onSessionSelect={() => undefined}
          />
        </SidebarProvider>
      </TooltipProvider>,
    )

    expect(html).toContain("Sessions (3)")
    expect(html).toContain("Claude")
    expect(html).toContain("Codex")
    expect(html).toContain(">2<")
    expect(html).toContain(">1<")

    const claudeIndex = html.indexOf('aria-label="Claude sessions"')
    const codexIndex = html.indexOf('aria-label="Codex sessions"')
    expect(claudeIndex).toBeGreaterThanOrEqual(0)
    expect(codexIndex).toBeGreaterThan(claudeIndex)
  })

  it("keeps same-agent and mixed-agent sessions distinguishable", () => {
    const html = renderToStaticMarkup(
      <TooltipProvider>
        <SidebarProvider>
          <RunHavenSidebar
            sessions={[
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
                title: "Claude - docs",
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

    expect(html).toContain("Claude - docs")
    expect(html).toContain("Codex - gamma")
    expect(html).toContain("Project docs / Session planning")
    expect(html).toContain("Project docs / Session review")
    expect(html).toContain("Project sandbox / Session gamma")
    expect(html).toContain("API key broker")
    expect(html).toContain("Ready")
    expect(html).toContain("Shared agent login")
    expect(html).toContain("Project")
    expect(html).toContain("Provider only")
    expect(html).toContain("Local only")
    expect(html).toContain("Internet")
    expect(html).toContain("Input ready")
    expect(html).toContain("Input blocked")
    expect(html.match(/data-active="true"/g) ?? []).toHaveLength(1)
    expect(html).toContain('aria-current="page"')
    expect(html).not.toContain("/Users/c/private")
    expect(html).not.toContain("TOKEN=value")
    expect(html).not.toContain("runhaven-codex-hidden-home")
  })

  it("shows compact auth labels without leaking unsafe auth values", () => {
    const html = renderToStaticMarkup(
      <TooltipProvider>
        <SidebarProvider>
          <RunHavenSidebar
            sessions={[
              session("login", {
                session: "saved",
                title: "Claude - saved",
                project: "/Users/c/private/saved",
                authStatus: "login-state",
                detailCommand:
                  "runhaven runs show /Users/c/private/saved --env TOKEN=value",
              }),
              session("signin", {
                agent: "codex",
                session: "signin",
                title: "Codex - signin",
                project: "signin",
                authStatus: "needs-sign-in",
                statusLabel: "Needs auth",
              }),
              session("local", {
                agent: "shell",
                session: "local",
                title: "Shell - local",
                project: "local",
                authStatus: "no-auth-needed",
                network: "internal",
              }),
              session("unsafe", {
                agent: "gemini",
                session: "unsafe",
                title: "Gemini - unsafe",
                project: "unsafe",
                authStatus: "/Users/c/private/TOKEN=value",
              }),
            ]}
            selectedSessionId="login"
            onStartSelect={() => undefined}
            onSessionSelect={() => undefined}
          />
        </SidebarProvider>
      </TooltipProvider>,
    )

    expect(html).toContain("Login state")
    expect(html).toContain("Needs sign-in")
    expect(html).toContain("No auth needed")
    expect(html).toContain("Unknown")
    expect(html).toContain("Project saved")
    expect(html).not.toContain("/Users/c/private")
    expect(html).not.toContain("TOKEN=value")
  })
})
