import type { ManagedSessionList, ManagedSessionSummary } from "@/commands/runhaven"

export function sameManagedSessionList(
  current: ManagedSessionList,
  next: ManagedSessionList,
): boolean {
  return (
    current.sessions.length === next.sessions.length &&
    current.sessions.every((session, index) =>
      sameManagedSession(session, next.sessions[index]),
    )
  )
}

export function sameManagedSession(
  current: ManagedSessionSummary,
  next: ManagedSessionSummary | undefined,
): boolean {
  return (
    next !== undefined &&
    current.id === next.id &&
    current.runId === next.runId &&
    current.agent === next.agent &&
    current.session === next.session &&
    current.title === next.title &&
    current.project === next.project &&
    current.status === next.status &&
    current.statusLabel === next.statusLabel &&
    current.lifecycle === next.lifecycle &&
    current.authStatus === next.authStatus &&
    current.authScope === next.authScope &&
    current.network === next.network &&
    current.startedAt === next.startedAt &&
    current.containerName === next.containerName &&
    current.detailCommand === next.detailCommand &&
    current.input.attached === next.input.attached &&
    current.input.status === next.input.status &&
    current.input.detail === next.input.detail &&
    sameSessionCleanup(current, next) &&
    current.nextAction.kind === next.nextAction.kind &&
    current.nextAction.label === next.nextAction.label &&
    current.nextAction.detail === next.nextAction.detail &&
    sameSessionEvents(current, next)
  )
}

function sameSessionCleanup(
  current: ManagedSessionSummary,
  next: ManagedSessionSummary,
): boolean {
  const currentCleanup = current.cleanup ?? emptyCleanup
  const nextCleanup = next.cleanup ?? emptyCleanup
  return (
    currentCleanup.available === nextCleanup.available &&
    currentCleanup.target === nextCleanup.target &&
    currentCleanup.label === nextCleanup.label &&
    currentCleanup.detail === nextCleanup.detail
  )
}

const emptyCleanup = {
  available: false,
  target: "",
  label: "",
  detail: "",
}

function sameSessionEvents(
  current: ManagedSessionSummary,
  next: ManagedSessionSummary,
): boolean {
  return (
    current.events.length === next.events.length &&
    current.events.every((event, index) => {
      const nextEvent = next.events[index]
      return (
        nextEvent !== undefined &&
        event.source === nextEvent.source &&
        event.label === nextEvent.label &&
        event.detail === nextEvent.detail &&
        event.timestamp === nextEvent.timestamp
      )
    })
  )
}
