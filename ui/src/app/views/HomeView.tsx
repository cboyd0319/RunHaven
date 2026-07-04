import { ArrowUpRight, CheckCircle2, CircleAlert, Loader2 } from "lucide-react"

import type { LaunchRunResponse } from "@/commands/runhaven"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Button } from "@/components/ui/button"

import { AgentSetupPanel } from "../dashboard/AgentSetupPanel"
import { NeedsReviewPanel } from "../dashboard/NeedsReviewPanel"
import { RecentActivityPanel } from "../dashboard/RecentActivityPanel"
import { SecureBoundaryPanel } from "../dashboard/SecureBoundaryPanel"
import { LastLaunchCard } from "../LastLaunchCard"
import { LaunchSheet } from "../LaunchSheet"
import { LaunchStatusCard } from "../LaunchStatusCard"
import { RunStatusCard } from "../RunStatusCard"
import { sessionAttention } from "../sessionAttention"
import { SetupCard } from "../SetupCard"
import type { MainViewProps } from "./props"
import { OutputView } from "./SessionView"

// Home (design variant 06, HIG foundation): a daily command surface. A readiness
// line and the one New run action lead, then two columns -- Needs review and
// Recent activity on the left, Agent setup and the Secure boundary summary on
// the right. Every panel is a read-only summary of backend-owned state; the New
// Run sheet is the only launch path and the Rust native confirm stays the
// mutation boundary. Setup blockers render inline here, never as a takeover.
export function HomeView(props: MainViewProps) {
  const needsReview = props.sessions.filter(
    (session) => sessionAttention(session) !== null,
  )
  // Recent activity shows the newest runs of any lifecycle EXCEPT the ones in
  // Needs review, so each run appears once. Including running sessions keeps a
  // just-launched session reachable now that the sidebar lists no sessions.
  const recent = props.sessions
    .filter((session) => sessionAttention(session) === null)
    .slice()
    .sort(
      (left, right) =>
        right.startedAt.localeCompare(left.startedAt) ||
        right.id.localeCompare(left.id, undefined, { numeric: true }),
    )
    .slice(0, 6)

  return (
    <div className="flex flex-col gap-5">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <ReadinessLine {...props} />
        <LaunchSheet
          flow={props.launchFlow}
          agents={props.dashboard?.agents ?? []}
          selectedAgent={props.selectedAgent}
          authStatus={props.authStatus}
          imageLoading={props.imageLoading}
          imageStatus={props.imageStatus}
          imageError={props.imageError}
          setupReady={props.setupReady}
          launchReady={props.launchReady}
          launchImageReady={props.launchImageReady}
          onSignedIn={props.onSignedIn}
          disabledReason={newRunDisabledReason(props)}
        />
      </div>

      {!props.setupReady ? (
        <SetupCard
          loading={props.loading}
          dashboard={props.dashboard}
          onCheckAgain={props.onRefresh}
        />
      ) : null}

      <div className="grid min-w-0 gap-5 lg:grid-cols-[minmax(0,1.7fr)_minmax(0,1fr)]">
        <div className="flex min-w-0 flex-col gap-5">
          {needsReview.length > 0 ? (
            <NeedsReviewPanel
              sessions={needsReview}
              onSelectSession={props.onSelectSession}
            />
          ) : null}
          <RecentActivityPanel
            sessions={recent}
            onSelectSession={props.onSelectSession}
            onOpenHistory={() => props.onNavigate({ kind: "home" })}
          />
        </div>
        <div className="flex min-w-0 flex-col gap-5">
          <AgentSetupPanel
            agents={props.dashboard?.agents ?? []}
            authStatus={props.authStatus}
            onOpenAgents={() => props.onNavigate({ kind: "agents" })}
          />
          <SecureBoundaryPanel
            onViewModel={() => props.onNavigate({ kind: "diagnostics" })}
          />
        </div>
      </div>

      {props.lastLaunch ? (
        props.lastLaunchSessionId ? (
          <LaunchedRunNotice
            sessionId={props.lastLaunchSessionId}
            onSelect={props.onSelectSession}
          />
        ) : (
          <RunView {...props} lastLaunch={props.lastLaunch} />
        )
      ) : null}
    </div>
  )
}

// The one-line readiness status: a green check and "Ready to run" when the Mac
// can launch, an amber/red line when setup is blocked or still loading, plus a
// Diagnostics link for the full checks. The blocking detail renders in the
// SetupCard below; this line is the at-a-glance summary.
function ReadinessLine(props: MainViewProps) {
  const state = props.setupReady
    ? "ready"
    : props.loading && props.dashboard === null
      ? "checking"
      : "blocked"
  return (
    <div className="flex flex-wrap items-center gap-x-4 gap-y-1 text-sm">
      <span className="flex items-center gap-2 font-medium">
        {state === "ready" ? (
          <CheckCircle2 className="size-4 text-success" aria-hidden />
        ) : state === "checking" ? (
          <Loader2 className="size-4 animate-spin text-muted-foreground" aria-hidden />
        ) : (
          <CircleAlert className="size-4 text-destructive" aria-hidden />
        )}
        {state === "ready"
          ? "Ready to run"
          : state === "checking"
            ? "Checking setup..."
            : "Setup needed"}
      </span>
      <button
        type="button"
        aria-label="Open diagnostics"
        onClick={() => props.onNavigate({ kind: "diagnostics" })}
        className="flex items-center gap-1 text-primary transition-colors hover:underline focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
      >
        Diagnostics
        <ArrowUpRight className="size-3.5" aria-hidden />
      </button>
    </div>
  )
}

// While setup is blocked, New run stays visible but disabled with the reason
// (spec: disabled, not hidden). The reason text sits next to the button.
function newRunDisabledReason(props: MainViewProps): string | null {
  if (props.setupReady) {
    return null
  }
  if (props.loading && props.dashboard === null) {
    return "Checking your Mac's setup..."
  }
  if (props.dashboard === null) {
    return "RunHaven could not read your Mac's setup. Try Check again."
  }
  return "Fix the setup problem above before starting a run."
}

// After a launch becomes a managed session, the live status, controls, and
// output live only in that session's view. Home shows a single confirmation
// that points at the session, so one run is never controllable from two places.
function LaunchedRunNotice({
  sessionId,
  onSelect,
}: {
  sessionId: string
  onSelect: (sessionId: string) => void
}) {
  return (
    <Alert role="status">
      <CheckCircle2 />
      <AlertTitle>Run started. Find it in Recent activity.</AlertTitle>
      <AlertDescription className="flex flex-col items-start gap-2">
        <span>Open the session to watch its status and controls.</span>
        <Button
          type="button"
          variant="outline"
          size="sm"
          onClick={() => onSelect(sessionId)}
        >
          Open session
        </Button>
      </AlertDescription>
    </Alert>
  )
}

// Fallback control surface for a launched run that never registered as a managed
// session (for example a failed launch), so stop and repair stay reachable.
function RunView(props: MainViewProps & { lastLaunch: LaunchRunResponse }) {
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
