import { useEffect, useState } from "react"
import { CheckCircle2, CircleAlert } from "lucide-react"

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Button } from "@/components/ui/button"

import type { NavDestination } from "./navigation"
import { TerminalPanel } from "./TerminalPanel"
import { useFocusOnChange } from "./useFocusOnChange"
import { sessionAllowsWorktreeReview } from "./worktreeReview"
import { AgentsView } from "./views/AgentsView"
import { DiagnosticsView } from "./views/DiagnosticsView"
import { HomeView } from "./views/HomeView"
import { MaintenanceView } from "./views/MaintenanceView"
import type { MainViewProps } from "./views/props"
import { RunsView } from "./views/RunsView"
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

  // The top-level action error clears on the next refresh, but a user should be
  // able to dismiss it right away once they have read it. The error lives in the
  // controller, so we suppress it view-locally. Reset the dismissal on ANY error
  // change (not just a clear): after dismissing A, an A->B->A flap is a fresh,
  // current error and must show again. Dismiss itself sets dismissedError=error
  // while error is unchanged, so this effect does not re-fire and the alert stays
  // hidden until the error actually changes.
  const [dismissedError, setDismissedError] = useState("")
  useEffect(() => {
    setDismissedError("")
  }, [error])
  const showError = error !== "" && error !== dismissedError

  // A live interactive run terminal stays MOUNTED here for its whole life,
  // shown only on its own destination and hidden (not unmounted) on every other.
  // This is load-bearing: the run's sandbox uses this terminal's host PTY as its
  // controlling TTY, so unmounting the panel would close the PTY and kill the
  // running agent (audit RT-1). Navigating away therefore hides it; it is torn
  // down only when the run terminal is cleared (End run), which unmounts the
  // panel and fires its teardown. The impossible null case (run-terminal
  // destination with no terminal in state) falls through to the normal views.
  const runTerminal = props.launchFlow.runTerminal
  const showTerminal = navigation.kind === "run-terminal" && runTerminal !== null

  return (
    <main className="flex min-h-0 flex-1 flex-col overflow-hidden">
      {runTerminal ? (
        <div
          // The bounded full-height layout the xterm needs: the container must
          // flex-fill a definite parent so FitAddon settles and never grows the
          // layout past the window. `hidden` keeps it mounted but out of view.
          hidden={!showTerminal}
          className="flex min-h-0 flex-1 flex-col gap-3 p-4"
        >
          {showTerminal && showError ? (
            <AppErrorAlert
              message={error}
              onDismiss={() => setDismissedError(error)}
            />
          ) : null}
          <div
            ref={showTerminal ? focusRef : undefined}
            tabIndex={-1}
            aria-label="Run terminal"
            className="flex min-h-0 flex-1 flex-col outline-none"
          >
            <TerminalPanel
              run={runTerminal}
              fill
              visible={showTerminal}
              onClose={props.launchFlow.closeRunTerminal}
            />
          </div>
        </div>
      ) : null}

      {!showTerminal ? (
        <div className="min-h-0 flex-1 overflow-y-auto p-4 sm:p-6">
          <section className="mx-auto flex w-full max-w-[1440px] min-w-0 flex-col gap-4">
            <div
              ref={focusRef}
              tabIndex={-1}
              aria-label={mainViewLabel(navigation)}
              className="flex flex-col gap-4 outline-none"
            >
              {showError ? (
                <AppErrorAlert
                  message={error}
                  onDismiss={() => setDismissedError(error)}
                />
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
        </div>
      ) : null}
    </main>
  )
}

// The shared top-level action error, with a Dismiss action so it does not
// outlive its cause while the user waits for the next refresh.
function AppErrorAlert({
  message,
  onDismiss,
}: {
  message: string
  onDismiss: () => void
}) {
  return (
    <Alert variant="destructive">
      <CircleAlert />
      <AlertTitle>Action failed</AlertTitle>
      <AlertDescription className="flex flex-col items-start gap-2">
        <span>{message}</span>
        <Button
          type="button"
          variant="outline"
          size="sm"
          onClick={onDismiss}
        >
          Dismiss
        </Button>
      </AlertDescription>
    </Alert>
  )
}

function MainView(props: AppMainContentProps) {
  switch (props.navigation.kind) {
    case "agents":
      return (
        <AgentsView
          agents={props.dashboard?.agents ?? []}
          agentReadiness={props.dashboard?.agentReadiness ?? []}
          setupReady={props.setupReady}
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
      return <MaintenanceView setupReady={props.setupReady} />
    case "runs":
      return (
        <RunsView
          filter={props.navigation.filter ?? "all"}
          sessions={props.sessions}
          onSelectSession={props.onSelectSession}
          onFilterChange={(filter) =>
            props.onNavigate({ kind: "runs", filter })
          }
        />
      )
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
    case "runs":
      return `runs:${navigation.filter ?? "all"}`
    default:
      return navigation.kind
  }
}

function mainViewLabel(navigation: NavDestination): string {
  switch (navigation.kind) {
    case "home":
      return "Home"
    case "session":
      return "Run details"
    case "agents":
      return "Agents"
    case "diagnostics":
      return "System Check"
    case "maintenance":
      return "Cleanup"
    case "runs":
      return "Run history"
    case "run-terminal":
      return "Run terminal"
  }
}
