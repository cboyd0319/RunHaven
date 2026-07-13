// @vitest-environment happy-dom
import { describe, expect, it } from "vitest"

import {
  loadTextSize,
  persistTextSize,
  sidebarDefaultOpen,
} from "../app/uiStatePersistence"

import { installLocalStorageStub } from "./local-storage"

installLocalStorageStub()

describe("text size persistence", () => {
  it("round-trips only the bounded accessibility scale", () => {
    persistTextSize(200)
    expect(loadTextSize()).toBe(200)
  })

  it("falls back to actual size for corrupt or unsupported values", () => {
    for (const value of ["huge", "90", "201", "null"]) {
      window.localStorage.setItem("runhaven.text-size", value)
      expect(loadTextSize()).toBe(100)
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
