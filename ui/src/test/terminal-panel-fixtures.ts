// Shared, non-hoisted helpers for the TerminalPanel test files. The
// `vi.mock`/`vi.hoisted` fake-Terminal wiring must stay in each spec file
// (vitest hoists those above imports and applies them per file), but the type,
// launch builder, and render helpers below are plain module code and are safe
// to import. TerminalPanel loaded through this module still receives the
// per-spec `@/commands/runhaven` mock because the mock covers the whole test
// file's module graph.
import { createElement } from "react"
import { createRoot } from "react-dom/client"
import { vi } from "vitest"

import { TerminalPanel } from "../app/TerminalPanel"
import { act } from "./render-hook"
import { managedSession, planRequest } from "./main-view-props"

// happy-dom exposes navigator.clipboard as a getter-only property, so a plain
// assignment throws; install a fake through defineProperty instead. readText
// resolves `readValue` (the pasted text); the returned writeText spy captures
// what Cmd-C copied for the caller to assert on.
export function stubClipboard(readValue: string) {
  const writeText = vi.fn(() => Promise.resolve())
  Object.defineProperty(navigator, "clipboard", {
    configurable: true,
    value: { writeText, readText: () => Promise.resolve(readValue) },
  })
  return writeText
}

// The shape of the fake xterm Terminal captured by the per-spec mock. Used both
// by the hoisted `xtermState` container and by the assertions in each spec.
export type FakeTerminalLike = {
  cols: number
  rows: number
  options: Record<string, unknown>
  writes: string[]
  disposed: boolean
  blurred: boolean
  dataHandlers: ((data: string) => void)[]
  keyEventHandler: ((event: KeyboardEvent) => boolean) | undefined
  getSelection: () => string
}

// A reviewed, confirmed interactive launch as the run panel receives it.
export function runLaunch() {
  return {
    sessionId: "runabc123",
    request: {
      plan: { ...planRequest(), agent: "claude" },
    },
  }
}

// Mounts TerminalPanel for a managed session, clicks "Open terminal" to
// activate the panel, and flushes the async xterm load + open.
export async function mountActiveTerminal(): Promise<{
  container: HTMLDivElement
  unmount: () => Promise<void>
}> {
  const container = document.createElement("div")
  document.body.appendChild(container)
  const root = createRoot(container)
  await act(async () => {
    root.render(createElement(TerminalPanel, { session: managedSession() }))
  })
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
