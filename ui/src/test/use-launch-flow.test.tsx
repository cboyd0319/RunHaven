// @vitest-environment happy-dom
import { describe, expect, it } from "vitest"

import { useLaunchFlow, type LaunchFlowDeps } from "../app/useLaunchFlow"
import type { NavDestination } from "../app/navigation"
import type {
  LaunchRunResponse,
  ManagedSessionList,
} from "../commands/runhaven"
import { dashboard } from "./main-view-props"
import { act, renderHook } from "./render-hook"
import { localAgent, providerAgent } from "./runhaven-fixtures"

// The preview command fallbacks (no Tauri runtime in tests) back the flow, so
// planRun and launchRun behave like the real intent-capture pipeline.
function deps(patch: Partial<LaunchFlowDeps> = {}): LaunchFlowDeps & {
  launched: LaunchRunResponse[]
  selected: string[]
  errors: string[]
  navigated: NavDestination[]
} {
  const launched: LaunchRunResponse[] = []
  const selected: string[] = []
  const errors: string[] = []
  const navigated: NavDestination[] = []
  return {
    launched,
    selected,
    errors,
    navigated,
    dashboard: null,
    defaultAgent: null,
    loadImageStatus: async () => {},
    loadRunStatus: async () => {},
    loadManagedSessions: async (): Promise<ManagedSessionList> => ({
      sessions: [],
    }),
    selectSession: (sessionId) => selected.push(sessionId),
    navigate: (destination) => navigated.push(destination),
    onLaunched: (launch) => launched.push(launch),
    onError: (message) => errors.push(message),
    ...patch,
  }
}

async function renderFlow(flowDeps: LaunchFlowDeps) {
  return renderHook(() => useLaunchFlow(flowDeps), undefined)
}

