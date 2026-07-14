// @vitest-environment happy-dom
import { readFileSync } from "node:fs"
import { join } from "node:path"

import { createElement } from "react"
import { createRoot } from "react-dom/client"
import { beforeEach, describe, expect, it, vi } from "vitest"

import { SessionInputCard } from "../app/SessionInputCard"
import {
  handleMenuCommand,
  MENU_CONTROL_ACTIONS,
  MENU_FOCUS_SELECTORS,
  useMenuEvents,
  type MenuActions,
} from "../app/useMenuEvents"

import { managedSession } from "./main-view-props"
import { act, renderHook } from "./render-hook"
import { installLocalStorageStub } from "./local-storage"

installLocalStorageStub()

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
    navigation: { kind: "new-session" },
    launchFlow: { setOpen: vi.fn() },
    navigate: vi.fn(),
    runControl: vi.fn().mockResolvedValue(undefined),
    supportReport: { create: vi.fn().mockResolvedValue(undefined) },
    ...patch,
  }
}

beforeEach(() => {
  capturedHandlers.length = 0
  unlisten.mockClear()
  document.body.innerHTML = ""
  document.documentElement.style.fontSize = ""
  window.localStorage.clear()
})

describe("handleMenuCommand", () => {
  it("opens the first-class New Session page when setup is ready", () => {
    const app = actions()
    handleMenuCommand("new-run", app)
    expect(app.launchFlow.setOpen).toHaveBeenCalledWith(false)
    expect(app.navigate).toHaveBeenCalledWith({ kind: "new-session" })
  })

  it("opens the in-app guide from RunHaven Help", () => {
    const app = actions()
    handleMenuCommand("show-help", app)
    expect(app.navigate).toHaveBeenCalledWith({
      kind: "help",
      returnTo: { kind: "new-session" },
    })
  })

  it("keeps the current destination when RunHaven Help opens", () => {
    const app = actions({ navigation: { kind: "new-session" } })
    handleMenuCommand("show-help", app)
    expect(app.navigate).toHaveBeenCalledWith({
      kind: "help",
      returnTo: { kind: "new-session" },
    })
  })

  it("runs Get Support through the shared support controller", () => {
    const app = actions()
    handleMenuCommand("get-support", app)
    expect(app.supportReport.create).toHaveBeenCalledTimes(1)
  })

  it("runs selected-session menu controls through the trusted command path", () => {
    const app = actions({ navigation: { kind: "session", id: "a1" } })
    handleMenuCommand("stop-session", app)
    handleMenuCommand("hard-stop-session", app)
    handleMenuCommand("cleanup-session", app)
    expect(app.runControl).toHaveBeenNthCalledWith(1, "stop")
    expect(app.runControl).toHaveBeenNthCalledWith(2, "kill")
    expect(app.runControl).toHaveBeenNthCalledWith(3, "cleanup")
    expect(app.navigate).not.toHaveBeenCalled()
  })

  it("focuses the rendered composer for Send Message without sending", async () => {
    const container = document.createElement("div")
    document.body.append(container)
    const root = createRoot(container)
    const onSend = vi.fn()
    await act(async () => {
      root.render(createElement(SessionInputCard, {
        session: managedSession(),
        draft: "draft stays put",
        sending: false,
        error: "",
        acceptedMessage: "",
        onDraftChange: vi.fn(),
        onSend,
      }))
    })
    const composer = container.querySelector("textarea")

    handleMenuCommand("send-message", actions())
    expect(document.activeElement).toBe(composer)
    expect(onSend).not.toHaveBeenCalled()
    await act(async () => root.unmount())
  })

  it("no-ops session actions when the control is not on screen", () => {
    const app = actions()
    for (const id of [
      ...Object.keys(MENU_FOCUS_SELECTORS),
      ...Object.keys(MENU_CONTROL_ACTIONS),
    ]) {
      expect(() => handleMenuCommand(id, app)).not.toThrow()
    }
    expect(app.navigate).not.toHaveBeenCalled()
    expect(app.runControl).not.toHaveBeenCalled()
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

  it("supports standard text sizing through the View menu", () => {
    handleMenuCommand("increase-text-size", actions())
    expect(document.documentElement.style.fontSize).toBe("125%")
    expect(window.localStorage.getItem("runhaven.text-size")).toBe("125")
    handleMenuCommand("increase-text-size", actions())
    expect(document.documentElement.style.fontSize).toBe("150%")
    handleMenuCommand("decrease-text-size", actions())
    expect(document.documentElement.style.fontSize).toBe("125%")
    handleMenuCommand("actual-text-size", actions())
    expect(document.documentElement.style.fontSize).toBe("100%")
  })

  it("moves one step from the lower text-size bound", () => {
    document.documentElement.style.fontSize = "80%"
    handleMenuCommand("increase-text-size", actions())
    expect(document.documentElement.style.fontSize).toBe("100%")

    document.documentElement.style.fontSize = "80%"
    handleMenuCommand("decrease-text-size", actions())
    expect(document.documentElement.style.fontSize).toBe("80%")
  })

  it("ignores unknown and placeholder ids", () => {
    const app = actions()
    for (const id of ["filter-sessions", "help", "not-a-real-id"]) {
      handleMenuCommand(id, app)
    }
    expect(app.navigate).not.toHaveBeenCalled()
    expect(app.launchFlow.setOpen).not.toHaveBeenCalled()
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
    capturedHandlers[0]("show-help")
    expect(next.navigate).toHaveBeenCalledWith({
      kind: "help",
      returnTo: { kind: "new-session" },
    })

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
    const sources = appSource("SessionInputCard.tsx")
    const labels = ["Type a message."]
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
      "show-help",
      "get-support",
      "toggle-sidebar",
      "increase-text-size",
      "decrease-text-size",
      "actual-text-size",
      ...Object.keys(MENU_FOCUS_SELECTORS),
      ...Object.keys(MENU_CONTROL_ACTIONS),
    ]
    for (const id of handledIds) {
      expect(rustMenu, id).toContain(`"${id}"`)
    }
    expect(rustMenu).toContain('"menu://command"')
  })
})
