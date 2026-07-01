import type { ManagedSessionList } from "@/commands/runhaven"

export function sessionIdForRun(
  managedSessions: ManagedSessionList,
  runId: string,
): string | null {
  return (
    managedSessions.sessions.find((session) => session.runId === runId)?.id ?? null
  )
}
