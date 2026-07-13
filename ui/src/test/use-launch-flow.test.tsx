// @vitest-environment happy-dom
import { describe, expect, it } from "vitest"

import type { NavDestination } from "../app/navigation"
import { useLaunchFlow, type LaunchFlowDeps } from "../app/useLaunchFlow"
import {
  planRun,
  type LaunchRunResponse,
  type ManagedSessionList,
} from "../commands/runhaven"

import { dashboard } from "./main-view-props"
import { act, renderHook } from "./render-hook"
import { localAgent, providerAgent } from "./runhaven-fixtures"

function deps(patch: Partial<LaunchFlowDeps> = {}): LaunchFlowDeps & {
  launched: LaunchRunResponse[]
  errors: string[]
  navigated: NavDestination[]
} {
  const launched: LaunchRunResponse[] = []
  const errors: string[] = []
  const navigated: NavDestination[] = []
  return {
    launched,
    errors,
    navigated,
    dashboard: null,
    managedBrowserPreview: false,
    loadImageStatus: async () => {},
    loadRunStatus: async () => {},
    loadManagedSessions: async (): Promise<ManagedSessionList> => ({
      sessions: [],
    }),
    selectSession: () => {},
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
  it("sends only the plan to the native confirmation boundary", async () => {
    const requests: unknown[] = []
    const { result } = await renderFlow(
      deps({
        launchRunCommand: async (request) => {
          requests.push(request)
          return null
        },
      }),
    )

    await act(async () => {
      result.current.updateRequest({
        agent: "shell",
        workspacePath: "/tmp/safe-project",
        worktree: true,
      })
    })
    await act(async () => {
      await result.current.startSession()
    })

    expect(requests).toHaveLength(1)
    expect(requests[0]).toEqual({
      plan: expect.objectContaining({
        agent: "shell",
        workspacePath: "/tmp/safe-project",
        worktree: true,
      }),
    })
    expect(requests[0]).not.toHaveProperty("confirmLaunch")
    expect(requests[0]).not.toHaveProperty("confirmedWarnings")
  })

  it("leaves warnings to the native sheet instead of opening a WebView review", async () => {
    const requests: unknown[] = []
    const { result } = await renderFlow(
      deps({
        launchRunCommand: async (request) => {
          requests.push(request)
          return null
        },
      }),
    )

    await act(async () => {
      result.current.updateRequest({
        agent: "shell",
        workspacePath: "/tmp/internet-project",
        networkMode: "internet",
        worktree: false,
      })
    })
    await act(async () => {
      await result.current.startSession()
    })

    expect(requests).toHaveLength(1)
    expect(result.current.launchStep).toBe("agent")
  })

  it("keeps hard blockers inline and never reaches launch", async () => {
    const requests: unknown[] = []
    const { result } = await renderFlow(
      deps({
        planRunCommand: async (request) => {
          const plan = await planRun(request)
          return {
            ...plan,
            access: {
              ...plan.access,
              blocking: true,
              label: "Sign-in needed",
              detail: "Codex is not signed in.",
              remedy: "Sign in, then try again.",
            },
          }
        },
        launchRunCommand: async (request) => {
          requests.push(request)
          return null
        },
      }),
    )

    await act(async () => {
      result.current.updateRequest({ workspacePath: "/tmp/blocked-project" })
    })
    await act(async () => {
      await result.current.startSession()
    })

    expect(requests).toEqual([])
    expect(result.current.error).toContain("Sign-in needed")
    expect(result.current.error).toContain("Sign in, then try again.")
  })

  it.each([
    [true, true],
    [false, false],
  ])(
    "replans protected-copy recovery with readOnlyWorkspace=%s",
    async (readOnlyWorkspace, expectedReadOnly) => {
      const requests: Array<{ plan: { worktree: boolean; readOnlyWorkspace: boolean } }> = []
      const { result } = await renderFlow(
        deps({
          launchRunCommand: async (request) => {
            requests.push(request)
            return null
          },
        }),
      )
      await act(async () => {
        result.current.updateRequest({
          agent: "shell",
          workspacePath: "/tmp/unprotected-project",
          worktree: true,
        })
      })
      await act(async () => {
        await result.current.retryWithoutProtectedCopy(readOnlyWorkspace)
      })

      expect(requests[0]?.plan.worktree).toBe(false)
      expect(requests[0]?.plan.readOnlyWorkspace).toBe(expectedReadOnly)
    },
  )

  it("keeps the draft when leaving but resets the visible step", async () => {
    const { result } = await renderFlow(deps())
    await act(async () => {
      result.current.updateRequest({
        agent: "codex",
        workspacePath: "/tmp/draft-project",
        worktree: true,
      })
      result.current.setLaunchStep("project")
      result.current.setOpen(false)
    })

    expect(result.current.launchStep).toBe("agent")
    expect(result.current.request.workspacePath).toBe("/tmp/draft-project")
    expect(result.current.request.worktree).toBe(true)
  })

  it("moves an accepted background session to History with plain feedback", async () => {
    const flowDeps = deps()
    const { result } = await renderFlow(flowDeps)
    await act(async () => {
      result.current.updateRequest({
        agent: "shell",
        workspacePath: "/tmp/launch-project",
      })
    })
    await act(async () => {
      await result.current.startSession()
    })

    expect(flowDeps.launched).toHaveLength(1)
    expect(flowDeps.navigated).toEqual([{ kind: "runs" }])
    expect(result.current.launchMessage).toBe("Starting Shell…")
    expect(result.current.launchMessage).not.toContain(
      flowDeps.launched[0]?.runId ?? "",
    )
  })

  it("routes every interactive product assistant to the session terminal", async () => {
    for (const agent of ["antigravity", "claude", "codex", "copilot"]) {
      const flowDeps = deps()
      const { result, unmount } = await renderFlow(flowDeps)
      await act(async () => {
        result.current.updateRequest({
          agent,
          workspacePath: `/tmp/${agent}-interactive`,
        })
      })
      await act(async () => {
        await result.current.startSession()
      })

      expect(result.current.runTerminal?.request).toEqual({
        plan: expect.objectContaining({ agent }),
      })
      expect(result.current.runTerminal?.sessionId).not.toContain("-")
      expect(flowDeps.navigated).toEqual([
        expect.objectContaining({ kind: "run-terminal" }),
      ])
      expect(flowDeps.launched).toEqual([])
      await unmount()
    }
  })

  it("keeps two interactive sessions instead of replacing the first", async () => {
    const flowDeps = deps()
    const { result } = await renderFlow(flowDeps)

    for (const agent of ["copilot", "antigravity"] as const) {
      await act(async () => {
        result.current.updateRequest({
          agent,
          workspacePath: `/tmp/${agent}-concurrent`,
        })
      })
      await act(async () => {
        await result.current.startSession()
      })
    }

    const terminals = (
      result.current as typeof result.current & {
        runTerminals: Array<{
          sessionId: string
          request: { plan: { agent: string; sessionName: string | null } }
        }>
      }
    ).runTerminals
    expect(terminals).toHaveLength(2)
    expect(terminals.map((terminal) => terminal.request.plan.agent)).toEqual([
      "copilot",
      "antigravity",
    ])
    expect(new Set(terminals.map((terminal) => terminal.sessionId)).size).toBe(2)
    expect(
      new Set(terminals.map((terminal) => terminal.request.plan.sessionName)).size,
    ).toBe(2)
    expect(flowDeps.navigated).toEqual([
      expect.objectContaining({ kind: "run-terminal" }),
      expect.objectContaining({ kind: "run-terminal" }),
    ])
  })

  it("uses managed preview sessions instead of a terminal in the browser", async () => {
    const flowDeps = deps({ managedBrowserPreview: true })
    const { result } = await renderFlow(flowDeps)
    await act(async () => {
      result.current.updateRequest({
        agent: "codex",
        workspacePath: "/tmp/browser-preview",
      })
    })
    await act(async () => {
      await result.current.startSession()
    })

    expect(result.current.runTerminal).toBeNull()
    expect(flowDeps.launched).toHaveLength(1)
    expect(flowDeps.navigated).toEqual([{ kind: "runs" }])
  })

  it("treats closing the native confirmation as quiet cancellation", async () => {
    const flowDeps = deps({ launchRunCommand: async () => null })
    const { result } = await renderFlow(flowDeps)
    await act(async () => {
      result.current.updateRequest({
        agent: "shell",
        workspacePath: "/tmp/cancel-project",
      })
    })
    await act(async () => {
      await result.current.startSession()
    })

    expect(flowDeps.launched).toEqual([])
    expect(flowDeps.navigated).toEqual([])
    expect(result.current.error).toBe("")
    expect(result.current.planning).toBe(false)
  })

  it("uses the safe default assistant once and avoids image checks before setup", async () => {
    let imageChecks = 0
    const { result } = await renderFlow(
      deps({ loadImageStatus: async () => void (imageChecks += 1) }),
    )
    await act(async () => {
      await result.current.applyDashboard({
        ...dashboard(false),
        agents: [providerAgent, localAgent],
      })
    })

    expect(result.current.request.agent).toBe("codex")
    expect(imageChecks).toBe(0)
  })

  it("checks the selected assistant after applying a saved setup", async () => {
    const checked: string[] = []
    const { result } = await renderFlow(
      deps({
        dashboard: dashboard(true),
        loadImageStatus: async (agentName) => void checked.push(agentName),
      }),
    )

    await act(async () => {
      result.current.applySavedSetup({
        ...result.current.request,
        agent: "claude",
        workspacePath: "/tmp/saved-project",
      })
    })

    expect(checked).toEqual(["claude"])
    expect(result.current.launchStep).toBe("project")
  })

  it("turns off the host API key broker for alternate network modes", async () => {
    const withCodex = dashboard(true)
    const { result } = await renderFlow(deps({ dashboard: withCodex }))
    await act(async () => {
      await result.current.applyDashboard(withCodex)
      result.current.updateRequest({ apiKeyBrokerEnv: "OPENAI_API_KEY" })
      result.current.updateNetworkMode("internet")
    })

    expect(result.current.request.networkMode).toBe("internet")
    expect(result.current.request.apiKeyBrokerEnv).toBeNull()
  })
})
