// @vitest-environment happy-dom
import { readFileSync } from "node:fs"
import { join } from "node:path"

import { beforeEach, describe, expect, it, vi } from "vitest"

import {
  handleMenuCommand,
  MENU_ACTION_SELECTORS,
  MENU_FOCUS_SELECTORS,
  useMenuEvents,
  type MenuActions,
} from "../app/useMenuEvents"

import { renderHook } from "./render-hook"

// vi.mock factories are hoisted above module-level const declarations, so the
// shared spies must be hoisted with them.
const { capturedHandlers, unlisten } = vi.hoisted(() => ({
  capturedHandlers: [] as Array<(id: string) => void>,
  unlisten: vi.fn(),
}))

vi.mock("@/commands/menu_events", () => ({
  MENU_COMMAND_EVENT: "menu://command",
  listenForMenuCommands: (handler: (id: string) => void) => {
    capturedHandlers.push(handler)
    return Promise.resolve(unlisten)
  },
}))

function actions(patch: Partial<MenuActions> = {}): MenuActions {
  return {
    navigation: { kind: "home" },
    launchFlow: { setOpen: vi.fn() },
    settings: { setOpen: vi.fn() },
    navigate: vi.fn(),
    openSessionInNewWindow: vi.fn(),
    ...patch,
  }
}

beforeEach(() => {
  capturedHandlers.length = 0
  unlisten.mockClear()
  document.body.innerHTML = ""
})

describe("handleMenuCommand", () => {
  it("opens the first-class New Session page when setup is ready", () => {
    const app = actions()
    handleMenuCommand("new-run", app)
    expect(app.launchFlow.setOpen).toHaveBeenCalledWith(false)
    expect(app.navigate).toHaveBeenCalledWith({ kind: "new-session" })
  })

  it("opens inline settings", () => {
    const app = actions()
    handleMenuCommand("settings", app)
    expect(app.settings.setOpen).toHaveBeenCalledWith(true)
    expect(app.launchFlow.setOpen).not.toHaveBeenCalled()
    expect(app.navigate).not.toHaveBeenCalled()
  })

  it("opens the monitor window only for the selected session", () => {
    const selected = actions({ navigation: { kind: "session", id: "a1" } })
    handleMenuCommand("open-session-window", selected)
    expect(selected.openSessionInNewWindow).toHaveBeenCalledWith("a1")
    // Read-only window creation only: nothing else moves.
    expect(selected.navigate).not.toHaveBeenCalled()

    // Without a selected session there is nothing to open; never guess one.
    for (const nav of [
      { kind: "home" } as const,
      { kind: "runs" } as const,
    ]) {
      const app = actions({ navigation: nav })
      handleMenuCommand("open-session-window", app)
      expect(app.openSessionInNewWindow).not.toHaveBeenCalled()
    }
  })

  it("activates an existing session action so its native confirmation opens", () => {
    const button = document.createElement("button")
    const onClick = vi.fn()
    button.setAttribute("aria-label", "Stop run")
    button.addEventListener("click", onClick)
    document.body.append(button)

    const app = actions({ navigation: { kind: "session", id: "a1" } })
    handleMenuCommand("stop-session", app)
    expect(onClick).toHaveBeenCalledTimes(1)
    // The button calls the normal command, whose Rust-owned native dialog is
    // still the mutation boundary.
    expect(app.navigate).not.toHaveBeenCalled()
  })

  it("focuses the composer for Send Message without sending an empty draft", () => {
    const composer = document.createElement("textarea")
    composer.placeholder = "Type a message for this run."
    document.body.append(composer)

    handleMenuCommand("send-message", actions())
    expect(document.activeElement).toBe(composer)
  })

  it("no-ops session actions when the control is not on screen", () => {
    const app = actions()
    for (const id of [
      ...Object.keys(MENU_FOCUS_SELECTORS),
      ...Object.keys(MENU_ACTION_SELECTORS),
    ]) {
      expect(() => handleMenuCommand(id, app)).not.toThrow()
    }
    expect(app.navigate).not.toHaveBeenCalled()
  })

  it("clicks the existing sidebar trigger for toggle-sidebar", () => {
    const trigger = document.createElement("button")
    trigger.setAttribute("data-sidebar", "trigger")
    const onClick = vi.fn()
    trigger.addEventListener("click", onClick)
    document.body.append(trigger)

    handleMenuCommand("toggle-sidebar", actions())
    expect(onClick).toHaveBeenCalledTimes(1)
  })

  it("ignores unknown and placeholder ids", () => {
    const app = actions()
    for (const id of ["filter-sessions", "help", "not-a-real-id"]) {
      handleMenuCommand(id, app)
    }
    expect(app.navigate).not.toHaveBeenCalled()
    expect(app.launchFlow.setOpen).not.toHaveBeenCalled()
    expect(app.settings.setOpen).not.toHaveBeenCalled()
  })
})

describe("useMenuEvents", () => {
  it("subscribes once, dispatches to the latest actions, and unsubscribes", async () => {
    const app = actions()
    const hook = await renderHook(useMenuEvents, app)
    expect(capturedHandlers).toHaveLength(1)

    capturedHandlers[0]("new-run")
    expect(app.navigate).toHaveBeenCalledWith({ kind: "new-session" })

    const next = actions()
    await hook.rerender(next)
    expect(capturedHandlers).toHaveLength(1)
    capturedHandlers[0]("settings")
    expect(next.settings.setOpen).toHaveBeenCalledWith(true)

    await hook.unmount()
    expect(unlisten).toHaveBeenCalledTimes(1)
  })
})

describe("menu contract tripwires", () => {
  // Vitest runs from ui/, and this file's environment is happy-dom, so
  // import.meta.url is not a file URL; resolve from the working directory.
  function appSource(name: string): string {
    return readFileSync(join(process.cwd(), "src", "app", name), "utf8")
  }

  it("session menu selectors match labels the session view components render", () => {
    const sources = [
      appSource("RunStatusCard.tsx"),
      appSource("SessionInputCard.tsx"),
      appSource("RunOutputCard.tsx"),
    ].join("\n")
    const labels = [
      'aria-label="Stop run"',
      'aria-label="Hard stop"',
      'aria-label="Clean up run"',
      "Type a message for this run.",
      "Show latest output",
    ]
    for (const label of labels) {
      expect(sources, label).toContain(label)
    }
  })

  it("every frontend-handled id exists in the Rust menu definition", () => {
    const rustMenuDir = join(process.cwd(), "..", "src-tauri", "src", "menu")
    const rustMenu =
      readFileSync(join(rustMenuDir, "spec.rs"), "utf8") +
      readFileSync(join(rustMenuDir, "mod.rs"), "utf8")
    const handledIds = [
      "new-run",
      "settings",
      "toggle-sidebar",
      "open-session-window",
      ...Object.keys(MENU_FOCUS_SELECTORS),
      ...Object.keys(MENU_ACTION_SELECTORS),
    ]
    for (const id of handledIds) {
      expect(rustMenu, id).toContain(`"${id}"`)
    }
    expect(rustMenu).toContain('"menu://command"')
  })
})
