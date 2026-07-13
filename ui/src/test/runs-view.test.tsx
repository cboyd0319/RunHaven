import { renderToStaticMarkup } from "react-dom/server"
import { describe, expect, it } from "vitest"

import { SessionLabelsProvider } from "../app/sessionLabels"
import { filterRuns, RunsView } from "../app/views/RunsView"

import { managedSession } from "./main-view-props"

describe("RunsView", () => {
  it("lists every run newest-first, keyboard-operable, and secret-free", () => {
    const html = renderToStaticMarkup(
      <SessionLabelsProvider>
        <RunsView
          filter="all"
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
          ]}
          onSelectSession={() => undefined}
          onFilterChange={() => undefined}
        />
      </SessionLabelsProvider>,
    )

    expect(html).toContain("Run history")
    expect(html).toContain("Codex - alpha")
    expect(html).toContain("Claude - beta")
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
          filter="all"
          sessions={[]}
          onSelectSession={() => undefined}
          onFilterChange={() => undefined}
        />
      </SessionLabelsProvider>,
    )
    expect(html).toContain("No runs yet.")
  })

  it("honors the run category selected in the sidebar", () => {
    const sessions = [
      managedSession({ id: "running", lifecycle: "running" }),
      managedSession({
        id: "failed",
        lifecycle: "failed",
        statusLabel: "Failed",
      }),
      managedSession({
        id: "past",
        lifecycle: "finished",
        statusLabel: "Ended",
      }),
    ]

    expect(filterRuns(sessions, "needs-you").map((session) => session.id)).toEqual([
      "failed",
    ])
    expect(filterRuns(sessions, "running").map((session) => session.id)).toEqual([
      "running",
    ])
    expect(filterRuns(sessions, "past").map((session) => session.id)).toEqual([
      "failed",
      "past",
    ])
  })
})
