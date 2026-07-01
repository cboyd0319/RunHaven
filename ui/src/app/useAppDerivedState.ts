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

type AppDerivedStateInput = {
  dashboard: DashboardStatus | null
  request: RunPlanRequest
  imageStatus: ImageStatusResponse | null
  plan: RunPlanResponse | null
  launchConfirmation: boolean
  confirmedWarnings: string[]
  managedSessions: ManagedSessionList
  selectedSessionId: string | null
  lastLaunch: LaunchRunResponse | null
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
  const pageTitle = selectedSession ? "Session" : "Launch"
  const pageDescription = selectedSession
    ? "Review status and access for the selected agent session."
    : "Choose an agent and project. Review access before launch."
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
  }
}
