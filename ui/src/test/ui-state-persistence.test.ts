// @vitest-environment happy-dom
import { afterEach, beforeEach, describe, expect, it } from "vitest"

import {
  DEFAULT_SETTINGS,
  MAX_SESSION_LABEL_LENGTH,
  loadLastDestination,
  loadSessionLabels,
  loadSettings,
  persistLastDestination,
  persistSessionLabels,
  persistSettings,
  sanitizeSessionLabel,
  sidebarDefaultOpen,
} from "../app/uiStatePersistence"

import { installLocalStorageStub } from "./local-storage"

const KEY = "runhaven.lastDestination"
const SETTINGS_KEY = "runhaven.settings"
const SESSION_LABELS_KEY = "runhaven.sessionLabels"

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

describe("session label sanitization", () => {
  it("strips control characters and trims", () => {
    expect(sanitizeSessionLabel("  Fix\tlogin\nbug \r")).toBe("Fixloginbug")
    expect(sanitizeSessionLabel("clean name")).toBe("clean name")
    expect(sanitizeSessionLabel("withdel")).toBe("withdel")
  })

  it("treats a blank or whitespace-only label as empty", () => {
    expect(sanitizeSessionLabel("")).toBe("")
    expect(sanitizeSessionLabel("   ")).toBe("")
    expect(sanitizeSessionLabel("\n\t")).toBe("")
  })

  it("caps the label length", () => {
    const long = "a".repeat(MAX_SESSION_LABEL_LENGTH + 40)
    expect(sanitizeSessionLabel(long)).toHaveLength(MAX_SESSION_LABEL_LENGTH)
  })
})

describe("session label persistence", () => {
  beforeEach(() => {
    window.localStorage.clear()
  })

  it("round-trips labels keyed by the opaque session id", () => {
    persistSessionLabels({ abc123: "Fix the login bug" })
    expect(loadSessionLabels()).toEqual({ abc123: "Fix the login bug" })
  })

  it("sanitizes on write and drops blank labels", () => {
    persistSessionLabels({
      abc123: "  Trimmed\tname  ",
      def456: "   ",
    })
    expect(loadSessionLabels()).toEqual({ abc123: "Trimmedname" })
  })

  it("rejects keys that are not opaque session ids", () => {
    persistSessionLabels({
      "../../etc/passwd": "evil",
      "id with spaces": "bad",
      abc123: "ok",
    })
    expect(loadSessionLabels()).toEqual({ abc123: "ok" })
  })

  it("reads an empty, malformed, or non-string store as no labels", () => {
    expect(loadSessionLabels()).toEqual({})

    window.localStorage.setItem(SESSION_LABELS_KEY, "not json")
    expect(loadSessionLabels()).toEqual({})

    window.localStorage.setItem(SESSION_LABELS_KEY, JSON.stringify("home"))
    expect(loadSessionLabels()).toEqual({})

    window.localStorage.setItem(
      SESSION_LABELS_KEY,
      JSON.stringify({ abc123: 42, def456: null }),
    )
    expect(loadSessionLabels()).toEqual({})
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

  it("never persists session labels from a monitor window", () => {
    window.history.replaceState(null, "", "/?monitor=1")
    persistSessionLabels({ abc123: "Fix the login bug" })
    expect(window.localStorage.getItem(SESSION_LABELS_KEY)).toBeNull()
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
