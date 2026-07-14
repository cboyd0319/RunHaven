// @vitest-environment happy-dom
import { createRoot } from "react-dom/client"
import { renderToStaticMarkup } from "react-dom/server"
import { describe, expect, it, vi } from "vitest"

import { NewSessionView } from "../app/views/NewSessionView"
import { dashboard, launchFlow, mainViewProps } from "./main-view-props"
import { act } from "./render-hook"

describe("NewSessionView", () => {
  it("presents one semantic three-step session sequence", () => {
    const html = renderToStaticMarkup(<NewSessionView {...mainViewProps()} />)

    expect(html).toContain('aria-label="Session setup progress"')
    expect(html).toContain('aria-current="step"')
    expect(html.indexOf("Choose assistant")).toBeLessThan(
      html.indexOf("Choose project"),
    )
    expect(html.indexOf("Choose project")).toBeLessThan(
      html.indexOf("Review and start"),
    )
    expect(html).toContain("Nothing starts until you review and confirm")
  })

  it("summarizes completed choices and lets people edit them", async () => {
    const flow = launchFlow()
    const setLaunchStep = vi.fn()
    const container = document.createElement("div")
    const root = createRoot(container)

    await act(async () => {
      root.render(
        <NewSessionView
          {...mainViewProps({
            launchFlow: {
              ...flow,
              launchStep: "review",
              request: {
                ...flow.request,
                workspacePath: "/Users/you/quiet-project",
                worktree: true,
              },
              setLaunchStep,
            },
          })}
        />,
      )
    })

    expect(container.textContent).toContain("Codex")
    expect(container.textContent).toContain("quiet-project")
    expect(container.textContent).toContain("Complete")
    const editAssistant = container.querySelector<HTMLButtonElement>(
      'button[aria-label="Edit assistant"]',
    )
    const editProject = container.querySelector<HTMLButtonElement>(
      'button[aria-label="Edit project"]',
    )
    expect(editAssistant).not.toBeNull()
    expect(editProject).not.toBeNull()

    await act(async () => editAssistant?.click())
    expect(setLaunchStep).toHaveBeenCalledWith("agent")
    await act(async () => editProject?.click())
    expect(setLaunchStep).toHaveBeenCalledWith("project")
    await act(async () => root.unmount())
  })

  it("restores focus to Review after native confirmation is canceled", async () => {
    const flow = launchFlow()
    const container = document.createElement("div")
    document.body.append(container)
    const root = createRoot(container)

    await act(async () => {
      root.render(
        <NewSessionView
          {...mainViewProps({
            launchFlow: {
              ...flow,
              launchStep: "review",
              reviewFocusRevision: 1,
              request: {
                ...flow.request,
                workspacePath: "/Users/you/quiet-project",
              },
            },
          })}
        />,
      )
    })

    expect(document.activeElement).toBe(
      container.querySelector('[role="heading"][aria-level="2"]'),
    )
    await act(async () => root.unmount())
    container.remove()
  })

  it("continues from project choices to review without starting", async () => {
    const flow = launchFlow()
    const setLaunchStep = vi.fn()
    const startSession = vi.fn(async () => {})
    const container = document.createElement("div")
    const root = createRoot(container)

    await act(async () => {
      root.render(
        <NewSessionView
          {...mainViewProps({
            launchFlow: {
              ...flow,
              launchStep: "project",
              request: {
                ...flow.request,
                workspacePath: "/Users/you/quiet-project",
              },
              setLaunchStep,
              startSession,
            },
          })}
        />,
      )
    })

    const continueButton = Array.from(
      container.querySelectorAll("button"),
    ).find((button) => button.textContent?.trim() === "Continue")
    expect(continueButton).toBeDefined()
    expect(continueButton?.disabled).toBe(false)
    await act(async () => {
      continueButton
        ?.closest("form")
        ?.dispatchEvent(
          new Event("submit", { bubbles: true, cancelable: true }),
        )
    })

    expect(setLaunchStep).toHaveBeenCalledWith("review")
    expect(startSession).not.toHaveBeenCalled()
    await act(async () => root.unmount())
  })

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

  it("shows setup recovery as the only grouped task when setup is blocked", () => {
    const html = renderToStaticMarkup(
      <NewSessionView
        {...mainViewProps({
          setupReady: false,
          dashboard: {
            ...mainViewProps().dashboard!,
            setup: {
              ...mainViewProps().dashboard!.setup,
              ok: false,
              blockerCount: 1,
            },
          },
        })}
      />,
    )

    expect(html).toContain("Setup needs attention")
    expect(html).not.toContain('aria-label="Session setup progress"')
    expect(html.match(/data-slot="card"/g) ?? []).toHaveLength(1)
  })

  it("focuses the session steps after an explicit setup check succeeds", async () => {
    const container = document.createElement("div")
    document.body.append(container)
    const root = createRoot(container)
    const onRefresh = vi.fn()

    await act(async () => {
      root.render(
        <NewSessionView
          {...mainViewProps({
            setupReady: false,
            dashboard: dashboard(false),
            onRefresh,
          })}
        />,
      )
    })
    const checkAgain = Array.from(container.querySelectorAll("button")).find(
      (button) => button.textContent?.trim() === "Check again",
    )
    await act(async () => checkAgain?.click())
    expect(onRefresh).toHaveBeenCalledOnce()

    await act(async () => {
      root.render(
        <NewSessionView
          {...mainViewProps({
            setupReady: true,
            dashboard: dashboard(true),
            onRefresh,
          })}
        />,
      )
    })
    expect(document.activeElement).toBe(
      container.querySelector('[role="heading"][aria-level="2"]'),
    )

    await act(async () => root.unmount())
    container.remove()
  })

  it("does not move focus when background setup refresh becomes ready", async () => {
    const container = document.createElement("div")
    document.body.append(container)
    const root = createRoot(container)
    const sentinel = document.createElement("button")
    document.body.append(sentinel)
    sentinel.focus()

    await act(async () => {
      root.render(
        <NewSessionView
          {...mainViewProps({
            setupReady: false,
            dashboard: dashboard(false),
          })}
        />,
      )
    })
    await act(async () => {
      root.render(
        <NewSessionView
          {...mainViewProps({
            setupReady: true,
            dashboard: dashboard(true),
          })}
        />,
      )
    })
    expect(document.activeElement).toBe(sentinel)

    await act(async () => root.unmount())
    sentinel.remove()
    container.remove()
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

    expect(setLaunchStep).toHaveBeenCalledWith("agent")
    expect(setOpen).not.toHaveBeenCalled()
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
    const lessSafe = html.indexOf(
      "Let assistant change original files (less safe)",
    )
    expect(safe).toBeGreaterThanOrEqual(0)
    expect(lessSafe).toBeGreaterThan(safe)
    expect(html).not.toContain("commit")
    expect(html).not.toContain("stash")
    expect(html).not.toContain("git")
    expect(html).not.toContain("worktree")
  })
})
