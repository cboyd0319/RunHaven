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
  const cleaned = status.trim()
  switch (cleaned.toLowerCase()) {
    case "api-key-broker":
    case "brokered":
      return "API key broker"
    case "login-state":
      return "Login state"
    case "needs-sign-in":
      return "Needs sign-in"
    case "no-auth-needed":
    case "n/a":
      return "No auth needed"
    case "ready":
    case "available":
      return "Ready"
    case "":
    case "-":
    case "unknown":
      return "Unknown"
    default:
      return safeAuthStatusLabel(cleaned)
  }
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

function safeAuthStatusLabel(status: string): string {
  if (!/^[a-z0-9][a-z0-9 _-]{0,39}$/i.test(status)) {
    return "Unknown"
  }

  return titleCase(status.replaceAll("_", " "))
}
