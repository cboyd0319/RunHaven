import { renderToStaticMarkup } from "react-dom/server"
import { describe, expect, it } from "vitest"

import { AgentsView } from "../app/views/AgentsView"
import type { AgentProfile } from "../commands/runhaven"

import { localAgent, providerAgent } from "./runhaven-fixtures"

const claudeAgent: AgentProfile = {
  name: "claude",
  description: "Claude",
  image: "runhaven/claude:0.1.0",
  defaultCommand: ["claude"],
  providerHosts: ["api.anthropic.com"],
  signIn: "runhaven login claude",
  broker: "no",
  supportsInAppSignIn: false,
}

const handlers = {
  refreshing: false,
  onSignedIn: () => undefined,
  onRefresh: () => undefined,
}

describe("AgentsView", () => {
  it("shows one row per agent with sign-in state, image state, and auth details", () => {
    const markup = renderToStaticMarkup(
      <AgentsView
        agents={[providerAgent, claudeAgent, localAgent]}
        authStatus={{
          status: "ok",
          runtime: "running",
          profiles: [
            { name: "codex", status: "api-key-broker" },
            { name: "claude", status: "needs-sign-in" },
          ],
        }}
        {...handlers}
      />,
    )

    expect(markup).toContain("Codex")
    expect(markup).toContain("Claude")
    expect(markup).toContain("Shell")
    // Per-agent auth status uses the shared label vocabulary.
    expect(markup).toContain("API key broker")
    expect(markup).toContain("Needs sign-in")
    // In-app sign-in for device-code agents; the in-app Claude sign-in terminal
    // (path A), with running it in your own terminal kept as a copy fallback.
    expect(markup).toContain("Sign in to Codex")
    expect(markup).toContain("Claude sign-in terminal")
    expect(markup).toContain("Prefer your own terminal?")
    expect(markup).toContain("runhaven login claude")
    expect(markup).toContain("Copy command")
    expect(markup).toContain("Auth details")
    // Per-row image state loads on entry; static markup shows the initial
    // loading line. Check again is the view's only refresh action.
    expect(markup).toContain("Checking image...")
    expect(markup).toContain("Check again")
  })

  it("hosts the confirm-gated image rebuild with agent choices", () => {
    const markup = renderToStaticMarkup(
      <AgentsView
        agents={[providerAgent, claudeAgent]}
        authStatus={null}
        {...handlers}
      />,
    )

    expect(markup).toContain("Rebuild agent image")
    expect(markup).toContain(">codex<")
    expect(markup).toContain(">claude<")
    // The rebuild button stays disabled until the user confirms.
    expect(markup).toContain("disabled")
  })

  it("renders a real empty state instead of dead navigation", () => {
    const markup = renderToStaticMarkup(
      <AgentsView agents={[]} authStatus={null} {...handlers} />,
    )
    expect(markup).toContain("No agents found yet")
  })
})
