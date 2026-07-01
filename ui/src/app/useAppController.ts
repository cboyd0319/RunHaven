import { useEffect, useRef, useState } from "react"

import {
  chooseProjectFolder,
  defaultRunPlanRequest,
  getDashboardStatus,
  getImageStatus,
  getManagedSessions,
  getRunStatus,
  killManagedSession,
  killRun,
  launchRun,
  nextLaunchSessionName,
  planRun,
  repairManagedSession,
  repairRun,
  secureNetworkDefault,
  stopManagedSession,
  stopRun,
  withLaunchSessionName,
  writeSessionInput,
  type DashboardStatus,
  type ImageStatusResponse,
  type LaunchRunResponse,
  type ManagedSessionList,
  type RunPlanRequest,
  type RunPlanResponse,
  type RunStatusResponse,
} from "@/commands/runhaven"

import {
  errorMessage,
  type LaunchStep,
  type RunControlAction,
} from "./types"
import { useDiagnosticsState } from "./useDiagnosticsState"
import { useAppDerivedState } from "./useAppDerivedState"

export function useAppController() {
  const [dashboard, setDashboard] = useState<DashboardStatus | null>(null)
  const [request, setRequest] = useState<RunPlanRequest>(() =>
    defaultRunPlanRequest(undefined),
  )
  const [launchStep, setLaunchStep] = useState<LaunchStep>("agent")
  const [plan, setPlan] = useState<RunPlanResponse | null>(null)
  const [loading, setLoading] = useState(true)
  const [planning, setPlanning] = useState(false)
  const [launching, setLaunching] = useState(false)
  const [imageLoading, setImageLoading] = useState(false)
  const [imageStatus, setImageStatus] = useState<ImageStatusResponse | null>(null)
  const [imageError, setImageError] = useState("")
  const [launchConfirmation, setLaunchConfirmation] = useState(false)
  const [confirmedWarnings, setConfirmedWarnings] = useState<string[]>([])
  const [launchMessage, setLaunchMessage] = useState("")
  const [lastLaunch, setLastLaunch] = useState<LaunchRunResponse | null>(null)
  const [managedSessions, setManagedSessions] = useState<ManagedSessionList>({
    sessions: [],
  })
  const [selectedSessionId, setSelectedSessionId] = useState<string | null>(null)
  const selectedSessionIdRef = useRef<string | null>(null)
  const [sessionInputDraft, setSessionInputDraft] = useState("")
  const [sessionInputSending, setSessionInputSending] = useState(false)
  const [sessionInputError, setSessionInputError] = useState("")
  const [sessionInputAcceptedMessage, setSessionInputAcceptedMessage] = useState("")
  const [runStatus, setRunStatus] = useState<RunStatusResponse | null>(null)
  const [runStatusLoading, setRunStatusLoading] = useState(false)
  const [runStatusError, setRunStatusError] = useState("")
  const [controlBusy, setControlBusy] = useState(false)
  const [controlError, setControlError] = useState("")
  const [controlMessage, setControlMessage] = useState("")
  const [controlConfirmations, setControlConfirmations] = useState<
    Record<RunControlAction, boolean>
  >({ stop: false, kill: false, repair: false })
  const diagnostics = useDiagnosticsState()
  const [error, setError] = useState("")

  const derived = useAppDerivedState({
    dashboard,
    request,
    imageStatus,
    plan,
    launchConfirmation,
    confirmedWarnings,
    managedSessions,
    selectedSessionId,
    lastLaunch,
  })

  useEffect(() => {
    void refresh()
  }, [])

  useEffect(() => {
    if (
      selectedSessionId &&
      !managedSessions.sessions.some((session) => session.id === selectedSessionId)
    ) {
      selectSession(null)
    }
  }, [managedSessions.sessions, selectedSessionId])

  useEffect(() => {
    selectedSessionIdRef.current = selectedSessionId
    setSessionInputDraft("")
    setSessionInputSending(false)
    setSessionInputError("")
    setSessionInputAcceptedMessage("")
  }, [selectedSessionId])

  useEffect(() => {
    setControlConfirmations({ stop: false, kill: false, repair: false })
    setControlError("")
    setControlMessage("")
  }, [derived.controlTargetId])

  async function refresh() {
    setLoading(true)
    setError("")
    try {
      const nextDashboard = await getDashboardStatus()
      setDashboard(nextDashboard)
      await loadManagedSessions()
      const agent =
        nextDashboard.agents.find((item) => item.name === request.agent) ??
        nextDashboard.agents[0]
      if (agent) {
        setRequest((current) => ({
          ...defaultRunPlanRequest(agent),
          workspacePath: current.workspacePath,
          sessionName: current.sessionName,
        }))
        await loadImageStatus(agent.name)
      }
      if (lastLaunch) {
        await loadRunStatus(lastLaunch.runId)
      }
    } catch (cause) {
      setError(errorMessage(cause))
    } finally {
      setLoading(false)
    }
  }

  async function loadImageStatus(agentName: string) {
    setImageLoading(true)
    setImageError("")
    try {
      setImageStatus(await getImageStatus(agentName))
    } catch (cause) {
      setImageStatus(null)
      setImageError(errorMessage(cause))
    } finally {
      setImageLoading(false)
    }
  }

  async function loadRunStatus(runId: string) {
    setRunStatusLoading(true)
    setRunStatusError("")
    try {
      setRunStatus(await getRunStatus(runId))
    } catch (cause) {
      setRunStatus(null)
      setRunStatusError(errorMessage(cause))
    } finally {
      setRunStatusLoading(false)
    }
  }

  async function loadManagedSessions() {
    setManagedSessions(await getManagedSessions())
  }

  function selectSession(sessionId: string | null) {
    selectedSessionIdRef.current = sessionId
    setSelectedSessionId(sessionId)
  }

  function changeAgent(agentName: string) {
    const agent = dashboard?.agents.find((item) => item.name === agentName)
    setRequest((current) => ({
      ...current,
      agent: agentName,
      networkMode: secureNetworkDefault(agent),
    }))
    invalidatePlan()
    void loadImageStatus(agentName)
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
      await loadRunStatus(started.runId)
      setLaunchMessage(`Run started: ${started.runId}`)
      await loadManagedSessions()
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
    if (!derived.selectedSession && !lastLaunch) {
      return
    }
    setControlBusy(true)
    setControlError("")
    setControlMessage("")
    try {
      if (derived.selectedSession) {
        const sessionId = derived.selectedSession.id
        if (action === "stop") {
          const { result } = await stopManagedSession(sessionId)
          setControlMessage(`Stop requested for ${result.runId}.`)
        } else if (action === "kill") {
          const { result } = await killManagedSession(sessionId)
          setControlMessage(`Hard stop requested for ${result.runId}.`)
        } else {
          const { result } = await repairManagedSession(sessionId)
          setControlMessage(
            result.markerRemoved
              ? `Stale marker cleared for ${result.runId}.`
              : `Repair for ${result.runId}: ${result.status}.`,
          )
        }
        await loadManagedSessions()
      } else {
        const runId = lastLaunch?.runId
        if (!runId) {
          return
        }
        if (action === "stop") {
          const result = await stopRun(runId)
          setControlMessage(`Stop requested for ${result.runId}.`)
        } else if (action === "kill") {
          const result = await killRun(runId)
          setControlMessage(`Hard stop requested for ${result.runId}.`)
        } else {
          const result = await repairRun(runId)
          setControlMessage(
            result.markerRemoved
              ? `Stale marker cleared for ${result.runId}.`
              : `Repair for ${result.runId}: ${result.status}.`,
          )
        }
        await loadRunStatus(runId)
        await loadManagedSessions()
      }
    } catch (cause) {
      setControlError(errorMessage(cause))
    } finally {
      setControlBusy(false)
    }
  }

  async function sendSelectedSessionInput() {
    if (!derived.selectedSession || sessionInputSending) {
      return
    }
    const targetSessionId = derived.selectedSession.id
    const input = sessionInputDraft
    setSessionInputSending(true)
    setSessionInputError("")
    setSessionInputAcceptedMessage("")
    try {
      const response = await writeSessionInput(targetSessionId, input)
      if (selectedSessionIdRef.current !== targetSessionId) {
        return
      }
      setSessionInputDraft("")
      setSessionInputAcceptedMessage(`Sent ${response.acceptedBytes} bytes.`)
      await loadManagedSessions()
    } catch (cause) {
      if (selectedSessionIdRef.current === targetSessionId) {
        setSessionInputError(errorMessage(cause))
      }
    } finally {
      if (selectedSessionIdRef.current === targetSessionId) {
        setSessionInputSending(false)
      }
    }
  }

  function changeSessionInputDraft(value: string) {
    setSessionInputDraft(value)
    setSessionInputError("")
    setSessionInputAcceptedMessage("")
  }

  function setControlConfirmation(action: RunControlAction, checked: boolean) {
    setControlConfirmations((current) => ({
      ...current,
      [action]: checked,
    }))
  }

  return {
    dashboard,
    request,
    launchStep,
    plan,
    loading,
    planning,
    launching,
    imageLoading,
    imageStatus,
    imageError,
    launchConfirmation,
    confirmedWarnings,
    launchMessage,
    lastLaunch,
    managedSessions,
    selectedSessionId,
    sessionInputDraft,
    sessionInputSending,
    sessionInputError,
    sessionInputAcceptedMessage,
    runStatus,
    runStatusLoading,
    runStatusError,
    controlBusy,
    controlError,
    controlMessage,
    controlConfirmations,
    diagnostics: diagnostics.data,
    diagnosticsLoading: diagnostics.loading,
    diagnosticsError: diagnostics.error,
    error,
    selectedAgent: derived.selectedAgent,
    setupKnown: derived.setupKnown,
    setupReady: derived.setupReady,
    launchImageReady: derived.launchImageReady,
    launchReady: derived.launchReady,
    selectedSession: derived.selectedSession,
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
    loadDiagnostics: diagnostics.load,
    setLaunchConfirmation,
    setLaunchStep,
    changeSessionInputDraft,
    setControlConfirmation,
  }
}
