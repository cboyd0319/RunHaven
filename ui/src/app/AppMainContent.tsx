import { useEffect, useState } from "react"
import { CheckCircle2, CircleAlert } from "lucide-react"

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Button } from "@/components/ui/button"

import { helpReturnLabel, type NavDestination } from "./navigation"
import { TerminalPanel } from "./TerminalPanel"
import { useFocusOnChange } from "./useFocusOnChange"
import { DiagnosticsView } from "./views/DiagnosticsView"
import { HelpView } from "./views/HelpView"
import { NewSessionView } from "./views/NewSessionView"
import type { MainViewProps } from "./views/props"
import { RunsView } from "./views/RunsView"
import { SessionView } from "./views/SessionView"

type AppMainContentProps = MainViewProps & { navigation: NavDestination }

// Routes the selected sidebar destination to its full view. Every destination
// always renders something real (no dead navigation), and the focus region key
// covers destination changes, session switches, and explicit worktree actions so
// keyboard and AT users land on the new view instead of <body>. A setup
// blocker renders in New session, never as a takeover: running sessions and
// System details stays reachable while blocked.
export function AppMainContent(props: AppMainContentProps) {
  const { navigation, error, launchMessage } = props
  const [sessionContentRevision, setSessionContentRevision] = useState(0)
  const focusRef = useFocusOnChange<HTMLDivElement>(
    mainViewFocusKey(navigation, sessionContentRevision),
  )
  const refreshWorktreeAndFocus = async () => {
    await props.onWorktreeResolved()
    setSessionContentRevision((revision) => revision + 1)
  }

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
  const showError =
    error !== "" && error !== dismissedError

  // A live interactive run terminal stays MOUNTED here for its whole life,
  // shown only on its own destination and hidden (not unmounted) on every other.
  // This is load-bearing: the run's sandbox uses this terminal's host PTY as its
  // controlling TTY, so unmounting the panel would close the PTY and kill the
  // running agent (audit RT-1). Navigating away therefore hides it; it is torn
  // down only when the run terminal is cleared (End session), which unmounts the
  // panel and fires its teardown. The impossible null case (run-terminal
  // destination with no terminal in state) falls through to the normal views.
  const runTerminals = props.launchFlow.runTerminals
  const selectedRunTerminal =
    navigation.kind === "run-terminal"
      ? (runTerminals.find(
          (terminal) =>
            terminal.runId === navigation.id ||
            terminal.sessionId === navigation.id,
        ) ?? (navigation.id ? null : runTerminals.at(-1) ?? null))
      : null
  const showTerminal = selectedRunTerminal !== null

  return (
    <main className="flex min-h-0 flex-1 flex-col overflow-hidden">
      {runTerminals.map((runTerminal) => {
        const terminalVisible = runTerminal === selectedRunTerminal
        return (
        <div
          key={runTerminal.sessionId}
          // The bounded full-height layout the xterm needs: the container must
          // flex-fill a definite parent so FitAddon settles and never grows the
          // layout past the window. `hidden` keeps it mounted but out of view.
          hidden={!terminalVisible}
          className="flex min-h-0 flex-1 flex-col gap-3 p-4"
        >
          {terminalVisible && showError ? (
            <AppErrorAlert
              message={error}
              onDismiss={() => setDismissedError(error)}
            />
          ) : null}
          <div
            ref={terminalVisible ? focusRef : undefined}
            tabIndex={-1}
            aria-label="Session terminal"
            className="flex min-h-0 flex-1 flex-col outline-none"
          >
            <TerminalPanel
              run={runTerminal}
              fill
              visible={terminalVisible}
              onOpened={(opened) => {
                props.launchFlow.markRunTerminalOpened(
                  runTerminal.sessionId,
                  opened,
                )
                if (terminalVisible) {
                  props.onNavigate({ kind: "run-terminal", id: opened.runId })
                }
              }}
              onEnded={() =>
                void props.launchFlow.closeRunTerminal(
                  runTerminal.sessionId,
                  terminalVisible,
                )
              }
              onClose={() =>
                void props.launchFlow.closeRunTerminal(runTerminal.sessionId)
              }
              onOpenCancelled={() =>
                props.launchFlow.cancelRunTerminal(runTerminal.sessionId)
              }
            />
          </div>
        </div>
        )
      })}

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
                <Alert role="status">
                  <CheckCircle2 />
                  <AlertTitle>{launchMessage}</AlertTitle>
                  <AlertDescription>You can follow it below.</AlertDescription>
                </Alert>
              ) : null}
              <MainView
                {...props}
                onWorktreeResolved={refreshWorktreeAndFocus}
              />
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
    <Alert variant="destructive" role="alert">
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
    case "new-session":
      return <NewSessionView {...props} />
    case "diagnostics":
      return (
        <DiagnosticsView
          dashboard={props.dashboard}
          refreshing={props.loading}
          onRefresh={props.onRefresh}
        />
      )
    case "runs":
      return (
        <RunsView
          sessions={props.sessions}
          onSelectSession={props.onSelectSession}
          onSignIn={(agent) => {
            props.launchFlow.changeAgent(agent)
            props.launchFlow.setLaunchStep("agent")
            props.onNavigate({ kind: "new-session" })
          }}
          onStart={() => props.onNavigate({ kind: "new-session" })}
        />
      )
    case "help": {
      const returnTo = props.navigation.returnTo
      return (
        <HelpView
          backLabel={helpReturnLabel(returnTo)}
          onBack={() => props.onNavigate(returnTo)}
        />
      )
    }
    case "session":
      // A removed record renavigates via the selection fallback. New Session is a safe,
      // useful frame while that reconciliation happens; an empty main area looks
      // like the app crashed and can strand the user after a fast run exit.
      return props.selectedSession ? (
        <SessionView {...props} selectedSession={props.selectedSession} />
      ) : (
        <NewSessionView {...props} />
      )
    case "run-terminal":
      // Reached only in the impossible case of a run-terminal destination with
      // no run terminal in state (AppMainContent renders the real terminal
      // before MainView otherwise). Fall back to New Session so navigation is never
      // dead and the user is never stranded on a blank view.
      return <NewSessionView {...props} />
  }
}

// One focus key for the whole content pane: destination changes, session
// switches, and explicit worktree actions move focus. Background review-state
// polls do not change the revision, so a quiet refresh never steals focus.
export function mainViewFocusKey(
  navigation: NavDestination,
  sessionContentRevision: number,
): string {
  switch (navigation.kind) {
    case "session":
      return `session:${navigation.id}:${sessionContentRevision}`
    default:
      return navigation.kind
  }
}

function mainViewLabel(navigation: NavDestination): string {
  switch (navigation.kind) {
    case "new-session":
      return "New session"
    case "session":
      return "Session details"
    case "diagnostics":
      return "System details"
    case "help":
      return "RunHaven Help"
    case "runs":
      return "Session history"
    case "run-terminal":
      return "Session terminal"
  }
}
