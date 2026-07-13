import { renderToStaticMarkup } from "react-dom/server"
import { describe, expect, it } from "vitest"

import { SessionView } from "../app/views/SessionView"

import { mainViewProps, managedSession } from "./main-view-props"

describe("SessionView", () => {
  it("orders a running session: controls near the top, output, then composer", () => {
    const session = managedSession()
    const html = renderToStaticMarkup(
      <SessionView
        {...mainViewProps({ selectedSession: session })}
        selectedSession={session}
      />,
    )

    // No banner for a quiet session; controls, status header, output, and
    // composer all render, with controls before the output region.
    expect(html).toContain("Run actions")
    expect(html).toContain("Selected run overview.")
    expect(html).toContain("Latest output")
    expect(html).toContain("Message")
    expect(html.indexOf("Run actions")).toBeLessThan(
      html.indexOf("Latest output"),
    )
    expect(html.indexOf("Latest output")).toBeLessThan(html.indexOf("Message"))
  })

  it("leads with the attention banner when the session needs you", () => {
    const session = managedSession({ lifecycle: "failed" })
    const html = renderToStaticMarkup(
      <SessionView
        {...mainViewProps({ selectedSession: session })}
        selectedSession={session}
      />,
    )

    expect(html).toContain("This run failed")
    expect(html.indexOf("This run failed")).toBeLessThan(
      html.indexOf("Selected run overview."),
    )
  })

  it("keeps the terminal in Advanced tools and only while running", () => {
    const disclosure = "RunHaven does not filter, read, store, or log it"

    const running = managedSession()
    const runningHtml = renderToStaticMarkup(
      <SessionView
        {...mainViewProps({ selectedSession: running })}
        selectedSession={running}
      />,
    )
    expect(runningHtml).toContain("Advanced tools")
    expect(runningHtml).toContain(disclosure)

    const finished = managedSession({ lifecycle: "finished" })
    const finishedHtml = renderToStaticMarkup(
      <SessionView
        {...mainViewProps({ selectedSession: finished })}
        selectedSession={finished}
      />,
    )
    expect(finishedHtml).not.toContain(disclosure)
  })

  it("does not offer Message or Terminal after a run ends", () => {
    for (const lifecycle of ["failed", "finished"] as const) {
      const session = managedSession({ lifecycle })
      const html = renderToStaticMarkup(
        <SessionView
          {...mainViewProps({ selectedSession: session })}
          selectedSession={session}
        />,
      )
      expect(html).not.toContain(">Message<")
      expect(html).not.toContain("Attach terminal")
    }
  })

  it("keeps runtime and diagnostics detail behind a closed Details disclosure", () => {
    const session = managedSession()
    const html = renderToStaticMarkup(
      <SessionView
        {...mainViewProps({ selectedSession: session })}
        selectedSession={session}
      />,
    )

    expect(html).toContain("Show details")
    expect(html).not.toContain("Runtime status")
    expect(html).not.toContain("Session diagnostics")
  })
})
