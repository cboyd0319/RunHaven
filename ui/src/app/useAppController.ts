import { useEffect, useState } from "react"

import {
  chooseProjectFolder,
  cleanupManagedSession,
  defaultRunPlanRequest,
  getDashboardStatus,
  getAuthStatus,
  killManagedSession,
  killRun,
  launchRun,
  nextLaunchSessionName,
  planRun,
  reconcileRunPlanRequest,
  repairManagedSession,
  repairRun,
  secureNetworkDefault,
  stopManagedSession,
  stopRun,
  withLaunchSessionName,
  writeSessionInput,
  type DashboardStatus,
  type AuthStatusResponse,
  type LaunchRunResponse,
  type RunPlanRequest,
  type RunPlanResponse,
} from "@/commands/runhaven"

import { sameDashboardStatus } from "./appStatusEquality"
import {
  errorMessage,
  type LaunchStep,
  type RunControlAction,
} from "./types"
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
      const agent =
        nextDashboard.agents.find((item) => item.name === request.agent) ??
        nextDashboard.agents[0]
      if (agent) {
        const initializingDashboard = dashboard === null
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

  async function runControl(action: RunControlAction) {
    if (!selectedSession && !lastLaunch) {
      return
    }
    runControlState.setBusy(true)
    runControlState.setError("")
    runControlState.setMessage("")
    try {
      if (selectedSession) {
        const sessionId = selectedSession.id
        if (action === "stop") {
          const { result } = await stopManagedSession(sessionId)
          runControlState.setMessage(`Stop requested for ${result.runId}.`)
        } else if (action === "kill") {
          const { result } = await killManagedSession(sessionId)
          runControlState.setMessage(`Hard stop requested for ${result.runId}.`)
        } else if (action === "repair") {
          const { result } = await repairManagedSession(sessionId)
          runControlState.setMessage(
            result.markerRemoved
              ? `Stale marker cleared for ${result.runId}.`
              : `Repair for ${result.runId}: ${result.status}.`,
          )
        } else {
          const { result } = await cleanupManagedSession(sessionId)
          runControlState.setMessage(
            `Cleaned up ${result.target} for ${result.runId}.`,
          )
        }
        await selectedSessionStatus.loadStatus()
        await loadManagedSessions()
      } else {
        const runId = lastLaunch?.runId
        if (!runId) {
          return
        }
        if (action === "stop") {
          const result = await stopRun(runId)
          runControlState.setMessage(`Stop requested for ${result.runId}.`)
        } else if (action === "kill") {
          const result = await killRun(runId)
          runControlState.setMessage(`Hard stop requested for ${result.runId}.`)
        } else if (action === "repair") {
          const result = await repairRun(runId)
          runControlState.setMessage(
            result.markerRemoved
              ? `Stale marker cleared for ${result.runId}.`
              : `Repair for ${result.runId}: ${result.status}.`,
          )
        } else {
          return
        }
        await statusLoaders.loadRunStatus(runId)
        await loadManagedSessions()
      }
    } catch (cause) {
      runControlState.setError(errorMessage(cause))
    } finally {
      runControlState.resetConfirmations()
      runControlState.setBusy(false)
    }
  }

  async function sendSelectedSessionInput() {
    if (!selectedSession || sessionInput.sending) {
      return
    }
    const targetSessionId = selectedSession.id
    const input = sessionInput.draft
    sessionInput.setSendingFor(targetSessionId, true)
    sessionInput.setErrorFor(targetSessionId, "")
    sessionInput.setAcceptedMessageFor(targetSessionId, "")
    try {
      const response = await writeSessionInput(targetSessionId, input)
      sessionInput.setDraftFor(targetSessionId, "")
      sessionInput.setAcceptedMessageFor(
        targetSessionId,
        `Sent ${response.acceptedBytes} bytes.`,
      )
      if (selectedSessionIdRef.current === targetSessionId) {
        await selectedSessionStatus.loadStatus()
      }
      await loadManagedSessions()
    } catch (cause) {
      sessionInput.setErrorFor(targetSessionId, errorMessage(cause))
    } finally {
      sessionInput.setSendingFor(targetSessionId, false)
    }
  }

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
    launchMessage,
    lastLaunch,
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
    runOutput: derived.runOutput,
    refresh,
    selectSession,
    changeAgent,
    updateNetworkMode,
    updateRequest,
    pickFolder,
    reviewPlan,
    setWarningConfirmation,
    startRun,
    runControl,
    sendSelectedSessionInput,
    loadSelectedSessionDiagnostics: selectedSessionDiagnostics.load,
    loadDiagnostics: diagnostics.load,
    setLaunchConfirmation,
    setLaunchStep,
    changeSessionInputDraft,
    setControlConfirmation: runControlState.setConfirmation,
  }
}
