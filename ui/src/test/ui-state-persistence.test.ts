// @vitest-environment happy-dom
import { beforeEach, describe, expect, it } from "vitest"

import {
  loadLastDestination,
  persistLastDestination,
  sidebarDefaultOpen,
} from "../app/uiStatePersistence"

import { installLocalStorageStub } from "./local-storage"

const KEY = "runhaven.lastDestination"

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
