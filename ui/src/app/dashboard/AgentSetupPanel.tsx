import { Users } from "lucide-react"

import type { AgentProfile, AuthStatusResponse } from "@/commands/runhaven"

import { agentReadiness, readinessSummary } from "./agentReadiness"
import { AgentCell } from "./AgentCell"
import { DashboardPanel, PanelFooterLink } from "./DashboardPanel"
import { StatusDot } from "./status"

// Home "Agent setup": a one-line readiness summary and a row per agent showing
// its truthful auth state as a status dot. The footer opens the Agents view,
// where sign-in and key setup actually happen (Home never performs a mutation).
export function AgentSetupPanel({
  agents,
  authStatus,
  onOpenAgents,
}: {
  agents: AgentProfile[]
  authStatus: AuthStatusResponse | null
  onOpenAgents: () => void
}) {
  const readiness = agentReadiness(agents, authStatus)
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
              tone={agent.tone}
              label={agent.label}
              className="whitespace-nowrap text-muted-foreground"
            />
          </li>
        ))}
      </ul>
    </DashboardPanel>
  )
}
