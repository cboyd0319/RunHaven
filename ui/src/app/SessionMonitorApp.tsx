import { useState } from "react"
import { CircleAlert, Eye } from "lucide-react"

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Button } from "@/components/ui/button"

import { ManagedSessionCard } from "./ManagedSessionCard"
import { ManagedSessionDiagnosticsCard } from "./ManagedSessionDiagnosticsCard"
import { ManagedSessionStatusCard } from "./ManagedSessionStatusCard"
import {
  useManagedSessionDiagnostics,
} from "./useManagedSessionDiagnostics"
import { useSessionMonitor, type SessionMonitorState } from "./useSessionMonitor"

// The whole app for a per-session monitor window (label session-<n>): a
// read-only view of one session for watching runs side by side. There are no
// controls here on purpose. Stop, hard stop, repair, cleanup, worktree
// review, messages, and output all live in the main window, and this
// window's Tauri capability cannot invoke any of them; the plain line below
// says so instead of showing disabled buttons. This component never mounts
// the sidebar, settings, menus, badge, or anything that writes shared UI
// state.
export default function SessionMonitorApp() {
  const monitor = useSessionMonitor()
  const diagnostics = useManagedSessionDiagnostics(monitor.sessionId)
  return <SessionMonitorView monitor={monitor} diagnostics={diagnostics} />
}

type MonitorDiagnostics = ReturnType<typeof useManagedSessionDiagnostics>

export function SessionMonitorView({
  monitor,
  diagnostics,
}: {
  monitor: SessionMonitorState
  diagnostics: MonitorDiagnostics
}) {
  return (
    <main className="min-h-screen overflow-y-auto p-4">
      <section className="mx-auto flex w-full max-w-3xl min-w-0 flex-col gap-4">
        <p className="flex items-center gap-2 text-sm text-muted-foreground">
          <Eye aria-hidden="true" className="size-4 shrink-0" />
          This window only watches the session. Controls live in the main
          window.
        </p>
        <MonitorBody monitor={monitor} diagnostics={diagnostics} />
      </section>
    </main>
  )
}

function MonitorBody({
  monitor,
  diagnostics,
}: {
  monitor: SessionMonitorState
  diagnostics: MonitorDiagnostics
}) {
  if (monitor.resolveError) {
    return (
      <Alert variant="destructive">
        <CircleAlert />
        <AlertTitle>Nothing to watch</AlertTitle>
        <AlertDescription>{monitor.resolveError}</AlertDescription>
      </Alert>
    )
  }
  if (!monitor.sessionId || (!monitor.status && !monitor.gone && !monitor.pollError)) {
    return <p className="text-sm text-muted-foreground">Loading session...</p>
  }
  return (
    <>
      {monitor.gone ? (
        <Alert role="status">
          <CircleAlert />
          <AlertTitle>This session is gone</AlertTitle>
          <AlertDescription>
            Its record was removed, so this view will not update anymore.
            Close this window when you are done (Cmd-W).
          </AlertDescription>
        </Alert>
      ) : null}
      {monitor.pollError ? (
        <Alert variant="destructive">
          <CircleAlert />
          <AlertTitle>Could not update</AlertTitle>
          <AlertDescription>{monitor.pollError}</AlertDescription>
        </Alert>
      ) : null}
      {monitor.status ? (
        <>
          <ManagedSessionCard session={monitor.status.session} />
          {monitor.gone ? null : (
            <MonitorDetails monitor={monitor} diagnostics={diagnostics} />
          )}
        </>
      ) : null}
    </>
  )
}

// Same temporary-disclosure pattern as the main window's session details:
// runtime status and read-only diagnostics render only while open. The
// toggle itself never loads or mutates anything.
function MonitorDetails({
  monitor,
  diagnostics,
}: {
  monitor: SessionMonitorState
  diagnostics: MonitorDiagnostics
}) {
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
            status={monitor.status}
            loading={false}
            error={monitor.pollError}
          />
          <ManagedSessionDiagnosticsCard
            diagnostics={diagnostics.data}
            loading={diagnostics.loading}
            error={diagnostics.error}
            onLoad={diagnostics.load}
          />
        </>
      ) : null}
    </section>
  )
}
