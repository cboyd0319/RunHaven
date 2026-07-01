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
