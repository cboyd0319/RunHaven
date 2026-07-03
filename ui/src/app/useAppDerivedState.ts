import { useMemo } from "react"

import {
  isLaunchReady,
  type DashboardStatus,
  type ImageStatusResponse,
  type LaunchRunResponse,
  type ManagedSessionList,
  type RunPlanRequest,
  type RunPlanResponse,
} from "@/commands/runhaven"

import type { NavDestination } from "./navigation"
import { sessionIdForRun } from "./sessionSelection"

type AppDerivedStateInput = {
  dashboard: DashboardStatus | null
  request: RunPlanRequest
  imageStatus: ImageStatusResponse | null
  plan: RunPlanResponse | null
  launchConfirmation: boolean
  confirmedWarnings: string[]
  managedSessions: ManagedSessionList
  navigation: NavDestination
  selectedSessionId: string | null
  lastLaunch: LaunchRunResponse | null
}

// Header title and description per destination, in the shared plain-language
// vocabulary.
const PAGE_HEADINGS: Record<
  NavDestination["kind"],
  { title: string; description: string }
> = {
  home: {
    title: "Home",
    description: "See what needs you, then start a new run.",
  },
  session: {
    title: "Session",
    description: "Review status and access for the selected agent session.",
  },
  agents: {
    title: "Agents",
    description: "Sign in and keep each agent ready to run.",
  },
  diagnostics: {
    title: "Diagnostics",
    description: "Read-only trust checks. Nothing here changes your Mac.",
  },
  maintenance: {
    title: "Maintenance",
    description: "Clean up what RunHaven left behind.",
  },
  "run-terminal": {
    title: "Run terminal",
    description: "Type your task for the interactive agent, live in its sandbox.",
  },
}

export function useAppDerivedState(input: AppDerivedStateInput) {
  const selectedAgent = useMemo(
    () => input.dashboard?.agents.find((agent) => agent.name === input.request.agent),
    [input.dashboard?.agents, input.request.agent],
  )
  const setupKnown = input.dashboard !== null
  const setupReady = input.dashboard?.setup.ok === true
  const launchImageReady = input.request.image
    ? true
    : input.imageStatus?.image.ready === true
  const launchReady =
    setupReady &&
    isLaunchReady(
      input.plan,
      input.launchConfirmation,
      new Set(input.confirmedWarnings),
      launchImageReady,
    )
  const selectedSession = useMemo(
    () =>
      input.managedSessions.sessions.find(
        (session) => session.id === input.selectedSessionId,
      ) ?? null,
    [input.managedSessions.sessions, input.selectedSessionId],
  )
  const { title: pageTitle, description: pageDescription } =
    PAGE_HEADINGS[input.navigation.kind]
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
    launchImageReady,
    launchReady,
    selectedSession,
    pageTitle,
    pageDescription,
    outputTargetId,
    outputSessionId,
    outputRunId,
    controlTargetId,
    lastLaunchSessionId,
  }
}
