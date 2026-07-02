// @vitest-environment happy-dom
import { afterEach, beforeEach, describe, expect, it } from "vitest"

import {
  DEFAULT_SETTINGS,
  loadLastDestination,
  loadSettings,
  persistLastDestination,
  persistSettings,
  sidebarDefaultOpen,
} from "../app/uiStatePersistence"

import { installLocalStorageStub } from "./local-storage"

const KEY = "runhaven.lastDestination"
const SETTINGS_KEY = "runhaven.settings"

installLocalStorageStub()

describe("last destination persistence", () => {
  beforeEach(() => {
    window.localStorage.clear()
  })

  it("round-trips system destinations and sessions", () => {
    persistLastDestination({ kind: "diagnostics" })
    expect(loadLastDestination()).toEqual({ kind: "diagnostics" })

    persistLastDestination({ kind: "session", id: "abc123" })
    expect(loadLastDestination()).toEqual({ kind: "session", id: "abc123" })
  })

  it("stores only the destination kind and the opaque session id", () => {
    const destination = {
      kind: "session",
      id: "abc123",
      // Anything beyond kind and id must never reach the store, even if a
      // future NavDestination variant carries more fields.
      project: "/Users/someone/secret-project",
    } as unknown as Parameters<typeof persistLastDestination>[0]
    persistLastDestination(destination)
    expect(JSON.parse(window.localStorage.getItem(KEY) ?? "")).toEqual({
      kind: "session",
      id: "abc123",
    })
  })

  it("reads an empty or malformed store as nothing", () => {
    expect(loadLastDestination()).toBeNull()

    window.localStorage.setItem(KEY, "not json")
    expect(loadLastDestination()).toBeNull()

    window.localStorage.setItem(KEY, JSON.stringify("home"))
    expect(loadLastDestination()).toBeNull()

    window.localStorage.setItem(KEY, JSON.stringify({ kind: "wizard" }))
    expect(loadLastDestination()).toBeNull()
  })

  it("rejects session ids that are not opaque tokens", () => {
    for (const id of [
      "",
      "../../etc/passwd",
      "run id with spaces",
      "a".repeat(65),
      42,
      null,
    ]) {
      window.localStorage.setItem(KEY, JSON.stringify({ kind: "session", id }))
      expect(loadLastDestination()).toBeNull()
    }
  })
})

describe("settings persistence", () => {
  beforeEach(() => {
    window.localStorage.clear()
  })

  it("defaults to badge on and no default agent", () => {
    expect(loadSettings()).toEqual(DEFAULT_SETTINGS)
    expect(DEFAULT_SETTINGS).toEqual({ dockBadge: true, defaultAgent: null })
  })

  it("round-trips the two allowed choices", () => {
    persistSettings({ dockBadge: false, defaultAgent: "codex" })
    expect(loadSettings()).toEqual({ dockBadge: false, defaultAgent: "codex" })

    persistSettings({ dockBadge: true, defaultAgent: null })
    expect(loadSettings()).toEqual({ dockBadge: true, defaultAgent: null })
  })

  it("stores exactly the two allowed fields, nothing extra", () => {
    const settings = {
      dockBadge: false,
      defaultAgent: "codex",
      // Anything beyond the allowed set must never reach the store, even if
      // a future AppSettings variant carries more fields.
      workspacePath: "/Users/someone/secret-project",
    } as unknown as Parameters<typeof persistSettings>[0]
    persistSettings(settings)
    expect(
      JSON.parse(window.localStorage.getItem(SETTINGS_KEY) ?? ""),
    ).toEqual({ dockBadge: false, defaultAgent: "codex" })
  })

  it("reads an empty or malformed store as the defaults", () => {
    for (const raw of ["not json", JSON.stringify("dark"), "42"]) {
      window.localStorage.setItem(SETTINGS_KEY, raw)
      expect(loadSettings()).toEqual(DEFAULT_SETTINGS)
    }
    window.localStorage.setItem(
      SETTINGS_KEY,
      JSON.stringify({ dockBadge: "yes", defaultAgent: 7 }),
    )
    expect(loadSettings()).toEqual(DEFAULT_SETTINGS)
  })

  it("rejects a default agent that is not an opaque token", () => {
    for (const defaultAgent of [
      "",
      "../../etc/passwd",
      "agent name with spaces",
      "a".repeat(65),
    ]) {
      window.localStorage.setItem(
        SETTINGS_KEY,
        JSON.stringify({ dockBadge: true, defaultAgent }),
      )
      expect(loadSettings().defaultAgent).toBeNull()

      persistSettings({ dockBadge: true, defaultAgent })
      expect(
        JSON.parse(window.localStorage.getItem(SETTINGS_KEY) ?? "").defaultAgent,
      ).toBeNull()
    }
  })
})

describe("monitor-window write guard", () => {
  beforeEach(() => {
    window.localStorage.clear()
  })

  afterEach(() => {
    window.history.replaceState(null, "", "/")
  })

  it("never persists a destination from a monitor window", () => {
    window.history.replaceState(null, "", "/?monitor=1")
    persistLastDestination({ kind: "session", id: "abc123" })
    expect(window.localStorage.getItem(KEY)).toBeNull()
  })

  it("never persists settings from a monitor window", () => {
    window.history.replaceState(null, "", "/?monitor=1")
    persistSettings({ dockBadge: false, defaultAgent: "codex" })
    expect(window.localStorage.getItem(SETTINGS_KEY)).toBeNull()
  })

  it("keeps persisting normally in the main window", () => {
    persistLastDestination({ kind: "session", id: "abc123" })
    expect(loadLastDestination()).toEqual({ kind: "session", id: "abc123" })
  })
})

describe("sidebarDefaultOpen", () => {
  it("defaults to open without a cookie", () => {
    document.cookie = "sidebar_state=; max-age=0"
    expect(sidebarDefaultOpen()).toBe(true)
  })

  it("follows the sidebar_state cookie the shadcn provider writes", () => {
    document.cookie = "sidebar_state=false"
    expect(sidebarDefaultOpen()).toBe(false)
    document.cookie = "sidebar_state=true"
    expect(sidebarDefaultOpen()).toBe(true)
  })
})
