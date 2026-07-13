// @vitest-environment happy-dom
import { createRoot } from "react-dom/client"
import { renderToStaticMarkup } from "react-dom/server"
import { describe, expect, it, vi } from "vitest"

import { HomeView } from "../app/views/HomeView"
import type { AgentProfile } from "../commands/runhaven"

import { dashboard, mainViewProps, managedSession } from "./main-view-props"
import { providerAgent } from "./runhaven-fixtures"
import { act } from "./render-hook"

const agentNames = ["claude", "codex", "antigravity", "copilot"] as const

function agent(name: (typeof agentNames)[number]): AgentProfile {
  return {
    ...providerAgent,
    name,
    description: `${name} coding agent`,
  }
}

function launchpadProps() {
  return mainViewProps({
    dashboard: {
      ...dashboard(true),
      agents: agentNames.map(agent),
    },
  })
}

describe("HomeView", () => {
  it("is an assistant launchpad with one clear choice per assistant", () => {
    const html = renderToStaticMarkup(<HomeView {...launchpadProps()} />)

    expect(html).toContain("Start a session")
    expect(html).not.toContain("Claude Code")
    expect(html).not.toContain("OpenAI Codex")
    expect(html).not.toContain("Google Antigravity")
    expect(html).not.toContain("GitHub Copilot")
    expect(html).not.toContain("Start with")
    expect(html.match(/data-agent-launch=/g) ?? []).toHaveLength(4)
    for (const monogram of ["CL", "CO", "AG", "CP"]) {
      expect(html).toContain(`>${monogram}</span>`)
    }
    expect(html).not.toContain("System ready")
    expect(html).not.toContain("Needs you")
    expect(html).not.toContain("Recent work")
    expect(html).not.toContain("Run history")
    expect(html).not.toContain("Setup checks")
    expect(html).toContain("Choose the assistant account you already use.")
    expect(html).toContain("You can go back without starting anything.")
    expect(html).toContain("Not sure?")
  })

  it("opens Help without starting a session", async () => {
    const onNavigate = vi.fn()
    const changeAgent = vi.fn()
    const container = document.createElement("div")
    const root = createRoot(container)
    await act(async () => {
      root.render(
        <HomeView
          {...launchpadProps()}
          launchFlow={{ ...launchpadProps().launchFlow, changeAgent }}
          onNavigate={onNavigate}
        />,
      )
    })

    const help = Array.from(container.querySelectorAll("button")).find(
      (button) => button.textContent?.trim() === "Not sure?",
    )
    expect(help).toBeDefined()
    await act(async () => help?.click())

    expect(onNavigate).toHaveBeenCalledWith({
      kind: "help",
      returnTo: { kind: "home" },
    })
    expect(changeAgent).not.toHaveBeenCalled()
    await act(async () => root.unmount())
  })

  it("does not put run failures or history on the landing screen", () => {
    const html = renderToStaticMarkup(
      <HomeView
        {...launchpadProps()}
        sessions={[
          managedSession({
            id: "failed",
            title: "Codex - failing",
            lifecycle: "failed",
          }),
          managedSession({ id: "active", title: "Claude - active" }),
        ]}
      />,
    )

    expect(html).not.toContain("Codex - failing")
    expect(html).not.toContain("Claude - active")
    expect(html).not.toContain("failed")
  })

  it("keeps the launchpad calm when system setup needs work", () => {
    const html = renderToStaticMarkup(
      <HomeView
        {...launchpadProps()}
        setupReady={false}
        dashboard={{ ...launchpadProps().dashboard!, setup: dashboard(false).setup }}
      />,
    )

    expect(html).toContain(">Claude<")
    expect(html).not.toContain("Apple container is not installed")
    expect(html).not.toContain("Setup needs attention")
    expect(html).not.toContain('disabled=""')
  })

  it("keeps the selected agent and moves directly to the project step", async () => {
    const changeAgent = vi.fn()
    const setLaunchStep = vi.fn()
    const onNavigate = vi.fn()
    const container = document.createElement("div")
    const root = createRoot(container)
    await act(async () => {
      root.render(
        <HomeView
          {...launchpadProps()}
          launchFlow={{
            ...launchpadProps().launchFlow,
            changeAgent,
            setLaunchStep,
          }}
          onNavigate={onNavigate}
        />,
      )
    })

    const firstStart = Array.from(container.querySelectorAll("button")).find(
      (button) => button.textContent?.includes("Claude"),
    )
    expect(firstStart).toBeDefined()
    await act(async () => firstStart?.click())

    expect(changeAgent).toHaveBeenCalledWith("claude")
    expect(setLaunchStep).toHaveBeenCalledWith("project")
    expect(onNavigate).toHaveBeenCalledWith({ kind: "new-session" })
    await act(async () => root.unmount())
  })
})