describe("useLaunchFlow", () => {
  it("keeps the request draft across close and reopen, resetting only flow state", async () => {
    const { result } = await renderFlow(deps())

    await act(async () => {
      result.current.setOpen(true)
      result.current.updateRequest({
        agent: "codex",
        workspacePath: "/tmp/draft-project",
        worktree: true,
      })
    })
    await act(async () => {
      await result.current.reviewPlan()
    })
    await act(async () => {
      result.current.setWarningConfirmation("some-warning", true)
    })
    expect(result.current.launchStep).toBe("review")
    expect(result.current.plan).not.toBeNull()

    // Cancel/Escape: no side effects on the draft, but the reviewed plan and
    // every confirmation clear so nothing stays silently confirmed.
    await act(async () => {
      result.current.setOpen(false)
    })
    expect(result.current.launchStep).toBe("agent")
    expect(result.current.plan).toBeNull()
    expect(result.current.confirmedWarnings).toEqual([])

    await act(async () => {
      result.current.setOpen(true)
    })
    expect(result.current.request.workspacePath).toBe("/tmp/draft-project")
    expect(result.current.request.worktree).toBe(true)
    expect(result.current.request.agent).toBe("codex")
  })

  it("editing the request invalidates the reviewed plan and confirmations", async () => {
    const { result } = await renderFlow(deps())

    await act(async () => {
      result.current.updateRequest({
        agent: "codex",
        workspacePath: "/tmp/plan-project",
      })
    })
    await act(async () => {
      await result.current.reviewPlan()
    })
    await act(async () => {
      result.current.updateRequest({ workspacePath: "/tmp/other-project" })
    })
    expect(result.current.plan).toBeNull()
  })

  it("closes the sheet, selects the session, and arms a new name on launch", async () => {
    const flowDeps = deps()
    // Once a launch is accepted, the sessions list reports it as a managed
    // session so the flow can navigate to it.
    flowDeps.loadManagedSessions = async () => ({
      sessions: flowDeps.launched.map((launch, index) => ({
        id: `session-${index + 1}`,
        runId: launch.runId,
        agent: "codex",
        session: "alpha",
        title: "Codex - alpha",
        project: "alpha",
        status: "running",
        statusLabel: "Running",
        authStatus: "ready",
        authScope: "project",
        network: "provider",
        startedAt: "2026-07-01T01:00:00Z",
        containerName: "runhaven-1",
        detailCommand: "runhaven runs show",
        lifecycle: "running",
        input: { attached: true, status: "attached", detail: "" },
        nextAction: { kind: "monitor", label: "Monitor", detail: "" },
        events: [],
      })),
    })
    const { result } = await renderFlow(flowDeps)

    await act(async () => {
      result.current.setOpen(true)
      result.current.updateRequest({
        agent: "codex",
        workspacePath: "/tmp/launch-project",
      })
    })
    await act(async () => {
      await result.current.reviewPlan()
    })
    const sessionNameBefore = result.current.request.sessionName
    await act(async () => {
      await result.current.startRun()
    })

    expect(flowDeps.launched).toHaveLength(1)
    expect(flowDeps.selected).toEqual(["session-1"])
    expect(flowDeps.errors).toEqual([])
    expect(result.current.launchStep).toBe("agent")
    expect(result.current.plan).toBeNull()
    expect(result.current.request.sessionName).not.toBe(sessionNameBefore)
    // The draft itself survives for the next run.
    expect(result.current.request.workspacePath).toBe("/tmp/launch-project")
  })

  it("initializes with the saved default agent when it still exists", async () => {
    const withAgents = {
      ...dashboard(true),
      agents: [providerAgent, localAgent],
    }
    const { result } = await renderFlow(deps({ defaultAgent: "shell" }))
    await act(async () => {
      await result.current.applyDashboard(withAgents)
    })
    expect(result.current.request.agent).toBe("shell")
  })

  it("falls back to the built-in pick when the saved default agent is gone", async () => {
    const withAgents = {
      ...dashboard(true),
      agents: [providerAgent, localAgent],
    }
    const { result } = await renderFlow(deps({ defaultAgent: "removed-agent" }))
    await act(async () => {
      await result.current.applyDashboard(withAgents)
    })
    // preferredDefaultAgent: the first agent that supports in-app sign-in.
    expect(result.current.request.agent).toBe("codex")
  })

  it("does not check agent images before setup is ready", async () => {
    let imageChecks = 0
    const flowDeps = deps({
      loadImageStatus: async () => {
        imageChecks += 1
      },
    })
    const { result } = await renderFlow(flowDeps)

    await act(async () => {
      await result.current.applyDashboard(dashboard(false))
    })

    expect(imageChecks).toBe(0)
  })

  it("allows alternate Claude networks without selecting the host broker", async () => {
    const claude = {
      ...providerAgent,
      name: "claude",
      description: "Claude",
      providerHosts: ["api.anthropic.com"],
      supportsInAppSignIn: false,
    }
    const withSignedInClaude = {
      ...dashboard(true),
      agents: [claude],
      agentReadiness: [
        { name: "claude", category: "signed-in" as const, label: "Signed in" },
      ],
    }
    const flowDeps = deps({ dashboard: withSignedInClaude })
    const { result } = await renderFlow(flowDeps)
    await act(async () => {
      await result.current.applyDashboard(withSignedInClaude)
      result.current.updateNetworkMode("internal")
    })

    expect(result.current.request.networkMode).toBe("internal")
    expect(result.current.request.apiKeyBrokerEnv).toBeNull()
  })

  it("turns off an explicit API key broker when an alternate network is chosen", async () => {
    const withCodex = dashboard(true)
    const flowDeps = deps({ dashboard: withCodex })
    const { result } = await renderFlow(flowDeps)
    await act(async () => {
      await result.current.applyDashboard(withCodex)
      result.current.updateRequest({ apiKeyBrokerEnv: "OPENAI_API_KEY" })
      result.current.updateNetworkMode("internet")
    })

    expect(result.current.request.networkMode).toBe("internet")
    expect(result.current.request.apiKeyBrokerEnv).toBeNull()
  })

  it("opens the run terminal as a main-content destination for an interactive agent instead of a background launch", async () => {
    const flowDeps = deps()
    const { result } = await renderFlow(flowDeps)

    await act(async () => {
      result.current.setOpen(true)
      result.current.updateRequest({
        agent: "claude",
        workspacePath: "/tmp/claude-project",
      })
    })
    await act(async () => {
      await result.current.reviewPlan()
    })
    await act(async () => {
      await result.current.startRun()
    })

    // Routed to the run-terminal destination carrying the reviewed, confirmed
    // request: the sheet closes (a live session never lives in the modal),
    // nothing launched in the background, and no session navigation happened.
    expect(result.current.runTerminal).not.toBeNull()
    expect(result.current.runTerminal?.request.plan.agent).toBe("claude")
    expect(result.current.runTerminal?.request.confirmLaunch).toBe(true)
    // The session key is a dash-free label so it passes the backend run-id rule.
    expect(result.current.runTerminal?.sessionId).not.toContain("-")
    expect(flowDeps.navigated).toEqual([{ kind: "run-terminal" }])
    expect(flowDeps.launched).toHaveLength(0)
    expect(flowDeps.selected).toEqual([])

    // Closing the run terminal clears it and returns Home; the panel unmounts,
    // and close-on-unmount tears down the backend child.
    await act(async () => {
      result.current.closeRunTerminal()
    })
    expect(result.current.runTerminal).toBeNull()
    expect(flowDeps.navigated).toEqual([
      { kind: "run-terminal" },
      { kind: "home" },
    ])
  })

  it("sends launch intent to the Rust-owned native confirmation without a WebView checkbox", async () => {
    const flowDeps = deps()
    const { result } = await renderFlow(flowDeps)

    await act(async () => {
      result.current.setOpen(true)
      result.current.updateRequest({
        agent: "codex",
        workspacePath: "/tmp/reject-project",
      })
    })
    await act(async () => {
      await result.current.reviewPlan()
    })
    await act(async () => {
      await result.current.startRun()
    })

    expect(flowDeps.launched).toHaveLength(1)
    expect(flowDeps.launched[0]?.status).toBe("started")
    expect(result.current.error).toBe("")
  })

  it("treats closing the native launch confirmation as quiet cancellation", async () => {
    const flowDeps = deps({
      launchRunCommand: async () => null,
    })
    const { result } = await renderFlow(flowDeps)

    await act(async () => {
      result.current.updateRequest({
        agent: "codex",
        workspacePath: "/tmp/cancel-project",
      })
    })
    await act(async () => {
      await result.current.reviewPlan()
    })
    expect(result.current.plan).not.toBeNull()
    await act(async () => {
      await result.current.startRun()
    })

    expect(flowDeps.launched).toEqual([])
    expect(flowDeps.errors).toEqual([])
    expect(result.current.error).toBe("")
    expect(result.current.launching).toBe(false)
  })
})
