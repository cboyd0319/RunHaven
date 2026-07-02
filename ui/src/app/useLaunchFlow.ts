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
  type LaunchRunResponse,
  type ManagedSessionList,
  type RunPlanRequest,
  type RunPlanResponse,
} from "@/commands/runhaven"

import { loadSessionsUntilRunPresent } from "./sessionSelection"
import { errorMessage, type LaunchStep } from "./types"

export type LaunchFlowDeps = {
  dashboard: DashboardStatus | null
  // The user's saved default agent (Settings), or null for the built-in pick.
  // Applied only if that agent still exists in the live dashboard.
  defaultAgent: string | null
  loadImageStatus: (agentName: string) => Promise<void>
  loadRunStatus: (runId: string) => Promise<void>
  loadManagedSessions: () => Promise<ManagedSessionList>
  selectSession: (sessionId: string) => void
  onLaunched: (launch: LaunchRunResponse) => void
  onError: (message: string) => void
}

export type LaunchFlow = ReturnType<typeof useLaunchFlow>

// The New Run sheet's whole state: the request draft, the step, the reviewed
// plan, and the in-flight flags. The sheet is intent capture only; the actual
// mutation boundary stays the Rust-owned native confirmation inside launch_run.
//
// Draft retention rule (spec 3.2): Cancel/Escape closes the sheet with no side
// effects, so the request draft (agent, project folder, advanced options)
// keeps its values for the rest of the app session. Closing does reset the
// step to "agent" and clears the reviewed plan, the launch confirmation, and
// any checked warnings: a warning confirmed inside a closed sheet must never
// stay silently confirmed for a later launch.
export function useLaunchFlow(deps: LaunchFlowDeps) {
  const [sheetOpen, setSheetOpen] = useState(false)
  const [request, setRequest] = useState<RunPlanRequest>(() =>
    defaultRunPlanRequest(undefined),
  )
  const [launchStep, setLaunchStep] = useState<LaunchStep>("agent")
  const [plan, setPlan] = useState<RunPlanResponse | null>(null)
  const [planning, setPlanning] = useState(false)
  const [launching, setLaunching] = useState(false)
  const [launchConfirmation, setLaunchConfirmation] = useState(false)
  const [confirmedWarnings, setConfirmedWarnings] = useState<string[]>([])
  const [launchMessage, setLaunchMessage] = useState("")
  // Errors from planning or launching render inside the sheet; the app-level
  // alert would sit behind the modal overlay where nobody can read it.
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
      setSheetOpen(true)
      return
    }
    setSheetOpen(false)
    setLaunchStep("agent")
    setPlan(null)
    setLaunchConfirmation(false)
    setConfirmedWarnings([])
    setError("")
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
    await deps.loadImageStatus(agent.name)
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
    void deps.loadImageStatus(agentName)
  }

  function updateNetworkMode(networkMode: RunPlanRequest["networkMode"]) {
    setRequest((current) => ({ ...current, networkMode }))
    invalidatePlan()
  }

  function updateRequest(patch: Partial<RunPlanRequest>) {
    setRequest((current) => ({ ...current, ...patch }))
    invalidatePlan()
  }

  function invalidatePlan() {
    setPlan(null)
    setLaunchConfirmation(false)
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

  async function startRun() {
    if (!plan) {
      return
    }
    setLaunching(true)
    setError("")
    setLaunchMessage("")
    let started: LaunchRunResponse
    try {
      const launchRequest = request.sessionName
        ? request
        : { ...request, sessionName: plan.session || nextLaunchSessionName() }
      started = await launchRun({
        plan: launchRequest,
        confirmLaunch: launchConfirmation,
        confirmedWarnings,
      })
    } catch (cause) {
      setError(errorMessage(cause))
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
      // The sheet is already closed, so follow-up failures surface app-wide.
      deps.onError(errorMessage(cause))
    } finally {
      setLaunching(false)
    }
  }

  return {
    sheetOpen,
    request,
    launchStep,
    plan,
    planning,
    launching,
    launchConfirmation,
    confirmedWarnings,
    launchMessage,
    error,
    setOpen,
    applyDashboard,
    changeAgent,
    updateNetworkMode,
    updateRequest,
    pickFolder,
    reviewPlan,
    setWarningConfirmation,
    startRun,
    setLaunchConfirmation,
    setLaunchStep,
  }
}
