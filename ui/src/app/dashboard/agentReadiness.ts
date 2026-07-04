import type { AgentProfile, AuthStatusResponse } from "@/commands/runhaven"

import { authStatusLabel } from "../labels"
import type { StatusTone } from "./status"

// Agent-setup readiness for the Home "Agent setup" panel. The label stays the
// truthful, backend-derived auth vocabulary (labels.ts), never a mock-only word
// the backend cannot substantiate; the tone and category are a presentation
// layer on top, used for the panel summary ("1 ready, 3 need setup, ...") and
// the status dot color. "API key broker" is a method, not a failure (design
// doc security rule), so it counts as ready.

export type ReadinessCategory = "ready" | "needs-setup" | "unavailable"

export type AgentReadiness = {
  name: string
  label: string
  tone: StatusTone
  category: ReadinessCategory
}

function categoryFor(status: string): { tone: StatusTone; category: ReadinessCategory } {
  switch (status.trim().toLowerCase()) {
    case "ready":
    case "available":
    case "n/a":
    case "no-auth-needed":
    case "keychain-oauth":
    case "api-key-broker":
    case "brokered":
      return { tone: "success", category: "ready" }
    case "needs-sign-in":
    case "login-state":
      return { tone: "active", category: "needs-setup" }
    case "design-only":
    case "unsupported":
      return { tone: "failed", category: "unavailable" }
    case "":
    case "-":
    case "unknown":
      return { tone: "neutral", category: "needs-setup" }
    default:
      // An unmapped but plausible status is treated as setup-pending, not a
      // hard failure: it may just be a state the UI has not named yet.
      return { tone: "active", category: "needs-setup" }
  }
}

export function agentReadiness(
  agents: AgentProfile[],
  authStatus: AuthStatusResponse | null,
): AgentReadiness[] {
  return agents.map((agent) => {
    const status =
      authStatus?.profiles.find((profile) => profile.name === agent.name)?.status ?? ""
    const { tone, category } = categoryFor(status)
    return {
      name: agent.name,
      label: authStatusLabel(status),
      tone,
      category,
    }
  })
}

// The one-line summary above the agent list: "1 ready, 3 need setup, 2 unavailable".
// Zero-count clauses are dropped so the line stays short and true.
export function readinessSummary(readiness: AgentReadiness[]): string {
  const ready = readiness.filter((item) => item.category === "ready").length
  const needSetup = readiness.filter((item) => item.category === "needs-setup").length
  const unavailable = readiness.filter((item) => item.category === "unavailable").length
  const parts: string[] = []
  if (ready > 0) parts.push(`${ready} ready`)
  if (needSetup > 0) parts.push(`${needSetup} need setup`)
  if (unavailable > 0) parts.push(`${unavailable} unavailable`)
  if (parts.length === 0) {
    return "No agents configured yet."
  }
  return `${parts.join(", ")}.`
}
