import type { NavDestination } from "./navigation"
import { projectName } from "./launchSummary"
import { agentLabel } from "./sessionIdentity"

export type TerminalWorkspaceSession = {
  sessionId: string
  runId?: string
  label?: string
  request?: { plan: { agent: string; workspacePath: string } }
  startedAt?: string
}

export function activeTerminalWorkspace<
  T extends TerminalWorkspaceSession,
>(terminals: readonly T[], id?: string): T | null {
  if (!id) {
    return terminals.at(-1) ?? null
  }
  return (
    terminals.find(
      (terminal) => terminal.runId === id || terminal.sessionId === id,
    ) ?? null
  )
}

export function terminalWorkspaceTitle(
  terminal: TerminalWorkspaceSession,
): string {
  const plan = terminal.request?.plan
  return (
    terminal.label ??
    (plan
      ? `${agentLabel(plan.agent)} · ${projectName(plan.workspacePath)}`
      : "Session starting")
  )
}

export function terminalSidebarLabel(
  terminal: TerminalWorkspaceSession,
  index: number,
  terminals: readonly TerminalWorkspaceSession[],
): string {
  const title = terminalWorkspaceTitle(terminal)
  const started = terminal.startedAt
    ? ` · ${new Intl.DateTimeFormat(undefined, {
        hour: "numeric",
        minute: "2-digit",
        second: "2-digit",
      }).format(new Date(terminal.startedAt))}`
    : ""
  const label = `${title}${started}`
  const occurrence =
    terminals
      .slice(0, index)
      .filter((candidate) => terminalWorkspaceTitle(candidate) === title).length + 1
  return occurrence > 1 ? `${label} (${occurrence})` : label
}

export function terminalWorkspacePageTitle(
  navigation: NavDestination,
  terminals: readonly TerminalWorkspaceSession[],
  fallback: string,
): string {
  if (navigation.kind !== "run-terminal") {
    return fallback
  }
  const terminal = activeTerminalWorkspace(terminals, navigation.id)
  return terminal ? terminalWorkspaceTitle(terminal) : fallback
}
