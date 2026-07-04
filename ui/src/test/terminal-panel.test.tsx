// @vitest-environment happy-dom
import { createElement } from "react"
import { createRoot } from "react-dom/client"
import { renderToStaticMarkup } from "react-dom/server"
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"

import type { TerminalMessage } from "../commands/runhaven"

import { AppMainContent } from "../app/AppMainContent"
import { act } from "./render-hook"
import {
  launchFlow,
  managedSession,
  mainViewProps,
} from "./main-view-props"
import {
  type FakeTerminalLike,
  mountActiveTerminal,
  runLaunch,
} from "./terminal-panel-fixtures"

// A fake xterm Terminal so the teardown, input, and message-handler contracts
// are testable without booting the real WebView terminal in happy-dom. The
// class stays inline because its `vi.mock` factory is hoisted above imports and
// cannot reference imported values; the reusable helpers live in the fixtures
// module.
const xtermState = vi.hoisted(() => ({
  instances: [] as FakeTerminalLike[],
}))

vi.mock("@xterm/xterm/css/xterm.css", () => ({}))

vi.mock("@xterm/xterm", () => {
  class FakeTerminal {
    cols = 80
    rows = 24
    options: Record<string, unknown> = {}
    writes: string[] = []
    disposed = false
    blurred = false
    dataHandlers: ((data: string) => void)[] = []
    keyEventHandler: ((event: KeyboardEvent) => boolean) | undefined
    constructor(options: Record<string, unknown>) {
      this.options = { ...options }
      xtermState.instances.push(this)
    }
    loadAddon() {}
    open() {}
    focus() {}
    blur() {
      this.blurred = true
    }
    attachCustomKeyEventHandler(handler: (event: KeyboardEvent) => boolean) {
      this.keyEventHandler = handler
    }
    onData(handler: (data: string) => void) {
      this.dataHandlers.push(handler)
    }
    onResize() {}
    write(chunk: Uint8Array | string) {
      this.writes.push(
        typeof chunk === "string" ? chunk : new TextDecoder().decode(chunk),
      )
    }
    writeln(line: string) {
      this.writes.push(`${line}\n`)
    }
    dispose() {
      this.disposed = true
    }
  }
  return { Terminal: FakeTerminal }
})

vi.mock("@xterm/addon-fit", () => ({
  FitAddon: class {
    fit() {}
  },
}))

vi.mock("@/commands/runhaven", async (importActual) => ({
  ...(await importActual<typeof import("../commands/runhaven")>()),
  hasTauriRuntime: vi.fn(() => true),
  openTerminalSession: vi.fn(() => Promise.resolve()),
  openClaudeSignInTerminal: vi.fn(() => Promise.resolve()),
  openAgentRunTerminal: vi.fn(() => Promise.resolve()),
  writeTerminalInput: vi.fn(() => Promise.resolve()),
  resizeTerminal: vi.fn(() => Promise.resolve()),
  closeTerminalSession: vi.fn(() => Promise.resolve()),
}))

import { TerminalPanel } from "../app/TerminalPanel"
import * as runhaven from "../commands/runhaven"

const DISCLOSURE =
  "RunHaven does not filter, read, store, or log it"

const hasTauriRuntime = vi.mocked(runhaven.hasTauriRuntime)
const openTerminalSession = vi.mocked(runhaven.openTerminalSession)
const openClaudeSignInTerminal = vi.mocked(runhaven.openClaudeSignInTerminal)
const openAgentRunTerminal = vi.mocked(runhaven.openAgentRunTerminal)
const closeTerminalSession = vi.mocked(runhaven.closeTerminalSession)
const writeTerminalInput = vi.mocked(runhaven.writeTerminalInput)

beforeEach(() => {
  xtermState.instances.length = 0
  vi.clearAllMocks()
  hasTauriRuntime.mockReturnValue(true)
  openTerminalSession.mockResolvedValue(undefined)
  openClaudeSignInTerminal.mockResolvedValue(undefined)
  openAgentRunTerminal.mockResolvedValue(undefined)
  closeTerminalSession.mockResolvedValue(undefined)
  writeTerminalInput.mockResolvedValue(undefined)
})

