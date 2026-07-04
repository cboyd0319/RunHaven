import { agentLabel } from "../sessionIdentity"
import { agentVisual } from "./agentVisuals"

// The agent identity cell shared by the Needs review and Recent activity tables
// and the Agent setup rows: a product-safe tinted glyph plus the agent's
// display name. One place so every agent reference looks the same.
export function AgentCell({ agent }: { agent: string }) {
  const { Icon, tint } = agentVisual(agent)
  return (
    <span className="inline-flex items-center gap-2">
      <Icon aria-hidden className={`size-4 ${tint}`} />
      <span className="truncate">{agentLabel(agent)}</span>
    </span>
  )
}
