import { renderToStaticMarkup } from "react-dom/server"
import { describe, expect, it } from "vitest"

import { SessionView } from "../app/views/SessionView"

import { mainViewProps, managedSession } from "./main-view-props"

describe("SessionView", () => {
  it("keeps the live composer primary and removes the duplicate actions card", () => {
    const session = managedSession()
    const html = renderToStaticMarkup(
      <SessionView
        {...mainViewProps({ selectedSession: session })}
        selectedSession={session}
      />,
    )

    expect(html).not.toContain("Session actions")
    expect(html).not.toContain("RunHaven asks for confirmation")
    expect(html).not.toContain("Selected run overview.")
    expect(html).toContain("Message")
    expect(html).toContain("Show details")
    expect(html).not.toContain("Latest output")
    expect(html.indexOf("Message")).toBeLessThan(html.indexOf("Show details"))
  })

  it("leads with the attention banner when the session needs you", () => {
    const session = managedSession({ lifecycle: "failed" })
    const html = renderToStaticMarkup(
      <SessionView
        {...mainViewProps({ selectedSession: session })}
        selectedSession={session}
      />,
    )

    expect(html).toContain("Session ended")
    expect(html).toContain("Show output")
    expect(html).toContain("Check System")
    expect(html.indexOf("Session ended")).toBeLessThan(
      html.indexOf("Show details"),
    )
  })

  it("keeps the terminal inside the single Details disclosure", () => {
    const disclosure = "RunHaven shows this live session without reading"

    const running = managedSession()
    const runningHtml = renderToStaticMarkup(
      <SessionView
        {...mainViewProps({ selectedSession: running })}
        selectedSession={running}
      />,
    )
    expect(runningHtml).toContain("Show details")
    expect(runningHtml).not.toContain("Advanced tools")
    expect(runningHtml).not.toContain(disclosure)

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

  it("names a session action failure without control jargon", () => {
    const session = managedSession()
    const html = renderToStaticMarkup(
      <SessionView
        {...mainViewProps({
          selectedSession: session,
          controlError: "Nothing changed.",
        })}
        selectedSession={session}
      />,
    )

    expect(html).toContain("Couldn’t update this session")
    expect(html).not.toContain("Session control failed")
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
    expect(html).not.toContain("Latest output")
    expect(html).not.toContain("Runtime status")
    expect(html).not.toContain("Session diagnostics")
  })
})
