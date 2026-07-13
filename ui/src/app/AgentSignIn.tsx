import { useMemo } from "react"

import type { AgentProfile } from "@/commands/runhaven"

import { agentLabel } from "./sessionIdentity"
import { TerminalPanel } from "./TerminalPanel"

export function AgentSignIn({
  agent,
  onSignedIn,
}: {
  agent: AgentProfile | undefined
  onSignedIn?: () => void
}) {
  const name = agent?.name ?? ""
  const sessionId = useMemo(
    () => `agent-sign-in-${name}-${crypto.randomUUID()}`,
    [name],
  )

  if (!agent || agent.signIn === "n/a") {
    return null
  }

  return (
    <TerminalPanel
      login={{
        agent: agent.name,
        agentLabel: agentLabel(agent.name),
        sessionId,
        onSignedIn,
      }}
    />
  )
}
