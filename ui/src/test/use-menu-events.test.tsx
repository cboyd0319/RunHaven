// @vitest-environment happy-dom
import { readFileSync } from "node:fs"
import { join } from "node:path"

import { beforeEach, describe, expect, it, vi } from "vitest"

import {
  handleMenuCommand,
  MENU_FOCUS_SELECTORS,
  useMenuEvents,
  type MenuActions,
} from "../app/useMenuEvents"
import type { ManagedSessionSummary } from "../commands/runhaven"

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

function session(patch: Partial<ManagedSessionSummary>): ManagedSessionSummary {
  return {
    id: "s1",
    runId: "run-s1",
    agent: "codex",
    session: "alpha",
    title: "Codex - alpha",
    project: "alpha",
    status: "running",
    statusLabel: "Running",
    authStatus: "ready",
    authScope: "project",
    network: "provider",
    startedAt: "2026-07-01T01:00:00Z",
    containerName: "runhaven-s1",
    detailCommand: "runhaven runs show run-s1",
    lifecycle: "running",
    input: {
      attached: true,
      status: "attached",
      detail: "Session input is ready.",
    },
    nextAction: {
      kind: "monitor",
      label: "Monitor session",
      detail: "RunHaven is tracking this session.",
    },
    events: [],
    ...patch,
  }
}

// Sidebar order for these fixtures: active (newest first) then finished.
const active = session({ id: "a1", startedAt: "2026-07-01T03:00:00Z" })
const activeOlder = session({ id: "a2", startedAt: "2026-07-01T02:00:00Z" })
const failed = session({
  id: "f1",
  lifecycle: "failed",
  startedAt: "2026-07-01T04:00:00Z",
})
const needsSignIn = session({
  id: "a3",
  authStatus: "needs-sign-in",
  startedAt: "2026-07-01T01:00:00Z",
})

function actions(patch: Partial<MenuActions> = {}): MenuActions {
  return {
    navigation: { kind: "home" },
    managedSessions: { sessions: [active, activeOlder, failed] },
    setupReady: true,
    launchFlow: { setOpen: vi.fn() },
    settings: { setOpen: vi.fn() },
    navigate: vi.fn(),
    selectSession: vi.fn(),
    refresh: vi.fn().mockResolvedValue(undefined),
    ...patch,
  }
}

beforeEach(() => {
  capturedHandlers.length = 0
  unlisten.mockClear()
  document.body.innerHTML = ""
})

