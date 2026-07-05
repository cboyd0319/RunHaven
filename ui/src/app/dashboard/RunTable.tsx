import type { ManagedSessionSummary } from "@/commands/runhaven"

import { safeBackendLabel } from "../labels"
import { useSessionLabels } from "../sessionLabels"
import { sessionDisplayTitle } from "../sessionIdentity"
import { AgentCell } from "./AgentCell"
import { StatusDot, runStatusTone } from "./status"
import { timeAgo } from "./timeAgo"

// The shared run table (Run | Agent | Status | Updated), used by Home's Recent
// activity and the full Run history view. Columns are table-fixed so they
// distribute; the Run title is a real keyboard-operable button (the row onClick
// is a mouse convenience on top). Renders only sanitized fields (title via
// sessionDisplayTitle, status via safeBackendLabel), never a raw path/container.
export function RunTable({
  sessions,
  onSelectSession,
}: {
  sessions: ManagedSessionSummary[]
  onSelectSession: (sessionId: string) => void
}) {
  const { labels } = useSessionLabels()
  return (
    <table className="w-full table-fixed border-collapse text-sm">
      <thead>
        <tr className="text-left text-xs text-muted-foreground">
          <th className="pb-2 pl-2 font-medium">Run</th>
          <th className="w-[20%] pb-2 font-medium">Agent</th>
          <th className="w-[20%] pb-2 font-medium">Status</th>
          <th className="w-[16%] pb-2 pr-2 font-medium">Updated</th>
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
              <td className="py-2.5 pl-2">
                <button
                  type="button"
                  onClick={(event) => {
                    event.stopPropagation()
                    onSelectSession(session.id)
                  }}
                  className="block max-w-full truncate text-left font-medium hover:underline focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
                >
                  {sessionDisplayTitle(session, labels)}
                </button>
              </td>
              <td className="py-2.5">
                <AgentCell agent={session.agent} />
              </td>
              <td className="py-2.5">
                <StatusDot tone={tone} label={label} />
              </td>
              <td className="py-2.5 pr-2 text-muted-foreground tabular-nums">
                {timeAgo(session.startedAt)}
              </td>
            </tr>
          )
        })}
      </tbody>
    </table>
  )
}
