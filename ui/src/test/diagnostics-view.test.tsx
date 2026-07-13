// @vitest-environment happy-dom
import { createElement } from "react"
import { createRoot } from "react-dom/client"
import { renderToStaticMarkup } from "react-dom/server"
import { describe, expect, it } from "vitest"

import { DiagnosticsView } from "../app/views/DiagnosticsView"

import { dashboard } from "./main-view-props"
import { act } from "./render-hook"

function viewProps(patch: Partial<Parameters<typeof DiagnosticsView>[0]> = {}) {
  return {
    dashboard: dashboard(true),
    refreshing: false,
    onRefresh: () => {},
    ...patch,
  }
}

describe("DiagnosticsView", () => {
  it("shows only setup problems with a direct next action", () => {
    const html = renderToStaticMarkup(
      <DiagnosticsView {...viewProps({ dashboard: dashboard(false) })} />,
    )

    expect(html).toContain("Setup problems")
    expect(html).toContain("Container CLI")
    expect(html).toContain("Apple container is not installed.")
    expect(html).toContain("Install Apple container.")
    expect(html).toContain("Check again")
    expect(html).not.toContain("Trust checks")
  })

  it("shows health, actionable cleanup, and hides technical detail by default", () => {
    const healthy = renderToStaticMarkup(<DiagnosticsView {...viewProps()} />)
    expect(healthy).toContain("Your Mac is ready for RunHaven.")
    expect(healthy).toContain("Advanced system information")
    expect(healthy).toContain("Leftover networks")
    expect(healthy).toContain("Remove stale networks")
    expect(healthy).not.toContain("Auth broker status")
    expect(healthy).not.toContain("Network trust")

    const blocked = renderToStaticMarkup(
      <DiagnosticsView {...viewProps({ dashboard: dashboard(false) })} />,
    )
    expect(blocked).toContain("1 setup problem needs your attention.")
  })

  it("hides the setup checklist when the Mac is ready", () => {
    const html = renderToStaticMarkup(<DiagnosticsView {...viewProps()} />)

    expect(html).not.toContain("Setup problems")
    expect(html).not.toContain("Container CLI")
    expect(html).not.toContain("Next step")
  })

  it("shows a real empty state when no checks are known yet", () => {
    const html = renderToStaticMarkup(
      <DiagnosticsView {...viewProps({ dashboard: null })} />,
    )

    expect(html).toContain("Setup status unavailable")
    expect(html).toContain("RunHaven could not load setup status.")
  })

  it("re-checks setup directly without a trust-log step", async () => {
    let refreshes = 0
    const props = viewProps({
      onRefresh: () => {
        refreshes += 1
      },
    })

    const container = document.createElement("div")
    const root = createRoot(container)
    await act(async () => {
      root.render(createElement(DiagnosticsView, props))
    })
    expect(refreshes).toBe(0)

    const checkAgain = Array.from(container.querySelectorAll("button")).find(
      (button) => button.textContent?.includes("Check again"),
    )
    expect(checkAgain).toBeDefined()
    await act(async () => {
      checkAgain?.click()
    })
    expect(refreshes).toBe(1)

    await act(async () => {
      root.unmount()
    })
  })
})