afterEach(() => {
  vi.restoreAllMocks()
})

describe("TerminalPanel Claude login mode", () => {
  it("shows the desktop-only placeholder and never signs in during preview", () => {
    hasTauriRuntime.mockReturnValue(false)
    const html = renderToStaticMarkup(
      <TerminalPanel login={{ sessionId: "loginabc" }} />,
    )
    expect(html).toContain(DISCLOSURE)
    expect(html).toContain("The sign-in terminal needs the desktop app")
    expect(html).not.toContain("Open sign-in terminal")
    expect(openClaudeSignInTerminal).not.toHaveBeenCalled()
  })

  it("opens the Claude login command (not attach) under the desktop runtime", async () => {
    const container = document.createElement("div")
    document.body.appendChild(container)
    const root = createRoot(container)
    await act(async () => {
      root.render(
        createElement(TerminalPanel, { login: { sessionId: "loginabc" } }),
      )
    })
    const openButton = [...container.querySelectorAll("button")].find((button) =>
      button.textContent?.includes("Open sign-in terminal"),
    )
    expect(openButton).toBeDefined()
    await act(async () => {
      openButton?.dispatchEvent(new MouseEvent("click", { bubbles: true }))
    })
    await act(async () => {})
    expect(openClaudeSignInTerminal).toHaveBeenCalledTimes(1)
    expect(openClaudeSignInTerminal.mock.calls[0][0]).toBe("loginabc")
    // The login path must not fall through to the attach command.
    expect(openTerminalSession).not.toHaveBeenCalled()
    await act(async () => {
      root.unmount()
    })
    container.remove()
  })
})

describe("TerminalPanel run mode", () => {
  it("shows the desktop-only placeholder and never launches during preview", () => {
    hasTauriRuntime.mockReturnValue(false)
    const html = renderToStaticMarkup(<TerminalPanel run={runLaunch()} />)
    // The standing no-filter/no-store disclosure still shows for a run terminal.
    expect(html).toContain(DISCLOSURE)
    expect(html).toContain("The run terminal needs the desktop app")
    expect(html).not.toContain("Start run")
    expect(openAgentRunTerminal).not.toHaveBeenCalled()
  })

  it("opens the run command with the reviewed request (not attach) under the desktop runtime", async () => {
    const launch = runLaunch()
    const container = document.createElement("div")
    document.body.appendChild(container)
    const root = createRoot(container)
    await act(async () => {
      root.render(createElement(TerminalPanel, { run: launch }))
    })
    const openButton = [...container.querySelectorAll("button")].find((button) =>
      button.textContent?.includes("Start run"),
    )
    expect(openButton).toBeDefined()
    await act(async () => {
      openButton?.dispatchEvent(new MouseEvent("click", { bubbles: true }))
    })
    await act(async () => {})
    expect(openAgentRunTerminal).toHaveBeenCalledTimes(1)
    expect(openAgentRunTerminal.mock.calls[0][0]).toBe(launch.sessionId)
    // The reviewed launch request is handed straight to the backend.
    expect(openAgentRunTerminal.mock.calls[0][1]).toBe(launch.request)
    // The run path must not fall through to attach or login.
    expect(openTerminalSession).not.toHaveBeenCalled()
    expect(openClaudeSignInTerminal).not.toHaveBeenCalled()
    // Close-on-unmount tears down the backend run child.
    closeTerminalSession.mockClear()
    await act(async () => {
      root.unmount()
    })
    expect(closeTerminalSession).toHaveBeenCalledWith(launch.sessionId)
    container.remove()
  })

  it("in the fill destination keeps a reachable Close before start and an End run after, tearing down on close", async () => {
    const launch = runLaunch()
    let closed = 0
    const onClose = () => {
      closed += 1
    }
    const container = document.createElement("div")
    document.body.appendChild(container)
    const root = createRoot(container)
    await act(async () => {
      root.render(
        createElement(TerminalPanel, { run: launch, fill: true, onClose }),
      )
    })
    // Before the run is started the disclosure is up and a Close escape hatch is
    // reachable, so the user is never stranded on the destination.
    expect(container.textContent).toContain(DISCLOSURE)
    const preClose = [...container.querySelectorAll("button")].find(
      (button) => button.textContent?.trim() === "Close",
    )
    expect(preClose).toBeDefined()

    // Start the run, then the terminal-level control ends the run (leaves the
    // destination) instead of dead-ending on a re-launch button.
    const startButton = [...container.querySelectorAll("button")].find((button) =>
      button.textContent?.includes("Start run"),
    )
    await act(async () => {
      startButton?.dispatchEvent(new MouseEvent("click", { bubbles: true }))
    })
    await act(async () => {})
    const endButton = [...container.querySelectorAll("button")].find((button) =>
      button.textContent?.includes("End run"),
    )
    expect(endButton).toBeDefined()
    await act(async () => {
      endButton?.dispatchEvent(new MouseEvent("click", { bubbles: true }))
    })
    expect(closed).toBe(1)

    // Unmounting (what the parent navigation does on close) tears down the child.
    closeTerminalSession.mockClear()
    await act(async () => {
      root.unmount()
    })
    expect(closeTerminalSession).toHaveBeenCalledWith(launch.sessionId)
    container.remove()
  })
})

