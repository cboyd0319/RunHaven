import { renderToStaticMarkup } from "react-dom/server"
import { describe, expect, it } from "vitest"

import { SessionLabelsProvider } from "../app/sessionLabels"
import { RunsView } from "../app/views/RunsView"

import { managedSession } from "./main-view-props"

describe("RunsView", () => {
  it("lists every run newest-first, keyboard-operable, and secret-free", () => {
    const html = renderToStaticMarkup(
      <SessionLabelsProvider>
        <RunsView
          sessions={[
            managedSession({
              id: "a",
              title: "Codex - alpha",
              startedAt: "2026-07-01T01:00:00Z",
            }),
            managedSession({
              id: "b",
              title: "Claude - beta",
              startedAt: "2026-07-01T03:00:00Z",
              project: "/Users/c/private/beta",
              containerName: "runhaven-hidden-home",
              detailCommand: "runhaven runs show --env TOKEN=value",
            }),
            managedSession({
              id: "login",
              title: "Claude sign in",
              project: "login-workspace",
            }),
            managedSession({ id: "shell", title: "Shell helper", agent: "shell" }),
          ]}
          onSelectSession={() => undefined}
          onSignIn={() => undefined}
        />
      </SessionLabelsProvider>,
    )

    expect(html).toContain("Run history")
    expect(html).toContain("Codex - alpha")
    expect(html).toContain("Claude - beta")
    expect(html).not.toContain("Claude sign in")
    expect(html).not.toContain("Shell helper")
    expect(html).not.toContain("Filter run history")
    // Newest-first: beta (03:00) before alpha (01:00).
    expect(html.indexOf("Claude - beta")).toBeLessThan(html.indexOf("Codex - alpha"))
    // Each run title is a real button (keyboard/SR affordance), not a bare cell.
    expect(html).toContain("<button")
    // No raw path, container, or env value leaks through.
    expect(html).not.toContain("/Users/c/private")
    expect(html).not.toContain("runhaven-hidden-home")
    expect(html).not.toContain("TOKEN=value")
  })

  it("shows a compact empty state when there are no runs", () => {
    const html = renderToStaticMarkup(
      <SessionLabelsProvider>
        <RunsView
          sessions={[]}
          onSelectSession={() => undefined}
          onSignIn={() => undefined}
        />
      </SessionLabelsProvider>,
    )
    expect(html).toContain("No runs yet.")
  })

  it("moves attention-needed runs here instead of Home", () => {
    const html = renderToStaticMarkup(
      <SessionLabelsProvider>
        <RunsView
          sessions={[
            managedSession({
              id: "failed",
              title: "Codex - needs help",
              lifecycle: "failed",
              nextAction: {
                kind: "failed",
                label: "Failed",
                detail: "The session reported a failure.",
              },
            }),
            managedSession({ id: "active", title: "Claude - active" }),
          ]}
          onSelectSession={() => undefined}
          onSignIn={() => undefined}
        />
      </SessionLabelsProvider>,
    )

    expect(html).toContain("Needs attention")
    expect(html).toContain("Codex - needs help")
    expect(html).toContain("The session reported a failure.")
    expect(html).toContain("Other runs")
    expect(html).toContain("Claude - active")
  })
})
