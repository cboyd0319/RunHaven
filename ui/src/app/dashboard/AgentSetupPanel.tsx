import { Users } from "lucide-react"

import type { AgentReadiness } from "@/commands/runhaven"

import { readinessSummary, readinessTone } from "./agentReadiness"
import { AgentCell } from "./AgentCell"
import { DashboardPanel, PanelFooterLink } from "./DashboardPanel"
import { StatusDot } from "./status"

// Home "Agent setup": a one-line summary and a row per agent showing its real,
// backend-derived sign-in state as a status dot. Every label is agent-state only
// (Signed in / Needs sign-in / Ready) and never implies RunHaven holds the
// credential; the credential lives in the isolated sandbox (or, for Claude's
// opt-in Keychain broker, the Keychain). The footer opens the Agents view, where
// sign-in actually happens (Home never performs a mutation).
export function AgentSetupPanel({
  readiness,
  onOpenAgents,
}: {
  readiness: AgentReadiness[]
  onOpenAgents: () => void
}) {
  return (
    <DashboardPanel
      title="Agent setup"
      description={readinessSummary(readiness)}
      footer={
        <PanelFooterLink icon={Users} label="Open Agents" onClick={onOpenAgents} />
      }
    >
      <ul className="flex flex-col text-sm">
        {readiness.map((agent) => (
          <li
            key={agent.name}
            className="flex items-center justify-between gap-3 border-t py-2.5 first:border-t-0"
          >
            <AgentCell agent={agent.name} />
            <StatusDot
              tone={readinessTone(agent.category)}
              label={agent.label}
              className="whitespace-nowrap text-muted-foreground"
            />
          </li>
        ))}
      </ul>
    </DashboardPanel>
  )
}
