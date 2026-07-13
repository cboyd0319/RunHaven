// @vitest-environment happy-dom
import { createElement } from "react"
import { createRoot } from "react-dom/client"
import { renderToStaticMarkup } from "react-dom/server"
import { describe, expect, it, vi } from "vitest"

import { AppMainContent, mainViewFocusKey } from "../app/AppMainContent"
import type { RunTerminalLaunch } from "../app/useLaunchFlow"

import {
  dashboard,
  launchFlow,
  mainViewProps,
  planRequest,
} from "./main-view-props"
import { act } from "./render-hook"

// A reviewed, confirmed interactive launch as it reaches the run-terminal
// destination.
function runTerminal(): RunTerminalLaunch {
  return {
    sessionId: "runabc123",
    startedAt: "2026-07-01T01:00:00Z",
    request: {
      plan: { ...planRequest(), agent: "claude" },
    },
  }
}

describe("AppMainContent", () => {
  it("moves focus for explicit session refreshes, not background review state", () => {
    const session = { kind: "session", id: "run-1" } as const

    expect(mainViewFocusKey(session, 0)).toBe("session:run-1:0")
    expect(mainViewFocusKey(session, 0)).toBe(mainViewFocusKey(session, 0))
    expect(mainViewFocusKey(session, 1)).not.toBe(mainViewFocusKey(session, 0))
  })

  it("renders a concise in-app guide at the Help destination", () => {
    const html = renderToStaticMarkup(
      <AppMainContent
        {...mainViewProps()}
        navigation={{ kind: "help", returnTo: { kind: "new-session" } }}
      />,
    )

    expect(html).toContain("RunHaven Help")
    expect(html).toContain("Start safely")
    expect(html).toContain("Find a previous session")
    expect(html).toContain("Fix a setup problem")
    expect(html).toContain("Back to New session")
  })

  it("returns from Help to the destination the user was using", async () => {
    const onNavigate = vi.fn()
    const container = document.createElement("div")
    const root = createRoot(container)
    await act(async () => {
      root.render(
        <AppMainContent
          {...mainViewProps({ onNavigate })}
          navigation={{ kind: "help", returnTo: { kind: "new-session" } }}
        />,
      )
    })

    const back = Array.from(container.querySelectorAll("button")).find(
      (button) => button.textContent?.trim() === "Back to New session",
    )
    expect(back).toBeDefined()
    await act(async () => back?.click())
    expect(onNavigate).toHaveBeenCalledWith({ kind: "new-session" })
    await act(async () => root.unmount())
  })

  it("renders the interactive run terminal in the bounded main content", () => {
    const html = renderToStaticMarkup(
      <AppMainContent
        {...mainViewProps({
          launchFlow: {
            ...launchFlow(),
            runTerminal: runTerminal(),
            runTerminals: [runTerminal()],
          },
        })}
        navigation={{ kind: "run-terminal" }}
      />,
    )

    // The run terminal renders as a main-content view: its title and the
    // standing no-filter/no-store disclosure are present, and there is no New
    // Run dialog around it (that lives on New session).
    expect(html).toContain("<main")
    expect(html).toContain("Session terminal")
    expect(html).toContain("RunHaven keeps a filtered copy in memory until you quit")
    expect(html).not.toContain("New run")
    // Browser preview (no Tauri runtime in tests) shows the placeholder and a
    // reachable Close, and never invokes the backend.
    expect(html).toContain("The session terminal needs the desktop app")
    expect(html).toContain("Close")
  })

  it("keeps the run terminal mounted but hidden when navigated away from it", () => {
    // RT-1: the run's sandbox uses this terminal's PTY as its controlling TTY,
    // so navigating away must NOT unmount the panel (that would close the PTY and
    // kill the run). With a live run and a non-run-terminal destination, the
    // panel stays rendered (hidden) AND the normal destination shows alongside.
    const html = renderToStaticMarkup(
      <AppMainContent
        {...mainViewProps({
          launchFlow: {
            ...launchFlow(),
            runTerminal: runTerminal(),
            runTerminals: [runTerminal()],
          },
        })}
        navigation={{ kind: "new-session" }}
      />,
    )

    // The terminal is still in the tree (its standing disclosure renders)...
    expect(html).toContain("RunHaven keeps a filtered copy in memory until you quit")
    expect(html).toContain("hidden")
    // ...and the New session flow renders alongside it.
    expect(html).toContain("Choose assistant")
  })

  it("offers a Dismiss action on the top-level error and hides it when used", async () => {
    const container = document.createElement("div")
    const root = createRoot(container)
    await act(async () => {
      root.render(
        createElement(AppMainContent, {
          ...mainViewProps({ error: "Something went wrong." }),
          navigation: { kind: "runs" },
        }),
      )
    })

    expect(container.textContent).toContain("Action failed")
    expect(container.textContent).toContain("Something went wrong.")
    const dismiss = Array.from(container.querySelectorAll("button")).find(
      (button) => button.textContent?.trim() === "Dismiss",
    )
    expect(dismiss).toBeDefined()

    await act(async () =>
      dismiss?.dispatchEvent(new MouseEvent("click", { bubbles: true })),
    )

    // The error clears view-locally without waiting for the next refresh.
    expect(container.textContent).not.toContain("Action failed")
    expect(container.textContent).not.toContain("Something went wrong.")

    await act(async () => {
      root.unmount()
    })
  })

  it("re-shows a dismissed error after an A -> B -> A flap", async () => {
    const container = document.createElement("div")
    const root = createRoot(container)
    function show(message: string) {
      return act(async () => {
        root.render(
          createElement(AppMainContent, {
            ...mainViewProps({ error: message }),
            navigation: { kind: "runs" },
          }),
        )
      })
    }

    await show("Error A.")
    const dismiss = Array.from(container.querySelectorAll("button")).find(
      (button) => button.textContent?.trim() === "Dismiss",
    )
    await act(async () =>
      dismiss?.dispatchEvent(new MouseEvent("click", { bubbles: true })),
    )
    expect(container.textContent).not.toContain("Error A.")

    // A different failure shows normally...
    await show("Error B.")
    expect(container.textContent).toContain("Error B.")

    // ...and Error A recurring (B -> A) is a fresh, current error, not the one
    // that was dismissed, so it must show again (the flap the review flagged).
    await show("Error A.")
    expect(container.textContent).toContain("Error A.")

    await act(async () => {
      root.unmount()
    })
  })

  it("falls back to New session when the destination has no live terminal", () => {
    const html = renderToStaticMarkup(
      <AppMainContent
        {...mainViewProps()}
        navigation={{ kind: "run-terminal" }}
      />,
    )

    // Never a dead view: the launchpad returns so the user is not stranded.
    expect(html).toContain("Choose assistant")
  })

  it("falls back to New session while a removed session selection is reconciling", () => {
    const html = renderToStaticMarkup(
      <AppMainContent
        {...mainViewProps({ selectedSession: null })}
        navigation={{ kind: "session", id: "removed-session" }}
      />,
    )

    expect(html).toContain("Choose assistant")
    expect(html).not.toBe("")
  })

  it("keeps operational errors visible on New session", () => {
    const html = renderToStaticMarkup(
      <AppMainContent
        {...mainViewProps({
          error: "Background refresh failed.",
          launchMessage: "Run accepted.",
        })}
        navigation={{ kind: "new-session" }}
      />,
    )

    expect(html).toContain("Choose assistant")
    expect(html).toContain("Action failed")
    expect(html).toContain("Background refresh failed.")
    expect(html).toContain("Run accepted.")
  })

  it("shows a setup problem only after the user starts a new session", () => {
    const html = renderToStaticMarkup(
      <AppMainContent
        {...mainViewProps({ dashboard: dashboard(false), setupReady: false })}
        navigation={{ kind: "new-session" }}
      />,
    )

    expect(html).toContain("Setup needs attention")
    expect(html).toContain("Apple container is not installed.")
    expect(html).toContain("System details")
    expect(html).not.toContain("Fix the setup problem on Home")
  })
})
