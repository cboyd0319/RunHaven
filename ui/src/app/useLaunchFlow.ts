import { useRef, useState } from "react"

import {
  chooseProjectFolder,
  defaultRunPlanRequest,
  launchRun,
  nextLaunchSessionName,
  planRun,
  preferredDefaultAgent,
  reconcileRunPlanRequest,
  secureNetworkDefault,
  withLaunchSessionName,
  type DashboardStatus,
  type LaunchRunRequest,
  type LaunchRunResponse,
  type ManagedSessionList,
  type RunPlanRequest,
} from "@/commands/runhaven"

import {
  HOME_NAV,
  NEW_SESSION_NAV,
  RUN_TERMINAL_NAV,
  type NavDestination,
} from "./navigation"
import { agentLabel } from "./sessionIdentity"
import { errorMessage, type LaunchStep } from "./types"

// The captured launch for an interactive assistant's terminal. Rust recomputes
// the exact plan and owns the native confirmation before anything starts.
export type RunTerminalLaunch = {
  sessionId: string
  request: LaunchRunRequest
}

// Each bundled coding agent starts an interactive terminal UI when invoked with
// its profile's default command. Running one through the headless captured path
// leaves it waiting on a TTY with no usable prompt. Shell remains the one
// background-capable profile.
const INTERACTIVE_AGENTS = new Set([
  "antigravity",
  "claude",
  "codex",
  "copilot",
])

function agentRunsInteractively(agent: string): boolean {
  return INTERACTIVE_AGENTS.has(agent)
}

// A fresh terminal session key for a run terminal. Like the sign-in terminal, it
// is only the registry key / run label (dash-free so it passes the backend run-id
// validator); it never steers what runs, which the backend decides from the
// reviewed request.
function newTerminalSessionId(): string {
  return (globalThis.crypto?.randomUUID?.() ?? `run-${Date.now()}`).replaceAll(
    "-",
    "",
  )
}

export type LaunchFlowDeps = {
  dashboard: DashboardStatus | null
  // Browser preview normally simulates a managed session so History, output,
  // diagnostics, and worktree review remain directly testable. The real Tauri
  // app and the explicit terminal preview use the live interactive terminal.
  managedBrowserPreview: boolean
  loadImageStatus: (agentName: string) => Promise<void>
  loadRunStatus: (runId: string) => Promise<void>
  loadManagedSessions: () => Promise<ManagedSessionList>
  selectSession: (sessionId: string) => void
  // Move the main window to a destination. Used to open the interactive run
  // terminal as a main-content view and to return Home when it closes.
  navigate: (destination: NavDestination) => void
  onLaunched: (launch: LaunchRunResponse) => void
  onError: (message: string) => void
  // Injection seam for the native confirm result. The command returns null
  // when the user closes or cancels the trusted dialog.
  launchRunCommand?: typeof launchRun
  planRunCommand?: typeof planRun
}

export type LaunchFlow = ReturnType<typeof useLaunchFlow>

