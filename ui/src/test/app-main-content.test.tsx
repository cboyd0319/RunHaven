import { renderToStaticMarkup } from "react-dom/server"
import { describe, expect, it } from "vitest"

import { AppMainContent } from "../app/AppMainContent"
import type { RunTerminalLaunch } from "../app/useLaunchFlow"

import { launchFlow, mainViewProps, planRequest } from "./main-view-props"

// A reviewed, confirmed interactive launch as it reaches the run-terminal
// destination.
function runTerminal(): RunTerminalLaunch {
  return {
    sessionId: "runabc123",
    request: {
      plan: { ...planRequest(), agent: "claude" },
      confirmLaunch: true,
      confirmedWarnings: [],
    },
  }
}

describe("AppMainContent run-terminal destination", () => {
  it("renders the interactive run terminal in main content, not the New Run modal", () => {
    const html = renderToStaticMarkup(
      <AppMainContent
        {...mainViewProps({
          launchFlow: { ...launchFlow(), runTerminal: runTerminal() },
        })}
        navigation={{ kind: "run-terminal" }}
      />,
    )

    // The run terminal renders as a main-content view: its title and the
    // standing no-filter/no-store disclosure are present, and there is no New
    // Run dialog around it (that lives on Home).
    expect(html).toContain("<main")
    expect(html).toContain("Run terminal")
    expect(html).toContain("RunHaven does not filter, read, store, or log it")
    expect(html).not.toContain("New run")
    // Browser preview (no Tauri runtime in tests) shows the placeholder and a
    // reachable Close, and never invokes the backend.
    expect(html).toContain("The run terminal needs the desktop app")
    expect(html).toContain("Close")
  })

  it("falls back to Home when the destination has no live run terminal", () => {
    const html = renderToStaticMarkup(
      <AppMainContent
        {...mainViewProps()}
        navigation={{ kind: "run-terminal" }}
      />,
    )

    // Never a dead view: the New Run action returns so the user is not stranded.
    expect(html).toContain("New run")
  })
})
