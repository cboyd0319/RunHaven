import { useEffect, useState } from "react"

import {
  getDashboardStatus,
  hasTauriRuntime,
  type DashboardStatus,
  type LaunchRunResponse,
} from "@/commands/runhaven"

import { sameDashboardStatus } from "./appStatusEquality"
import { sessionAllowsRunControl } from "./ManagedSessionCard"
import { errorMessage } from "./types"
import { createSessionActions } from "./sessionActions"
import { useAppDerivedState } from "./useAppDerivedState"
import { useLaunchFlow } from "./useLaunchFlow"
import { useManagedSessionsState } from "./useManagedSessionsState"
import { useManagedSessionDiagnostics } from "./useManagedSessionDiagnostics"
import { useManagedSessionStatus } from "./useManagedSessionStatus"
import { useLaunchStatus } from "./useLaunchStatus"
import { useRunOutput } from "./useRunOutput"
import { useSavedSetups } from "./useSavedSetups"
import { useRunControlState } from "./useRunControlState"
import { useSessionInputState } from "./useSessionInputState"
import { useStatusLoaders } from "./useStatusLoaders"

export function useAppController() {
  const [dashboard, setDashboard] = useState<DashboardStatus | null>(null)
  const [loading, setLoading] = useState(true)
  // When the last successful status read completed, for the setup freshness
  // freshness line. Null until the first read lands.
  const [lastRefreshedAt, setLastRefreshedAt] = useState<string | null>(null)
  const [lastLaunch, setLastLaunch] = useState<LaunchRunResponse | null>(null)
  const {
    managedSessions,
    navigation,
    selectedSessionId,
    selectedSessionIdRef,
    navigate,
    selectSession,
    loadManagedSessions,
  } = useManagedSessionsState(lastLaunch?.runId ?? null)
  const statusLoaders = useStatusLoaders()
  const [error, setError] = useState("")
  const launchFlow = useLaunchFlow({
    dashboard,
    managedBrowserPreview:
      !hasTauriRuntime() &&
      new URLSearchParams(window.location.search).get("preview") !== "terminal",
    loadImageStatus: statusLoaders.loadImageStatus,
    loadRunStatus: statusLoaders.loadRunStatus,
    loadManagedSessions,
    selectSession,
    navigate,
    onLaunched: setLastLaunch,
    onError: setError,
  })
  const savedSetups = useSavedSetups({
    applyRequest: launchFlow.applySavedSetup,
    navigate,
  })

  const derived = useAppDerivedState({
    dashboard,
    request: launchFlow.request,
    managedSessions,
    navigation,
    selectedSessionId,
    lastLaunch,
  })
  const statusSessionId =
    derived.selectedSession && sessionAllowsRunControl(derived.selectedSession)
      ? selectedSessionId
      : null
  const selectedSessionStatus = useManagedSessionStatus(statusSessionId)
  const selectedSessionDiagnostics = useManagedSessionDiagnostics(selectedSessionId)
  const selectedSession =
    selectedSessionStatus.data?.session ?? derived.selectedSession
  const launchStatus = useLaunchStatus(
    lastLaunch && !selectedSession ? lastLaunch.runId : null,
  )
  const sessionInput = useSessionInputState(selectedSessionId)
  const runControlState = useRunControlState(derived.controlTargetId)
  const runOutput = useRunOutput({
    targetId: derived.outputTargetId,
    sessionId: derived.outputSessionId,
    runId: derived.outputRunId,
  })

  useEffect(() => {
    void refresh()
  }, [])

  async function refresh() {
    setLoading(true)
    setError("")
    try {
      const nextDashboard = await getDashboardStatus()
      setDashboard((current) =>
        current && sameDashboardStatus(current, nextDashboard)
          ? current
          : nextDashboard,
      )
      // The status read succeeded; stamp it for the setup freshness line.
      setLastRefreshedAt(new Date().toISOString())
      await loadManagedSessions()
      await launchFlow.applyDashboard(nextDashboard)
      if (lastLaunch) {
        await statusLoaders.loadRunStatus(lastLaunch.runId)
      }
    } catch (cause) {
      setError(errorMessage(cause))
    } finally {
      setLoading(false)
    }
  }

  const sessionActions = createSessionActions({
    selectedSession,
    lastLaunch,
    selectedSessionIdRef,
    selectedSessionStatus,
    sessionInput,
    runControlState,
    statusLoaders,
    loadManagedSessions,
  })

  function changeSessionInputDraft(value: string) {
    sessionInput.changeDraft(value)
  }

  return {
    dashboard,
    launchFlow,
    savedSetups,
    loading,
    lastRefreshedAt,
    imageLoading: statusLoaders.imageLoading,
    imageStatus: statusLoaders.imageStatus,
    imageError: statusLoaders.imageError,
    // The "Run accepted" alert lives only until the launched run shows up as a
    // managed session; after that the one-line launch notice is the durable
    // affordance, so the alert clears instead of persisting forever.
    launchMessage: derived.lastLaunchSessionId ? "" : launchFlow.launchMessage,
    lastLaunch,
    lastLaunchSessionId: derived.lastLaunchSessionId,
    managedSessions,
    navigation,
    selectedSessionId,
    sessionInputDraft: sessionInput.draft,
    sessionInputSending: sessionInput.sending,
    sessionInputError: sessionInput.error,
    sessionInputAcceptedMessage: sessionInput.acceptedMessage,
    runStatus: statusLoaders.runStatus,
    runStatusLoading: statusLoaders.runStatusLoading,
    runStatusError: statusLoaders.runStatusError,
    controlBusy: runControlState.busy,
    controlError: runControlState.error,
    controlMessage: runControlState.message,
    error,
    selectedAgent: derived.selectedAgent,
    setupKnown: derived.setupKnown,
    setupReady: derived.setupReady,
    selectedSession,
    selectedSessionStatus: selectedSessionStatus.data,
    selectedSessionStatusLoading: selectedSessionStatus.loading,
    selectedSessionStatusError: selectedSessionStatus.error,
    selectedSessionDiagnostics: selectedSessionDiagnostics.data,
    selectedSessionDiagnosticsLoading: selectedSessionDiagnostics.loading,
    selectedSessionDiagnosticsError: selectedSessionDiagnostics.error,
    launchStatus: launchStatus.data,
    launchStatusLoading: launchStatus.loading,
    launchStatusError: launchStatus.error,
    pageTitle: derived.pageTitle,
    runOutput,
    refresh,
    navigate,
    selectSession,
    runControl: sessionActions.runControl,
    refreshAfterWorktree: sessionActions.refreshAfterWorktree,
    sendSelectedSessionInput: sessionActions.sendSelectedSessionInput,
    loadSelectedSessionDiagnostics: selectedSessionDiagnostics.load,
    changeSessionInputDraft,
  }
}