// The New Session flow's whole state: the request draft, step, and in-flight
// flags. The page is intent capture only; the actual
// mutation boundary stays the Rust-owned native confirmation inside launch_run.
//
// Draft retention rule: Cancel leaves the page with no side effects, so the
// request draft (agent, project folder, advanced options) keeps its values for
// the rest of the app session. Leaving does reset the
// step to "agent" and clears local errors.
export function useLaunchFlow(deps: LaunchFlowDeps) {
  const [request, setRequest] = useState<RunPlanRequest>(() =>
    defaultRunPlanRequest(undefined),
  )
  const [launchStep, setLaunchStep] = useState<LaunchStep>("agent")
  const [planning, setPlanning] = useState(false)
  const [launchMessage, setLaunchMessage] = useState("")
  // The active interactive run terminal, set when launch routes an interactive
  // assistant to the run-terminal main-content destination
  // instead of a background run. It is independent of the New Session page;
  // closeRunTerminal clears it and navigates Home, which
  // unmounts the panel and tears down the backend child.
  const [runTerminal, setRunTerminal] = useState<RunTerminalLaunch | null>(null)
  // Errors from planning or launching render inside the flow so they stay next
  // to the action that failed.
  const [error, setError] = useState("")
  // The agent name the UI currently targets, readable from async refresh
  // completions. Null until the first refresh applies the default selection;
  // that null doubles as the "apply defaults exactly once" guard, so two
  // refreshes racing before the first render commit (a render-closure read of
  // the dashboard sees null twice) cannot both take the initializing branch and
  // reset the user's networkMode/apiKeyBrokerEnv edits or the default agent.
  const agentNameRef = useRef<string | null>(null)

  function setOpen(open: boolean) {
    if (open) {
      return
    }
    setLaunchStep("agent")
    setError("")
    // The run terminal is a main-content destination, so leaving New Session
    // must not tear it down. It is cleared only by
    // closeRunTerminal (navigating Home unmounts the panel and kills the child).
  }

  // Reconcile the request draft with a fresh dashboard: make the built-in pick
  // exactly once, then keep following the user's explicit agent choice.
  async function applyDashboard(dashboard: DashboardStatus) {
    const initializing = agentNameRef.current === null
    const agent = initializing
      ? preferredDefaultAgent(dashboard.agents)
      : (dashboard.agents.find((item) => item.name === agentNameRef.current) ??
        dashboard.agents[0])
    if (!agent) {
      return
    }
    agentNameRef.current = agent.name
    setRequest((current) =>
      reconcileRunPlanRequest(
        initializing
          ? {
              ...defaultRunPlanRequest(agent),
              workspacePath: current.workspacePath,
              sessionName: current.sessionName,
            }
          : current,
        agent,
      ),
    )
    if (dashboard.setup.ok) {
      await deps.loadImageStatus(agent.name)
    }
  }

  function changeAgent(agentName: string) {
    agentNameRef.current = agentName
    const agent = deps.dashboard?.agents.find((item) => item.name === agentName)
    setRequest((current) =>
      agent
        ? reconcileRunPlanRequest(current, agent)
        : {
            ...current,
            agent: agentName,
            networkMode: secureNetworkDefault(undefined),
            apiKeyBrokerEnv: null,
          },
    )
    invalidatePlan()
    if (deps.dashboard?.setup.ok) {
      void deps.loadImageStatus(agentName)
    }
  }

  function updateNetworkMode(networkMode: RunPlanRequest["networkMode"]) {
    setRequest((current) => ({
      ...current,
      networkMode,
      // A host broker only works through RunHaven's provider network. Choosing
      // another supported mode opts out of that broker; Rust independently
      // guarantees no host secret crosses into the run.
      apiKeyBrokerEnv:
        networkMode === "provider" ? current.apiKeyBrokerEnv : null,
    }))
    invalidatePlan()
  }

  function updateRequest(patch: Partial<RunPlanRequest>) {
    setError("")
    setRequest((current) => {
      const next = { ...current, ...patch }
      return patch.apiKeyBrokerEnv
        ? { ...next, networkMode: "provider" }
        : next
    })
    invalidatePlan()
  }

  function invalidatePlan() {
    setLaunchMessage("")
  }

  async function pickFolder() {
    const selected = await chooseProjectFolder()
    if (selected) {
      updateRequest({ workspacePath: selected })
    }
  }

  async function startSession() {
    await planAndContinue(withLaunchSessionName(request))
  }

  async function retryWithoutProtectedCopy(readOnlyWorkspace: boolean) {
    await planAndContinue(
      withLaunchSessionName({
        ...request,
        worktree: false,
        readOnlyWorkspace,
      }),
    )
  }

  async function planAndContinue(planRequest: RunPlanRequest) {
    setPlanning(true)
    setError("")
    invalidatePlan()
    try {
      if (planRequest !== request) {
        setRequest(planRequest)
      }
      const nextPlan = await (deps.planRunCommand ?? planRun)(planRequest)
      if (nextPlan.access.blocking) {
        setError(
          [
            nextPlan.access.label,
            nextPlan.access.detail,
            nextPlan.access.remedy,
          ]
            .filter(Boolean)
            .join(" "),
        )
        return
      }
      await startPlannedRun(planRequest)
    } catch (cause) {
      setError(errorMessage(cause))
    } finally {
      setPlanning(false)
    }
  }

  // Leave the interactive run terminal: clearing state and navigating Home
  // unmounts the panel, whose close-on-unmount tears down the backend child.
  // Home (or the sessions list) is where the user lands, never stranded.
  function closeRunTerminal() {
    setRunTerminal(null)
    deps.navigate(HOME_NAV)
  }

  // The trusted native sheet appears after the terminal host is mounted so its
  // output channel is ready before a child can write. If the user cancels that
  // sheet, remove the temporary terminal destination and return to the retained
  // project choices. A canceled launch must never look like an empty session.
  function cancelRunTerminal() {
    setRunTerminal(null)
    setLaunchStep("project")
    setError("")
    deps.navigate(NEW_SESSION_NAV)
  }

  async function startPlannedRun(requestDraft: RunPlanRequest) {
    const launchRequest = requestDraft.sessionName
      ? requestDraft
      : { ...requestDraft, sessionName: nextLaunchSessionName() }
    const fullRequest: LaunchRunRequest = {
      plan: launchRequest,
    }
    // Interactive agent (Claude Code): its prompt lives in its own TUI and a
    // RunHaven session carries no prompt, so open the in-app terminal instead
    // of a headless background session. Rust still owns final confirmation.
    if (
      agentRunsInteractively(launchRequest.agent) &&
      !deps.managedBrowserPreview
    ) {
      setError("")
      setLaunchMessage("")
      setRunTerminal({ sessionId: newTerminalSessionId(), request: fullRequest })
      // A live interactive session fills the bounded main content area. Reset
      // the launch flow and open the run-terminal destination; setOpen(false)
      // leaves runTerminal intact.
      setOpen(false)
      deps.navigate(RUN_TERMINAL_NAV)
      return
    }
    setError("")
    setLaunchMessage("")
    let started: LaunchRunResponse | null
    try {
      started = await (deps.launchRunCommand ?? launchRun)(fullRequest)
    } catch (cause) {
      setError(errorMessage(cause))
      return
    }
    if (started === null) {
      return
    }
    // Accepted: leave New Session, keep the draft values, and
    // arm the next launch with a fresh session name.
    deps.onLaunched(started)
    setLaunchMessage(`Starting ${agentLabel(launchRequest.agent)}…`)
    setOpen(false)
    // The native command has accepted the launch, but its managed-session
    // record may take several seconds to appear while the sandbox starts.
    // History is a useful, stable waiting surface; useManagedSessionsState
    // opens the exact session as soon as polling finds its backend-owned run id.
    deps.navigate({ kind: "runs" })
    setRequest((current) => ({
      ...current,
      sessionName: nextLaunchSessionName(),
    }))
    try {
      await deps.loadRunStatus(started.runId)
      await deps.loadManagedSessions()
    } catch (cause) {
      // The flow is already closed, so follow-up failures surface app-wide.
      deps.onError(errorMessage(cause))
    }
  }

  return {
    request,
    launchStep,
    planning,
    launchMessage,
    error,
    runTerminal,
    setOpen,
    applyDashboard,
    changeAgent,
    updateNetworkMode,
    updateRequest,
    pickFolder,
    startSession,
    retryWithoutProtectedCopy,
    closeRunTerminal,
    cancelRunTerminal,
    setLaunchStep,
  }
}
