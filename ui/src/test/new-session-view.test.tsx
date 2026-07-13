// @vitest-environment happy-dom
import { createRoot } from "react-dom/client"
import { renderToStaticMarkup } from "react-dom/server"
import { describe, expect, it, vi } from "vitest"

import { NewSessionView } from "../app/views/NewSessionView"
import { launchFlow, mainViewProps } from "./main-view-props"
import { act } from "./render-hook"

describe("NewSessionView", () => {
  it("marks the folder picker as requiring more input", () => {
    const flow = launchFlow()
    const html = renderToStaticMarkup(
      <NewSessionView
        {...mainViewProps({
          launchFlow: {
            ...flow,
            launchStep: "project",
          },
          savedSetups: {
            ...mainViewProps().savedSetups,
            error: "Project folder can’t be found. Choose it again.",
          },
        })}
      />,
    )

    expect(html).toContain("Choose folder again…")
  })

  it("returns to the assistant step without leaving New session", async () => {
    const flow = launchFlow()
    const setLaunchStep = vi.fn()
    const setOpen = vi.fn()
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
              setOpen,
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

    expect(setOpen).toHaveBeenCalledWith(false)
    expect(onNavigate).not.toHaveBeenCalled()
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
