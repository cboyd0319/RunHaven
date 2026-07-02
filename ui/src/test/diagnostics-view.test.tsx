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
    diagnostics: null,
    diagnosticsLoading: false,
    diagnosticsError: "",
    onLoadDiagnostics: async () => {},
    onRefresh: () => {},
    ...patch,
  }
}

describe("DiagnosticsView", () => {
  it("shows full doctor detail: every setup check with its result", () => {
    const html = renderToStaticMarkup(
      <DiagnosticsView {...viewProps({ dashboard: dashboard(false) })} />,
    )

    expect(html).toContain("Setup checks")
    expect(html).toContain("Container CLI")
    expect(html).toContain("Needs attention")
    expect(html).toContain("Apple container is not installed.")
    expect(html).toContain("Install Apple container.")
    expect(html).toContain("Check again")
    expect(html).toContain("Trust checks")
  })

  it("shows the read-only system panels: health rollup, versions, images, networks", () => {
    const healthy = renderToStaticMarkup(<DiagnosticsView {...viewProps()} />)
    expect(healthy).toContain("Your Mac passes every setup check.")
    expect(healthy).toContain("Versions and pins")
    expect(healthy).toContain("RunHaven app")
    expect(healthy).toContain("Agent images")
    expect(healthy).toContain("Stale networks")

    const blocked = renderToStaticMarkup(
      <DiagnosticsView {...viewProps({ dashboard: dashboard(false) })} />,
    )
    expect(blocked).toContain("1 setup check needs attention.")
  })

  it("marks passing checks and shows no remedy for them", () => {
    const html = renderToStaticMarkup(<DiagnosticsView {...viewProps()} />)

    expect(html).toContain("Passed")
    expect(html).not.toContain("Next step")
  })

  it("shows a real empty state when no checks are known yet", () => {
    const html = renderToStaticMarkup(
      <DiagnosticsView {...viewProps({ dashboard: null })} />,
    )

    expect(html).toContain("No setup checks recorded yet.")
  })

  it("loads trust data on entry and re-checks everything from Check again", async () => {
    let loads = 0
    let refreshes = 0
    const props = viewProps({
      onLoadDiagnostics: async () => {
        loads += 1
      },
      onRefresh: () => {
        refreshes += 1
      },
    })

    const container = document.createElement("div")
    const root = createRoot(container)
    await act(async () => {
      root.render(createElement(DiagnosticsView, props))
    })
    // Read-only surface: it loads by itself when opened.
    expect(loads).toBe(1)
    expect(refreshes).toBe(0)

    const checkAgain = Array.from(container.querySelectorAll("button")).find(
      (button) => button.textContent?.includes("Check again"),
    )
    expect(checkAgain).toBeDefined()
    await act(async () => {
      checkAgain?.click()
    })
    expect(loads).toBe(2)
    expect(refreshes).toBe(1)

    await act(async () => {
      root.unmount()
    })
  })
})
