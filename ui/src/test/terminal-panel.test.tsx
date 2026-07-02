// @vitest-environment happy-dom
import { createElement } from "react"
import { createRoot } from "react-dom/client"
import { renderToStaticMarkup } from "react-dom/server"
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"

import type { TerminalMessage } from "../commands/runhaven"

import { act } from "./render-hook"
import { managedSession } from "./main-view-props"

// A fake xterm Terminal so the teardown, input, and message-handler contracts
// are testable without booting the real WebView terminal in happy-dom.
const xtermState = vi.hoisted(() => ({
  instances: [] as FakeTerminalLike[],
}))

type FakeTerminalLike = {
  cols: number
  rows: number
  options: Record<string, unknown>
  writes: string[]
  disposed: boolean
  dataHandlers: ((data: string) => void)[]
}

vi.mock("@xterm/xterm/css/xterm.css", () => ({}))

vi.mock("@xterm/xterm", () => {
  class FakeTerminal {
    cols = 80
    rows = 24
    options: Record<string, unknown> = {}
    writes: string[] = []
    disposed = false
    dataHandlers: ((data: string) => void)[] = []
    constructor(options: Record<string, unknown>) {
      this.options = { ...options }
      xtermState.instances.push(this)
    }
    loadAddon() {}
    open() {}
    focus() {}
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
const closeTerminalSession = vi.mocked(runhaven.closeTerminalSession)
const writeTerminalInput = vi.mocked(runhaven.writeTerminalInput)

beforeEach(() => {
  xtermState.instances.length = 0
  vi.clearAllMocks()
  hasTauriRuntime.mockReturnValue(true)
  openTerminalSession.mockResolvedValue(undefined)
  closeTerminalSession.mockResolvedValue(undefined)
  writeTerminalInput.mockResolvedValue(undefined)
})

afterEach(() => {
  vi.restoreAllMocks()
})

async function mountActiveTerminal(): Promise<{
  container: HTMLDivElement
  unmount: () => Promise<void>
}> {
  const container = document.createElement("div")
  document.body.appendChild(container)
  const root = createRoot(container)
  await act(async () => {
    root.render(createElement(TerminalPanel, { session: managedSession() }))
  })
  // Click "Open terminal" to activate the panel, then flush the async xterm
  // load + open.
  const openButton = [...container.querySelectorAll("button")].find((button) =>
    button.textContent?.includes("Open terminal"),
  )
  if (!openButton) {
    throw new Error("Open terminal button not found")
  }
  await act(async () => {
    openButton.dispatchEvent(new MouseEvent("click", { bubbles: true }))
  })
  await act(async () => {})
  return {
    container,
    async unmount() {
      await act(async () => {
        root.unmount()
      })
      container.remove()
    },
  }
}

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
