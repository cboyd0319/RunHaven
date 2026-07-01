import { CheckCircle2, CircleAlert } from "lucide-react"

import type {
  AgentProfile,
  AuthStatusResponse,
  DashboardStatus,
  ImageStatusResponse,
  LaunchRunResponse,
  LaunchStatusResponse,
  ManagedSessionStatusResponse,
  ManagedSessionDiagnosticsResponse,
  ManagedSessionSummary,
  RunPlanRequest,
  RunPlanResponse,
  RunStatusResponse,
} from "@/commands/runhaven"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"

import { DiagnosticsCard } from "./DiagnosticsCard"
import { LastLaunchCard } from "./LastLaunchCard"
import { MaintenanceCard } from "./MaintenanceCard"
import { LaunchStatusCard } from "./LaunchStatusCard"
import { LaunchPlanCard } from "./LaunchPlanCard"
import {
  ManagedSessionCard,
  sessionAllowsCleanup,
  sessionAllowsRunControl,
} from "./ManagedSessionCard"
import { ManagedSessionDiagnosticsCard } from "./ManagedSessionDiagnosticsCard"
import { ManagedSessionStatusCard } from "./ManagedSessionStatusCard"
import { PlanReviewCard } from "./PlanReviewCard"
import { RunOutputCard } from "./RunOutputCard"
import { RunStatusCard } from "./RunStatusCard"
import { SessionInputCard } from "./SessionInputCard"
import { SetupCard } from "./SetupCard"
import type { DiagnosticsState, LaunchStep, RunControlAction } from "./types"
import { useFocusOnChange } from "./useFocusOnChange"
import type { useRunOutput } from "./useRunOutput"
import { useWorktreeReview } from "./useWorktreeReview"
import { sessionAllowsWorktreeReview } from "./worktreeReview"
import { WorktreeReviewCard } from "./WorktreeReviewCard"

type AppMainContentProps = {
  error: string
  launchMessage: string
  loading: boolean
  dashboard: DashboardStatus | null
  authStatus: AuthStatusResponse | null
  setupReady: boolean
  selectedSession: ManagedSessionSummary | null
  selectedSessionStatus: ManagedSessionStatusResponse | null
  selectedSessionStatusLoading: boolean
  selectedSessionStatusError: string
  selectedSessionDiagnostics: ManagedSessionDiagnosticsResponse | null
  selectedSessionDiagnosticsLoading: boolean
  selectedSessionDiagnosticsError: string
  onLoadSelectedSessionDiagnostics: () => Promise<void>
  launchStatus: LaunchStatusResponse | null
  launchStatusLoading: boolean
  launchStatusError: string
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
  onWorktreeResolved: () => Promise<void>
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
  onSignedIn: () => void
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
            <AlertDescription>You can follow it below.</AlertDescription>
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

        {setupReady ? (
          <details className="rounded-xl border bg-card p-4 text-card-foreground">
            <summary className="cursor-pointer text-sm font-medium">Maintenance</summary>
            <div className="mt-4">
              <MaintenanceCard agents={(dashboard?.agents ?? []).map((agent) => agent.name)} />
            </div>
          </details>
        ) : null}
      </section>
    </main>
  )
}

function SessionView(
  props: AppMainContentProps & { selectedSession: ManagedSessionSummary },
) {
  const runControlsAvailable = sessionAllowsRunControl(props.selectedSession)
  const worktreeReviewEligible = sessionAllowsWorktreeReview(props.selectedSession)
  // The worktree review card owns discard, so hide the standalone delete control
  // when it is showing to avoid two ways to remove the same copy.
  const cleanupAvailable =
    sessionAllowsCleanup(props.selectedSession) && !worktreeReviewEligible
  const showRunControlCard =
    runControlsAvailable ||
    cleanupAvailable ||
    Boolean(props.controlMessage || props.controlError)

  // Land focus on this view when a session is opened (launch success or sidebar
  // select) and again when the worktree review resolves and its card unmounts,
  // so keyboard and AT users are not dropped onto <body>.
  const focusRef = useFocusOnChange<HTMLDivElement>(
    `${props.selectedSession.id}:${worktreeReviewEligible}`,
    { focusOnMount: true },
  )

  return (
    <div
      ref={focusRef}
      tabIndex={-1}
      aria-label="Session details"
      className="flex flex-col gap-4 outline-none"
    >
      <ManagedSessionCard session={props.selectedSession} />
      {worktreeReviewEligible ? (
        <SessionWorktreeReview
          session={props.selectedSession}
          onResolved={props.onWorktreeResolved}
        />
      ) : null}
      <ManagedSessionStatusCard
        status={props.selectedSessionStatus}
        loading={props.selectedSessionStatusLoading}
        error={props.selectedSessionStatusError}
      />
      <ManagedSessionDiagnosticsCard
        diagnostics={props.selectedSessionDiagnostics}
        loading={props.selectedSessionDiagnosticsLoading}
        error={props.selectedSessionDiagnosticsError}
        onLoad={props.onLoadSelectedSessionDiagnostics}
      />
      <SessionInputCard
        session={props.selectedSession}
        draft={props.sessionInputDraft}
        sending={props.sessionInputSending}
        error={props.sessionInputError}
        acceptedMessage={props.sessionInputAcceptedMessage}
        onDraftChange={props.onSessionInputDraftChange}
        onSend={props.onSendSelectedSessionInput}
      />
      {showRunControlCard ? (
        <RunStatusCard
          title="Session controls"
          controlTarget="session"
          showStatus={false}
          controlsAvailable={runControlsAvailable}
          cleanupAvailable={cleanupAvailable}
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
      ) : null}
      <OutputView runOutput={props.runOutput} />
    </div>
  )
}

function LaunchView(props: AppMainContentProps) {
  // Focus the launch view when the step changes (Back/Continue/Review), but not
  // on first paint since the launch view can be the initial screen.
  const focusRef = useFocusOnChange<HTMLDivElement>(props.launchStep)
  const content =
    props.launchStep === "review" && props.plan ? (
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
    ) : (
      <LaunchPlanCard
        step={props.launchStep === "review" ? "project" : props.launchStep}
        agents={props.dashboard?.agents ?? []}
        request={props.request}
        selectedAgent={props.selectedAgent}
        authStatus={props.authStatus}
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
        onSignedIn={props.onSignedIn}
      />
    )
  return (
    <div
      ref={focusRef}
      tabIndex={-1}
      aria-label="Launch a run"
      className="flex flex-col gap-4 outline-none"
    >
      {content}
    </div>
  )
}

function RunView(
  props: AppMainContentProps & { lastLaunch: LaunchRunResponse },
) {
  return (
    <>
      <LastLaunchCard lastLaunch={props.lastLaunch} />
      <LaunchStatusCard
        status={props.launchStatus}
        loading={props.launchStatusLoading}
        error={props.launchStatusError}
      />
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

function SessionWorktreeReview({
  session,
  onResolved,
}: {
  session: ManagedSessionSummary
  onResolved: () => Promise<void>
}) {
  const review = useWorktreeReview(session.id, onResolved)
  return (
    <WorktreeReviewCard
      review={review.review}
      loading={review.loading}
      loadError={review.loadError}
      confirmations={review.confirmations}
      busy={review.busy}
      message={review.message}
      nextSteps={review.nextSteps}
      actionError={review.actionError}
      onConfirmationChange={review.setConfirmation}
      onAction={review.runAction}
    />
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
