import type { ManagedSessionSummary } from "@/commands/runhaven"

export const PRODUCT_AGENTS = ["claude", "codex", "antigravity", "copilot"] as const

function productAgentRank(agent: string): number {
  const rank = PRODUCT_AGENTS.indexOf(agent as (typeof PRODUCT_AGENTS)[number])
  return rank === -1 ? PRODUCT_AGENTS.length : rank
}

export function orderProductAgents<T extends { name: string }>(agents: readonly T[]): T[] {
  return [...agents].sort(
    (left, right) => productAgentRank(left.name) - productAgentRank(right.name),
  )
}

export function displayValue(value: string): string {
  return value.trim() || "Unknown"
}

export function agentLabel(value: string): string {
  return titleCase(value)
}

export function titleCase(value: string): string {
  const cleaned = value.trim().replaceAll("-", " ")
  if (!cleaned) {
    return "Unknown"
  }
  return cleaned
    .split(/\s+/)
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(" ")
}

export function sessionIdentityLabel(session: ManagedSessionSummary): string {
  const project = identityValue(session.project)
  const sessionName = identityValue(session.session)
  const projectText = `Project ${project}`

  if (
    sessionName === "Unknown" ||
    sessionName.toLowerCase() === project.toLowerCase()
  ) {
    return projectText
  }

  return `${projectText} / Session ${sessionName}`
}

export function sessionInputLabel(session: ManagedSessionSummary): string {
  return session.input.attached ? "Messages available" : "Messages unavailable"
}

// The backend provides the single display title used across History and session
// detail. It is derived from the sanitized assistant and project labels, never
// a raw host path, container name, or token.
export function sessionDisplayTitle(session: ManagedSessionSummary): string {
  return session.title
}

function identityValue(value: string): string {
  const cleaned = displayValue(value)
  if (cleaned === "Unknown" || cleaned === "-") {
    return "Unknown"
  }

  const parts = cleaned.split(/[\\/]+/).filter(Boolean)
  return parts[parts.length - 1] ?? cleaned
}
