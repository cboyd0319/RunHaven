import type { AuthExplainResponse } from "@/commands/runhaven"

// Pure mapping and label helpers for per-agent auth details. Shared by the
// launch access summary and the on-demand auth details section so the two views
// speak about auth state with one vocabulary. No secrets, tokens, or host paths
// are ever produced here; input is already the secret-free backend contract.

export type AuthDetailSection = {
  label: string
  items: string[]
}

export function authStatusBadgeVariant(
  status: string,
): "destructive" | "secondary" | "success" | "warning" {
  switch (status.trim().toLowerCase()) {
    case "api-key-broker":
    case "keychain-oauth":
    case "brokered":
    case "ready":
    case "available":
    case "n/a":
      return "success"
    case "design-only":
      return "warning"
    default:
      return "secondary"
  }
}

// Turn the auth details response into labeled lists for display. Empty lists are
// dropped so the view shows only sections that have something to say.
export function authDetailSections(
  response: AuthExplainResponse,
): AuthDetailSection[] {
  return [
    { label: "Supported sign-in", items: response.supportedAuth },
    { label: "What this Mac keeps", items: response.hostKeeps },
    { label: "What the agent gets", items: response.guestReceives },
    { label: "Provider hosts", items: response.providerHosts },
    { label: "Notes", items: response.notes },
  ].filter((section) => section.items.length > 0)
}
