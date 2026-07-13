// @vitest-environment happy-dom
import { createRoot } from "react-dom/client"
import { renderToStaticMarkup } from "react-dom/server"
import { describe, expect, it, vi } from "vitest"

import { NewSessionView } from "../app/views/NewSessionView"
import { launchFlow, mainViewProps } from "./main-view-props"
import { act } from "./render-hook"

describe("NewSessionView", () => {
  it("returns Home from Project instead of reopening an agent picker", async () => {
    const flow = launchFlow()
    const setLaunchStep = vi.fn()
    const onNavigate = vi.fn()
    const container = document.createElement("div")
    const root = createRoot(container)

    await act(async () => {
      root.render(
        <NewSessionView
          {...mainViewProps({
            launchFlow: {
              ...flow,
              launchStep: "project",
              setLaunchStep,
            },
            onNavigate,
          })}
        />,
      )
    })

    const back = Array.from(container.querySelectorAll("button")).find(
      (button) => button.textContent?.trim() === "Back",
    )
    expect(back).toBeDefined()
    await act(async () => back?.click())

    expect(onNavigate).toHaveBeenCalledWith({ kind: "home" })
    expect(setLaunchStep).not.toHaveBeenCalledWith("agent")
    await act(async () => root.unmount())
  })

  it("offers a safe read-only fallback before the warned original-folder choice", () => {
    const flow = launchFlow()
    const html = renderToStaticMarkup(
      <NewSessionView
        {...mainViewProps({
          launchFlow: {
            ...flow,
            request: { ...flow.request, worktree: true },
            launchStep: "project",
            error:
              "RunHaven can't make a protected copy while this project has unfinished changes.",
          },
        })}
      />,
    )

    const safe = html.indexOf("Start without changing files")
    const lessSafe = html.indexOf("Let assistant change original files (less safe)")
    expect(safe).toBeGreaterThanOrEqual(0)
    expect(lessSafe).toBeGreaterThan(safe)
    expect(html).not.toContain("commit")
    expect(html).not.toContain("stash")
    expect(html).not.toContain("git")
    expect(html).not.toContain("worktree")
  })
})