describe("handleMenuCommand", () => {
  it("opens the launch sheet for new-run when setup is ready", () => {
    const app = actions()
    handleMenuCommand("new-run", app)
    expect(app.launchFlow.setOpen).toHaveBeenCalledWith(true)
    expect(app.navigate).not.toHaveBeenCalled()
  })

  it("routes new-run to Home when setup is blocked", () => {
    const app = actions({ setupReady: false })
    handleMenuCommand("new-run", app)
    expect(app.launchFlow.setOpen).not.toHaveBeenCalled()
    expect(app.navigate).toHaveBeenCalledWith({ kind: "home" })
  })

  it("opens the settings dialog for settings", () => {
    const app = actions()
    handleMenuCommand("settings", app)
    expect(app.settings.setOpen).toHaveBeenCalledWith(true)
    expect(app.launchFlow.setOpen).not.toHaveBeenCalled()
    expect(app.navigate).not.toHaveBeenCalled()
  })

  it("maps navigation ids onto destinations", () => {
    const cases: Array<[string, { kind: string }]> = [
      ["go-home", { kind: "home" }],
      ["go-agents", { kind: "agents" }],
      ["go-diagnostics", { kind: "diagnostics" }],
      ["go-maintenance", { kind: "maintenance" }],
      ["agent-sign-in", { kind: "agents" }],
      ["agent-rebuild-image", { kind: "agents" }],
      ["agent-auth-details", { kind: "agents" }],
    ]
    for (const [id, destination] of cases) {
      const app = actions()
      handleMenuCommand(id, app)
      expect(app.navigate, id).toHaveBeenCalledWith(destination)
    }
  })

  it("selects session N in sidebar order and ignores out-of-range slots", () => {
    const app = actions()
    // Active newest-first (a1, a2), then finished (f1).
    handleMenuCommand("select-session-1", app)
    expect(app.selectSession).toHaveBeenLastCalledWith("a1")
    handleMenuCommand("select-session-3", app)
    expect(app.selectSession).toHaveBeenLastCalledWith("f1")
    handleMenuCommand("select-session-9", app)
    expect(app.selectSession).toHaveBeenCalledTimes(2)
  })

  it("cycles next and previous session with wrap-around", () => {
    const onSecond = actions({ navigation: { kind: "session", id: "a2" } })
    handleMenuCommand("next-session", onSecond)
    expect(onSecond.selectSession).toHaveBeenLastCalledWith("f1")
    handleMenuCommand("previous-session", onSecond)
    expect(onSecond.selectSession).toHaveBeenLastCalledWith("a1")

    const onLast = actions({ navigation: { kind: "session", id: "f1" } })
    handleMenuCommand("next-session", onLast)
    expect(onLast.selectSession).toHaveBeenLastCalledWith("a1")

    const fromHome = actions()
    handleMenuCommand("next-session", fromHome)
    expect(fromHome.selectSession).toHaveBeenLastCalledWith("a1")
    handleMenuCommand("previous-session", fromHome)
    expect(fromHome.selectSession).toHaveBeenLastCalledWith("f1")

    const empty = actions({ managedSessions: { sessions: [] } })
    handleMenuCommand("next-session", empty)
    expect(empty.selectSession).not.toHaveBeenCalled()
  })

  it("jumps to the highest-priority needs-you session, then walks the rest", () => {
    const app = actions({
      managedSessions: { sessions: [active, failed, needsSignIn] },
    })
    // Failed outranks needs-sign-in regardless of sidebar order.
    handleMenuCommand("jump-needs-you", app)
    expect(app.selectSession).toHaveBeenLastCalledWith("f1")

    const onFailed = actions({
      managedSessions: { sessions: [active, failed, needsSignIn] },
      navigation: { kind: "session", id: "f1" },
    })
    handleMenuCommand("jump-needs-you", onFailed)
    expect(onFailed.selectSession).toHaveBeenLastCalledWith("a3")

    const calm = actions({ managedSessions: { sessions: [active] } })
    handleMenuCommand("jump-needs-you", calm)
    expect(calm.selectSession).not.toHaveBeenCalled()
  })

  it("refreshes on check-again", () => {
    const app = actions()
    handleMenuCommand("check-again", app)
    expect(app.refresh).toHaveBeenCalledTimes(1)
  })

  it("focuses the existing confirm control for session actions", () => {
    const checkbox = document.createElement("button")
    checkbox.setAttribute("aria-label", "Confirm stopping this session.")
    document.body.append(checkbox)

    const app = actions({ navigation: { kind: "session", id: "a1" } })
    handleMenuCommand("stop-session", app)
    expect(document.activeElement).toBe(checkbox)
    // Focus only: the stop command itself must stay behind the user's own
    // confirm click plus the Rust-owned native dialog.
    expect(app.selectSession).not.toHaveBeenCalled()
    expect(app.navigate).not.toHaveBeenCalled()
  })

  it("no-ops session actions when the control is not on screen", () => {
    const app = actions()
    for (const id of Object.keys(MENU_FOCUS_SELECTORS)) {
      expect(() => handleMenuCommand(id, app)).not.toThrow()
    }
    expect(app.selectSession).not.toHaveBeenCalled()
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
    expect(app.selectSession).not.toHaveBeenCalled()
    expect(app.launchFlow.setOpen).not.toHaveBeenCalled()
    expect(app.settings.setOpen).not.toHaveBeenCalled()
    expect(app.refresh).not.toHaveBeenCalled()
  })
})

describe("useMenuEvents", () => {
  it("subscribes once, dispatches to the latest actions, and unsubscribes", async () => {
    const app = actions()
    const hook = await renderHook(useMenuEvents, app)
    expect(capturedHandlers).toHaveLength(1)

    capturedHandlers[0]("go-agents")
    expect(app.navigate).toHaveBeenCalledWith({ kind: "agents" })

    const next = actions()
    await hook.rerender(next)
    expect(capturedHandlers).toHaveLength(1)
    capturedHandlers[0]("go-diagnostics")
    expect(next.navigate).toHaveBeenCalledWith({ kind: "diagnostics" })

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

  it("focus selectors match the labels the session view components render", () => {
    const sources = [
      appSource("RunStatusCard.tsx"),
      appSource("SessionInputCard.tsx"),
      appSource("RunOutputCard.tsx"),
    ].join("\n")
    const labels = [
      "Confirm stopping this",
      "Confirm hard-stopping this",
      "Confirm clearing this",
      "Confirm deleting this session's saved worktree.",
      "Type a message for this session.",
      "I understand raw output may include private data.",
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
      "go-home",
      "go-agents",
      "go-diagnostics",
      "go-maintenance",
      "agent-sign-in",
      "agent-rebuild-image",
      "agent-auth-details",
      "next-session",
      "previous-session",
      "jump-needs-you",
      "toggle-sidebar",
      "check-again",
      ...Array.from({ length: 9 }, (_, i) => `select-session-${i + 1}`),
      ...Object.keys(MENU_FOCUS_SELECTORS),
    ]
    for (const id of handledIds) {
      expect(rustMenu, id).toContain(`"${id}"`)
    }
    expect(rustMenu).toContain('"menu://command"')
  })
})
