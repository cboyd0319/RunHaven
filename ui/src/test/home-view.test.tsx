import { renderToStaticMarkup } from "react-dom/server"
import { describe, expect, it } from "vitest"

import { HomeView } from "../app/views/HomeView"
import type { AgentProfile } from "../commands/runhaven"

import { dashboard, mainViewProps, managedSession } from "./main-view-props"
import { providerAgent } from "./runhaven-fixtures"

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
  it("is an agent launchpad with one clear start action per agent", () => {
    const html = renderToStaticMarkup(<HomeView {...launchpadProps()} />)

    expect(html).toContain("Choose an agent")
    for (const name of ["Claude", "Codex", "Antigravity", "Copilot"]) {
      expect(html).toContain(`Start with ${name}`)
    }
    expect(html).not.toContain("System ready")
    expect(html).not.toContain("Needs you")
    expect(html).not.toContain("Recent work")
    expect(html).not.toContain("Run history")
    expect(html).not.toContain("Setup checks")
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

    expect(html).toContain("Start with Claude")
    expect(html).not.toContain("Apple container is not installed")
    expect(html).not.toContain("Setup needs attention")
    expect(html).not.toContain('disabled=""')
  })
})
