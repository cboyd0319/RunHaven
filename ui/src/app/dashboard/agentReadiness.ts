import type { AgentReadiness } from "@/commands/runhaven"

import type { StatusTone } from "./status"

// Presentation for the backend-derived per-agent sign-in state. The category and
// label are computed by the backend from what RunHaven can truthfully know
// (Shell needs no auth; product agents use secret-free native sign-in state);
// the frontend only maps the category to a dot tone and rolls the categories
// into a one-line summary. No auth logic and no "via RunHaven" framing lives
// here.

export function readinessTone(category: AgentReadiness["category"]): StatusTone {
  switch (category) {
    case "ready":
    case "signed-in":
      return "success"
    case "needs-sign-in":
      return "warning"
    default:
      return "neutral"
  }
}

// "1 signed in, 1 ready, 3 need sign-in". Zero-count clauses are dropped so the
// line stays short and true. "ready" is the no-sign-in-needed state (Shell).
export function readinessSummary(items: AgentReadiness[]): string {
  const signedIn = items.filter((item) => item.category === "signed-in").length
  const ready = items.filter((item) => item.category === "ready").length
  const needs = items.filter((item) => item.category === "needs-sign-in").length
  const parts: string[] = []
  if (signedIn > 0) parts.push(`${signedIn} signed in`)
  if (ready > 0) parts.push(`${ready} ready`)
  if (needs > 0) parts.push(`${needs} need sign-in`)
  if (parts.length === 0) {
    return "No agents configured yet."
  }
  return `${parts.join(", ")}.`
}
