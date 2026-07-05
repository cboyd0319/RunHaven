import { History } from "lucide-react"

import type { ManagedSessionSummary } from "@/commands/runhaven"

import { DashboardPanel, PanelFooterLink } from "./DashboardPanel"
import { RunTable } from "./RunTable"

// Home "Recent activity": the newest runs of any lifecycle, so a just-launched
// session is reachable here even though the sidebar no longer lists individual
// sessions. Row click opens the session. The footer opens the full run history.
export function RecentActivityPanel({
  sessions,
  onSelectSession,
  onOpenHistory,
}: {
  sessions: ManagedSessionSummary[]
  onSelectSession: (sessionId: string) => void
  onOpenHistory: () => void
}) {
  return (
    <DashboardPanel
      title="Recent activity"
      footer={
        <PanelFooterLink
          icon={History}
          label="Open full run history"
          onClick={onOpenHistory}
        />
      }
    >
      {sessions.length === 0 ? (
        <p className="py-2 text-sm text-muted-foreground">No runs yet.</p>
      ) : (
        <RunTable sessions={sessions} onSelectSession={onSelectSession} />
      )}
    </DashboardPanel>
  )
}
