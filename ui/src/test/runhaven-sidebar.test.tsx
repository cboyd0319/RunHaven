import { renderToStaticMarkup } from "react-dom/server"
import { describe, expect, it } from "vitest"

import { RunHavenSidebar } from "../app/RunHavenSidebar"
import type { NavDestination } from "../app/navigation"
import { SidebarProvider } from "../components/ui/sidebar"
import { TooltipProvider } from "../components/ui/tooltip"
function renderSidebar(
  navigation: NavDestination,
  runTerminalActive = false,
): string {
  return renderToStaticMarkup(
    <TooltipProvider>
      <SidebarProvider>
        <RunHavenSidebar
          navigation={navigation}
          onNavigate={() => undefined}
          onOpenSettings={() => undefined}
          runTerminalActive={runTerminalActive}
          appVersion="9.9.9"
        />
      </SidebarProvider>
    </TooltipProvider>,
  )
}

describe("RunHavenSidebar", () => {
  it("keeps primary navigation to Home, History, and Settings", () => {
    const html = renderSidebar({ kind: "home" })

    const home = html.indexOf(">Home<")
    const history = html.indexOf(">History<")
    expect(home).toBeGreaterThanOrEqual(0)
    expect(history).toBeGreaterThan(home)
    expect(html).not.toContain(">Agents<")
    expect(html).not.toContain(">System Check<")
    expect(html).not.toContain(">Cleanup<")
    expect(html).not.toContain(">Needs you<")
    expect(html).not.toContain(">Running<")
    expect(html).not.toContain(">Past<")
    expect(html).toContain(">Settings<")
    expect(html).toContain("RunHaven 9.9.9")

    // Home is the one selected destination.
    expect(html.match(/aria-current="page"/g) ?? []).toHaveLength(1)
    expect(html.match(/data-active="true"/g) ?? []).toHaveLength(1)
  })

  it("shows a Run terminal handle only while a run terminal is live", () => {
    // A live interactive run is not a managed session, so without this fixed row
    // navigating away would leave no way back to it (audit RT-1/RT-2).
    expect(renderSidebar({ kind: "home" }, false)).not.toContain(">Run terminal<")

    const live = renderSidebar({ kind: "run-terminal" }, true)
    expect(live).toContain(">Run terminal<")
    expect(live.indexOf(">Run terminal<")).toBeGreaterThan(live.indexOf(">Home<"))
    expect(live.match(/aria-current="page"/g) ?? []).toHaveLength(1)
  })

  it("marks History with exactly one current destination", () => {
    const html = renderSidebar({ kind: "runs" })
    expect(html.match(/aria-current="page"/g) ?? []).toHaveLength(1)
    const currentIndex = html.indexOf('aria-current="page"')
    expect(html.indexOf(">History<")).toBeGreaterThan(currentIndex)
  })
})
