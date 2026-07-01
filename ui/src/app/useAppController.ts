import { useEffect, useRef, useState } from "react"

import {
  chooseProjectFolder,
  defaultRunPlanRequest,
  getDashboardStatus,
  getAuthStatus,
  launchRun,
  nextLaunchSessionName,
  planRun,
  preferredDefaultAgent,
  reconcileRunPlanRequest,
  secureNetworkDefault,
  withLaunchSessionName,
  type DashboardStatus,
  type AuthStatusResponse,
  type LaunchRunResponse,
  type RunPlanRequest,
  type RunPlanResponse,
} from "@/commands/runhaven"

import { sameDashboardStatus } from "./appStatusEquality"
import { errorMessage, type LaunchStep } from "./types"
import { createSessionActions } from "./sessionActions"
import { loadSessionsUntilRunPresent } from "./sessionSelection"
import {
  sameAuthStatusResponse,
  useDiagnosticsState,
} from "./useDiagnosticsState"
import { useAppDerivedState } from "./useAppDerivedState"
import { useManagedSessionsState } from "./useManagedSessionsState"
import { useManagedSessionDiagnostics } from "./useManagedSessionDiagnostics"
import { useManagedSessionStatus } from "./useManagedSessionStatus"
import { useLaunchStatus } from "./useLaunchStatus"
import { useRunOutput } from "./useRunOutput"
import { useRunControlState } from "./useRunControlState"
import { useSessionInputState } from "./useSessionInputState"
import { useStatusLoaders } from "./useStatusLoaders"

