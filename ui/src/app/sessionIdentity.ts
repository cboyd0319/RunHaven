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

// The display name for a session: the user's GUI-only label when one is set,
// otherwise the derived "Agent - project" title. The label map is keyed by the
// opaque session id and its values are already sanitized in persistence; the
// trim guard here keeps a blank label from ever winning over the derived title.
// The display title, used across the sidebar and the Home run tables. Both
// sources are secret-free by contract: the GUI label map is sanitized in
// persistence, and `session.title` is the backend's sanctioned "Agent - project"
// display field (derived from the sanitized project basename, never a raw host
// path, container name, or token). This is the one rendered field with no extra
// frontend shape guard, so that backend contract is the guarantee; keep it.
export function sessionDisplayTitle(
  session: ManagedSessionSummary,
  labels: Record<string, string>,
): string {
  const label = labels[session.id]
  return label && label.trim() ? label : session.title
}

function identityValue(value: string): string {
  const cleaned = displayValue(value)
  if (cleaned === "Unknown" || cleaned === "-") {
    return "Unknown"
  }

  const parts = cleaned.split(/[\\/]+/).filter(Boolean)
  return parts[parts.length - 1] ?? cleaned
}
