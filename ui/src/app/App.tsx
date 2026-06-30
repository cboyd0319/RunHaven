import { useEffect, useMemo, useState } from "react"
import { CheckCircle2, CircleAlert, RefreshCw } from "lucide-react"

import {
  chooseProjectFolder,
  defaultRunPlanRequest,
  getAuthLog,
  getAuthStatus,
  getDashboardStatus,
  getEgressLog,
  getImageStatus,
  getLogSnapshot,
  getManagedSessions,
  getRunStatus,
  isLaunchReady,
  killRun,
  launchRun,
  planRun,
  repairRun,
  secureNetworkDefault,
  stopRun,
  type DashboardStatus,
  type ImageStatusResponse,
  type LaunchRunResponse,
  type LogSnapshotResponse,
  type ManagedSessionList,
  type RunPlanRequest,
  type RunPlanResponse,
  type RunStatusResponse,
} from "@/commands/runhaven"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import {
  SidebarInset,
  SidebarProvider,
  SidebarTrigger,
} from "@/components/ui/sidebar"
import { TooltipProvider } from "@/components/ui/tooltip"

import { DiagnosticsCard } from "./DiagnosticsCard"
import { LastLaunchCard } from "./LastLaunchCard"
import { LaunchPlanCard } from "./LaunchPlanCard"
import { ManagedSessionCard } from "./ManagedSessionCard"
import { PlanReviewCard } from "./PlanReviewCard"
import { RunHavenSidebar } from "./RunHavenSidebar"
import { RunOutputCard } from "./RunOutputCard"
import { RunStatusCard } from "./RunStatusCard"
import { SetupCard } from "./SetupCard"
import {
  errorMessage,
  type DiagnosticsState,
  type LaunchStep,
  type RunControlAction,
} from "./types"

