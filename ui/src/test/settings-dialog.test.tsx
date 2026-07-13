// @vitest-environment happy-dom
import { renderToStaticMarkup } from "react-dom/server"
import { beforeEach, describe, expect, it } from "vitest"

import { SettingsForm } from "../app/SettingsDialog"
import { useSettings, type SettingsState } from "../app/useSettings"

import { installLocalStorageStub } from "./local-storage"
import { act, renderHook } from "./render-hook"
import { localAgent, providerAgent } from "./runhaven-fixtures"

installLocalStorageStub()

function state(patch: Partial<SettingsState> = {}): SettingsState {
  return {
    open: true,
    setOpen: () => {},
    dockBadge: true,
    defaultAgent: null,
    setDockBadge: () => {},
    setDefaultAgent: () => {},
    ...patch,
  }
}

describe("SettingsForm", () => {
  it("offers the convenience settings in plain words", () => {
    const markup = renderToStaticMarkup(
      <SettingsForm
        settings={state()}
        agents={[providerAgent, localAgent]}
      />,
    )
    expect(markup).toContain("Show a number on the app icon")
    expect(markup).toContain("The number counts sessions that need you.")
    expect(markup).toContain("Start new runs with")
    expect(markup).toContain("You can still change")
    // Appearance is honest prose, not a dead control.
    expect(markup).toContain("follows your Mac")
  })

  it("reflects the saved choices and shows no paths or commands", () => {
    const markup = renderToStaticMarkup(
      <SettingsForm
        settings={state({ dockBadge: false, defaultAgent: "codex" })}
        agents={[providerAgent]}
      />,
    )
    expect(markup).toContain('data-state="unchecked"')
    expect(markup).not.toContain("/Users")
    expect(markup).not.toContain("container ")
  })

  it("checks the badge box when the badge is on", () => {
    const markup = renderToStaticMarkup(
      <SettingsForm settings={state()} agents={[providerAgent]} />,
    )
    expect(markup).toContain('data-state="checked"')
  })
})

describe("useSettings", () => {
  beforeEach(() => {
    window.localStorage.clear()
  })

  it("starts closed with the defaults and opens on request", async () => {
    const { result } = await renderHook(() => useSettings(), undefined)
    expect(result.current.open).toBe(false)
    expect(result.current.dockBadge).toBe(true)
    expect(result.current.defaultAgent).toBeNull()

    await act(async () => {
      result.current.setOpen(true)
    })
    expect(result.current.open).toBe(true)
  })

  it("persists each choice so a fresh mount reads it back", async () => {
    const first = await renderHook(() => useSettings(), undefined)
    await act(async () => {
      first.result.current.setDockBadge(false)
    })
    await act(async () => {
      first.result.current.setDefaultAgent("codex")
    })
    expect(first.result.current.dockBadge).toBe(false)
    expect(first.result.current.defaultAgent).toBe("codex")
    await first.unmount()

    const second = await renderHook(() => useSettings(), undefined)
    expect(second.result.current.dockBadge).toBe(false)
    expect(second.result.current.defaultAgent).toBe("codex")
  })

  it("clears the default agent back to choose-for-me", async () => {
    const { result } = await renderHook(() => useSettings(), undefined)
    await act(async () => {
      result.current.setDefaultAgent("codex")
    })
    await act(async () => {
      result.current.setDefaultAgent(null)
    })
    expect(result.current.defaultAgent).toBeNull()
  })
})
