import { renderToStaticMarkup } from "react-dom/server"
import { describe, expect, it } from "vitest"

import { AppHeader } from "../app/AppHeader"
import { SidebarProvider } from "../components/ui/sidebar"

import { managedSession } from "./main-view-props"

describe("AppHeader", () => {
  it("keeps global setup status out of the title bar", () => {
    const html = renderToStaticMarkup(
      <SidebarProvider>
        <AppHeader
          title="Home"
        />
      </SidebarProvider>,
    )

    expect(html).toContain("Home")
    expect(html).not.toContain(">Ready<")
    expect(html).not.toContain(">Blocked<")
    expect(html).not.toContain("Refresh status")
    expect(html).toContain('aria-label="Hide Sidebar"')
  })

  it("names the collapsed-sidebar action", () => {
    const html = renderToStaticMarkup(
      <SidebarProvider defaultOpen={false}>
        <AppHeader title="Home" />
      </SidebarProvider>,
    )

    expect(html).toContain('aria-label="Show Sidebar"')
  })

  it("uses the selected-session toolbar for status and the normal stop action", () => {
    const session = managedSession({
      title: "Codex - website",
      statusLabel: "Running",
    })
    const html = renderToStaticMarkup(
      <SidebarProvider>
        <AppHeader
          title={session.title}
          session={session}
          controlBusy={false}
          onControl={async () => undefined}
        />
      </SidebarProvider>,
    )

    expect(html).toContain("Codex - website")
    expect(html).toContain("Running")
    expect(html).toContain("Stop")
    expect(html).not.toContain("Force Stop")
    expect(html).not.toContain("Session actions")
  })
})
