import { useEffect, useRef, useState } from "react"
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
import { sessionAttention } from "../sessionAttention"
import { TerminalPanel } from "../TerminalPanel"
import type { useRunOutput } from "../useRunOutput"
import { useWorktreeReview } from "../useWorktreeReview"
import {
  sessionAllowsWorktreeReview,
  sessionWorktreeReviewState,
} from "../worktreeReview"
import { WorktreeReviewCard } from "../WorktreeReviewCard"
import type { MainViewProps } from "./props"

export function SessionView(
  props: MainViewProps & { selectedSession: ManagedSessionSummary },
) {
  const attention = sessionAttention(props.selectedSession)
  const issueDetails = Boolean(
    attention && attention.kind !== "ready-to-review",
  )
  const [detailsOpen, setDetailsOpen] = useState(issueDetails)
  const loadedIssueDetails = useRef("")
  const issueDetailsKey = issueDetails
    ? `${props.selectedSession.id}:${attention?.kind}`
    : ""

  useEffect(() => {
    if (!issueDetailsKey) {
      return
    }
    setDetailsOpen(true)
    if (loadedIssueDetails.current === issueDetailsKey) {
      return
    }
    loadedIssueDetails.current = issueDetailsKey
    void props.onLoadSelectedSessionDiagnostics()
  }, [issueDetailsKey, props.onLoadSelectedSessionDiagnostics])
  const runControlsAvailable = sessionAllowsRunControl(props.selectedSession)
  const inputAvailable =
    runControlsAvailable &&
    props.selectedSession.input.attached &&
    props.selectedSession.nextAction.kind === "monitor"
  const hasLiveTerminal = props.launchFlow.runTerminals.some(
    (terminal) => terminal.runId === props.selectedSession.runId,
  )
  const returnToTerminal =
    runControlsAvailable &&
    hasLiveTerminal &&
    !props.selectedSession.input.attached &&
    props.selectedSession.nextAction.kind === "monitor"
  const worktreeReviewEligible = sessionAllowsWorktreeReview(props.selectedSession)
  const cleanWorktree =
    sessionWorktreeReviewState(props.selectedSession) === "clean"
  // The worktree review card owns discard, so hide the standalone delete control
  // when it is showing to avoid two ways to remove the same copy.
  const cleanupAvailable =
    sessionAllowsCleanup(props.selectedSession) &&
    !worktreeReviewEligible &&
    !cleanWorktree
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
        onStartNew={(agent) => {
          props.launchFlow.changeAgent(agent)
          props.launchFlow.setLaunchStep("agent")
          props.onNavigate({ kind: "new-session" })
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
      {cleanWorktree ? (
        <CleanWorktreeCard
          busy={props.controlBusy}
          onDelete={() => void props.onControl("cleanup")}
        />
      ) : null}
      <SessionControlFeedback
        message={props.controlMessage}
        error={props.controlError}
      />
      {returnToTerminal ? (
        <div>
          <Button
            type="button"
            variant="outline"
            onClick={() =>
              props.onNavigate({
                kind: "run-terminal",
                id: props.selectedSession.runId,
              })
            }
          >
            Return to terminal
          </Button>
        </div>
      ) : null}
      {inputAvailable ? (
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
        hasLiveTerminal={hasLiveTerminal}
        cleanupAvailable={cleanupAvailable}
        issue={issueDetails}
      />
    </div>
  )
}

function CleanWorktreeCard({
  busy,
  onDelete,
}: {
  busy: boolean
  onDelete: () => void
}) {
  return (
    <section className="flex flex-col items-start gap-3 border-t pt-5">
      <div className="grid gap-1">
        <h2 className="text-base font-semibold">No changes to review</h2>
        <p className="text-sm text-muted-foreground">
          The assistant did not change any files. Your original project is
          unchanged.
        </p>
        <p className="text-sm text-muted-foreground">
          RunHaven kept the protected copy. You can leave it or delete it now.
        </p>
      </div>
      <Button
        type="button"
        variant="outline"
        disabled={busy}
        onClick={onDelete}
      >
        Delete protected copy…
      </Button>
    </section>
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
      <Alert variant="destructive" role="alert">
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
    hasLiveTerminal: boolean
    cleanupAvailable: boolean
    issue: boolean
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
          {props.issue
            ? props.open
              ? "Hide issue details"
              : "Show issue details"
            : props.open
              ? "Hide details"
              : "Show details"}
        </Button>
      </div>
      {props.open ? (
        <>
          {props.selectedSession.outputAvailable === false ? (
            <Alert>
              <CircleAlert />
              <AlertTitle>Output was not kept</AlertTitle>
              <AlertDescription>
                RunHaven keeps session output only until you quit.
              </AlertDescription>
            </Alert>
          ) : (
            <OutputView runOutput={props.runOutput} />
          )}
          {props.runControlsAvailable && !props.hasLiveTerminal ? (
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
                Delete protected copy…
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
      onRetry={review.retry}
      onAction={review.runAction}
    />
  )
}

// Shared by the session view and failed-launch fallback: the same
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
