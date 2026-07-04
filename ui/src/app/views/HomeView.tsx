import { CheckCircle2 } from "lucide-react"

import type {
  LaunchRunResponse,
  ManagedSessionSummary,
} from "@/commands/runhaven"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"

import { authStatusLabel, safeBackendLabel } from "../labels"
import { LastLaunchCard } from "../LastLaunchCard"
import { LaunchSheet } from "../LaunchSheet"
import { LaunchStatusCard } from "../LaunchStatusCard"
import { groupSessions } from "../navigation"
import { RunStatusCard } from "../RunStatusCard"
import { sessionAttention } from "../sessionAttention"
import { useSessionLabels } from "../sessionLabels"
import { agentLabel, sessionDisplayTitle } from "../sessionIdentity"
import { SetupCard } from "../SetupCard"
import type { MainViewProps } from "./props"
import { OutputView } from "./SessionView"

// The Home destination (spec 3.1): is this Mac ready, and what needs you right
// now? Setup health leads (the blocker renders here instead of a full-screen
// takeover), New run is the one prominent action, and the lists below point at
// agents and sessions that need attention. The launch flow itself lives in the
// New Run modal sheet.
export function HomeView(props: MainViewProps) {
  const needsYou = props.sessions.filter(
    (session) => sessionAttention(session) !== null,
  )
  // A session in the Needs you list (failed, ready to review) never repeats in
  // the finished list below, so each run has exactly one Home row.
  const recentFinished = groupSessions(props.sessions)
    .finished.filter((session) => sessionAttention(session) === null)
    .slice(0, 5)
  const firstRun = props.sessions.length === 0 && !props.lastLaunch

  return (
    <div className="flex flex-col gap-4">
      <SetupHealth {...props} />
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
      {firstRun && props.setupReady ? <FirstRunCard /> : null}
      <AgentReadinessStrip {...props} />
      {needsYou.length > 0 ? (
        <SessionListCard
          title="Needs you"
          description="These sessions are waiting on you."
          sessions={needsYou}
          statusFor={(session) => sessionAttention(session)?.label ?? ""}
          onSelectSession={props.onSelectSession}
        />
      ) : null}
      {recentFinished.length > 0 ? (
        <SessionListCard
          title="Recent finished runs"
          description="Open one to review or clean it up."
          sessions={recentFinished}
          statusFor={(session) =>
            safeBackendLabel(session.statusLabel, "Finished")
          }
          onSelectSession={props.onSelectSession}
        />
      ) : null}
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

// While setup is blocked, New run stays visible but disabled with the reason
// (spec: disabled, not hidden). The reason text sits next to the button.
function newRunDisabledReason(props: MainViewProps): string | null {
  if (props.setupReady) {
    return null
  }
  // Only the very first setup read (still loading, nothing loaded yet) shows
  // "Checking...". A transient manual refresh keeps the loaded dashboard, so
  // setupReady above already governs whether New run is disabled; loading does
  // not disable it on its own.
  if (props.loading && props.dashboard === null) {
    return "Checking your Mac's setup..."
  }
  // A failed or never-completed first read leaves no dashboard to point at, so
  // do not imply a fixed blocker above; ask the user to check again.
  if (props.dashboard === null) {
    return "RunHaven could not read your Mac's setup. Try Check again."
  }
  return "Fix the setup problem above before starting a run."
}

// Setup health: one quiet line when healthy; the single blocker with its fix
// and a Check Again action when not. Full doctor detail stays in Diagnostics.
function SetupHealth(props: MainViewProps) {
  if (props.setupReady) {
    return (
      <p className="flex items-center gap-2 text-sm text-muted-foreground">
        <CheckCircle2 className="size-4 text-success" aria-hidden />
        Your Mac is ready to run agents. Full checks live in Diagnostics.
      </p>
    )
  }
  return (
    <SetupCard
      loading={props.loading}
      dashboard={props.dashboard}
      onCheckAgain={props.onRefresh}
    />
  )
}

// One line per agent: signed-in state, linking to the Agents view for the
// sign-in and image jobs themselves.
function AgentReadinessStrip(props: MainViewProps) {
  const agents = props.dashboard?.agents ?? []
  if (agents.length === 0) {
    return null
  }
  return (
    <Card>
      <CardHeader>
        <CardTitle role="heading" aria-level={2}>
          Agents
        </CardTitle>
        <CardDescription>
          Sign in and manage agents in the Agents view.
        </CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-2">
        <ul className="flex flex-col gap-1 text-sm">
          {agents.map((agent) => (
            <li key={agent.name} className="flex justify-between gap-3">
              <span className="font-medium">{agentLabel(agent.name)}</span>
              <span className="text-muted-foreground">
                {authStatusLabel(
                  props.authStatus?.profiles.find(
                    (profile) => profile.name === agent.name,
                  )?.status ?? "",
                )}
              </span>
            </li>
          ))}
        </ul>
        <div>
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={() => props.onNavigate({ kind: "agents" })}
          >
            Open Agents
          </Button>
        </div>
      </CardContent>
    </Card>
  )
}

function SessionListCard({
  title,
  description,
  sessions,
  statusFor,
  onSelectSession,
}: {
  title: string
  description: string
  sessions: ManagedSessionSummary[]
  statusFor: (session: ManagedSessionSummary) => string
  onSelectSession: (sessionId: string) => void
}) {
  const { labels } = useSessionLabels()
  return (
    <Card>
      <CardHeader>
        <CardTitle role="heading" aria-level={2}>
          {title}
        </CardTitle>
        <CardDescription>{description}</CardDescription>
      </CardHeader>
      <CardContent>
        <ul className="flex flex-col gap-2">
          {sessions.map((session) => (
            <li key={session.id}>
              <Button
                type="button"
                variant="outline"
                className="w-full justify-between gap-3"
                onClick={() => onSelectSession(session.id)}
              >
                <span className="truncate">
                  {sessionDisplayTitle(session, labels)}
                </span>
                <span className="text-muted-foreground">
                  {statusFor(session)}
                </span>
              </Button>
            </li>
          ))}
        </ul>
      </CardContent>
    </Card>
  )
}

// First-run empty state: what RunHaven does and never shares, in two
// sentences. The New run button above is the next action. The sidebar header
// is the single brand anchor, so this card carries no logo of its own.
function FirstRunCard() {
  return (
    <Card>
      <CardContent>
        <p className="text-sm text-muted-foreground">
          RunHaven runs AI coding agents in a sandbox on your Mac. An agent
          sees only the project folder you pick, never your home folder, keys,
          or logins.
        </p>
      </CardContent>
    </Card>
  )
}

// After a launch becomes a managed session, the live status, controls, and
// output live only in that session's view. Home shows a single confirmation
// that points at the sidebar, so one run is never controllable from two places
// at once.
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
      <AlertTitle>Run started - find it in your sidebar</AlertTitle>
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