describe("TerminalPanel disclosure", () => {
  it("always renders the exact no-filter/no-store disclosure in preview mode", () => {
    hasTauriRuntime.mockReturnValue(false)
    const html = renderToStaticMarkup(
      <TerminalPanel session={managedSession()} />,
    )
    expect(html).toContain(DISCLOSURE)
    expect(html).toContain("visible on this screen")
  })

  it("shows the desktop-only placeholder and never opens a session in preview", () => {
    hasTauriRuntime.mockReturnValue(false)
    const html = renderToStaticMarkup(
      <TerminalPanel session={managedSession()} />,
    )
    expect(html).toContain("Terminal needs the desktop app")
    expect(html).not.toContain("Open terminal")
    expect(openTerminalSession).not.toHaveBeenCalled()
  })

  it("offers the Open terminal button under the desktop runtime", () => {
    const html = renderToStaticMarkup(
      <TerminalPanel session={managedSession()} />,
    )
    expect(html).toContain(DISCLOSURE)
    expect(html).toContain("Open terminal")
    expect(html).not.toContain("Terminal needs the desktop app")
  })
})

describe("TerminalPanel teardown contract", () => {
  it("closes the backend session on unmount", async () => {
    const panel = await mountActiveTerminal()
    expect(openTerminalSession).toHaveBeenCalledTimes(1)
    closeTerminalSession.mockClear()
    await panel.unmount()
    expect(closeTerminalSession).toHaveBeenCalledWith(managedSession().id)
  })

  it("closes the backend session on window pagehide", async () => {
    const panel = await mountActiveTerminal()
    closeTerminalSession.mockClear()
    await act(async () => {
      window.dispatchEvent(new Event("pagehide"))
    })
    expect(closeTerminalSession).toHaveBeenCalledWith(managedSession().id)
    await panel.unmount()
  })
})

describe("TerminalPanel message handling", () => {
  it("writes output bytes to xterm and ends on exit", async () => {
    let handler: ((message: TerminalMessage) => void) | undefined
    openTerminalSession.mockImplementation((_sessionId, onMessage) => {
      handler = onMessage
      return Promise.resolve()
    })
    const panel = await mountActiveTerminal()
    expect(handler).toBeDefined()
    const term = xtermState.instances[0]
    expect(term).toBeDefined()

    await act(async () => {
      handler?.({ event: "output", data: [104, 105] })
    })
    expect(term.writes.join("")).toContain("hi")

    await act(async () => {
      handler?.({ event: "exit", code: 0 })
    })
    expect(term.writes.join("")).toContain("This session ended")
    expect(term.options.disableStdin).toBe(true)
    await panel.unmount()
  })

  it("sends typed input to the backend as utf-8 bytes", async () => {
    const panel = await mountActiveTerminal()
    const term = xtermState.instances[0]
    await act(async () => {
      term.dataHandlers.forEach((handler) => handler("a"))
    })
    expect(writeTerminalInput).toHaveBeenCalledWith(managedSession().id, [97])
    await panel.unmount()
  })
})

