// @vitest-environment happy-dom
import { createElement } from "react"
import { createRoot } from "react-dom/client"
import { beforeEach, describe, expect, it, vi } from "vitest"

import { TerminalPanel } from "../app/TerminalPanel"
import * as runhaven from "../commands/runhaven"
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
  writeTerminalInput: vi.fn(() => Promise.resolve()),
  endTerminalSession: vi.fn(() => Promise.resolve(true)),
  closeTerminalSession: vi.fn(() => Promise.resolve()),
}))

const openAgentRunTerminal = vi.mocked(runhaven.openAgentRunTerminal)
const endTerminalSession = vi.mocked(runhaven.endTerminalSession)
const closeTerminalSession = vi.mocked(runhaven.closeTerminalSession)

beforeEach(() => {
  vi.clearAllMocks()
  openAgentRunTerminal.mockResolvedValue({
    runId: "run-live",
    label: "Claude · project",
  })
  endTerminalSession.mockResolvedValue(true)
  closeTerminalSession.mockResolvedValue(undefined)
})

async function mountRun(onClose: () => void) {
  const launch = runLaunch()
  const container = document.createElement("div")
  document.body.appendChild(container)
  const root = createRoot(container)
  await act(async () => {
    root.render(createElement(TerminalPanel, { run: launch, fill: true, onClose }))
  })
  await act(async () => {})
  return { container, launch, root }
}

describe("TerminalPanel run end", () => {
  it("ends a run only after the native end confirmation succeeds", async () => {
    const onClose = vi.fn()
    const panel = await mountRun(onClose)
    const endButton = [...panel.container.querySelectorAll("button")].find(
      (button) => button.textContent?.trim() === "End session…",
    )
    expect(endButton).toBeDefined()
    await act(async () => {
      endButton?.dispatchEvent(new MouseEvent("click", { bubbles: true }))
    })

    expect(endTerminalSession).toHaveBeenCalledWith(panel.launch.sessionId)
    expect(onClose).toHaveBeenCalledTimes(1)
    closeTerminalSession.mockClear()
    await act(async () => panel.root.unmount())
    expect(closeTerminalSession).toHaveBeenCalledWith(panel.launch.sessionId)
    panel.container.remove()
  })

  it("keeps the run terminal open when End session is canceled", async () => {
    endTerminalSession.mockResolvedValue(false)
    const onClose = vi.fn()
    const panel = await mountRun(onClose)
    const endButton = [...panel.container.querySelectorAll("button")].find(
      (button) => button.textContent?.trim() === "End session…",
    )
    await act(async () => {
      endButton?.dispatchEvent(new MouseEvent("click", { bubbles: true }))
    })

    expect(endTerminalSession).toHaveBeenCalledWith(panel.launch.sessionId)
    expect(onClose).not.toHaveBeenCalled()
    expect(
      panel.container.querySelector('[aria-label="Session terminal"]'),
    ).not.toBeNull()
    await act(async () => panel.root.unmount())
    panel.container.remove()
  })

  it("closes a failed-open destination without forging an end confirmation", async () => {
    openAgentRunTerminal.mockRejectedValue(new Error("session did not start"))
    const onClose = vi.fn()
    const panel = await mountRun(onClose)

    expect(panel.container.textContent).toContain("Terminal did not open")
    const closeButton = [...panel.container.querySelectorAll("button")].find(
      (button) => button.textContent?.trim() === "Close",
    )
    expect(closeButton).toBeDefined()
    await act(async () => {
      closeButton?.dispatchEvent(new MouseEvent("click", { bubbles: true }))
    })

    expect(endTerminalSession).not.toHaveBeenCalled()
    expect(onClose).toHaveBeenCalledTimes(1)
    await act(async () => panel.root.unmount())
    panel.container.remove()
  })
})
