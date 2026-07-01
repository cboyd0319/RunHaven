import { CheckCircle2, CircleAlert } from "lucide-react"

import type {
  AgentProfile,
  DashboardStatus,
  ImageStatusResponse,
  LaunchRunResponse,
  ManagedSessionSummary,
  RunPlanRequest,
  RunPlanResponse,
  RunStatusResponse,
} from "@/commands/runhaven"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"

import { DiagnosticsCard } from "./DiagnosticsCard"
import { LastLaunchCard } from "./LastLaunchCard"
import { LaunchPlanCard } from "./LaunchPlanCard"
import { ManagedSessionCard } from "./ManagedSessionCard"
import { PlanReviewCard } from "./PlanReviewCard"
import { RunOutputCard } from "./RunOutputCard"
import { RunStatusCard } from "./RunStatusCard"
import { SessionInputCard } from "./SessionInputCard"
import { SetupCard } from "./SetupCard"
import type { DiagnosticsState, LaunchStep, RunControlAction } from "./types"
import type { useRunOutput } from "./useRunOutput"

type AppMainContentProps = {
  error: string
  launchMessage: string
  loading: boolean
  dashboard: DashboardStatus | null
  setupReady: boolean
  selectedSession: ManagedSessionSummary | null
  sessionInputDraft: string
  sessionInputSending: boolean
  sessionInputError: string
  sessionInputAcceptedMessage: string
  onSessionInputDraftChange: (value: string) => void
  onSendSelectedSessionInput: () => Promise<void>
  runOutput: ReturnType<typeof useRunOutput>
  controlBusy: boolean
  controlError: string
  controlMessage: string
  controlConfirmations: Record<RunControlAction, boolean>
  onControlConfirmationChange: (action: RunControlAction, checked: boolean) => void
  onControl: (action: RunControlAction) => Promise<void>
  launchStep: LaunchStep
  plan: RunPlanResponse | null
  launchConfirmation: boolean
  confirmedWarnings: string[]
  launchReady: boolean
  launching: boolean
  launchImageReady: boolean
  onLaunchConfirmationChange: (checked: boolean) => void
  onConfirmWarning: (code: string, checked: boolean) => void
  onLaunchStepChange: (step: LaunchStep) => void
  onStart: () => Promise<void>
  request: RunPlanRequest
  selectedAgent: AgentProfile | undefined
  imageLoading: boolean
  imageStatus: ImageStatusResponse | null
  imageError: string
  planning: boolean
  onAgentChange: (agentName: string) => void
  onPickFolder: () => Promise<void>
  onRequestChange: (patch: Partial<RunPlanRequest>) => void
  onNetworkChange: (networkMode: RunPlanRequest["networkMode"]) => void
  onReviewPlan: () => Promise<void>
  lastLaunch: LaunchRunResponse | null
  runStatus: RunStatusResponse | null
  runStatusLoading: boolean
  runStatusError: string
  diagnostics: DiagnosticsState | null
  diagnosticsLoading: boolean
  diagnosticsError: string
  onLoadDiagnostics: () => Promise<void>
}

export function AppMainContent(props: AppMainContentProps) {
  const {
    error,
    launchMessage,
    loading,
    dashboard,
    setupReady,
    selectedSession,
    lastLaunch,
  } = props

  return (
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
            <AlertDescription>RunHaven is tracking the run below.</AlertDescription>
          </Alert>
        ) : null}

        {!setupReady ? (
          <SetupCard loading={loading} dashboard={dashboard} />
        ) : selectedSession ? (
          <SessionView {...props} selectedSession={selectedSession} />
        ) : (
          <LaunchView {...props} />
        )}

        {!selectedSession && lastLaunch ? <RunView {...props} lastLaunch={lastLaunch} /> : null}

        <details className="rounded-xl border bg-card p-4 text-card-foreground">
          <summary className="cursor-pointer text-sm font-medium">Diagnostics</summary>
          <div className="mt-4">
            <DiagnosticsCard
              diagnostics={props.diagnostics}
              loading={props.diagnosticsLoading}
              error={props.diagnosticsError}
              onLoad={props.onLoadDiagnostics}
            />
          </div>
        </details>
      </section>
    </main>
  )
}

