import { History } from "lucide-react"

import type { ManagedSessionSummary } from "@/commands/runhaven"

import { safeBackendLabel } from "../labels"
import { useSessionLabels } from "../sessionLabels"
import { sessionDisplayTitle } from "../sessionIdentity"
import { AgentCell } from "./AgentCell"
import { DashboardPanel, PanelFooterLink } from "./DashboardPanel"
import { StatusDot, runStatusTone } from "./status"
import { timeAgo } from "./timeAgo"

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
  const { labels } = useSessionLabels()
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
        <table className="w-full border-collapse text-sm">
          <thead>
            <tr className="text-left text-xs text-muted-foreground">
              <th className="pb-2 pl-2 font-medium">Run</th>
              <th className="pb-2 font-medium">Agent</th>
              <th className="pb-2 font-medium">Status</th>
              <th className="pb-2 pr-2 font-medium">Updated</th>
            </tr>
          </thead>
          <tbody>
            {sessions.map((session) => {
              const label = safeBackendLabel(session.statusLabel, "Unknown")
              const tone = runStatusTone(session.lifecycle, session.statusLabel)
              return (
                <tr
                  key={session.id}
                  className="cursor-pointer border-t transition-colors hover:bg-muted/50"
                  onClick={() => onSelectSession(session.id)}
                >
                  <td className="w-full py-2.5 pl-2">
                    <span className="block truncate font-medium">
                      {sessionDisplayTitle(session, labels)}
                    </span>
                  </td>
                  <td className="py-2.5 whitespace-nowrap">
                    <AgentCell agent={session.agent} />
                  </td>
                  <td className="py-2.5 whitespace-nowrap">
                    <StatusDot tone={tone} label={label} />
                  </td>
                  <td className="py-2.5 pr-2 whitespace-nowrap text-muted-foreground tabular-nums">
                    {timeAgo(session.startedAt)}
                  </td>
                </tr>
              )
            })}
          </tbody>
        </table>
      )}
    </DashboardPanel>
  )
}
