import { renderToStaticMarkup } from "react-dom/server"
import { describe, expect, it } from "vitest"

import { RunsView } from "../app/views/RunsView"

import { managedSession } from "./main-view-props"

describe("RunsView", () => {
  it("lists every run newest-first, keyboard-operable, and secret-free", () => {
    const html = renderToStaticMarkup(
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
          onStart={() => undefined}
      />,
    )

    expect(html).toContain("Session history")
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
      <RunsView
        sessions={[]}
        onSelectSession={() => undefined}
        onSignIn={() => undefined}
        onStart={() => undefined}
      />,
    )
    expect(html).toContain("No sessions yet")
    expect(html).toContain("Sessions you start will appear here.")
    expect(html).toContain("Start a session")
    expect(html).not.toContain("Session history")
  })

  it("keeps attention-needed runs in History", () => {
    const html = renderToStaticMarkup(
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
            managedSession({
              id: "clean-copy",
              title: "Codex - no file changes",
              lifecycle: "finished",
              statusLabel: "Finished",
              cleanup: {
                available: true,
                reviewState: "clean",
                target: "worktree",
                label: "Delete protected copy",
                detail: "A protected copy exists.",
              },
            }),
            managedSession({
              id: "review",
              title: "Claude - review a very long project title without clipping",
              nextAction: {
                kind: "monitor",
                label: "Monitor",
                detail: "RunHaven is tracking status for this session.",
              },
              cleanup: {
                available: true,
                reviewState: "changes",
                target: "worktree",
                label: "Review saved copy",
                detail: "A protected copy is ready.",
              },
            }),
          ]}
          onSelectSession={() => undefined}
          onSignIn={() => undefined}
          onStart={() => undefined}
      />,
    )

    expect(html).toContain("Needs attention")
    expect(html).toContain("Codex - needs help")
    expect(html).toContain("The session reported a failure.")
    expect(html).toContain(
      "Claude - review a very long project title without clipping",
    )
    expect(html).toContain("The assistant changed files in the protected copy.")
    expect(html).not.toContain("RunHaven is tracking status for this session.")
    expect(html).toContain("Review changes")
    expect(html).toContain("shrink-0 self-start")
    const attentionPanel = html.slice(
      html.indexOf("Needs attention"),
      html.indexOf("Other sessions"),
    )
    expect(attentionPanel).not.toContain("table-fixed")
    expect(attentionPanel).toContain("min-w-0 break-words")
    expect(attentionPanel).not.toContain("Codex - no file changes")
    expect(attentionPanel).not.toContain(
      'class="truncate">Claude - review a very long project title',
    )
    expect(html).toContain("Other sessions")
    expect(html).toContain("Claude - active")
    expect(html).toContain("Codex - no file changes")
  })
})
