import { renderToStaticMarkup } from "react-dom/server"
import { describe, expect, it } from "vitest"

import { SessionView } from "../app/views/SessionView"

import {
  launchFlow,
  mainViewProps,
  managedSession,
  planRequest,
} from "./main-view-props"

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
        {...mainViewProps({
          selectedSession: session,
          launchFlow: {
            ...launchFlow(),
            runTerminals: [
              {
                sessionId: "terminal-s1",
                runId: session.runId,
                startedAt: "2026-07-01T01:00:00Z",
                request: { plan: planRequest() },
              },
            ],
          },
        })}
        selectedSession={session}
      />,
    )

    expect(html).toContain("Session ended")
    expect(html).toContain("Show output")
    expect(html).toContain("Start new session…")
    expect(html).toContain("Hide issue details")
    expect(html).toContain("Session diagnostics")
    expect(html.indexOf("Session ended")).toBeLessThan(
      html.indexOf("Hide issue details"),
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

  it("shows clean protected copies as neutral optional cleanup", () => {
    const session = managedSession({
      lifecycle: "finished",
      cleanup: {
        available: true,
        reviewState: "clean",
        target: "worktree",
        label: "Delete protected copy",
        detail: "Delete this protected copy.",
      },
    })
    const html = renderToStaticMarkup(
      <SessionView
        {...mainViewProps({ selectedSession: session })}
        selectedSession={session}
      />,
    )

    expect(html).toContain("No changes to review")
    expect(html).toContain("Your original project is unchanged.")
    expect(html).toContain("Delete protected copy…")
    expect(html).not.toContain("Changes ready to review")
    expect(html).not.toContain("Apply changes")
    expect(html).not.toContain("Discard changes")
  })

  it("returns a detached interactive session to its live terminal", () => {
    const session = managedSession({
      input: {
        attached: false,
        status: "detached",
        detail: "Use the session terminal.",
      },
    })
    const html = renderToStaticMarkup(
      <SessionView
        {...mainViewProps({
          selectedSession: session,
          launchFlow: {
            ...launchFlow(),
            runTerminals: [
              {
                sessionId: "terminal-s1",
                runId: session.runId,
                startedAt: "2026-07-01T01:00:00Z",
                request: { plan: planRequest() },
              },
            ],
          },
        })}
        selectedSession={session}
      />,
    )

    expect(html).toContain("Return to terminal")
    expect(html).not.toContain(">Message<")
    expect(html).not.toContain("This session can’t receive messages")
  })

  it("does not offer a dead terminal link after the app has relaunched", () => {
    const session = managedSession({
      input: {
        attached: false,
        status: "detached",
        detail: "Use the session terminal.",
      },
    })
    const html = renderToStaticMarkup(
      <SessionView
        {...mainViewProps({ selectedSession: session })}
        selectedSession={session}
      />,
    )

    expect(html).not.toContain("Return to terminal")
    expect(html).not.toContain(">Message<")
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
