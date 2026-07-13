import { useState } from "react"
import { CheckCircle2, CircleAlert } from "lucide-react"

import type { ManagedSessionSummary } from "@/commands/runhaven"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Button } from "@/components/ui/button"

import {
  sessionAllowsCleanup,
  sessionAllowsRunControl,
} from "../ManagedSessionCard"
import { SessionAttentionBanner } from "../SessionAttentionBanner"
import { ManagedSessionDiagnosticsCard } from "../ManagedSessionDiagnosticsCard"
import { ManagedSessionStatusCard } from "../ManagedSessionStatusCard"
import { RunOutputCard } from "../RunOutputCard"
import { SessionInputCard } from "../SessionInputCard"
import { TerminalPanel } from "../TerminalPanel"
import type { useRunOutput } from "../useRunOutput"
import { useWorktreeReview } from "../useWorktreeReview"
import { sessionAllowsWorktreeReview } from "../worktreeReview"
import { WorktreeReviewCard } from "../WorktreeReviewCard"
import type { MainViewProps } from "./props"

export function SessionView(
  props: MainViewProps & { selectedSession: ManagedSessionSummary },
) {
  const [detailsOpen, setDetailsOpen] = useState(false)
  const runControlsAvailable = sessionAllowsRunControl(props.selectedSession)
  const worktreeReviewEligible = sessionAllowsWorktreeReview(props.selectedSession)
  // The worktree review card owns discard, so hide the standalone delete control
  // when it is showing to avoid two ways to remove the same copy.
  const cleanupAvailable =
    sessionAllowsCleanup(props.selectedSession) && !worktreeReviewEligible
  // The native-style toolbar owns the selected-session title, status, and
  // normal Stop action. The body keeps the user's work primary and puts
  // output, terminal, maintenance, and diagnostics behind one disclosure.
  return (
    <div className="flex flex-col gap-4">
      <SessionAttentionBanner
        session={props.selectedSession}
        onNavigate={props.onNavigate}
        onShowOutput={() => {
          setDetailsOpen(true)
          void props.runOutput.loadSnapshot()
        }}
        onSignIn={(agent) => {
          props.launchFlow.changeAgent(agent)
          props.launchFlow.setLaunchStep("agent")
          props.onNavigate({ kind: "new-session" })
        }}
        onRepair={() => void props.onControl("repair")}
        outputError={props.runOutput.error}
      />
      {worktreeReviewEligible ? (
        <SessionWorktreeReview
          session={props.selectedSession}
          onResolved={props.onWorktreeResolved}
        />
      ) : null}
      <SessionControlFeedback
        message={props.controlMessage}
        error={props.controlError}
      />
      {runControlsAvailable ? (
        <SessionInputCard
          session={props.selectedSession}
          draft={props.sessionInputDraft}
          sending={props.sessionInputSending}
          error={props.sessionInputError}
          acceptedMessage={props.sessionInputAcceptedMessage}
          onDraftChange={props.onSessionInputDraftChange}
          onSend={props.onSendSelectedSessionInput}
        />
      ) : null}
      <SessionDetails
        {...props}
        open={detailsOpen}
        onOpenChange={setDetailsOpen}
        runControlsAvailable={runControlsAvailable}
        cleanupAvailable={cleanupAvailable}
      />
    </div>
  )
}

function SessionControlFeedback({
  message,
  error,
}: {
  message: string
  error: string
}) {
  if (error) {
    return (
      <Alert variant="destructive">
        <CircleAlert />
        <AlertTitle>Couldn’t update this session</AlertTitle>
        <AlertDescription>{error}</AlertDescription>
      </Alert>
    )
  }
  return message ? (
    <Alert role="status">
      <CheckCircle2 />
      <AlertTitle>{message}</AlertTitle>
    </Alert>
  ) : null
}

// Runtime status and per-session diagnostics as a temporary disclosure, not
// permanent panes: the cards render only while Details is open. The toggle is
// plain show/hide state; it never loads or mutates anything by itself.
function SessionDetails(
  props: MainViewProps & {
    selectedSession: ManagedSessionSummary
    open: boolean
    onOpenChange: (open: boolean) => void
    runControlsAvailable: boolean
    cleanupAvailable: boolean
  },
) {
  return (
    <section className="flex flex-col gap-4">
      <div>
        <Button
          type="button"
          variant="outline"
          aria-expanded={props.open}
          onClick={() => props.onOpenChange(!props.open)}
        >
          {props.open ? "Hide details" : "Show details"}
        </Button>
      </div>
      {props.open ? (
        <>
          <OutputView runOutput={props.runOutput} />
          {props.runControlsAvailable ? (
            <TerminalPanel session={props.selectedSession} />
          ) : null}
          {props.runControlsAvailable ? (
            <ManagedSessionStatusCard
              status={props.selectedSessionStatus}
              loading={props.selectedSessionStatusLoading}
              error={props.selectedSessionStatusError}
            />
          ) : null}
          <ManagedSessionDiagnosticsCard
            diagnostics={props.selectedSessionDiagnostics}
            loading={props.selectedSessionDiagnosticsLoading}
            error={props.selectedSessionDiagnosticsError}
            onLoad={props.onLoadSelectedSessionDiagnostics}
          />
          {props.cleanupAvailable ? (
            <div>
              <Button
                type="button"
                variant="outline"
                disabled={props.controlBusy}
                onClick={() => void props.onControl("cleanup")}
              >
                Delete protected copy
              </Button>
            </div>
          ) : null}
        </>
      ) : null}
    </section>
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
      busy={review.busy}
      message={review.message}
      nextSteps={review.nextSteps}
      actionError={review.actionError}
      onAction={review.runAction}
    />
  )
}

// Shared by the session view and the Home failed-launch fallback: the same
// acknowledgment-gated output surface for both.
export function OutputView({
  runOutput,
}: {
  runOutput: ReturnType<typeof useRunOutput>
}) {
  return (
    <RunOutputCard
      logLoading={runOutput.loading}
      logError={runOutput.error}
      logSnapshot={runOutput.visibleSnapshot}
      onLoad={runOutput.loadSnapshot}
    />
  )
}
