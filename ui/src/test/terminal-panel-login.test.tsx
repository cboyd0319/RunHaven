// @vitest-environment happy-dom
import { createElement } from "react"
import { createRoot } from "react-dom/client"
import { renderToStaticMarkup } from "react-dom/server"
import { beforeEach, describe, expect, it, vi } from "vitest"

import { TerminalPanel } from "../app/TerminalPanel"
import * as runhaven from "../commands/runhaven"
import type { TerminalMessage } from "../commands/runhaven"
import { act } from "./render-hook"

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
  openTerminalSession: vi.fn(() => Promise.resolve(true)),
  openAgentSignInTerminal: vi.fn(() => Promise.resolve(true)),
  resizeTerminal: vi.fn(() => Promise.resolve()),
  writeTerminalInput: vi.fn(() => Promise.resolve()),
  closeTerminalSession: vi.fn(() => Promise.resolve()),
}))

const hasTauriRuntime = vi.mocked(runhaven.hasTauriRuntime)
const openTerminalSession = vi.mocked(runhaven.openTerminalSession)
const openAgentSignInTerminal = vi.mocked(runhaven.openAgentSignInTerminal)
const closeTerminalSession = vi.mocked(runhaven.closeTerminalSession)

beforeEach(() => {
  vi.clearAllMocks()
  hasTauriRuntime.mockReturnValue(true)
  openTerminalSession.mockResolvedValue(true)
  openAgentSignInTerminal.mockResolvedValue(true)
  closeTerminalSession.mockResolvedValue(undefined)
})

function mountLogin(onSignedIn?: () => void) {
  const container = document.createElement("div")
  document.body.appendChild(container)
  const root = createRoot(container)
  const login = {
    agent: "claude",
    agentLabel: "Claude",
    sessionId: "loginabc",
    onSignedIn,
  }
  return { container, login, root }
}

async function openLogin(container: HTMLElement) {
  const openButton = [...container.querySelectorAll("button")].find((button) =>
    button.textContent?.includes("Open sign-in terminal"),
  )
  expect(openButton).toBeDefined()
  await act(async () => {
    openButton?.dispatchEvent(new MouseEvent("click", { bubbles: true }))
  })
  await act(async () => {})
}

describe("TerminalPanel agent login mode", () => {
  it("shows the desktop-only placeholder and never signs in during preview", () => {
    hasTauriRuntime.mockReturnValue(false)
    const html = renderToStaticMarkup(
      <TerminalPanel
        login={{ agent: "claude", agentLabel: "Claude", sessionId: "loginabc" }}
      />,
    )
    expect(html).toContain("RunHaven does not filter, read, store, or log it")
    expect(html).toContain("The sign-in terminal needs the desktop app")
    expect(html).not.toContain("Open sign-in terminal")
    expect(openAgentSignInTerminal).not.toHaveBeenCalled()
  })

  it("opens the agent login command instead of attach", async () => {
    const panel = mountLogin()
    await act(async () => panel.root.render(<TerminalPanel login={panel.login} />))
    await openLogin(panel.container)
    expect(openAgentSignInTerminal).toHaveBeenCalledTimes(1)
    expect(openAgentSignInTerminal.mock.calls[0][0]).toBe("claude")
    expect(openAgentSignInTerminal.mock.calls[0][1]).toBe("loginabc")
    expect(openTerminalSession).not.toHaveBeenCalled()
    await act(async () => panel.root.unmount())
    panel.container.remove()
  })

  it("returns to the open action without an error when confirmation is canceled", async () => {
    openAgentSignInTerminal.mockResolvedValue(false)
    const panel = mountLogin()
    await act(async () => panel.root.render(<TerminalPanel login={panel.login} />))
    await openLogin(panel.container)

    expect(panel.container.textContent).toContain("Open sign-in terminal")
    expect(panel.container.textContent).not.toContain("Terminal did not open")
    await act(async () => panel.root.unmount())
    panel.container.remove()
  })

  it("gives the login terminal a fixed viewport so xterm fit cannot grow its observer target", async () => {
    const panel = mountLogin()
    await act(async () => panel.root.render(<TerminalPanel login={panel.login} />))
    await openLogin(panel.container)

    const viewport = panel.container.querySelector('[aria-label="Sandbox terminal"]')
    expect(viewport).not.toBeNull()
    expect(viewport?.classList.contains("h-72")).toBe(true)
    expect(viewport?.classList.contains("min-h-72")).toBe(false)

    await act(async () => panel.root.unmount())
    panel.container.remove()
  })

  it("keeps a live login open across callback rerenders and calls the latest callback", async () => {
    let handler: ((message: TerminalMessage) => void) | undefined
    openAgentSignInTerminal.mockImplementation((_agent, _sessionId, onMessage) => {
      handler = onMessage
      return Promise.resolve(true)
    })
    const first = vi.fn()
    const latest = vi.fn()
    const panel = mountLogin(first)
    await act(async () => panel.root.render(<TerminalPanel login={panel.login} />))
    await openLogin(panel.container)
    closeTerminalSession.mockClear()

    await act(async () => {
      panel.root.render(
        createElement(TerminalPanel, {
          login: { ...panel.login, onSignedIn: latest },
        }),
      )
    })
    expect(closeTerminalSession).not.toHaveBeenCalled()
    expect(openAgentSignInTerminal).toHaveBeenCalledTimes(1)

    await act(async () => {
      handler?.({ event: "exit", code: 0, completionRecorded: true })
    })
    expect(first).not.toHaveBeenCalled()
    expect(latest).toHaveBeenCalledTimes(1)
    await act(async () => panel.root.unmount())
    panel.container.remove()
  })

  it("reports when readiness state could not be saved", async () => {
    let handler: ((message: TerminalMessage) => void) | undefined
    openAgentSignInTerminal.mockImplementation((_agent, _sessionId, onMessage) => {
      handler = onMessage
      return Promise.resolve(true)
    })
    const signedIn = vi.fn()
    const panel = mountLogin(signedIn)
    panel.login.agent = "codex"
    panel.login.agentLabel = "Codex"
    await act(async () => panel.root.render(<TerminalPanel login={panel.login} />))
    await openLogin(panel.container)
    await act(async () => {
      handler?.({ event: "exit", code: 0, completionRecorded: false })
    })
    expect(panel.container.textContent).toContain("Sign-in status was not saved")
    expect(panel.container.textContent).toContain("agent may still be signed in")
    expect(signedIn).not.toHaveBeenCalled()
    await act(async () => panel.root.unmount())
    panel.container.remove()
  })
})
