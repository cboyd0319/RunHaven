import type { ManagedSessionSummary } from "@/commands/runhaven"

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

export function projectLabel(project: string): string {
  return `Project ${identityValue(project)}`
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
  return session.input.attached ? "Input ready" : "Input blocked"
}

function identityValue(value: string): string {
  const cleaned = displayValue(value)
  if (cleaned === "Unknown" || cleaned === "-") {
    return "Unknown"
  }

  const parts = cleaned.split(/[\\/]+/).filter(Boolean)
  return parts[parts.length - 1] ?? cleaned
}