describe("TerminalPanel keyboard escape (WCAG 2.1.2, audit A11Y-1)", () => {
  it("Shift+Esc escapes to the primary control without forwarding; bare Esc and normal keys reach xterm", async () => {
    const panel = await mountActiveTerminal()
    const term = xtermState.instances[0]
    expect(term.keyEventHandler).toBeDefined()
    const closeButton = [...panel.container.querySelectorAll("button")].find(
      (button) => button.textContent?.trim() === "Close terminal",
    )
    writeTerminalInput.mockClear()
    let handled: boolean | undefined
    await act(async () => {
      handled = term.keyEventHandler?.(
        new KeyboardEvent("keydown", { key: "Escape", shiftKey: true }),
      )
    })
    // Chord kept out of the PTY (false), terminal blurred, focus on the visible
    // control so a keyboard-only user is never trapped.
    expect(handled).toBe(false)
    expect(term.blurred).toBe(true)
    expect(document.activeElement).toBe(closeButton)
    expect(writeTerminalInput).not.toHaveBeenCalled()
    // Bare Esc (Claude uses single/double Esc) and normal keys still reach xterm.
    const bareEsc = new KeyboardEvent("keydown", { key: "Escape" })
    expect(term.keyEventHandler?.(bareEsc)).toBe(true)
    expect(term.keyEventHandler?.(new KeyboardEvent("keydown", { key: "a" }))).toBe(true)
    await panel.unmount()
  })
})

// Pins the P0 fix (audit RT-1): the run's sandbox uses this terminal's PTY as
// its controlling TTY, so navigating away must keep the panel MOUNTED (not
// unmount-and-kill). Guards against a future refactor silently reintroducing
// kill-on-nav, which every static-markup test would miss.
describe("AppMainContent keeps a live run mounted across navigation", () => {
  it("does not tear down the run on nav change and preserves the terminal node, ending only on unmount", async () => {
    const launch = runLaunch()
    const container = document.createElement("div")
    document.body.appendChild(container)
    const root = createRoot(container)

    async function show(kind: "run-terminal" | "home") {
      await act(async () => {
        root.render(
          createElement(AppMainContent, {
            ...mainViewProps({
              launchFlow: { ...launchFlow(), runTerminal: launch },
            }),
            navigation: { kind },
          }),
        )
      })
    }

    // Start the run on its destination (active), then flush the async xterm open.
    await show("run-terminal")
    const start = [...container.querySelectorAll("button")].find((button) =>
      button.textContent?.includes("Start run"),
    )
    if (!start) {
      throw new Error("Start run button not found")
    }
    await act(async () => {
      start.dispatchEvent(new MouseEvent("click", { bubbles: true }))
    })
    await act(async () => {})
    expect(openAgentRunTerminal).toHaveBeenCalledTimes(1)
    const node = container.querySelector('[aria-label="Sandbox terminal"]')
    expect(node).not.toBeNull()

    // Navigate AWAY to Home: the run must NOT be torn down, and the xterm node
    // identity must be preserved (proves no remount, so the agent survives).
    closeTerminalSession.mockClear()
    await show("home")
    expect(closeTerminalSession).not.toHaveBeenCalled()
    expect(container.querySelector('[aria-label="Sandbox terminal"]')).toBe(node)

    // Navigate BACK: still the same live node, still no teardown.
    await show("run-terminal")
    expect(closeTerminalSession).not.toHaveBeenCalled()
    expect(container.querySelector('[aria-label="Sandbox terminal"]')).toBe(node)

    // Only a real unmount (End run clears runTerminal) tears down the child.
    await act(async () => {
      root.unmount()
    })
    expect(closeTerminalSession).toHaveBeenCalledWith(launch.sessionId)
    container.remove()
  })
})
