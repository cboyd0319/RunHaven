// @vitest-environment happy-dom
import { createRoot } from "react-dom/client"
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
  it("keeps primary navigation to Home, History, and System", () => {
    const html = renderSidebar({ kind: "home" })

    const home = html.indexOf(">Home<")
    const history = html.indexOf(">History<")
    const system = html.indexOf(">System<")
    expect(home).toBeGreaterThanOrEqual(0)
    expect(history).toBeGreaterThan(home)
    expect(system).toBeGreaterThan(history)
    expect(html).not.toContain(">Agents<")
    expect(html).not.toContain(">System Check<")
    expect(html).not.toContain(">Cleanup<")
    expect(html).not.toContain(">Needs you<")
    expect(html).not.toContain(">Running<")
    expect(html).not.toContain(">Past<")
    expect(html).not.toContain(">Settings<")
    expect(html).toContain("RunHaven 9.9.9")

    // Home is the one selected destination.
    expect(html.match(/aria-current="page"/g) ?? []).toHaveLength(1)
    expect(html.match(/data-active="true"/g) ?? []).toHaveLength(1)
  })

  it("shows a Session terminal handle only while a session terminal is live", () => {
    // A live interactive run is not a managed session, so without this fixed row
    // navigating away would leave no way back to it (audit RT-1/RT-2).
    expect(renderSidebar({ kind: "home" }, false)).not.toContain(">Session terminal<")

    const live = renderSidebar({ kind: "run-terminal" }, true)
    expect(live).toContain(">Session terminal<")
    expect(live.indexOf(">Session terminal<")).toBeGreaterThan(live.indexOf(">Home<"))
    expect(live.match(/aria-current="page"/g) ?? []).toHaveLength(1)
  })

  it("marks History with exactly one current destination", () => {
    const html = renderSidebar({ kind: "runs" })
    expect(html.match(/aria-current="page"/g) ?? []).toHaveLength(1)
    const currentIndex = html.indexOf('aria-current="page"')
    expect(html.indexOf(">History<")).toBeGreaterThan(currentIndex)
  })

  it("hides the sidebar completely instead of leaving a clipped icon rail", () => {
    const html = renderSidebar({ kind: "home" }, false, false)
    expect(html).toContain('data-collapsible="offcanvas"')
    expect(html).not.toContain('data-collapsible="icon"')
  })

  it("does not add a hidden pointer rail", () => {
    const html = renderSidebar({ kind: "home" })

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
              navigation={{ kind: "home" }}
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
