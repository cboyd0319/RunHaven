import { renderToStaticMarkup } from "react-dom/server"
import { describe, expect, it } from "vitest"

import { AgentsView, ImageBuildFeedback } from "../app/views/AgentsView"
import { ImageReadiness } from "../app/LaunchAccess"
import type { AgentProfile } from "../commands/runhaven"

import { localAgent, providerAgent } from "./runhaven-fixtures"

const claudeAgent: AgentProfile = {
  name: "claude",
  description: "Claude",
  image: "runhaven/claude:0.7.0",
  defaultCommand: ["claude"],
  providerHosts: ["api.anthropic.com"],
  signIn: "runhaven login claude",
  broker: "yes",
  supportsInAppSignIn: true,
}

const handlers = {
  setupReady: true,
  refreshing: false,
  onSignedIn: () => undefined,
  onRefresh: () => undefined,
}

describe("AgentsView", () => {
  it("shows a compact agent list and opens the agent that needs attention", () => {
    const markup = renderToStaticMarkup(
      <AgentsView
        agents={[providerAgent, claudeAgent, localAgent]}
        agentReadiness={[
          { name: "codex", category: "signed-in", label: "Signed in" },
          { name: "claude", category: "needs-sign-in", label: "Needs sign-in" },
          { name: "shell", category: "ready", label: "Ready" },
        ]}
        {...handlers}
      />,
    )

    expect(markup).toContain("Codex")
    expect(markup).toContain("Claude")
    expect(markup).toContain("Shell")
    // Per-agent readiness is about the agent's native sign-in, not broker
    // capability.
    expect(markup).toContain("Signed in")
    expect(markup).toContain("Needs sign-in")
    // Every supported agent uses the same native in-container terminal path.
    expect(markup).not.toContain("Sign in to Codex")
    expect(markup).toContain("Claude sign-in terminal")
    expect(markup).not.toContain("Sign in on your Mac")
    expect(markup).not.toContain("runhaven login claude")
    expect(markup).toContain("Sign-in details")
    expect(markup).toContain('aria-current="true"')
    // Per-row image state loads on entry; static markup shows the initial
    // loading line. Check again is the view's only refresh action.
    expect(markup).toContain("Checking image...")
    expect(markup).toContain("Check again")
  })

  it("hides the sign-in terminal when native readiness is signed in", () => {
    const markup = renderToStaticMarkup(
      <AgentsView
        agents={[claudeAgent]}
        agentReadiness={[
          { name: "claude", category: "signed-in", label: "Signed in" },
        ]}
        {...handlers}
      />,
    )

    expect(markup).not.toContain("Claude sign-in terminal")
    expect(markup).toContain("This agent is ready")
    expect(markup).toContain("Sign-in details")
  })

  it("makes a missing agent image a visible first-time install action", () => {
    const markup = renderToStaticMarkup(
      <ImageReadiness
        imageLoading={false}
        imageError=""
        imageStatus={{
          agent: "codex",
          image: {
            agent: "codex",
            image: "runhaven/codex:0.7.0",
            status: "missing",
            ready: false,
            expectedSourceDigest: "expected",
            localSourceDigest: null,
            fixCommand: "runhaven image build codex",
          },
          builder: {
            status: "ready",
            detail: "Builder ready",
            image: null,
            cpus: null,
            memory: null,
            rosetta: null,
            startedDate: null,
            ipv4Address: null,
            warning: null,
          },
        }}
        installing={false}
        onInstall={() => undefined}
      />,
    )

    expect(markup).toContain("Agent setup needed")
    expect(markup).toContain("Install Codex")
    expect(markup).not.toContain("runhaven image build codex")
  })

  it("hosts image repair behind a technical disclosure and relies on native confirmation", () => {
    const markup = renderToStaticMarkup(
      <AgentsView
        agents={[providerAgent, claudeAgent]}
        agentReadiness={[]}
        {...handlers}
      />,
    )

    expect(markup).toContain("Reinstall agent")
    expect(markup).toContain(">codex<")
    expect(markup).toContain(">claude<")
    expect(markup).toContain("Repair agent installation")
    expect(markup).not.toContain("Yes, reinstall this agent")
  })

  it("renders a failed agent installation as an error, not a success", () => {
    const markup = renderToStaticMarkup(
      <ImageBuildFeedback
        error=""
        status={{
          buildId: "build-1",
          agent: "codex",
          tag: "runhaven/codex:0.7.0",
          state: "failed",
          label: "Installation failed",
          detail: "The agent installation finished with errors.",
          outputTail: "",
          updatedAt: "2026-07-11T20:00:00Z",
        }}
      />,
    )

    expect(markup).toContain('data-slot="alert"')
    expect(markup).toContain("text-destructive")
    expect(markup).toContain("circle-alert")
    expect(markup).not.toContain("circle-check")
  })

  it("renders a real empty state instead of dead navigation", () => {
    const markup = renderToStaticMarkup(
      <AgentsView agents={[]} agentReadiness={[]} {...handlers} />,
    )
    expect(markup).toContain("No agents found yet")
  })
})
