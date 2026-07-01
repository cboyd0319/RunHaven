import type { ManagedSessionSummary } from "@/commands/runhaven"

export function displayValue(value: string): string {
  return value.trim() || "Unknown"
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

export function networkLabel(network: string): string {
  switch (network.trim().toLowerCase()) {
    case "provider":
      return "Provider only"
    case "internal":
      return "Local only"
    case "internet":
      return "Internet"
    default:
      return displayValue(network)
  }
}

export function authScopeLabel(scope: string): string {
  switch (scope.trim().toLowerCase()) {
    case "agent":
      return "Shared agent login"
    case "project":
      return "Project"
    default:
      return displayValue(scope)
  }
}

export function authLabel(status: string): string {
  if (status.trim().toLowerCase() === "api-key-broker") {
    return "API key broker"
  }
  return titleCase(status)
}

export function sessionInputLabel(session: ManagedSessionSummary): string {
  return session.input.attached ? "Input ready" : "Input blocked"
}
