import { useState } from "react"

import type { ManagedSessionSummary } from "@/commands/runhaven"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"

import {
  ManagedSessionCard,
  sessionAllowsCleanup,
  sessionAllowsRunControl,
} from "../ManagedSessionCard"
import { useSessionLabels } from "../sessionLabels"
import { MAX_SESSION_LABEL_LENGTH } from "../uiStatePersistence"
import { SessionAttentionBanner } from "../SessionAttentionBanner"
import { ManagedSessionDiagnosticsCard } from "../ManagedSessionDiagnosticsCard"
import { ManagedSessionStatusCard } from "../ManagedSessionStatusCard"
import { RunOutputCard } from "../RunOutputCard"
import { RunStatusCard } from "../RunStatusCard"
import { SessionInputCard } from "../SessionInputCard"
import type { useRunOutput } from "../useRunOutput"
import { useWorktreeReview } from "../useWorktreeReview"
import { sessionAllowsWorktreeReview } from "../worktreeReview"
import { WorktreeReviewCard } from "../WorktreeReviewCard"
import type { MainViewProps } from "./props"

export function SessionView(
  props: MainViewProps & { selectedSession: ManagedSessionSummary },
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

  // Reading order per the navigation spec: the attention banner leads with
  // the one fix, run controls sit at the top as the toolbar approximation
  // until the native menu bar lands (never at the window bottom), a finished
  // worktree leads with its review, then the status header, output, and
  // composer. Runtime and diagnostics detail is demoted behind Details.
  return (
    <div className="flex flex-col gap-4">
      <SessionAttentionBanner
        session={props.selectedSession}
        onNavigate={props.onNavigate}
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
      {worktreeReviewEligible ? (
        <SessionWorktreeReview
          session={props.selectedSession}
          onResolved={props.onWorktreeResolved}
        />
      ) : null}
      <SessionLabelEditor session={props.selectedSession} />
      <ManagedSessionCard session={props.selectedSession} />
      <OutputView runOutput={props.runOutput} />
      <SessionInputCard
        session={props.selectedSession}
        draft={props.sessionInputDraft}
        sending={props.sessionInputSending}
        error={props.sessionInputError}
        acceptedMessage={props.sessionInputAcceptedMessage}
        onDraftChange={props.onSessionInputDraftChange}
        onSend={props.onSendSelectedSessionInput}
      />
      <SessionDetails {...props} />
    </div>
  )
}

// A GUI-only display name for the session, shown right above the title it
// renames. This is a cosmetic label held only in this Mac's WebView storage:
// it never renames the run's real volume identity key, crosses no backend
// command, and needs no capability or native dialog. A blank name clears the
// label so the title falls back to the derived "Agent - project" text.
function SessionLabelEditor({ session }: { session: ManagedSessionSummary }) {
  const { labels, setLabel } = useSessionLabels()
  const current = labels[session.id] ?? ""
  const [editing, setEditing] = useState(false)
  const [draft, setDraft] = useState(current)
  const inputId = `session-label-${session.id}`

  function start() {
    setDraft(current)
    setEditing(true)
  }

  function save() {
    setLabel(session.id, draft)
    setEditing(false)
  }

  if (!editing) {
    return (
      <div>
        <Button type="button" variant="outline" size="sm" onClick={start}>
          {current ? "Rename session" : "Name this session"}
        </Button>
      </div>
    )
  }

  return (
    <form
      className="flex flex-col gap-2"
      onSubmit={(event) => {
        event.preventDefault()
        save()
      }}
    >
      <Label htmlFor={inputId}>Session name</Label>
      <div className="flex flex-wrap items-center gap-2">
        <Input
          id={inputId}
          value={draft}
          maxLength={MAX_SESSION_LABEL_LENGTH}
          autoFocus
          placeholder={session.title}
          className="max-w-xs"
          onChange={(event) => setDraft(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === "Escape") {
              setEditing(false)
            }
          }}
        />
        <Button type="submit" size="sm">
          Save
        </Button>
        <Button
          type="button"
          variant="ghost"
          size="sm"
          onClick={() => setEditing(false)}
        >
          Cancel
        </Button>
      </div>
      <p className="text-xs text-muted-foreground">
        This name shows only in this app on this Mac. Leave it blank to use the
        agent and project name. It does not rename the session itself.
      </p>
    </form>
  )
}

// Runtime status and per-session diagnostics as a temporary disclosure, not
// permanent panes: the cards render only while Details is open. The toggle is
// plain show/hide state; it never loads or mutates anything by itself.
function SessionDetails(props: MainViewProps) {
  const [open, setOpen] = useState(false)
  return (
    <section className="flex flex-col gap-4">
      <div>
        <Button
          type="button"
          variant="outline"
          aria-expanded={open}
          onClick={() => setOpen((current) => !current)}
        >
          {open ? "Hide details" : "Show details"}
        </Button>
      </div>
      {open ? (
        <>
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

// Shared by the session view and the Home failed-launch fallback: the same
// acknowledgment-gated output surface for both.
export function OutputView({
  runOutput,
}: {
  runOutput: ReturnType<typeof useRunOutput>
}) {
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
