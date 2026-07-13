// @vitest-environment happy-dom
import { createElement } from "react"
import { createRoot } from "react-dom/client"
import { renderToStaticMarkup } from "react-dom/server"
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"

import type { TerminalMessage } from "../commands/runhaven"

import { act } from "./render-hook"
import { managedSession } from "./main-view-props"
import {
  type FakeTerminalLike,
  mountActiveTerminal,
  runLaunch,
  stubClipboard,
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
    getSelection = () => ""
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
  openTerminalSession: vi.fn(() => Promise.resolve(true)),
  openAgentSignInTerminal: vi.fn(() => Promise.resolve(true)),
  openAgentRunTerminal: vi.fn(() => Promise.resolve(true)),
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
const openAgentSignInTerminal = vi.mocked(runhaven.openAgentSignInTerminal)
const openAgentRunTerminal = vi.mocked(runhaven.openAgentRunTerminal)
const closeTerminalSession = vi.mocked(runhaven.closeTerminalSession)
const writeTerminalInput = vi.mocked(runhaven.writeTerminalInput)

beforeEach(() => {
  xtermState.instances.length = 0
  vi.clearAllMocks()
  hasTauriRuntime.mockReturnValue(true)
  openTerminalSession.mockResolvedValue(true)
  openAgentSignInTerminal.mockResolvedValue(true)
  openAgentRunTerminal.mockResolvedValue(true)
  closeTerminalSession.mockResolvedValue(undefined)
  writeTerminalInput.mockResolvedValue(undefined)
})

afterEach(() => {
  vi.restoreAllMocks()
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

  it("starts the reviewed run automatically after leaving the review sheet", async () => {
    const launch = runLaunch()
    const container = document.createElement("div")
    document.body.appendChild(container)
    const root = createRoot(container)
    await act(async () => {
      root.render(createElement(TerminalPanel, { run: launch }))
    })
    await act(async () => {})
    expect(container.textContent).not.toContain("Start run")
    expect(openAgentRunTerminal).toHaveBeenCalledTimes(1)
    expect(openAgentRunTerminal.mock.calls[0][0]).toBe(launch.sessionId)
    // The reviewed launch request is handed straight to the backend.
    expect(openAgentRunTerminal.mock.calls[0][1]).toBe(launch.request)
    // The run path must not fall through to attach or login.
    expect(openTerminalSession).not.toHaveBeenCalled()
    expect(openAgentSignInTerminal).not.toHaveBeenCalled()
    // Close-on-unmount tears down the backend run child.
    closeTerminalSession.mockClear()
    await act(async () => {
      root.unmount()
    })
    expect(closeTerminalSession).toHaveBeenCalledWith(launch.sessionId)
    container.remove()
  })

  it("in the fill destination keeps an End run escape hatch while starting", async () => {
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
    // The run starts automatically, while the disclosure and an End run escape
    // hatch stay reachable so the user is never stranded on the destination.
    expect(container.textContent).toContain(DISCLOSURE)
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
      return Promise.resolve(true)
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
    expect(term.writes.join("")).toContain("This terminal ended")
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

  it("tells the user when typed input does not reach the terminal", async () => {
    writeTerminalInput.mockRejectedValueOnce(new Error("terminal channel closed"))
    const panel = await mountActiveTerminal()
    const term = xtermState.instances[0]

    await act(async () => {
      term.dataHandlers.forEach((handler) => handler("a"))
    })
    await act(async () => {})

    expect(panel.container.textContent).toContain("Terminal input failed")
    expect(panel.container.textContent).toContain("terminal channel closed")
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

describe("TerminalPanel clipboard (audit NAT-8)", () => {
  it("Cmd-C copies a selection and blocks ^C, passes a no-selection Cmd-C through, and Cmd-V pastes utf-8 into the PTY", async () => {
    const writeText = stubClipboard("pasted")
    await mountActiveTerminal()
    const term = xtermState.instances[0]
    const id = managedSession().id
    const cmd = (k: string) =>
      new KeyboardEvent("keydown", { key: k, metaKey: true })
    term.getSelection = () => "copied"
    expect(term.keyEventHandler?.(cmd("c"))).toBe(false)
    expect(writeText).toHaveBeenCalledWith("copied")
    term.getSelection = () => ""
    expect(term.keyEventHandler?.(cmd("c"))).toBe(true)
    expect(term.keyEventHandler?.(cmd("v"))).toBe(false)
    await act(async () => {})
    const bytes = [...new TextEncoder().encode("pasted")]
    expect(writeTerminalInput).toHaveBeenCalledExactlyOnceWith(id, bytes)
  })
})
