import { CheckCircle2, CircleAlert } from "lucide-react"

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"

import { DiagnosticsCard } from "./DiagnosticsCard"
import type { NavDestination } from "./navigation"
import { useFocusOnChange } from "./useFocusOnChange"
import { sessionAllowsWorktreeReview } from "./worktreeReview"
import { AgentsView } from "./views/AgentsView"
import { HomeView } from "./views/HomeView"
import { MaintenanceView } from "./views/MaintenanceView"
import type { MainViewProps } from "./views/props"
import { SessionView } from "./views/SessionView"

type AppMainContentProps = MainViewProps & { navigation: NavDestination }

// Routes the selected sidebar destination to its full view. Every destination
// always renders something real (no dead navigation), and the focus region key
// covers destination changes, session switches, and worktree resolution so
// keyboard and AT users land on the new view instead of <body>. A setup
// blocker renders as Home content, never as a takeover: running sessions and
// System destinations stay reachable while blocked.
export function AppMainContent(props: AppMainContentProps) {
  const { navigation, error, launchMessage } = props
  const worktreeReviewEligible = props.selectedSession
    ? sessionAllowsWorktreeReview(props.selectedSession)
    : false
  const focusRef = useFocusOnChange<HTMLDivElement>(
    mainViewFocusKey(navigation, worktreeReviewEligible),
  )

  return (
    <main className="min-h-0 flex-1 overflow-y-auto p-4">
      <section className="mx-auto flex w-full max-w-5xl min-w-0 flex-col gap-4">
        <div
          ref={focusRef}
          tabIndex={-1}
          aria-label={mainViewLabel(navigation)}
          className="flex flex-col gap-4 outline-none"
        >
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
          <MainView {...props} />
        </div>
      </section>
    </main>
  )
}

function MainView(props: AppMainContentProps) {
  switch (props.navigation.kind) {
    case "agents":
      return (
        <AgentsView
          agents={props.dashboard?.agents ?? []}
          authStatus={props.authStatus}
          onSignedIn={props.onSignedIn}
        />
      )
    case "diagnostics":
      return (
        <DiagnosticsCard
          diagnostics={props.diagnostics}
          loading={props.diagnosticsLoading}
          error={props.diagnosticsError}
          onLoad={props.onLoadDiagnostics}
        />
      )
    case "maintenance":
      return <MaintenanceView />
    case "session":
      // A removed record renavigates via the selection fallback; render nothing
      // for the one frame in between instead of flashing another view.
      return props.selectedSession ? (
        <SessionView {...props} selectedSession={props.selectedSession} />
      ) : null
    case "home":
      return <HomeView {...props} />
  }
}

// One focus key for the whole content pane: destination changes, session
// switches, and worktree-review resolution all move focus; background polls
// change none of these inputs, so quiet refresh stays quiet. The New Run
// sheet manages its own focus (dialog trap plus a per-step region).
function mainViewFocusKey(
  navigation: NavDestination,
  worktreeReviewEligible: boolean,
): string {
  switch (navigation.kind) {
    case "session":
      return `session:${navigation.id}:${worktreeReviewEligible}`
    default:
      return navigation.kind
  }
}

function mainViewLabel(navigation: NavDestination): string {
  switch (navigation.kind) {
    case "home":
      return "Home"
    case "session":
      return "Session details"
    case "agents":
      return "Agents"
    case "diagnostics":
      return "Diagnostics"
    case "maintenance":
      return "Maintenance"
  }
}
