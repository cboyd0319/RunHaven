import { useMemo } from "react"

import {
  type DashboardStatus,
  type LaunchRunResponse,
  type ManagedSessionList,
  type RunPlanRequest,
} from "@/commands/runhaven"

import type { NavDestination } from "./navigation"
import { sessionIdForRun } from "./sessionSelection"

type AppDerivedStateInput = {
  dashboard: DashboardStatus | null
  request: RunPlanRequest
  managedSessions: ManagedSessionList
  navigation: NavDestination
  selectedSessionId: string | null
  lastLaunch: LaunchRunResponse | null
}

// Header title and description per destination, in the shared plain-language
// vocabulary.
const PAGE_TITLES: Record<NavDestination["kind"], string> = {
  home: "Home",
  "new-session": "New session",
  runs: "History",
  session: "Session",
  diagnostics: "System",
  help: "Help",
  "run-terminal": "Session terminal",
}

export function useAppDerivedState(input: AppDerivedStateInput) {
  const selectedAgent = useMemo(
    () => input.dashboard?.agents.find((agent) => agent.name === input.request.agent),
    [input.dashboard?.agents, input.request.agent],
  )
  const setupKnown = input.dashboard !== null
  const setupReady = input.dashboard?.setup.ok === true
  const selectedSession = useMemo(
    () =>
      input.managedSessions.sessions.find(
        (session) => session.id === input.selectedSessionId,
      ) ?? null,
    [input.managedSessions.sessions, input.selectedSessionId],
  )
  const pageTitle = PAGE_TITLES[input.navigation.kind]
  const outputSessionId = selectedSession?.id ?? null
  const outputRunId = selectedSession ? null : input.lastLaunch?.runId ?? null
  const outputTargetId = outputSessionId
    ? `session:${outputSessionId}`
    : outputRunId
      ? `run:${outputRunId}`
      : null
  const controlTargetId = selectedSession
    ? `session:${selectedSession.id}`
    : input.lastLaunch
      ? `run:${input.lastLaunch.runId}`
      : null
  // The managed-session id (if any) for the most recent launch. Once a launch
  // becomes a managed session, its live status and controls belong only to that
  // session view, so the launch screen collapses to a one-line pointer instead
  // of a second control surface for the same run.
  const lastLaunchSessionId = input.lastLaunch
    ? sessionIdForRun(input.managedSessions, input.lastLaunch.runId)
    : null

  return {
    selectedAgent,
    setupKnown,
    setupReady,
    selectedSession,
    pageTitle,
    outputTargetId,
    outputSessionId,
    outputRunId,
    controlTargetId,
    lastLaunchSessionId,
  }
}
