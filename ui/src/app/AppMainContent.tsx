import { CheckCircle2, CircleAlert } from "lucide-react"

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"

import type { NavDestination } from "./navigation"
import { TerminalPanel } from "./TerminalPanel"
import { useFocusOnChange } from "./useFocusOnChange"
import { sessionAllowsWorktreeReview } from "./worktreeReview"
import { AgentsView } from "./views/AgentsView"
import { DiagnosticsView } from "./views/DiagnosticsView"
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

  // The interactive run terminal is a bounded full-height destination, not the
  // scrolling max-width column the other views use: the xterm container must
  // flex-fill a definite parent so FitAddon settles at a stable size and can
  // never grow the layout past the window. It renders only with a live run
  // terminal in state; the impossible null case falls through to the normal
  // views (MainView renders Home) so navigation is never dead.
  if (navigation.kind === "run-terminal" && props.launchFlow.runTerminal) {
    return (
      <main className="flex min-h-0 flex-1 flex-col gap-3 p-4">
        {error ? (
          <Alert variant="destructive">
            <CircleAlert />
            <AlertTitle>Action failed</AlertTitle>
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        ) : null}
        <div
          ref={focusRef}
          tabIndex={-1}
          aria-label="Run terminal"
          className="flex min-h-0 flex-1 flex-col outline-none"
        >
          <TerminalPanel
            run={props.launchFlow.runTerminal}
            fill
            onClose={props.launchFlow.closeRunTerminal}
          />
        </div>
      </main>
    )
  }

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
          refreshing={props.loading}
          onSignedIn={props.onSignedIn}
          onRefresh={props.onRefresh}
        />
      )
    case "diagnostics":
      return (
        <DiagnosticsView
          dashboard={props.dashboard}
          refreshing={props.loading}
          diagnostics={props.diagnostics}
          diagnosticsLoading={props.diagnosticsLoading}
          diagnosticsError={props.diagnosticsError}
          onLoadDiagnostics={props.onLoadDiagnostics}
          onRefresh={props.onRefresh}
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
    case "run-terminal":
      // Reached only in the impossible case of a run-terminal destination with
      // no run terminal in state (AppMainContent renders the real terminal
      // before MainView otherwise). Fall back to Home so navigation is never
      // dead and the user is never stranded on a blank view.
      return <HomeView {...props} />
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
    case "run-terminal":
      return "Run terminal"
  }
}
