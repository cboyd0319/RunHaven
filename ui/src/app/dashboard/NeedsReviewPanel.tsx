import type { ManagedSessionSummary } from "@/commands/runhaven"
import { Button } from "@/components/ui/button"

import { sessionAttention } from "../sessionAttention"
import { useSessionLabels } from "../sessionLabels"
import { sessionDisplayTitle } from "../sessionIdentity"
import { AgentCell } from "./AgentCell"
import { DashboardPanel } from "./DashboardPanel"
import { timeAgo } from "./timeAgo"

// Home "Needs review": the runs waiting on the user, the strongest signal on the
// page. Each row names the run, its agent, the issue, when it last changed, and
// a Review action that opens the session. The whole row is also clickable so the
// hit area is large; the Review button is the labelled affordance for keyboard
// and screen-reader users. Empty state hides the panel (the caller checks the
// count), never an empty grid.
export function NeedsReviewPanel({
  sessions,
  onSelectSession,
}: {
  sessions: ManagedSessionSummary[]
  onSelectSession: (sessionId: string) => void
}) {
  const { labels } = useSessionLabels()
  return (
    <DashboardPanel title="Needs review" count={sessions.length}>
      <table className="w-full border-collapse text-sm">
        <thead>
          <tr className="text-left text-xs text-muted-foreground">
            <th className="pb-2 pl-2 font-medium">Run</th>
            <th className="pb-2 font-medium">Agent</th>
            <th className="pb-2 font-medium">Issue</th>
            <th className="pb-2 font-medium">Updated</th>
            <th className="pb-2 pr-2 text-right font-medium">Action</th>
          </tr>
        </thead>
        <tbody>
          {sessions.map((session) => {
            const attention = sessionAttention(session)
            return (
              <tr
                key={session.id}
                className="cursor-pointer border-t transition-colors hover:bg-muted/50"
                onClick={() => onSelectSession(session.id)}
              >
                <td className="max-w-0 py-2.5 pl-2">
                  <span className="flex items-center gap-2 truncate font-medium">
                    <span
                      aria-hidden
                      className="size-2 shrink-0 rounded-full bg-destructive"
                    />
                    <span className="truncate">
                      {sessionDisplayTitle(session, labels)}
                    </span>
                  </span>
                </td>
                <td className="py-2.5 whitespace-nowrap">
                  <AgentCell agent={session.agent} />
                </td>
                <td className="py-2.5 text-muted-foreground">
                  {attention?.label ?? "Needs review"}
                </td>
                <td className="py-2.5 whitespace-nowrap text-muted-foreground tabular-nums">
                  {timeAgo(session.startedAt)}
                </td>
                <td className="py-2.5 pr-2 text-right">
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    onClick={(event) => {
                      event.stopPropagation()
                      onSelectSession(session.id)
                    }}
                  >
                    Review
                  </Button>
                </td>
              </tr>
            )
          })}
        </tbody>
      </table>
    </DashboardPanel>
  )
}