function SessionView(
  props: AppMainContentProps & { selectedSession: ManagedSessionSummary },
) {
  return (
    <>
      <ManagedSessionCard session={props.selectedSession} />
      <SessionInputCard
        session={props.selectedSession}
        draft={props.sessionInputDraft}
        sending={props.sessionInputSending}
        error={props.sessionInputError}
        acceptedMessage={props.sessionInputAcceptedMessage}
        onDraftChange={props.onSessionInputDraftChange}
        onSend={props.onSendSelectedSessionInput}
      />
      <RunStatusCard
        title="Session controls"
        controlTarget="session"
        showStatus={false}
        runStatus={null}
        runStatusLoading={false}
        runStatusError=""
        controlBusy={props.controlBusy}
        controlError={props.controlError}
        controlMessage={props.controlMessage}
        controlConfirmations={props.controlConfirmations}
        onConfirmationChange={props.onControlConfirmationChange}
        onControl={props.onControl}
      />
      <OutputView runOutput={props.runOutput} />
    </>
  )
}

function LaunchView(props: AppMainContentProps) {
  if (props.launchStep === "review" && props.plan) {
    return (
      <PlanReviewCard
        plan={props.plan}
        launchConfirmation={props.launchConfirmation}
        confirmedWarnings={props.confirmedWarnings}
        launchReady={props.launchReady}
        launching={props.launching}
        setupOk={props.setupReady}
        launchImageReady={props.launchImageReady}
        onLaunchConfirmationChange={props.onLaunchConfirmationChange}
        onConfirmWarning={props.onConfirmWarning}
        onBack={() => props.onLaunchStepChange("project")}
        onStart={props.onStart}
      />
    )
  }
  return (
    <LaunchPlanCard
      step={props.launchStep === "review" ? "project" : props.launchStep}
      agents={props.dashboard?.agents ?? []}
      request={props.request}
      selectedAgent={props.selectedAgent}
      imageLoading={props.imageLoading}
      imageStatus={props.imageStatus}
      imageError={props.imageError}
      planning={props.planning}
      onAgentChange={props.onAgentChange}
      onPickFolder={props.onPickFolder}
      onRequestChange={props.onRequestChange}
      onNetworkChange={props.onNetworkChange}
      onBack={() => props.onLaunchStepChange("agent")}
      onContinue={() => props.onLaunchStepChange("project")}
      onReviewPlan={props.onReviewPlan}
    />
  )
}

function RunView(
  props: AppMainContentProps & { lastLaunch: LaunchRunResponse },
) {
  return (
    <>
      <LastLaunchCard lastLaunch={props.lastLaunch} />
      <RunStatusCard
        runStatus={props.runStatus}
        runStatusLoading={props.runStatusLoading}
        runStatusError={props.runStatusError}
        controlBusy={props.controlBusy}
        controlError={props.controlError}
        controlMessage={props.controlMessage}
        controlConfirmations={props.controlConfirmations}
        onConfirmationChange={props.onControlConfirmationChange}
        onControl={props.onControl}
      />
      <OutputView runOutput={props.runOutput} />
    </>
  )
}

function OutputView({ runOutput }: { runOutput: ReturnType<typeof useRunOutput> }) {
  return (
    <RunOutputCard
      logAcknowledged={runOutput.acknowledged}
      logLoading={runOutput.loading}
      logError={runOutput.error}
      logSnapshot={runOutput.visibleSnapshot}
      liveOutputAvailable={runOutput.liveAvailable}
      liveOutputEnabled={runOutput.liveEnabled}
      liveOutputRefreshing={runOutput.liveRefreshing}
      onAcknowledgedChange={runOutput.setAcknowledged}
      onLoad={runOutput.loadSnapshot}
      onLiveOutputChange={runOutput.setLiveEnabled}
    />
  )
}
