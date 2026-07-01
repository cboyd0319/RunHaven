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

function sameManagedSession(
  current: ManagedSessionSummary,
  next: ManagedSessionSummary | undefined,
): boolean {
  return (
    next !== undefined &&
    current.id === next.id &&
    current.runId === next.runId &&
    current.status === next.status &&
    current.statusLabel === next.statusLabel &&
    current.lifecycle === next.lifecycle &&
    current.authStatus === next.authStatus &&
    current.authScope === next.authScope &&
    current.network === next.network &&
    current.input.attached === next.input.attached &&
    current.input.status === next.input.status &&
    current.input.detail === next.input.detail &&
    current.nextAction.kind === next.nextAction.kind &&
    current.nextAction.label === next.nextAction.label &&
    current.nextAction.detail === next.nextAction.detail &&
    sameSessionEvents(current, next)
  )
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