export function useAppController() {
  const [dashboard, setDashboard] = useState<DashboardStatus | null>(null)
  const [authStatus, setAuthStatus] = useState<AuthStatusResponse | null>(null)
  const [request, setRequest] = useState<RunPlanRequest>(() =>
    defaultRunPlanRequest(undefined),
  )
  const [launchStep, setLaunchStep] = useState<LaunchStep>("agent")
  const [plan, setPlan] = useState<RunPlanResponse | null>(null)
  const [loading, setLoading] = useState(true)
  const [planning, setPlanning] = useState(false)
  const [launching, setLaunching] = useState(false)
  const [launchConfirmation, setLaunchConfirmation] = useState(false)
  const [confirmedWarnings, setConfirmedWarnings] = useState<string[]>([])
  const [launchMessage, setLaunchMessage] = useState("")
  const [lastLaunch, setLastLaunch] = useState<LaunchRunResponse | null>(null)
  const {
    managedSessions,
    selectedSessionId,
    selectedSessionIdRef,
    selectSession,
    loadManagedSessions,
  } = useManagedSessionsState(lastLaunch?.runId ?? null)
  const statusLoaders = useStatusLoaders()
  const diagnostics = useDiagnosticsState()
  const [error, setError] = useState("")
  // The agent name the UI currently targets, readable from async refresh
  // completions. Null until the first refresh applies the default selection;
  // that null doubles as the "apply defaults exactly once" guard, so two
  // refreshes racing before the first render commit (a render-closure read of
  // `dashboard` sees null twice) cannot both take the initializing branch and
  // reset the user's networkMode/apiKeyBrokerEnv edits or the default agent.
  const agentNameRef = useRef<string | null>(null)

  const derived = useAppDerivedState({
    dashboard,
    request,
    imageStatus: statusLoaders.imageStatus,
    plan,
    launchConfirmation,
    confirmedWarnings,
    managedSessions,
    selectedSessionId,
    lastLaunch,
  })
  const selectedSessionStatus = useManagedSessionStatus(selectedSessionId)
  const selectedSessionDiagnostics = useManagedSessionDiagnostics(selectedSessionId)
  const selectedSession =
    selectedSessionStatus.data?.session ?? derived.selectedSession
  const launchStatus = useLaunchStatus(
    lastLaunch && !selectedSession ? lastLaunch.runId : null,
  )
  const sessionInput = useSessionInputState(selectedSessionId)
  const runControlState = useRunControlState(derived.controlTargetId)
  const runOutput = useRunOutput({
    targetId: derived.outputTargetId,
    sessionId: derived.outputSessionId,
    runId: derived.outputRunId,
  })

  useEffect(() => {
    void refresh()
  }, [])

  async function refresh() {
    setLoading(true)
    setError("")
    try {
      const nextDashboard = await getDashboardStatus()
      const nextAuthStatus = await getAuthStatus()
      setDashboard((current) =>
        current && sameDashboardStatus(current, nextDashboard)
          ? current
          : nextDashboard,
      )
      setAuthStatus((current) =>
        current && sameAuthStatusResponse(current, nextAuthStatus)
          ? current
          : nextAuthStatus,
      )
      await loadManagedSessions()
      const initializingDashboard = agentNameRef.current === null
      const agent = initializingDashboard
        ? preferredDefaultAgent(nextDashboard.agents)
        : nextDashboard.agents.find(
            (item) => item.name === agentNameRef.current,
          ) ?? nextDashboard.agents[0]
      if (agent) {
        agentNameRef.current = agent.name
        setRequest((current) =>
          reconcileRunPlanRequest(
            initializingDashboard
              ? {
                  ...defaultRunPlanRequest(agent),
                  workspacePath: current.workspacePath,
                  sessionName: current.sessionName,
                }
              : current,
            agent,
          ),
        )
        await statusLoaders.loadImageStatus(agent.name)
      }
      if (lastLaunch) {
        await statusLoaders.loadRunStatus(lastLaunch.runId)
      }
    } catch (cause) {
      setError(errorMessage(cause))
    } finally {
      setLoading(false)
    }
  }

  function changeAgent(agentName: string) {
    agentNameRef.current = agentName
    const agent = dashboard?.agents.find((item) => item.name === agentName)
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
    void statusLoaders.loadImageStatus(agentName)
  }

  function updateNetworkMode(networkMode: RunPlanRequest["networkMode"]) {
    setRequest((current) => ({ ...current, networkMode }))
    invalidatePlan()
  }

  function invalidatePlan() {
    setPlan(null)
    resetLaunchConfirmation()
  }

  function resetLaunchConfirmation() {
    setLaunchConfirmation(false)
    setConfirmedWarnings([])
    setLaunchMessage("")
  }

  function updateRequest(patch: Partial<RunPlanRequest>) {
    setRequest((current) => ({ ...current, ...patch }))
    invalidatePlan()
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
    setPlan(null)
    resetLaunchConfirmation()
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
    try {
      const launchRequest = request.sessionName
        ? request
        : { ...request, sessionName: plan.session || nextLaunchSessionName() }
      const started = await launchRun({
        plan: launchRequest,
        confirmLaunch: launchConfirmation,
        confirmedWarnings,
      })
      setLastLaunch(started)
      await statusLoaders.loadRunStatus(started.runId)
      setLaunchMessage(`Run accepted: ${started.runId}`)
      const { sessionId } = await loadSessionsUntilRunPresent(
        loadManagedSessions,
        started.runId,
      )
      if (sessionId) {
        selectSession(sessionId)
      }
      setPlan(null)
      setLaunchConfirmation(false)
      setConfirmedWarnings([])
      setLaunchStep("agent")
      setRequest((current) => ({
        ...current,
        sessionName: nextLaunchSessionName(),
      }))
    } catch (cause) {
      setError(errorMessage(cause))
    } finally {
      setLaunching(false)
    }
  }

  const sessionActions = createSessionActions({
    selectedSession,
    lastLaunch,
    selectedSessionIdRef,
    selectedSessionStatus,
    sessionInput,
    runControlState,
    statusLoaders,
    loadManagedSessions,
  })

  function changeSessionInputDraft(value: string) {
    sessionInput.changeDraft(value)
  }

  return {
    dashboard,
    authStatus,
    request,
    launchStep,
    plan,
    loading,
    planning,
    launching,
    imageLoading: statusLoaders.imageLoading,
    imageStatus: statusLoaders.imageStatus,
    imageError: statusLoaders.imageError,
    launchConfirmation,
    confirmedWarnings,
    // The "Run accepted" alert lives only until the launched run shows up as a
    // managed session; after that the one-line sidebar notice is the durable
    // affordance, so the alert clears instead of persisting forever.
    launchMessage: derived.lastLaunchSessionId ? "" : launchMessage,
    lastLaunch,
    lastLaunchSessionId: derived.lastLaunchSessionId,
    managedSessions,
    selectedSessionId,
    sessionInputDraft: sessionInput.draft,
    sessionInputSending: sessionInput.sending,
    sessionInputError: sessionInput.error,
    sessionInputAcceptedMessage: sessionInput.acceptedMessage,
    runStatus: statusLoaders.runStatus,
    runStatusLoading: statusLoaders.runStatusLoading,
    runStatusError: statusLoaders.runStatusError,
    controlBusy: runControlState.busy,
    controlError: runControlState.error,
    controlMessage: runControlState.message,
    controlConfirmations: runControlState.confirmations,
    diagnostics: diagnostics.data,
    diagnosticsLoading: diagnostics.loading,
    diagnosticsError: diagnostics.error,
    error,
    selectedAgent: derived.selectedAgent,
    setupKnown: derived.setupKnown,
    setupReady: derived.setupReady,
    launchImageReady: derived.launchImageReady,
    launchReady: derived.launchReady,
    selectedSession,
    selectedSessionStatus: selectedSessionStatus.data,
    selectedSessionStatusLoading: selectedSessionStatus.loading,
    selectedSessionStatusError: selectedSessionStatus.error,
    selectedSessionDiagnostics: selectedSessionDiagnostics.data,
    selectedSessionDiagnosticsLoading: selectedSessionDiagnostics.loading,
    selectedSessionDiagnosticsError: selectedSessionDiagnostics.error,
    launchStatus: launchStatus.data,
    launchStatusLoading: launchStatus.loading,
    launchStatusError: launchStatus.error,
    pageTitle: derived.pageTitle,
    pageDescription: derived.pageDescription,
    runOutput,
    refresh,
    selectSession,
    changeAgent,
    updateNetworkMode,
    updateRequest,
    pickFolder,
    reviewPlan,
    setWarningConfirmation,
    startRun,
    runControl: sessionActions.runControl,
    refreshAfterWorktree: sessionActions.refreshAfterWorktree,
    sendSelectedSessionInput: sessionActions.sendSelectedSessionInput,
    loadSelectedSessionDiagnostics: selectedSessionDiagnostics.load,
    loadDiagnostics: diagnostics.load,
    setLaunchConfirmation,
    setLaunchStep,
    changeSessionInputDraft,
    setControlConfirmation: runControlState.setConfirmation,
  }
}
