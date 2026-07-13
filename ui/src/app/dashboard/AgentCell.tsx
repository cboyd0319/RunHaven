import { AgentMonogram } from "../AgentMonogram"
import { agentLabel } from "../sessionIdentity"

// The assistant identity shared by History surfaces. The product name is the
// accessible label; the compact monogram is decorative.
export function AgentCell({ agent }: { agent: string }) {
  return (
    <span className="inline-flex min-w-0 items-center gap-2">
      <AgentMonogram agent={agent} className="size-5 rounded text-[0.5rem]" />
      <span className="truncate">{agentLabel(agent)}</span>
    </span>
  )
}
