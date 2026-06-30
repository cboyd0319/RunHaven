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
import { PlanReviewCard } from "./PlanReviewCard"
import { RunHavenSidebar } from "./RunHavenSidebar"
import { RunOutputCard } from "./RunOutputCard"
import { RunStatusCard } from "./RunStatusCard"
import { SafetyCard } from "./SafetyCard"
import { SetupCard } from "./SetupCard"
import {
  errorMessage,
  type DiagnosticsState,
  type RunControlAction,
} from "./types"

export default function App() {
  const [dashboard, setDashboard] = useState<DashboardStatus | null>(null)
  const [request, setRequest] = useState<RunPlanRequest>(() =>
    defaultRunPlanRequest(undefined),
  )
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
  const launchImageReady = request.image ? true : imageStatus?.image.ready === true
  const launchReady =
    Boolean(dashboard?.setup.ok) &&
    isLaunchReady(
      plan,
      launchConfirmation,
      new Set(confirmedWarnings),
      launchImageReady,
    )

  useEffect(() => {
    void refresh()
  }, [])

  async function refresh() {
    setLoading(true)
    setError("")
    try {
      const nextDashboard = await getDashboardStatus()
      setDashboard(nextDashboard)
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
      setPlan(await planRun(request))
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
      setPlan(null)
      setLaunchConfirmation(false)
      setConfirmedWarnings([])
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
        <RunHavenSidebar />
        <SidebarInset>
          <header className="flex h-14 shrink-0 items-center gap-3 border-b px-4">
            <SidebarTrigger variant="outline" />
            <div className="min-w-0">
              <h1 className="text-base font-medium">Start safely</h1>
              <p className="text-sm text-muted-foreground">
                Choose an agent, see what it can access, then start.
              </p>
            </div>
            <div className="ml-auto flex items-center gap-2">
              <Badge variant="secondary">macOS local</Badge>
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

          <main className="grid gap-4 p-4 lg:grid-cols-[minmax(0,1fr)_360px]">
            <section className="flex min-w-0 flex-col gap-4">
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

              <LaunchPlanCard
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
                onReviewPlan={reviewPlan}
              />

              {plan ? (
                <PlanReviewCard
                  plan={plan}
                  launchConfirmation={launchConfirmation}
                  confirmedWarnings={confirmedWarnings}
                  launchReady={launchReady}
                  launching={launching}
                  setupOk={Boolean(dashboard?.setup.ok)}
                  launchImageReady={launchImageReady}
                  onLaunchConfirmationChange={setLaunchConfirmation}
                  onConfirmWarning={setWarningConfirmation}
                  onStart={startRun}
                />
              ) : null}

              {lastLaunch ? <LastLaunchCard lastLaunch={lastLaunch} /> : null}

              {lastLaunch ? (
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

              {lastLaunch ? (
                <RunOutputCard
                  logAcknowledged={logAcknowledged}
                  logLoading={logLoading}
                  logError={logError}
                  logSnapshot={logSnapshot}
                  onAcknowledgedChange={setLogAcknowledged}
                  onLoad={loadLogSnapshot}
                />
              ) : null}
            </section>

            <aside className="flex min-w-0 flex-col gap-4">
              <SetupCard loading={loading} dashboard={dashboard} />
              <SafetyCard request={request} imageReady={launchImageReady} />
              <DiagnosticsCard
                diagnostics={diagnostics}
                loading={diagnosticsLoading}
                error={diagnosticsError}
                onLoad={loadDiagnostics}
              />
            </aside>
          </main>
        </SidebarInset>
      </SidebarProvider>
    </TooltipProvider>
  )
}
