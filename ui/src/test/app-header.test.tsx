import { renderToStaticMarkup } from "react-dom/server"
import { describe, expect, it } from "vitest"

import { AppHeader } from "../app/AppHeader"
import { SidebarProvider } from "../components/ui/sidebar"

describe("AppHeader", () => {
  it("keeps global setup status out of the title bar", () => {
    const html = renderToStaticMarkup(
      <SidebarProvider>
        <AppHeader
          title="Home"
          description="Start a run."
          loading={false}
          onRefresh={() => undefined}
        />
      </SidebarProvider>,
    )

    expect(html).toContain("Home")
    expect(html).not.toContain(">Ready<")
    expect(html).not.toContain(">Blocked<")
  })
})
