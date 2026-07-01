import type { ManagedSessionList } from "@/commands/runhaven"

type LoadSessionsUntilRunPresentOptions = {
  attempts?: number
  delayMs?: number
  sleep?: (delayMs: number) => Promise<void>
}

type LoadSessionsUntilRunPresentResult = {
  sessions: ManagedSessionList
  sessionId: string | null
}

export function sessionIdForRun(
  managedSessions: ManagedSessionList,
  runId: string,
): string | null {
  return (
    managedSessions.sessions.find((session) => session.runId === runId)?.id ?? null
  )
}

export async function loadSessionsUntilRunPresent(
  loadSessions: () => Promise<ManagedSessionList>,
  runId: string,
  options: LoadSessionsUntilRunPresentOptions = {},
): Promise<LoadSessionsUntilRunPresentResult> {
  const attempts = Math.max(1, options.attempts ?? 6)
  const delayMs = options.delayMs ?? 125
  const sleep = options.sleep ?? defaultSleep
  let sessions: ManagedSessionList = { sessions: [] }

  for (let attempt = 0; attempt < attempts; attempt += 1) {
    sessions = await loadSessions()
    const sessionId = sessionIdForRun(sessions, runId)
    if (sessionId || attempt === attempts - 1) {
      return { sessions, sessionId }
    }
    await sleep(delayMs)
  }

  return { sessions, sessionId: null }
}

function defaultSleep(delayMs: number): Promise<void> {
  return new Promise((resolve) => window.setTimeout(resolve, delayMs))
}