export default function App() {
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
  const [imageStatus, setImageStatus] = useState<ImageStatusResponse | null>(
    null,
  )
  const [imageError, setImageError] = useState("")
  const [launchConfirmation, setLaunchConfirmation] = useState(false)
  const [confirmedWarnings, setConfirmedWarnings] = useState<string[]>([])
  const [launchMessage, setLaunchMessage] = useState("")
  const [lastLaunch, setLastLaunch] = useState<LaunchRunResponse | null>(null)
  const [managedSessions, setManagedSessions] = useState<ManagedSessionList>({
    sessions: [],
  })
  const [selectedSessionId, setSelectedSessionId] = useState<string | null>(
    null,
  )
  const [runStatus, setRunStatus] = useState<RunStatusResponse | null>(null)
  const [runStatusLoading, setRunStatusLoading] = useState(false)
  const [runStatusError, setRunStatusError] = useState("")
  const [logSnapshot, setLogSnapshot] = useState<LogSnapshotResponse | null>(
    null,
  )
  const [logAcknowledged, setLogAcknowledged] = useState(false)
  const [logLoading, setLogLoading] = useState(false)
  const [logError, setLogError] = useState("")
  const [controlBusy, setControlBusy] = useState(false)
  const [controlError, setControlError] = useState("")
  const [controlMessage, setControlMessage] = useState("")
  const [controlConfirmations, setControlConfirmations] = useState<
    Record<RunControlAction, boolean>
  >({ stop: false, kill: false, repair: false })
  const [diagnostics, setDiagnostics] = useState<DiagnosticsState | null>(null)
  const [diagnosticsLoading, setDiagnosticsLoading] = useState(false)
  const [diagnosticsError, setDiagnosticsError] = useState("")
  const [error, setError] = useState("")

  const selectedAgent = useMemo(
    () => dashboard?.agents.find((agent) => agent.name === request.agent),
    [dashboard?.agents, request.agent],
  )
  const setupKnown = dashboard !== null
  const setupReady = setupKnown && dashboard.setup.ok
  const launchImageReady = request.image ? true : imageStatus?.image.ready === true
  const launchReady =
    setupReady &&
    isLaunchReady(
      plan,
      launchConfirmation,
      new Set(confirmedWarnings),
      launchImageReady,
    )
  const selectedSession = useMemo(
    () =>
      managedSessions.sessions.find(
        (session) => session.id === selectedSessionId,
      ) ?? null,
    [managedSessions.sessions, selectedSessionId],
  )
  const pageTitle = selectedSession ? "Session" : "Launch"
  const pageDescription = selectedSession
    ? "Review status and access for the selected agent session."
    : "Choose an agent and project. Review access before launch."

  useEffect(() => {
    void refresh()
  }, [])

  useEffect(() => {
    if (
      selectedSessionId &&
      !managedSessions.sessions.some((session) => session.id === selectedSessionId)
    ) {
      setSelectedSessionId(null)
    }
  }, [managedSessions.sessions, selectedSessionId])

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
      const nextPlan = await planRun(request)
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
      const started = await launchRun({
        plan: request,
        confirmLaunch: launchConfirmation,
        confirmedWarnings,
      })
      setLastLaunch(started)
      setLogSnapshot(null)
      setLogAcknowledged(false)
      setLogError("")
      await loadRunStatus(started.runId)
      setLaunchMessage(`Run started: ${started.runId}`)
      await loadManagedSessions()
      setPlan(null)
      setLaunchConfirmation(false)
      setConfirmedWarnings([])
      setLaunchStep("agent")
    } catch (cause) {
      setError(errorMessage(cause))
    } finally {
      setLaunching(false)
    }
  }

  async function loadLogSnapshot() {
    if (!lastLaunch) {
      return
    }
    setLogLoading(true)
    setLogError("")
    try {
      setLogSnapshot(
        await getLogSnapshot(lastLaunch.runId, {
          confirmSensitiveOutput: logAcknowledged,
        }),
      )
    } catch (cause) {
      setLogSnapshot(null)
      setLogError(errorMessage(cause))
    } finally {
      setLogLoading(false)
    }
  }

  async function runControl(action: RunControlAction) {
    if (!lastLaunch) {
      return
    }
    const runId = lastLaunch.runId
    setControlBusy(true)
    setControlError("")
    setControlMessage("")
    try {
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
    } catch (cause) {
      setControlError(errorMessage(cause))
    } finally {
      setControlBusy(false)
    }
  }

  async function loadDiagnostics() {
    setDiagnosticsLoading(true)
    setDiagnosticsError("")
    try {
      const [authStatus, authLog, egressLog] = await Promise.all([
        getAuthStatus(),
        getAuthLog(),
        getEgressLog(),
      ])
      setDiagnostics({ authStatus, authLog, egressLog })
    } catch (cause) {
      setDiagnostics(null)
      setDiagnosticsError(errorMessage(cause))
    } finally {
      setDiagnosticsLoading(false)
    }
  }

  return (
    <TooltipProvider>
      <SidebarProvider>
        <RunHavenSidebar
          sessions={managedSessions.sessions}
          selectedSessionId={selectedSessionId}
          onStartSelect={() => setSelectedSessionId(null)}
          onSessionSelect={setSelectedSessionId}
        />
        <SidebarInset className="h-screen overflow-hidden">
          <header className="flex h-14 shrink-0 items-center gap-3 border-b px-4">
            <SidebarTrigger variant="outline" />
            <div className="min-w-0">
              <h1 className="text-base font-medium">{pageTitle}</h1>
              <p className="text-sm text-muted-foreground">{pageDescription}</p>
            </div>
            <div className="ml-auto flex items-center gap-2">
              <Badge
                variant={
                  !setupKnown || loading
                    ? "outline"
                    : setupReady
                      ? "success"
                      : "destructive"
                }
              >
                {!setupKnown || loading
                  ? "Checking"
                  : setupReady
                    ? "Ready"
                    : "Blocked"}
              </Badge>
              <Button
                type="button"
                variant="outline"
                size="icon-sm"
                onClick={() => void refresh()}
                aria-label="Refresh status"
              >
                <RefreshCw />
              </Button>
            </div>
          </header>

          <main className="min-h-0 flex-1 overflow-y-auto p-4">
            <section className="mx-auto flex w-full max-w-5xl min-w-0 flex-col gap-4">
              {error ? (
                <Alert variant="destructive">
                  <CircleAlert />
                  <AlertTitle>Action failed</AlertTitle>
                  <AlertDescription>{error}</AlertDescription>
                </Alert>
              ) : null}
              {launchMessage ? (
                <Alert>
                  <CheckCircle2 />
                  <AlertTitle>{launchMessage}</AlertTitle>
                  <AlertDescription>
                    RunHaven is tracking the run below.
                  </AlertDescription>
                </Alert>
              ) : null}

              {!setupReady ? (
                <SetupCard loading={loading} dashboard={dashboard} />
              ) : selectedSession ? (
                <ManagedSessionCard session={selectedSession} />
              ) : (
                <>
                  {launchStep === "review" && plan ? (
                    <PlanReviewCard
                      plan={plan}
                      launchConfirmation={launchConfirmation}
                      confirmedWarnings={confirmedWarnings}
                      launchReady={launchReady}
                      launching={launching}
                      setupOk={setupReady}
                      launchImageReady={launchImageReady}
                      onLaunchConfirmationChange={setLaunchConfirmation}
                      onConfirmWarning={setWarningConfirmation}
                      onBack={() => setLaunchStep("project")}
                      onStart={startRun}
                    />
                  ) : (
                    <LaunchPlanCard
                      step={launchStep === "review" ? "project" : launchStep}
                      agents={dashboard?.agents ?? []}
                      request={request}
                      selectedAgent={selectedAgent}
                      imageLoading={imageLoading}
                      imageStatus={imageStatus}
                      imageError={imageError}
                      planning={planning}
                      onAgentChange={changeAgent}
                      onPickFolder={pickFolder}
                      onRequestChange={updateRequest}
                      onNetworkChange={updateNetworkMode}
                      onBack={() => setLaunchStep("agent")}
                      onContinue={() => setLaunchStep("project")}
                      onReviewPlan={reviewPlan}
                    />
                  )}
                </>
              )}

              {!selectedSession && lastLaunch ? (
                <LastLaunchCard lastLaunch={lastLaunch} />
              ) : null}

              {!selectedSession && lastLaunch ? (
                <RunStatusCard
                  runStatus={runStatus}
                  runStatusLoading={runStatusLoading}
                  runStatusError={runStatusError}
                  controlBusy={controlBusy}
                  controlError={controlError}
                  controlMessage={controlMessage}
                  controlConfirmations={controlConfirmations}
                  onConfirmationChange={(action, checked) =>
                    setControlConfirmations((current) => ({
                      ...current,
                      [action]: checked,
                    }))
                  }
                  onControl={runControl}
                />
              ) : null}

              {!selectedSession && lastLaunch ? (
                <RunOutputCard
                  logAcknowledged={logAcknowledged}
                  logLoading={logLoading}
                  logError={logError}
                  logSnapshot={logSnapshot}
                  onAcknowledgedChange={setLogAcknowledged}
                  onLoad={loadLogSnapshot}
                />
              ) : null}

              <details className="rounded-xl border bg-card p-4 text-card-foreground">
                <summary className="cursor-pointer text-sm font-medium">
                  Diagnostics
                </summary>
                <div className="mt-4">
                  <DiagnosticsCard
                    diagnostics={diagnostics}
                    loading={diagnosticsLoading}
                    error={diagnosticsError}
                    onLoad={loadDiagnostics}
                  />
                </div>
              </details>
            </section>
          </main>
        </SidebarInset>
      </SidebarProvider>
    </TooltipProvider>
  )
}
