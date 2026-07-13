// @vitest-environment happy-dom
import { createRoot } from "react-dom/client"
import { createElement } from "react"
import { renderToStaticMarkup } from "react-dom/server"
import { describe, expect, it } from "vitest"

import { RunHavenSidebar } from "../app/RunHavenSidebar"
import type { NavDestination } from "../app/navigation"
import { SidebarProvider, SidebarTrigger } from "../components/ui/sidebar"
import { TooltipProvider } from "../components/ui/tooltip"
import { act } from "./render-hook"
function renderSidebar(
  navigation: NavDestination,
  runTerminalActive = false,
  defaultOpen = true,
): string {
  return renderToStaticMarkup(
    <TooltipProvider>
      <SidebarProvider defaultOpen={defaultOpen}>
        <RunHavenSidebar
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
  it("keeps primary navigation to New session, History, and System", () => {
    const html = renderSidebar({ kind: "new-session" })

    const start = html.indexOf(">New session<")
    const history = html.indexOf(">History<")
    const system = html.indexOf(">System<")
    expect(start).toBeGreaterThanOrEqual(0)
    expect(history).toBeGreaterThan(start)
    expect(system).toBeGreaterThan(history)
    expect(html).not.toContain(">Home<")
    expect(html).not.toContain(">Agents<")
    expect(html).not.toContain(">System Check<")
    expect(html).not.toContain(">Cleanup<")
    expect(html).not.toContain(">Needs you<")
    expect(html).not.toContain(">Running<")
    expect(html).not.toContain(">Past<")
    expect(html).not.toContain(">Settings<")
    expect(html).toContain("RunHaven 9.9.9")
    expect(html).toContain('<nav aria-label="RunHaven navigation"')

    // New session is the one selected destination.
    expect(html.match(/aria-current="page"/g) ?? []).toHaveLength(1)
    expect(html.match(/data-active="true"/g) ?? []).toHaveLength(1)
  })

  it("does not use one ambiguous generic terminal row", () => {
    expect(renderSidebar({ kind: "new-session" }, false)).not.toContain(">Session terminal<")
    expect(renderSidebar({ kind: "run-terminal" }, true)).not.toContain(
      ">Session terminal<",
    )
  })

  it("uses the spare sidebar space for distinct running sessions and saved setups", () => {
    const html = renderToStaticMarkup(
      <TooltipProvider>
        <SidebarProvider>
          {createElement(RunHavenSidebar, {
            navigation: { kind: "new-session" },
            onNavigate: () => undefined,
            runTerminalActive: true,
            runTerminals: [
              {
                sessionId: "terminal-a",
                runId: "run-a",
                label: "Copilot · RunHaven",
              },
              {
                sessionId: "terminal-b",
                runId: "run-b",
                label: "Antigravity · website",
              },
            ],
            savedSetups: [
              { id: "setup-a", name: "Codex · RunHaven" },
            ],
            appVersion: "9.9.9",
          } as never)}
        </SidebarProvider>
      </TooltipProvider>,
    )

    expect(html).toContain(">Running<")
    expect(html).toContain("Copilot · RunHaven")
    expect(html).toContain("Antigravity · website")
    expect(html).toContain(">Saved setups<")
    expect(html).toContain("Codex · RunHaven")
    expect(html).not.toContain(">Session terminal<")
  })

  it("keeps starting terminal labels distinct without rendering a raw path", () => {
    const html = renderToStaticMarkup(
      <TooltipProvider>
        <SidebarProvider>
          <RunHavenSidebar
            navigation={{ kind: "new-session" }}
            onNavigate={() => undefined}
            runTerminals={[
              {
                sessionId: "terminal-a",
                request: {
                  plan: {
                    agent: "claude",
                    workspacePath: "/tmp/secret-project\u202ebad",
                  },
                },
              },
              {
                sessionId: "terminal-b",
                request: {
                  plan: {
                    agent: "claude",
                    workspacePath: "/tmp/secret-project\u202ebad",
                  },
                },
              },
            ]}
            appVersion="9.9.9"
          />
        </SidebarProvider>
      </TooltipProvider>,
    )

    expect(html).toContain("Claude · Starting")
    expect(html).toContain("Claude · Starting (2)")
    expect(html).not.toContain("secret-project")
  })

  it("announces unavailable setups and shows sidebar errors beside the action", () => {
    const html = renderToStaticMarkup(
      <TooltipProvider>
        <SidebarProvider>
          <RunHavenSidebar
            navigation={{ kind: "runs" }}
            onNavigate={() => undefined}
            savedSetups={[
              {
                id: "setup-a",
                name: "Codex · RunHaven",
                available: false,
              },
            ]}
            savedSetupsError="RunHaven could not update this saved setup."
            onUnpinSavedSetup={() => undefined}
            appVersion="9.9.9"
          />
        </SidebarProvider>
      </TooltipProvider>,
    )

    expect(html).toContain('aria-describedby="saved-setup-setup-a-status"')
    expect(html).toContain('aria-label="Unpin Codex · RunHaven"')
    expect(html).toContain("text-warning-text")
    expect(html).not.toContain("dark:text-")
    expect(html).not.toContain('text-xs text-warning"')
    expect(html).toContain('role="alert"')
    expect(html).toContain("RunHaven could not update this saved setup.")
  })

  it("marks History with exactly one current destination", () => {
    const html = renderSidebar({ kind: "runs" })
    expect(html.match(/aria-current="page"/g) ?? []).toHaveLength(1)
    const currentIndex = html.indexOf('aria-current="page"')
    expect(html.indexOf(">History<")).toBeGreaterThan(currentIndex)
  })

  it("hides the sidebar completely instead of leaving a clipped icon rail", () => {
    const html = renderSidebar({ kind: "new-session" }, false, false)
    expect(html).toContain('data-collapsible="offcanvas"')
    expect(html).not.toContain('data-collapsible="icon"')
  })

  it("does not add a hidden pointer rail", () => {
    const html = renderSidebar({ kind: "new-session" })

    expect(html).not.toContain("Toggle Sidebar")
    expect(html).not.toContain('data-sidebar="rail"')
  })

  it("moves focus out of the sidebar and hides it from accessibility", async () => {
    const container = document.createElement("div")
    document.body.append(container)
    const root = createRoot(container)
    await act(async () => {
      root.render(
        <TooltipProvider>
          <SidebarProvider>
            <RunHavenSidebar
              navigation={{ kind: "new-session" }}
              onNavigate={() => undefined}
              runTerminalActive={false}
              appVersion="9.9.9"
            />
            <SidebarTrigger />
          </SidebarProvider>
        </TooltipProvider>,
      )
    })
    const history = Array.from(container.querySelectorAll("button")).find(
      (button) => button.textContent === "History",
    )
    const trigger = container.querySelector<HTMLElement>(
      '[data-sidebar="trigger"]',
    )
    expect(history).toBeDefined()
    expect(trigger).not.toBeNull()
    history?.focus()

    await act(async () => trigger?.click())

    const sidebar = document.querySelector<HTMLElement>(
      '[data-slot="sidebar-container"]',
    )
    expect(document.activeElement).toBe(trigger)
    expect(sidebar?.getAttribute("aria-hidden")).toBe("true")
    expect(sidebar?.hasAttribute("inert")).toBe(true)
    await act(async () => root.unmount())
    container.remove()
  })
})
