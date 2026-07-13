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
  type RunPlanResponse,
} from "@/commands/runhaven"

import { HOME_NAV, RUN_TERMINAL_NAV, type NavDestination } from "./navigation"
import { loadSessionsUntilRunPresent } from "./sessionSelection"
import { errorMessage, type LaunchStep } from "./types"

// The captured, reviewed launch for an interactive agent's run terminal: the
// full LaunchRunRequest (with launch intent and confirmed warnings) plus the
// terminal session key. The backend fires the native launch confirmation before
// anything runs, so this is intent capture, not the mutation boundary.
export type RunTerminalLaunch = {
  sessionId: string
  request: LaunchRunRequest
}

// Which agents run as an interactive TUI, so New Session opens the in-app
// run terminal (task #23, Option A) instead of a headless background run. Claude
// Code is the only one today. Seam to generalize: when core exposes a per-agent
// "interactive" flag on the profile, read it here instead of hard-coding claude.
function agentRunsInteractively(agent: string): boolean {
  return agent === "claude"
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
  // The user's saved default agent (Settings), or null for the built-in pick.
  // Applied only if that agent still exists in the live dashboard.
  defaultAgent: string | null
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
}

export type LaunchFlow = ReturnType<typeof useLaunchFlow>

// The New Session flow's whole state: the request draft, the step, the reviewed
// plan, and the in-flight flags. The page is intent capture only; the actual
// mutation boundary stays the Rust-owned native confirmation inside launch_run.
//
// Draft retention rule: Cancel leaves the page with no side effects, so the
// request draft (agent, project folder, advanced options) keeps its values for
// the rest of the app session. Leaving does reset the
// step to "agent" and clears the reviewed plan, the launch confirmation, and
// any checked warnings: a warning confirmed on an abandoned review must never
// stay silently confirmed for a later launch.
export function useLaunchFlow(deps: LaunchFlowDeps) {
  const [request, setRequest] = useState<RunPlanRequest>(() =>
    defaultRunPlanRequest(undefined),
  )
  const [launchStep, setLaunchStep] = useState<LaunchStep>("agent")
  const [plan, setPlan] = useState<RunPlanResponse | null>(null)
  const [planning, setPlanning] = useState(false)
  const [launching, setLaunching] = useState(false)
  const [confirmedWarnings, setConfirmedWarnings] = useState<string[]>([])
  const [launchMessage, setLaunchMessage] = useState("")
  // The active interactive run terminal, set when startRun routes a reviewed,
  // confirmed interactive launch to the run-terminal main-content destination
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
    setPlan(null)
    setConfirmedWarnings([])
    setError("")
    // The run terminal is a main-content destination now, not sheet content, so
    // closing the sheet must not tear it down. It is cleared only by
    // closeRunTerminal (navigating Home unmounts the panel and kills the child).
  }

  // Reconcile the request draft with a fresh dashboard: pick the default agent
  // exactly once, then keep following the user's chosen agent across refreshes.
  async function applyDashboard(dashboard: DashboardStatus) {
    const initializing = agentNameRef.current === null
    const agent = initializing
      ? (dashboard.agents.find((item) => item.name === deps.defaultAgent) ??
        preferredDefaultAgent(dashboard.agents))
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
    setRequest((current) => {
      const next = { ...current, ...patch }
      return patch.apiKeyBrokerEnv
        ? { ...next, networkMode: "provider" }
        : next
    })
    invalidatePlan()
  }

  function invalidatePlan() {
    setPlan(null)
    setConfirmedWarnings([])
    setLaunchMessage("")
  }

  async function pickFolder() {
    const selected = await chooseProjectFolder()
    if (selected) {
      updateRequest({ workspacePath: selected })
    }
  }

  async function reviewPlan() {
    setPlanning(true)
    setError("")
    invalidatePlan()
    try {
      const planRequest = withLaunchSessionName(request)
      if (planRequest !== request) {
        setRequest(planRequest)
      }
      const nextPlan = await planRun(planRequest)
      setPlan(nextPlan)
      setLaunchStep("review")
    } catch (cause) {
      setError(errorMessage(cause))
    } finally {
      setPlanning(false)
    }
  }

  function setWarningConfirmation(code: string, checked: boolean) {
    setConfirmedWarnings((current) => {
      const next = new Set(current)
      if (checked) {
        next.add(code)
      } else {
        next.delete(code)
      }
      return Array.from(next)
    })
  }

  // Leave the interactive run terminal: clearing state and navigating Home
  // unmounts the panel, whose close-on-unmount tears down the backend child.
  // Home (or the sessions list) is where the user lands, never stranded.
  function closeRunTerminal() {
    setRunTerminal(null)
    deps.navigate(HOME_NAV)
  }

  async function startRun() {
    if (!plan) {
      return
    }
    const launchRequest = request.sessionName
      ? request
      : { ...request, sessionName: plan.session || nextLaunchSessionName() }
    const fullRequest: LaunchRunRequest = {
      plan: launchRequest,
      // Clicking Start run expresses WebView intent. The trusted mutation
      // boundary remains the Rust-owned native confirmation dialog.
      confirmLaunch: true,
      confirmedWarnings,
    }
    // Interactive agent (Claude Code): its prompt lives in its own TUI and a
    // RunHaven run carries no prompt, so open the in-app run terminal instead of
    // a headless background run. The reviewed request (confirmLaunch + confirmed
    // warnings) is handed straight to the backend, which fires the Rust-owned
    // native launch confirmation before anything runs.
    if (agentRunsInteractively(launchRequest.agent)) {
      setError("")
      setLaunchMessage("")
      setRunTerminal({ sessionId: newTerminalSessionId(), request: fullRequest })
      // A live interactive session fills the bounded main content area. Reset
      // the launch review and open the run-terminal destination; setOpen(false)
      // leaves runTerminal intact.
      setOpen(false)
      deps.navigate(RUN_TERMINAL_NAV)
      return
    }
    setLaunching(true)
    setError("")
    setLaunchMessage("")
    let started: LaunchRunResponse | null
    try {
      started = await (deps.launchRunCommand ?? launchRun)(fullRequest)
    } catch (cause) {
      setError(errorMessage(cause))
      setLaunching(false)
      return
    }
    if (started === null) {
      setLaunching(false)
      return
    }
    // Accepted: close the sheet first (spec 3.2), keep the draft values, and
    // arm the next launch with a fresh session name.
    deps.onLaunched(started)
    setLaunchMessage(`Run accepted: ${started.runId}`)
    setOpen(false)
    setRequest((current) => ({
      ...current,
      sessionName: nextLaunchSessionName(),
    }))
    try {
      await deps.loadRunStatus(started.runId)
      const { sessionId } = await loadSessionsUntilRunPresent(
        deps.loadManagedSessions,
        started.runId,
      )
      if (sessionId) {
        // User-initiated navigation: launching is the action that asked for it.
        deps.selectSession(sessionId)
      }
    } catch (cause) {
      // The flow is already closed, so follow-up failures surface app-wide.
      deps.onError(errorMessage(cause))
    } finally {
      setLaunching(false)
    }
  }

  return {
    request,
    launchStep,
    plan,
    planning,
    launching,
    confirmedWarnings,
    launchMessage,
    error,
    runTerminal,
    setOpen,
    applyDashboard,
    changeAgent,
    updateNetworkMode,
    updateRequest,
    pickFolder,
    reviewPlan,
    setWarningConfirmation,
    startRun,
    closeRunTerminal,
    setLaunchStep,
  }
}
