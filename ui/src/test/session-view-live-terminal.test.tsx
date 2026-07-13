// @vitest-environment happy-dom
import { createRoot } from "react-dom/client"
import { describe, expect, it, vi } from "vitest"

import { SessionView } from "../app/views/SessionView"
import { act } from "./render-hook"
import {
  launchFlow,
  mainViewProps,
  managedSession,
  planRequest,
} from "./main-view-props"

vi.mock("../app/TerminalPanel", () => ({
  TerminalPanel: () => <div>Attach terminal</div>,
}))

describe("SessionView live terminal details", () => {
  it("opens and loads same-page issue details for a session that needs attention", async () => {
    const session = managedSession({ lifecycle: "failed" })
    const loadDiagnostics = vi.fn(async () => {})
    const container = document.createElement("div")
    const root = createRoot(container)

    await act(async () => {
      root.render(
        <SessionView
          {...mainViewProps({
            selectedSession: session,
            onLoadSelectedSessionDiagnostics: loadDiagnostics,
          })}
          selectedSession={session}
        />,
      )
    })

    expect(container.textContent).toContain("Hide issue details")
    expect(container.textContent).toContain("Session diagnostics")
    expect(loadDiagnostics).toHaveBeenCalledTimes(1)
    await act(async () => root.unmount())
  })

  it("does not offer a second terminal for the same live run", async () => {
    const session = managedSession()
    const container = document.createElement("div")
    document.body.append(container)
    const root = createRoot(container)

    await act(async () => {
      root.render(
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
    })
    await act(async () => {
      Array.from(container.querySelectorAll("button"))
        .find((button) => button.textContent === "Show details")
        ?.click()
    })

    expect(container.textContent).not.toContain("Attach terminal")
    await act(async () => root.unmount())
    container.remove()
  })
})
