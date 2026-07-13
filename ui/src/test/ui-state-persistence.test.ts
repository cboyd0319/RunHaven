// @vitest-environment happy-dom
import { beforeEach, describe, expect, it } from "vitest"

import {
  DEFAULT_SETTINGS,
  loadSettings,
  persistSettings,
  sidebarDefaultOpen,
} from "../app/uiStatePersistence"

import { installLocalStorageStub } from "./local-storage"

const SETTINGS_KEY = "runhaven.settings"

installLocalStorageStub()

describe("settings persistence", () => {
  beforeEach(() => {
    window.localStorage.clear()
  })

  it("defaults to the Dock badge on", () => {
    expect(loadSettings()).toEqual(DEFAULT_SETTINGS)
    expect(DEFAULT_SETTINGS).toEqual({
      dockBadge: true,
    })
  })

  it("round-trips the allowed choice", () => {
    persistSettings({ dockBadge: false })
    expect(loadSettings()).toEqual({
      dockBadge: false,
    })

    persistSettings({ dockBadge: true })
    expect(loadSettings()).toEqual({
      dockBadge: true,
    })
  })

  it("stores exactly the allowed fields, nothing extra", () => {
    const settings = {
      dockBadge: false,
      // Anything beyond the allowed set must never reach the store, even if
      // a future AppSettings variant carries more fields.
      workspacePath: "/Users/someone/secret-project",
    } as unknown as Parameters<typeof persistSettings>[0]
    persistSettings(settings)
    expect(
      JSON.parse(window.localStorage.getItem(SETTINGS_KEY) ?? ""),
    ).toEqual({ dockBadge: false })
  })

  it("reads an empty or malformed store as the defaults", () => {
    for (const raw of ["not json", JSON.stringify("dark"), "42"]) {
      window.localStorage.setItem(SETTINGS_KEY, raw)
      expect(loadSettings()).toEqual(DEFAULT_SETTINGS)
    }
    window.localStorage.setItem(
      SETTINGS_KEY,
      JSON.stringify({ dockBadge: "yes" }),
    )
    expect(loadSettings()).toEqual(DEFAULT_SETTINGS)
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
