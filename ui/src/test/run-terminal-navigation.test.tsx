// @vitest-environment happy-dom
import { createElement } from "react"
import { createRoot } from "react-dom/client"
import { beforeEach, expect, it, vi } from "vitest"

import { AppMainContent } from "../app/AppMainContent"
import * as runhaven from "../commands/runhaven"
import { launchFlow, mainViewProps } from "./main-view-props"
import { act } from "./render-hook"
import { runLaunch } from "./terminal-panel-fixtures"

vi.mock("@xterm/xterm/css/xterm.css", () => ({}))
vi.mock("@xterm/xterm", () => ({
  Terminal: class {
    cols = 80
    rows = 24
    options: Record<string, unknown> = {}
    loadAddon() {}
    open() {}
    focus() {}
    blur() {}
    attachCustomKeyEventHandler() {}
    onData() {}
    onResize() {}
    write() {}
    writeln() {}
    dispose() {}
  },
}))
vi.mock("@xterm/addon-fit", () => ({
  FitAddon: class {
    fit() {}
  },
}))
vi.mock("@/commands/runhaven", async (importActual) => ({
  ...(await importActual<typeof import("../commands/runhaven")>()),
  hasTauriRuntime: vi.fn(() => true),
  openAgentRunTerminal: vi.fn(() =>
    Promise.resolve({ runId: "run-live", label: "Claude · project" }),
  ),
  resizeTerminal: vi.fn(() => Promise.resolve()),
  endTerminalSession: vi.fn(() => Promise.resolve(true)),
  closeTerminalSession: vi.fn(() => Promise.resolve()),
}))

const openAgentRunTerminal = vi.mocked(runhaven.openAgentRunTerminal)
const closeTerminalSession = vi.mocked(runhaven.closeTerminalSession)

beforeEach(() => {
  vi.clearAllMocks()
  openAgentRunTerminal.mockResolvedValue({
    runId: "run-live",
    label: "Claude · project",
  })
  closeTerminalSession.mockResolvedValue(undefined)
})

it("keeps a live run mounted across navigation and ends it only on unmount", async () => {
  const launch = runLaunch()
  const container = document.createElement("div")
  document.body.appendChild(container)
  const root = createRoot(container)

  async function show(kind: "run-terminal" | "new-session") {
    await act(async () => {
      root.render(
        createElement(AppMainContent, {
          ...mainViewProps({
            launchFlow: {
              ...launchFlow(),
              runTerminal: launch,
              runTerminals: [launch],
            },
          }),
          navigation: { kind },
        }),
      )
    })
  }

  await show("run-terminal")
  await act(async () => {})
  expect(openAgentRunTerminal).toHaveBeenCalledTimes(1)
  const node = container.querySelector('[aria-label$="terminal workspace"]')
  expect(node).not.toBeNull()

  closeTerminalSession.mockClear()
  await show("new-session")
  expect(closeTerminalSession).not.toHaveBeenCalled()
  expect(container.querySelector('[aria-label$="terminal workspace"]')).toBe(node)

  await show("run-terminal")
  expect(closeTerminalSession).not.toHaveBeenCalled()
  expect(container.querySelector('[aria-label$="terminal workspace"]')).toBe(node)

  await act(async () => root.unmount())
  expect(closeTerminalSession).toHaveBeenCalledWith(launch.sessionId)
  container.remove()
})
